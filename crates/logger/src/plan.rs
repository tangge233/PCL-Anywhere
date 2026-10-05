//! 把 [`Rotation`] 映射为 flexi_logger 的 `FileSpec` 与轮转配置。

use std::path::Path;

use flexi_logger::{Age, Cleanup, Criterion, FileSpec, Naming};

use crate::BASENAME;
use crate::options::{Options, Rotation};

/// 按轮转方式构造 `FileSpec`。
pub(crate) fn file_spec(options: &Options, dir: &Path, session: &str) -> FileSpec {
    let spec = FileSpec::default().directory(dir).basename(BASENAME);
    match options.rotation {
        // 按大小轮转时把会话标记写进文件名，否则重新启动会从 `r00000` 重新编号，覆盖上次
        // 运行产生的分片。
        Rotation::BySize { .. } => spec.discriminant(session),
        // 会话模式沿用 FileSpec 默认的启动时间戳，每次启动得到新文件；按日模式由轮转命名，
        // 启动时间戳会被自动省略。
        Rotation::BySession | Rotation::ByDate => spec,
    }
}

/// 轮转配置；会话模式返回 `None`。
pub(crate) fn rotation(options: &Options) -> Option<(Criterion, Naming, Cleanup)> {
    match options.rotation {
        Rotation::BySize { max_bytes } => Some((
            Criterion::Size(max_bytes),
            Naming::Numbers,
            Cleanup::KeepLogFiles(options.keep),
        )),
        Rotation::ByDate => Some((
            Criterion::Age(Age::Day),
            Naming::Timestamps,
            Cleanup::KeepLogFiles(options.keep),
        )),
        Rotation::BySession => None,
    }
}

/// 是否需要追加写入。按日模式的文件名跨启动不变，不追加会截掉当天先前的内容。
pub(crate) fn appends(options: &Options) -> bool {
    matches!(options.rotation, Rotation::ByDate)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options(rotation: Rotation) -> Options {
        Options {
            rotation,
            ..Options::default()
        }
    }

    #[test]
    fn by_size_stamps_the_session() {
        let options = options(Rotation::BySize { max_bytes: 1024 });
        let spec = file_spec(&options, Path::new("/log"), "2026-10-04_12-00-00");
        let path = spec.as_pathbuf(Some("rCURRENT"));
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        // 启动时间戳由 flexi_logger 补在会话标记之后，这里只断言前缀与轮转标记。
        assert!(
            name.starts_with("pcl-anywhere_2026-10-04_12-00-00_"),
            "{name}"
        );
        assert!(name.ends_with("_rCURRENT.log"), "{name}");
    }

    #[test]
    fn by_session_keeps_the_start_timestamp() {
        let spec = file_spec(&options(Rotation::BySession), Path::new("/log"), "ignored");
        // 会话文件名由 flexi_logger 在打开文件时生成，这里只确认没有写入会话标记。
        assert!(!spec.as_pathbuf(None).to_string_lossy().contains("ignored"));
    }

    #[test]
    fn rotation_matches_the_mode() {
        assert!(rotation(&options(Rotation::BySession)).is_none());
        assert!(matches!(
            rotation(&options(Rotation::BySize { max_bytes: 10 })),
            Some((
                Criterion::Size(10),
                Naming::Numbers,
                Cleanup::KeepLogFiles(_)
            ))
        ));
        assert!(matches!(
            rotation(&options(Rotation::ByDate)),
            Some((
                Criterion::Age(Age::Day),
                Naming::Timestamps,
                Cleanup::KeepLogFiles(_)
            ))
        ));
    }

    #[test]
    fn only_by_date_appends() {
        assert!(appends(&options(Rotation::ByDate)));
        assert!(!appends(&options(Rotation::BySession)));
        assert!(!appends(&options(Rotation::BySize { max_bytes: 10 })));
    }
}
