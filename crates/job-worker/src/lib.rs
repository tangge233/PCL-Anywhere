//! 以拓扑分层驱动的任务工作流。
//!
//! 一个 [`Job`] 由若干 [`Work`] 组成，依赖在 [`JobBuilder::add`] 时声明。构建时按
//! 依赖深度切层：第 n 层只依赖前 n-1 层，于是层内全部并发、层间严格串行。层间用
//! `JoinSet` 收敛，天然带 happens-before——上游写进 [`SharedData`] 的值，下游读到
//! 的就是写入后的结果，不需要额外的同步。
//!
//! 每个 Job 在调用方提供的 runtime 上只占一个 driver task，work 的并发都由这个
//! task 内的 `JoinSet` 展开；对外接口全部不阻塞，可以直接在 UI 线程轮询
//! [`JobWorker::job`] 读取实时状态。
//!
//! # 失败与取消
//!
//! work 失败会立刻取消整个 Job：同层正在跑的 work 通过 [`WorkContext::cancelled`]
//! 拿到信号，尚未启动的 work 标为 [`WorkStatus::Skipped`]。work 必须主动响应取消
//! 才会被真正打断——调度器不会 abort 任何任务。
//!
//! # 进度
//!
//! work 通过 [`WorkContext::report`] 上报自己的 `(current, total)`，单位由 work
//! 决定；Job 级进度按已成功的 work 数统计，避免把不同单位的数字相加。
//!
//! # 例子
//!
//! ```
//! use std::sync::Arc;
//!
//! use async_trait::async_trait;
//! use na_job_worker::{JobBuilder, JobStatus, JobWorker, Work, WorkContext, WorkError};
//!
//! struct Resolve;
//!
//! #[async_trait]
//! impl Work for Resolve {
//!     fn label(&self) -> &'static str {
//!         "解析版本"
//!     }
//!
//!     async fn run(&self, ctx: WorkContext) -> Result<(), WorkError> {
//!         ctx.shared().insert(vec!["client.jar", "assets"]);
//!         Ok(())
//!     }
//! }
//!
//! struct Download;
//!
//! #[async_trait]
//! impl Work for Download {
//!     async fn run(&self, ctx: WorkContext) -> Result<(), WorkError> {
//!         let files = ctx.shared().get::<Vec<&'static str>>().expect("上游已完成");
//!         let total = files.len() as u64;
//!         for done in 1..=total {
//!             ctx.report(done, total);
//!         }
//!         Ok(())
//!     }
//! }
//!
//! let runtime = Arc::new(tokio::runtime::Runtime::new().unwrap());
//! let worker = JobWorker::new(Arc::clone(&runtime));
//!
//! let mut builder = JobBuilder::new("安装示例");
//! let resolve = builder.add(Resolve, &[]);
//! builder.add(Download, &[resolve]);
//!
//! let id = worker.submit(builder.build());
//! while !worker.status(id).unwrap().is_finished() {
//!     std::thread::sleep(std::time::Duration::from_millis(10));
//! }
//!
//! assert_eq!(worker.status(id), Some(JobStatus::Succeeded));
//! assert_eq!(worker.job(id).unwrap().progress(), (2, 2));
//! ```

mod data;
mod executor;
mod job;
mod state;
mod work;
mod worker;

pub use data::SharedData;
pub use job::{Job, JobBuilder, JobId, WorkId, WorkView};
pub use state::{JobStatus, WorkStatus};
pub use work::{Work, WorkContext, WorkError};
pub use worker::JobWorker;
