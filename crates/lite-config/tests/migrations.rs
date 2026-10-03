//! 版本闸门与迁移链。

mod common;

use std::fs;
use std::time::Duration;

use common::{Migrating, Plain, fixture, read, temp_config};
use lite_config::{Config, Error, Schema};

const TIMEOUT: Duration = Duration::from_secs(5);

#[test]
fn every_schema_has_a_contiguous_migration_chain() {
    // schema 声明了 VERSION 却忘了写迁移步 —— 加载任何文件时都会硬失败，
    // 这里把它提前到测试里。
    for (version, steps) in [
        (Plain::VERSION, Plain::migrations().len()),
        (Migrating::VERSION, Migrating::migrations().len()),
    ] {
        assert_eq!(
            steps as u32 + 1,
            version,
            "v{version} 需要 {} 步迁移，实际 {steps} 步",
            version - 1
        );
    }
}

#[test]
fn v1_file_walks_the_whole_chain() {
    let (_dir, path) = fixture("v1.toml");
    let cfg = Config::<Migrating>::open(&path).expect("打开 v1");

    // 4096 MiB → 4 GiB；v2→v3 补上 harden。
    assert_eq!(cfg.read(|c| c.max_memory_gib), 4);
    assert!(cfg.read(|c| c.harden));
}

#[test]
fn migration_does_not_touch_the_file() {
    let (_dir, path) = fixture("v1.toml");
    let before = fs::read(&path).expect("读原件");
    let mtime_before = fs::metadata(&path)
        .expect("元数据")
        .modified()
        .expect("mtime");

    let cfg = Config::<Migrating>::open(&path).expect("打开");
    let _ = cfg;

    // 启动即改用户的文件是副作用；迁移结果留在内存里，等第一次真正改动再落盘。
    assert_eq!(fs::read(&path).expect("再读"), before, "迁移不该改写文件");
    assert_eq!(
        fs::metadata(&path)
            .expect("元数据")
            .modified()
            .expect("mtime"),
        mtime_before
    );
}

#[test]
fn a_later_mutation_writes_the_current_shape() {
    let (_dir, path) = fixture("v1.toml");
    let cfg = Config::<Migrating>::open(&path).expect("打开");

    cfg.mutate(|c| c.max_memory_gib = 12);
    cfg.flush(TIMEOUT).expect("落盘");

    let text = read(&path);
    assert!(text.starts_with("version = 3\n"), "{text}");
    assert!(text.contains("max_memory_gib = 12"), "{text}");
    assert!(!text.contains("max_memory_mb"), "旧键名不该留下：\n{text}");
    assert!(text.contains("harden = true"), "{text}");
}

#[test]
fn replace_if_only_fixes_the_wrong_default() {
    // 停在错的默认值上 → 被修正。
    let (_dir, path) = fixture("v2-default.toml");
    let cfg = Config::<Migrating>::open(&path).expect("打开");
    assert_eq!(cfg.read(|c| c.max_memory_gib), 15);
    assert!(cfg.read(|c| c.harden));

    // 用户自己调过 → 不动。
    let (_dir, path) = fixture("v2-custom.toml");
    let cfg = Config::<Migrating>::open(&path).expect("打开");
    assert_eq!(cfg.read(|c| c.max_memory_gib), 8);
    assert!(cfg.read(|c| c.harden));
}

#[test]
fn a_missing_version_key_counts_as_v1() {
    let (_dir, path) = fixture("no-version.toml");
    let cfg = Config::<Migrating>::open(&path).expect("打开");
    // 1024 MiB → 1 GiB。
    assert_eq!(cfg.read(|c| c.max_memory_gib), 1);
}

#[test]
fn a_newer_file_is_refused_and_left_alone() {
    let (_dir, path) = temp_config();
    fs::write(&path, "version = 999\nmax_memory_gib = 4\n").expect("写夹具");
    let before = fs::read(&path).expect("读原件");

    let error = Config::<Migrating>::open(&path).expect_err("版本过新必须拒绝");
    assert!(
        matches!(
            error,
            Error::VersionTooNew {
                found: 999,
                supported: 3,
                ..
            }
        ),
        "实际：{error}"
    );
    assert_eq!(
        fs::read(&path).expect("再读"),
        before,
        "拒绝加载不该改动文件"
    );

    // open_or_default_replace 也不能替换它：那是刻意的版本闸门，替换会永久吃掉
    // 新版本写的字段。
    let error = Config::<Migrating>::open_or_default_replace(&path).expect_err("同样拒绝");
    assert!(
        matches!(error, Error::VersionTooNew { .. }),
        "实际：{error}"
    );
    assert_eq!(fs::read(&path).expect("再读"), before);
}

#[test]
fn a_nonsense_version_is_corrupt() {
    for text in ["version = \"abc\"\n", "version = 0\n", "version = -1\n"] {
        let (_dir, path) = temp_config();
        fs::write(&path, text).expect("写夹具");
        let error = Config::<Migrating>::open(&path).expect_err("版本号无效必须报错");
        assert!(
            matches!(error, Error::BadVersion { .. }),
            "{text:?} 实际：{error}"
        );
    }
}

#[test]
fn a_broken_chain_is_reported_before_anything_else() {
    // 声明 v3 却一步迁移都不写：直接暴露成错误，而不是把老配置悄悄读成默认值。
    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Default)]
    #[serde(default)]
    struct Broken {
        value: u32,
    }
    impl Schema for Broken {
        const VERSION: u32 = 3;
    }

    let (_dir, path) = temp_config();
    fs::write(&path, "version = 1\nvalue = 4\n").expect("写夹具");

    let error = Config::<Broken>::open(&path).expect_err("迁移链断裂必须报错");
    assert!(
        matches!(
            error,
            Error::MigrationChainBroken {
                version: 3,
                steps: 0,
                expected: 2
            }
        ),
        "实际：{error}"
    );
}

#[test]
fn a_migration_that_meets_an_unexpected_shape_fails_loudly() {
    let (_dir, path) = temp_config();
    // `max_memory_mb` 是整数这个前提被破坏 —— 宁可硬失败，也不要 unwrap_or_default()
    // 把用户的配置静默重置。
    fs::write(&path, "version = 1\nmax_memory_mb = \"4 GiB\"\n").expect("写夹具");

    let error = Config::<Migrating>::open(&path).expect_err("形状不符必须报错");
    assert!(
        matches!(error, Error::Migrate { version: 1, .. }),
        "实际：{error}"
    );
}
