//! 三份配置文件的位置。
//!
//! 只回答「文件在哪」，不含优先级或合并语义——读哪份、谁覆盖谁由调用方决定。
//!
//! 路径统一用 `typed-path` 的平台路径类型处理：`normalize` 负责折叠 `.` / `..`，
//! `Utf8PlatformPath` 保留 Windows 的盘符 / 前缀语义，不再用 `std::path::Component`
//! 手写平台分支。非 UTF-8 路径直接报错，不做有损转换。

use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use typed_path::{Utf8PlatformPath, Utf8PlatformPathBuf};

use crate::error::Error;

/// 全局配置：`$XDG_CONFIG_HOME/neko-anywhere/config.toml`。
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
    let config_home = utf8_env(config_home, "XDG_CONFIG_HOME")?;
    let home = utf8_env(home, "HOME")?;

    let base = match config_home.map(Utf8PlatformPath::new) {
        // XDG 规范：`XDG_CONFIG_HOME` 是相对路径时无效，应当忽略。
        Some(dir) if dir.is_absolute() => dir.normalize(),
        _ => home
            .map(Utf8PlatformPath::new)
            .filter(|dir| dir.is_absolute())
            .ok_or(Error::NoConfigDir("HOME 未设置或不是绝对路径"))?
            .normalize()
            .join(".config"),
    };
    Ok(base.join("neko-anywhere").join("config.toml").into())
}

/// 环境变量值必须是 UTF-8：路径会作为实例 / 配置身份的一部分，不能做有损转换。
fn utf8_env<'a>(value: Option<&'a OsStr>, name: &'static str) -> Result<Option<&'a str>, Error> {
    value
        .map(|value| value.to_str().ok_or(Error::NonUtf8Path(name)))
        .transpose()
}

/// 游戏文件夹：包含 `.minecraft` 的那一层目录。
///
/// 身份就是它的绝对路径。实例名在不同游戏文件夹里会重名，所以实例必须带上游戏文件夹。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GameFolder(Arc<Utf8PlatformPathBuf>);

impl GameFolder {
    /// `root` 是包含 `.minecraft` 的那一层目录。
    ///
    /// 入参必须是 UTF-8；从 OS 路径构造用 [`GameFolder::from_path`]。
    pub fn new(root: impl AsRef<str>) -> Self {
        let path = Utf8PlatformPath::new(root.as_ref()).normalize();
        Self(Arc::new(path))
    }

    /// 从 OS 路径构造；非 UTF-8 路径返回错误，不做有损转换。
    pub fn from_path(root: &Path) -> Result<Self, Error> {
        let root = root.to_str().ok_or(Error::NonUtf8Path("游戏文件夹路径"))?;
        Ok(Self::new(root))
    }

    /// `<root>/.minecraft`。
    pub fn minecraft(&self) -> Utf8PlatformPathBuf {
        self.as_path().join(".minecraft")
    }

    /// `<root>/.minecraft/Neko-Anywhere/config.toml`。
    pub fn config_file(&self) -> Utf8PlatformPathBuf {
        self.minecraft().join("Neko-Anywhere").join("config.toml")
    }

    /// `<root>/.minecraft/versions/<name>`。
    ///
    /// 实例名必须可跨平台作为单个目录名：空名、`.` / `..`、路径分隔符与冒号都会被拒绝。
    pub fn instance_key(&self, name: &str) -> Option<InstanceKey> {
        InstanceKey::with_name(self.clone(), name)
    }

    /// 游戏文件夹的绝对路径。
    pub fn as_path(&self) -> &Utf8PlatformPath {
        self.0.as_path()
    }
}

/// 一个实例的路径身份。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InstanceKey {
    folder: GameFolder,
    name: Arc<str>,
}

impl InstanceKey {
    fn with_name(folder: GameFolder, name: &str) -> Option<Self> {
        // 拒绝目录段与两种平台的分隔符：路径身份必须能安全地拼进 versions/。
        let invalid = name.is_empty()
            || name == "."
            || name == ".."
            || name.chars().any(|c| matches!(c, '/' | '\\' | '\0' | ':'));
        if invalid {
            return None;
        }
        Some(Self {
            folder,
            name: Arc::from(name),
        })
    }

    /// 所属游戏文件夹。
    pub fn game_folder(&self) -> &GameFolder {
        &self.folder
    }

    /// 实例名（`versions/` 下的目录名）。
    pub fn name(&self) -> &str {
        &self.name
    }

    /// `<root>/.minecraft/versions/<name>`。
    pub fn version_dir(&self) -> Utf8PlatformPathBuf {
        // 实例名在构造时已校验，不含分隔符或绝对前缀，`join` 不会逃出 versions/。
        self.folder
            .minecraft()
            .join("versions")
            .join(self.name.as_ref())
    }

    /// 从 `<root>/.minecraft/versions/<name>` 反推实例身份。
    ///
    /// 路径先规范化；不是 UTF-8、不是绝对路径或不符合标准布局时返回 `None`。
    pub fn from_version_dir(path: &Path) -> Option<Self> {
        let path = path.to_str()?;
        let path = Utf8PlatformPath::new(path).normalize();
        if !path.is_absolute() {
            return None;
        }
        let name = path.file_name()?;
        let versions = path.parent()?;
        if versions.file_name()? != "versions" {
            return None;
        }
        let minecraft = versions.parent()?;
        if minecraft.file_name()? != ".minecraft" {
            return None;
        }
        let root = minecraft.parent()?;
        Self::with_name(GameFolder::new(root.as_str()), name)
    }

    /// `<root>/.minecraft/versions/<name>/Neko-Anywhere/config.toml`。
    pub fn config_file(&self) -> Utf8PlatformPathBuf {
        self.version_dir().join("Neko-Anywhere").join("config.toml")
    }
}
