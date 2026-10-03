//! 三份配置文件的路径推导。只回答"文件在哪"，不含优先级语义。

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use lite_config::{GameFolder, global_config_file_in};

#[test]
fn xdg_config_home_wins() {
    let path =
        global_config_file_in(Some(OsStr::new("/xdg")), Some(OsStr::new("/home/u"))).expect("解析");
    assert_eq!(path, PathBuf::from("/xdg/pcl-anywhere/config.toml"));
}

#[test]
fn xdg_falls_back_to_dot_config() {
    let path = global_config_file_in(None, Some(OsStr::new("/home/u"))).expect("解析");
    // macOS 上也走 `.config`：`directories::ProjectDirs` 在那边会返回
    // `~/Library/Application Support`，与本项目的路径约定不符，所以没采用它。
    assert_eq!(
        path,
        PathBuf::from("/home/u/.config/pcl-anywhere/config.toml")
    );
}

#[test]
fn a_relative_xdg_config_home_is_ignored() {
    // XDG 规范：相对路径无效。
    let path = global_config_file_in(
        Some(OsStr::new("relative/xdg")),
        Some(OsStr::new("/home/u")),
    )
    .expect("解析");
    assert_eq!(
        path,
        PathBuf::from("/home/u/.config/pcl-anywhere/config.toml")
    );
}

#[test]
fn a_missing_home_is_an_error_not_a_guess() {
    assert!(global_config_file_in(None, None).is_err());
    assert!(global_config_file_in(None, Some(OsStr::new("relative"))).is_err());
    assert!(global_config_file_in(Some(OsStr::new("relative")), None).is_err());
}

#[test]
fn game_folder_layout() {
    let folder = GameFolder::new("/games/主目录");
    assert_eq!(folder.as_path(), Path::new("/games/主目录"));
    assert_eq!(
        folder.minecraft(),
        PathBuf::from("/games/主目录/.minecraft")
    );
    assert_eq!(
        folder.config_file(),
        PathBuf::from("/games/主目录/.minecraft/PCL/config.toml")
    );
}

#[test]
fn instance_layout() {
    let instance = GameFolder::new("/games/a").instance("1.21");
    assert_eq!(
        instance.dir(),
        PathBuf::from("/games/a/.minecraft/version/1.21")
    );
    assert_eq!(
        instance.config_file(),
        PathBuf::from("/games/a/.minecraft/version/1.21/PCL/config.toml")
    );
    assert_eq!(instance.name(), "1.21");
    assert_eq!(instance.game_folder().as_path(), Path::new("/games/a"));
}

#[test]
fn paths_are_normalized_so_a_directory_has_one_spelling() {
    let folder = GameFolder::new("/games/b/../a/./");
    assert_eq!(folder.as_path(), Path::new("/games/a"));
    assert_eq!(folder, GameFolder::new("/games/a"));
}

#[test]
fn the_same_instance_name_in_two_folders_is_two_configs() {
    // 实例名会在不同游戏文件夹里重名，所以身份必须带上游戏文件夹。
    let left = GameFolder::new("/games/a").instance("1.21");
    let right = GameFolder::new("/games/b").instance("1.21");
    assert_ne!(left, right);
    assert_ne!(left.config_file(), right.config_file());
}
