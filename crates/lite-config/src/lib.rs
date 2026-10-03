//! 带版本迁移的配置文件读写。
//!
//! 一个 [`Config`] 管一份文件。格式由 [`Schema`] 定义，读取时按 `version` 键逐级迁移到
//! 当前版本。
//!
//! # 保证
//!
//! * **崩溃安全**：进程被杀、磁盘写满、序列化失败——文件要么是完整的旧内容，要么是完整的
//!   新内容，不会出现半截文件。
//! * **读无锁**：[`Config::snapshot`] 与 [`Config::read`] 不取锁、不做 IO。
//! * **写有界**：进程内一把 `Mutex`，只护「克隆 → 改 → 发布快照」，不跨 IO。
//! * **不静默丢数据**：内容坏掉或版本过新时报错，不会被默认值静默覆盖。
//!
//! # 不保证
//!
//! * 断电可能丢掉最后一次写入（`rename` 已原子，但未做 `F_FULLFSYNC`）。
//! * 跨进程无互斥：两个进程各写整份文件，结果永远是其中之一的完整状态，后写者覆盖先写者。
//! * 跨文件无原子性：改两份配置是两次独立提交。
//! * 写入是整份重写，文件里手写的注释与未知键会被丢弃。
//!
//! # 用法
//!
//! ```
//! use lite_config::{Config, Schema};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
//! #[serde(default)]
//! struct MyConfig {
//!     memory_gib: u32,
//! }
//!
//! impl Schema for MyConfig {
//!     const VERSION: u32 = 1;
//! }
//!
//! # fn main() -> Result<(), lite_config::Error> {
//! let dir = std::env::temp_dir().join("lite-config-doc-example");
//! let path = dir.join("config.toml");
//! let _ = std::fs::remove_dir_all(&dir);
//!
//! let cfg = Config::<MyConfig>::open_or_default_replace(&path)?;
//! assert_eq!(cfg.read(|c| c.memory_gib), 0);
//!
//! cfg.mutate(|c| c.memory_gib = 8);
//! cfg.flush(std::time::Duration::from_secs(2))?;
//!
//! let text = std::fs::read_to_string(&path).unwrap();
//! assert!(text.contains("version = 1"));
//! # let _ = std::fs::remove_dir_all(&dir);
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]

mod config;
mod doc;
mod error;
mod io;
mod location;
mod schema;
mod writer;

pub use config::Config;
pub use doc::DocAccess;
pub use error::{Error, MigrateError};
pub use location::{GameFolder, InstanceKey, global_config_file, global_config_file_in};
pub use schema::{Migration, Schema};

// 迁移步直接操作 `DocumentMut`，把这些类型转出，调用方不必自己依赖 toml_edit。
pub use toml_edit::{DocumentMut, Item, Value, value};

/// 构造数组节点，等价于 `toml_edit::value(toml_edit::Array::from_iter(values))`。
///
/// ```
/// use lite_config::{DocAccess, DocumentMut, arr};
///
/// let mut doc = DocumentMut::new();
/// doc.set_path(&["ui", "hidden"], arr(["page_download", "setup_launch"]))
///     .unwrap();
/// assert_eq!(doc["ui"]["hidden"].to_string(), r#"["page_download", "setup_launch"]"#);
/// ```
pub fn arr(values: impl IntoIterator<Item = impl Into<Value>>) -> Item {
    Item::Value(toml_edit::Array::from_iter(values).into())
}
