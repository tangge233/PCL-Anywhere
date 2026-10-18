//! 通用多源选择器：支持 P2C 与 Softmax SWRR。
//!
//! 使用方为每个上报结构 `T` 注入一个评分函数，选择器只维护分数和源集合：
//!
//! - `register` / `remove` 管理 String ID；
//! - `report(id, T)` 调用评分函数更新该源的分数；
//! - `select()` 使用配置里的默认策略，`select_with(strategy)` 可以显式指定。
//!
//! 一个选择器固定一种 `T`。不同连接类型需要各建一个选择器，或者把多种结果包成
//! 一个 `enum` 作为 `T`。
//!
//! # 评分
//!
//! 评分函数返回 `u8`（`0..=255`），**越小越好**。连接失败由调用方在评分函数里
//! 主动打高分，通常直接用 `255`；选择器本身不区分成功/失败，也不做熔断。
//!
//! 未上报过的源在选择时按最低分 0 处理，因此新源会先被探索。调用方应保证每次
//! 选择之后对返回的 ID 调用 `report`，否则它会一直保持 0 分。
//!
//! # 配置
//!
//! `Config` 同时保存两套策略的参数：
//!
//! - `default`：`select()` 使用的策略，默认 `Strategy::P2c`；
//! - `p2c.seed`：P2C 的随机种子；
//! - `swrr.temperature`：Softmax 温度；
//! - `ewma_alpha`：两种策略共享的分数平滑系数。
//!
//! # 用法
//!
//! ```
//! use source_selector::{Config, MultiSelector};
//! use std::time::Duration;
//!
//! struct HttpResult {
//!     bytes: u64,
//!     elapsed: Duration,
//!     status: u16,
//! }
//!
//! let selector = MultiSelector::<HttpResult>::new(Config::default(), |r| {
//!     let secs = r.elapsed.as_secs_f64().max(0.001);
//!     let mbps = r.bytes as f64 / secs / 1_000_000.0;
//!     let score = (100.0 / mbps.max(0.1)).min(255.0) as u8;
//!
//!     if (200..300).contains(&r.status) {
//!         score
//!     } else {
//!         255
//!     }
//! });
//!
//! selector.register("mirror-a").unwrap();
//! selector.register("mirror-b").unwrap();
//!
//! let id = selector.select().unwrap();
//! selector.report(&id, HttpResult {
//!     bytes: 1_000_000,
//!     elapsed: Duration::from_millis(80),
//!     status: 200,
//! });
//! ```
//!
//! # 并发
//!
//! P2C 读取无锁快照；SWRR 在内部串行化 current weight 的更新。`report` 只在查找
//! ID 时短暂持有读锁；`register` / `remove` 会重建一次快照。选择器本身可安全地在
//! 多线程间共享。

#![warn(missing_docs)]

mod config;
mod entry;
mod error;
mod rng;
mod selector;

pub use config::{Config, P2cConfig, Strategy, SwrrConfig};
pub use error::RegisterError;
pub use selector::MultiSelector;
