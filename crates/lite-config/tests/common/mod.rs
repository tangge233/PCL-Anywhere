//! 测试共用的 schema 与夹具。
//!
//! 这里定义的 schema 刻意不放进 crate：`lite-config` 只认 `PathBuf` + `Schema`，
//! 真正的全局/游戏文件夹/实例配置由应用层定义。

#![allow(dead_code)]

use std::path::{Path, PathBuf};

use lite_config::{DocAccess, MigrateError, Migration, Schema, value};
use serde::{Deserialize, Serialize};

/// 无历史包袱的 schema：用于往返、并发、损坏降级这些与版本无关的测试。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Plain {
    pub memory_gib: u32,
    /// 并发测试用它撑大文档，让"写到一半"有可观察的窗口。
    pub payload: Vec<u32>,
    pub theme: Theme,
}

impl Default for Plain {
    fn default() -> Self {
        Self {
            memory_gib: 15,
            payload: Vec::new(),
            theme: Theme::default(),
        }
    }
}

impl Schema for Plain {
    const VERSION: u32 = 1;
}

/// 嵌套结构 + 枚举 + `Option`：覆盖落盘的常见形状。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Theme {
    pub mode: ThemeMode,
    pub font: Option<String>,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            mode: ThemeMode::System,
            font: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ThemeMode {
    #[default]
    System,
    Light,
    Dark,
}

/// 三级版本、两步迁移的脚手架。
///
/// v1 的 `max_memory_mb`（MiB）→ v2 的 `max_memory_gib`（GiB）→ v3 引入 `harden`，
/// 并强制修正 v2 那个错的默认值 2 GiB。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Migrating {
    pub max_memory_gib: u32,
    pub harden: bool,
}

impl Default for Migrating {
    fn default() -> Self {
        Self {
            max_memory_gib: 15,
            harden: false,
        }
    }
}

impl Schema for Migrating {
    const VERSION: u32 = 3;

    fn migrations() -> &'static [Migration] {
        &[
            Migration {
                note: "max_memory_mb（MiB）改为 max_memory_gib（GiB）",
                apply: |doc| {
                    // 老文件常常只写了部分键，缺就缺——补默认值是 `#[serde(default)]` 的事。
                    let Some(item) = doc.get_path(&["max_memory_mb"]) else {
                        return Ok(());
                    };
                    let mib = item
                        .as_integer()
                        .ok_or(MigrateError::Shape("max_memory_mb 不是整数"))?;
                    doc.remove_path(&["max_memory_mb"]);
                    doc.set_path(&["max_memory_gib"], value(mib / 1024))?;
                    Ok(())
                },
            },
            Migration {
                note: "引入 harden；把 v2 那个错的默认值 2 GiB 修正为 15",
                apply: |doc| {
                    doc.set_path(&["harden"], value(true))?;
                    doc.replace_if(&["max_memory_gib"], value(2i64), value(15i64))?;
                    Ok(())
                },
            },
        ]
    }
}

/// 把夹具复制到临时目录里再交给测试——测试会改文件，不能动仓库里的原件。
pub fn fixture(name: &str) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("建临时目录");
    let path = dir.path().join("config.toml");
    std::fs::copy(source(name), &path).expect("复制夹具");
    (dir, path)
}

/// 夹具原件路径。
pub fn source(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

/// 创建一个空的临时配置文件路径（父目录已存在）。
pub fn temp_config() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("建临时目录");
    let path = dir.path().join("config.toml");
    (dir, path)
}

pub fn read(path: &Path) -> String {
    std::fs::read_to_string(path).expect("读配置")
}
