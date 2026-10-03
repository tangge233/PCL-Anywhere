//! 读写与版本闸门。
//!
//! 加载顺序固定：读文本 → 解析文档 → 查版本 → 迁移 → 反序列化。迁移在反序列化**之前**，
//! 所以 [`Schema`] 看不到历史形状，迁移步是唯一知道历史形状的地方。

use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use toml_edit::{DocumentMut, value};

use crate::error::Error;
use crate::schema::{Migration, Schema};

/// 落盘形态：`version` 在最前，其余字段展平到同一层。
#[derive(Serialize)]
struct File<'a, S: Schema> {
    version: u32,
    #[serde(flatten)]
    body: &'a S,
}

/// 内容损坏时的处理方式。
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum OnCorrupt {
    Fail,
    /// 改名备份成 `.corrupt-<时间戳>`，再用默认值重建。
    Replace,
}

/// 按策略加载。文件不存在时用默认值创建。
pub(crate) fn load<S: Schema>(path: &Path, on_corrupt: OnCorrupt) -> Result<S, Error> {
    let text = read_or_create::<S>(path)?;
    match decode::<S>(path, &text) {
        Ok(loaded) => Ok(loaded),
        Err(error) if on_corrupt == OnCorrupt::Replace && error.is_corrupt() => {
            backup(path)?;
            let value = S::default();
            write_atomically(path, &value)?;
            Ok(value)
        }
        Err(error) => Err(error),
    }
}

/// 读文本。文件不存在时先原子写入默认值，让用户打开就能看到全部可选项。
fn read_or_create<S: Schema>(path: &Path) -> Result<String, Error> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(text),
        Err(source) if source.kind() == io::ErrorKind::NotFound => {
            let text = serialize(path, &S::default())?;
            write_text(path, &text)?;
            Ok(text)
        }
        Err(source) => Err(Error::Read {
            path: path.into(),
            source,
        }),
    }
}

/// 解析文档、过版本闸门、跑迁移、反序列化成 [`Schema`]。
///
/// 迁移结果**不回写**：第一次 `mutate` 时整份重写，新版本号才会落下去。没被改过的文件因此
/// 一直保持原格式，降级回老版本仍然能读。
fn decode<S: Schema>(path: &Path, text: &str) -> Result<S, Error> {
    let mut doc: DocumentMut =
        text.parse()
            .map_err(|source: toml_edit::TomlError| Error::Parse {
                path: path.into(),
                source: source.into(),
            })?;

    let found = read_version(path, &doc)?;
    if found > S::VERSION {
        return Err(Error::VersionTooNew {
            path: path.into(),
            found,
            supported: S::VERSION,
        });
    }
    if found < S::VERSION {
        migrate(path, &mut doc, found, S::VERSION, S::migrations())?;
    }

    toml_edit::de::from_document(doc).map_err(|source| Error::Parse {
        path: path.into(),
        source,
    })
}

/// 读 `version` 键。缺失按 v1 处理，手写的、只有几个键的文件因此不需要任何版本标记。
fn read_version(path: &Path, doc: &DocumentMut) -> Result<u32, Error> {
    let bad_version = || Error::BadVersion { path: path.into() };
    match doc.get("version") {
        None => Ok(1),
        Some(item) => item
            .as_integer()
            .and_then(|raw| u32::try_from(raw).ok())
            .filter(|version| *version >= 1)
            .ok_or_else(bad_version),
    }
}

/// 逐级执行迁移，每步之后同步 `version` 键。
fn migrate(
    path: &Path,
    doc: &mut DocumentMut,
    from: u32,
    to: u32,
    steps: &[Migration],
) -> Result<(), Error> {
    // 链的连续性硬校验：声明了 VERSION 却忘了写迁移步会被当场抓住，而不是悄悄把老配置
    // 读成默认值——后者是最难查的一类 bug。
    if steps.len().saturating_add(1) as u64 != u64::from(to) {
        return Err(Error::MigrationChainBroken {
            version: to,
            steps: steps.len(),
            expected: to.saturating_sub(1),
        });
    }
    for version in from..to {
        let step = &steps[(version - 1) as usize];
        (step.apply)(doc).map_err(|source| Error::Migrate {
            path: path.into(),
            version,
            note: step.note,
            source,
        })?;
        doc.insert("version", value(i64::from(version) + 1));
    }
    Ok(())
}

/// 把 [`Schema`] 序列化成落盘文本。
pub(crate) fn serialize<S: Schema>(path: &Path, body: &S) -> Result<String, Error> {
    let mut text = toml_edit::ser::to_string_pretty(&File {
        version: S::VERSION,
        body,
    })
    .map_err(|source| Error::Serialize {
        path: path.into(),
        source,
    })?;
    if !text.ends_with('\n') {
        text.push('\n');
    }
    Ok(text)
}

/// 序列化后原子替换目标文件。
pub(crate) fn write_atomically<S: Schema>(path: &Path, body: &S) -> Result<(), Error> {
    let text = serialize(path, body)?;
    write_text(path, &text)
}

/// 原子替换文件内容。
///
/// `atomic-write-file` 执行的是同目录临时文件 → `fsync(文件)` → `rename` → `fsync(父目录)`，
/// 崩在任何一步都只会留下完整的旧内容。
fn write_text(path: &Path, text: &str) -> Result<(), Error> {
    let parent = path.parent().filter(|dir| !dir.as_os_str().is_empty());
    if let Some(parent) = parent {
        create_private_dir(parent)?;
    }
    let io_error = |source| Error::Write {
        path: path.into(),
        source,
    };
    // 新文件 0600；已存在的文件保持原 mode（`atomic-write-file` 默认沿用）。
    let mut file = atomic_write_file::AtomicWriteFile::options()
        .mode(0o600)
        .open(path)
        .map_err(io_error)?;
    file.write_all(text.as_bytes()).map_err(io_error)?;
    file.commit().map_err(io_error)
}

/// 递归建目录，权限 0700。
///
/// 配置里会有账号路径、Java 参数之类的信息，默认不给同机其它用户读。`mode` 只作用于新建的
/// 层，不改已存在目录的权限。
fn create_private_dir(dir: &Path) -> Result<(), Error> {
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true).mode(0o700);
    builder.create(dir).map_err(|source| Error::Write {
        path: dir.into(),
        source,
    })
}

/// 把损坏的文件改名成 `<名字>.corrupt-<毫秒时间戳>`。
///
/// 不直接覆盖：要的是「路径下有可用配置」，不是销毁证据。多一个 `.corrupt-*` 文件是可接受
/// 的代价，反过来不可逆。
fn backup(path: &Path) -> Result<(), Error> {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis())
        .unwrap_or(0);
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(format!(".corrupt-{millis}"));
    match fs::rename(path, path.with_file_name(name)) {
        Ok(()) => Ok(()),
        // 没有原文件就没什么可备份的。
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(Error::Backup {
            path: path.into(),
            source,
        }),
    }
}
