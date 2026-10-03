//! 三份配置文件的位置。
//!
//! 只回答「文件在哪」，不含优先级或合并语义——读哪份、谁覆盖谁由调用方决定。

use std::env;
use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use crate::error::Error;

/// 全局配置：`$XDG_CONFIG_HOME/pcl-anywhere/config.toml`。
pub fn global_config_file() -> Result<PathBuf, Error> {
    global_config_file_in(
        env::var_os("XDG_CONFIG_HOME").as_deref(),
        env::var_os("HOME").as_deref(),
    )
}

/// [`global_config_file`] 的纯函数版本，便于测试——环境变量是进程全局状态。
///
/// 两个平台一律遵循 XDG，macOS 上也用 `~/.config`。不用 `directories::ProjectDirs`：它在
/// macOS 上返回 `~/Library/Application Support`。
pub fn global_config_file_in(
    config_home: Option<&OsStr>,
    home: Option<&OsStr>,
) -> Result<PathBuf, Error> {
    let base = match config_home.map(Path::new) {
        // XDG 规范：`XDG_CONFIG_HOME` 是相对路径时无效，应当忽略。
        Some(dir) if dir.is_absolute() => dir.to_path_buf(),
        _ => home
            .map(Path::new)
            .filter(|dir| dir.is_absolute())
            .ok_or(Error::NoConfigDir("HOME 未设置或不是绝对路径"))?
            .join(".config"),
    };
    Ok(base.join("pcl-anywhere").join("config.toml"))
}

/// 游戏文件夹：包含 `.minecraft` 的那一层目录。
///
/// 身份就是它的绝对路径。实例名在不同游戏文件夹里会重名，所以实例必须带上游戏文件夹。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GameFolder(Arc<Path>);

impl GameFolder {
    /// `root` 是包含 `.minecraft` 的那一层目录。
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self(Arc::from(normalize(&root.into()).as_path()))
    }

    /// `<root>/.minecraft`。
    pub fn minecraft(&self) -> PathBuf {
        self.0.join(".minecraft")
    }

    /// `<root>/.minecraft/PCL/config.toml`。
    pub fn config_file(&self) -> PathBuf {
        self.minecraft().join("PCL").join("config.toml")
    }

    /// `<root>/.minecraft/version/<name>`。
    pub fn instance(&self, name: &str) -> InstanceKey {
        InstanceKey {
            folder: self.clone(),
            name: Arc::from(name),
        }
    }

    /// 游戏文件夹的绝对路径。
    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

/// 一个实例的路径身份。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InstanceKey {
    folder: GameFolder,
    name: Arc<str>,
}

impl InstanceKey {
    /// 所属游戏文件夹。
    pub fn game_folder(&self) -> &GameFolder {
        &self.folder
    }

    /// 实例名（`version/` 下的目录名）。
    pub fn name(&self) -> &str {
        &self.name
    }

    /// `<root>/.minecraft/version/<name>`。
    pub fn dir(&self) -> PathBuf {
        self.folder
            .minecraft()
            .join("version")
            .join(self.name.as_ref())
    }

    /// `<root>/.minecraft/version/<name>/PCL/config.toml`。
    pub fn config_file(&self) -> PathBuf {
        self.dir().join("PCL").join("config.toml")
    }
}

/// 合并多余分隔符、解析 `.` 与 `..`：路径是身份，写法必须唯一。
fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for part in path.components() {
        match part {
            Component::RootDir => out.push("/"),
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}
