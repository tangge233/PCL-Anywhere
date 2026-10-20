//! Job 的提交、查询与取消。

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use dashmap::DashMap;
use tokio::runtime::Runtime;

use crate::executor;
use crate::job::{Job, JobId};
use crate::state::JobStatus;

/// 持有全局 Tokio runtime 的 Job 管理器。
///
/// 每个 Job 只 spawn 一个 driver task，work 的并发由 driver 内部的 `JoinSet` 展开。
/// 所有方法都不阻塞，可以直接在 UI 线程调用。
pub struct JobWorker {
    runtime: Arc<Runtime>,
    jobs: DashMap<JobId, Arc<Job>>,
    next_id: AtomicU64,
}

impl JobWorker {
    /// 复用调用方的 runtime；JobWorker 自己不建线程池。
    pub fn new(runtime: Arc<Runtime>) -> Self {
        Self {
            runtime,
            jobs: DashMap::new(),
            next_id: AtomicU64::new(0),
        }
    }

    /// 提交并立即开始调度，返回后续查询用的编号。
    pub fn submit(&self, job: Job) -> JobId {
        let id = JobId(self.next_id.fetch_add(1, Ordering::Relaxed));
        let job = Arc::new(job);
        self.jobs.insert(id, Arc::clone(&job));
        self.runtime.spawn(executor::drive(job));
        id
    }

    /// 取 Job 的共享句柄，可以反复读实时状态。
    pub fn job(&self, id: JobId) -> Option<Arc<Job>> {
        self.jobs.get(&id).map(|entry| Arc::clone(entry.value()))
    }

    /// 只关心状态时的快捷方式。
    pub fn status(&self, id: JobId) -> Option<JobStatus> {
        self.job(id).map(|job| job.status())
    }

    /// 请求取消：运行中的 work 收到信号，未启动的 work 不再启动。
    ///
    /// 需要 work 主动响应（见 [`crate::WorkContext::cancelled`]），调度器不会 abort。
    pub fn cancel(&self, id: JobId) {
        if let Some(job) = self.job(id) {
            job.cancel();
        }
    }

    /// 从注册表移除；只影响查询，不影响已经在跑的 driver。
    pub fn remove(&self, id: JobId) -> Option<Arc<Job>> {
        self.jobs.remove(&id).map(|(_, job)| job)
    }
}
