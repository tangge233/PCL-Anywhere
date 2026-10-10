//! 损坏内容的两种处理：报错，或者用全新默认值代替。

mod common;

use std::fs;
use std::path::Path;

use common::{Plain, fixture, read, temp_config};
use lite_config::{Config, Error};

/// 目录里以 `<config.toml>.corrupt-` 开头的备份文件。
fn backups(dir: &Path) -> Vec<String> {
    fs::read_dir(dir)
        .expect("列目录")
        .map(|entry| {
            entry
                .expect("目录项")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .filter(|name| name.starts_with("config.toml.corrupt-"))
        .collect()
}

#[test]
fn open_reports_a_torn_file() {
    let (_dir, path) = fixture("torn.toml");
    let before = fs::read(&path).expect("读原件");

    let error = Config::<Plain>::open(&path).expect_err("半截文件必须报错");
    assert!(matches!(error, Error::Parse { .. }), "实际：{error}");
    // 报错归报错，原文件一个字节都不该动。
    assert_eq!(fs::read(&path).expect("再读"), before);
}

#[test]
fn open_or_default_replace_rebuilds_a_torn_file() {
    let (dir, path) = fixture("torn.toml");
    let cfg = Config::<Plain>::open_or_default_replace(&path).expect("应当自愈");

    assert_eq!(cfg.read(|c| c.memory_gib), 15, "内容应为全新默认值");

    let text = read(&path);
    assert!(
        text.starts_with("version = 1\n"),
        "重建后是当前版本：\n{text}"
    );
    assert!(text.contains("memory_gib = 15"));

    // 原件被挪走而不是被销毁：一次手滑的编辑不该变成不可逆的数据丢失。
    let moved = backups(dir.path());
    assert_eq!(moved.len(), 1, "应当留下一个备份：{moved:?}");
    let kept = read(&dir.path().join(&moved[0]));
    assert!(kept.contains("max_memory_mb"), "备份里是原来那份：\n{kept}");
}

#[test]
fn open_or_default_replace_keeps_a_healthy_file_untouched() {
    let (_dir, path) = temp_config();
    {
        let cfg = Config::<Plain>::open(&path).expect("打开");
        cfg.mutate(|c| c.memory_gib = 6);
        cfg.persist(std::time::Duration::from_secs(5))
            .expect("落盘");
    }
    let before = fs::read(&path).expect("读原件");

    let cfg = Config::<Plain>::open_or_default_replace(&path).expect("打开");
    assert_eq!(cfg.read(|c| c.memory_gib), 6);
    assert_eq!(fs::read(&path).expect("再读"), before, "好文件不该被碰");
}

#[test]
fn rebuild_also_works_for_an_invalid_version_key() {
    let (dir, path) = temp_config();
    fs::write(&path, "version = \"第一版\"\n").expect("写夹具");

    let cfg = Config::<Plain>::open_or_default_replace(&path).expect("应当自愈");
    assert_eq!(cfg.read(|c| c.memory_gib), 15);
    assert_eq!(backups(dir.path()).len(), 1);
}

#[test]
fn rebuild_also_works_for_a_failed_migration() {
    let (dir, path) = temp_config();
    // `max_memory_mb` 不是整数，v1→v2 迁移会拒绝。
    fs::write(&path, "version = 1\nmax_memory_mb = [1, 2]\n").expect("写夹具");

    let cfg = Config::<common::Migrating>::open_or_default_replace(&path).expect("应当自愈");
    assert_eq!(cfg.read(|c| c.max_memory_gib), 15);
    assert!(!cfg.read(|c| c.harden));
    assert_eq!(backups(dir.path()).len(), 1);
}
