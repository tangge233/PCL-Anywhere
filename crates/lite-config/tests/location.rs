//! 三份配置文件的路径推导。只回答"文件在哪"，不含优先级语义。

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use lite_config::{
    GameFolder, InstanceKey, Utf8PlatformPath, Utf8PlatformPathBuf, global_config_file_in,
};

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
    assert_eq!(folder.as_path(), Utf8PlatformPath::new("/games/主目录"));
    assert_eq!(
        folder.minecraft(),
        Utf8PlatformPathBuf::from("/games/主目录/.minecraft")
    );
    assert_eq!(
        folder.config_file(),
        Utf8PlatformPathBuf::from("/games/主目录/.minecraft/PCL/config.toml")
    );
}

#[test]
fn instance_layout() {
    let instance = GameFolder::new("/games/a")
        .instance_key("1.21")
        .expect("合法实例名");
    assert_eq!(
        instance.version_dir(),
        Utf8PlatformPathBuf::from("/games/a/.minecraft/versions/1.21")
    );
    assert_eq!(
        instance.config_file(),
        Utf8PlatformPathBuf::from("/games/a/.minecraft/versions/1.21/PCL/config.toml")
    );
    assert_eq!(instance.name(), "1.21");
    assert_eq!(
        instance.game_folder().as_path(),
        Utf8PlatformPath::new("/games/a")
    );
}

#[test]
fn instance_names_cannot_escape_versions_dir() {
    let folder = GameFolder::new("/games/a");
    for name in ["", ".", "..", "a/b", "a\\b", "C:evil"] {
        assert!(
            folder.instance_key(name).is_none(),
            "{name:?} 不应被接受为实例名"
        );
    }
}

#[cfg(unix)]
#[test]
fn non_utf8_env_paths_are_rejected() {
    use std::os::unix::ffi::OsStrExt;

    let bad = OsStr::from_bytes(b"/tmp/\xff");
    assert!(matches!(
        global_config_file_in(Some(bad), Some(OsStr::new("/home/u"))),
        Err(lite_config::Error::NonUtf8Path("XDG_CONFIG_HOME"))
    ));
}

#[test]
fn instance_key_round_trips_through_version_dir() {
    let key = GameFolder::new("/games/a")
        .instance_key("1.21")
        .expect("合法实例名");
    assert_eq!(
        InstanceKey::from_version_dir(key.version_dir().as_ref()),
        Some(key)
    );
}

#[test]
fn from_version_dir_normalizes_and_checks_the_layout() {
    let key = InstanceKey::from_version_dir(Path::new(
        "/games/a/.minecraft/versions/../versions/./1.21/",
    ))
    .expect("标准布局应当可以解析");
    assert_eq!(
        key,
        GameFolder::new("/games/a")
            .instance_key("1.21")
            .expect("合法实例名")
    );

    // 单数 version/、相对路径、缺少实例名都不是标准布局。
    assert!(InstanceKey::from_version_dir(Path::new("/games/a/.minecraft/version/1.21")).is_none());
    assert!(
        InstanceKey::from_version_dir(Path::new("relative/.minecraft/versions/1.21")).is_none()
    );
    assert!(InstanceKey::from_version_dir(Path::new("/games/a/.minecraft/versions")).is_none());
}

#[test]
fn paths_are_normalized_so_a_directory_has_one_spelling() {
    let folder = GameFolder::new("/games/b/../a/./");
    assert_eq!(folder.as_path(), Utf8PlatformPath::new("/games/a"));
    assert_eq!(folder, GameFolder::new("/games/a"));
}

#[test]
fn the_same_instance_name_in_two_folders_is_two_configs() {
    // 实例名会在不同游戏文件夹里重名，所以身份必须带上游戏文件夹。
    let left = GameFolder::new("/games/a")
        .instance_key("1.21")
        .expect("合法实例名");
    let right = GameFolder::new("/games/b")
        .instance_key("1.21")
        .expect("合法实例名");
    assert_ne!(left, right);
    assert_ne!(left.config_file(), right.config_file());
}
