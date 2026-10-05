//! 日志目录的位置。
//!
//! 只计算路径，不创建目录、不删除文件。与 lite-config 一样以 `$XDG_CONFIG_HOME` 为基准，
//! 但不依赖 lite-config：两边各自计算。

use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crate::error::Error;

/// 日志目录：`$XDG_CONFIG_HOME/pcl-anywhere/logs`，默认 `~/.config/pcl-anywhere/logs`。
pub fn log_dir() -> Result<PathBuf, Error> {
    log_dir_in(
        env::var_os("XDG_CONFIG_HOME").as_deref(),
        env::var_os("HOME").as_deref(),
    )
}

/// [`log_dir`] 的纯函数版本，供测试使用：环境变量是进程全局状态。
pub fn log_dir_in(config_home: Option<&OsStr>, home: Option<&OsStr>) -> Result<PathBuf, Error> {
    let base = match config_home.map(Path::new) {
        // XDG 规范：`XDG_CONFIG_HOME` 是相对路径时无效，应当忽略。
        Some(dir) if dir.is_absolute() => dir.to_path_buf(),
        _ => home
            .map(Path::new)
            .filter(|dir| dir.is_absolute())
            .ok_or(Error::NoLogDir("HOME 未设置或不是绝对路径"))?
            .join(".config"),
    };
    Ok(base.join("pcl-anywhere").join("logs"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_home_wins() {
        let dir = log_dir_in(Some(OsStr::new("/xdg")), Some(OsStr::new("/home/u"))).unwrap();
        assert_eq!(dir, Path::new("/xdg/pcl-anywhere/logs"));
    }

    #[test]
    fn falls_back_to_home() {
        let dir = log_dir_in(None, Some(OsStr::new("/home/u"))).unwrap();
        assert_eq!(dir, Path::new("/home/u/.config/pcl-anywhere/logs"));
    }

    #[test]
    fn relative_config_home_is_ignored() {
        let dir = log_dir_in(Some(OsStr::new("relative")), Some(OsStr::new("/home/u"))).unwrap();
        assert_eq!(dir, Path::new("/home/u/.config/pcl-anywhere/logs"));
    }

    #[test]
    fn no_home_is_an_error() {
        assert!(matches!(log_dir_in(None, None), Err(Error::NoLogDir(_))));
    }
}
