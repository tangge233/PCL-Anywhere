//! work / job 的运行状态：状态用 [`ArcSwap`] 交换，进度用原子累加。

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use arc_swap::ArcSwap;

/// 单个 work 的状态。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum WorkStatus {
    /// 还没轮到执行。
    #[default]
    Pending,
    Running,
    Succeeded,
    /// 执行失败，附带错误文本。
    Failed(String),
    /// 前置 work 失败或被取消，本 work 不会启动。
    Skipped,
    /// 执行途中被取消。
    Cancelled,
}

impl WorkStatus {
    /// 是否已到终态。
    pub fn is_finished(&self) -> bool {
        matches!(
            self,
            Self::Succeeded | Self::Failed(_) | Self::Skipped | Self::Cancelled
        )
    }
}

/// 整个 Job 的状态。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum JobStatus {
    #[default]
    Pending,
    Running,
    Succeeded,
    /// 某个 work 失败，附带它的错误文本。
    Failed(String),
    Cancelled,
}

impl JobStatus {
    /// 是否已到终态。
    pub fn is_finished(&self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed(_) | Self::Cancelled)
    }
}

/// `(current, total)` 进度。
///
/// 两个原子分开读写，读到的不是严格一致的快照；进度只用于展示，`Relaxed` 足够。
#[derive(Default)]
pub(crate) struct Progress {
    current: AtomicU64,
    total: AtomicU64,
}

impl Progress {
    pub(crate) fn set(&self, current: u64, total: u64) {
        self.total.store(total, Ordering::Relaxed);
        self.current.store(current, Ordering::Relaxed);
    }

    pub(crate) fn advance(&self, delta: u64) {
        self.current.fetch_add(delta, Ordering::Relaxed);
    }

    pub(crate) fn get(&self) -> (u64, u64) {
        (
            self.current.load(Ordering::Relaxed),
            self.total.load(Ordering::Relaxed),
        )
    }
}

pub(crate) struct WorkState {
    status: ArcSwap<WorkStatus>,
    progress: Progress,
}

impl WorkState {
    pub(crate) fn new() -> Self {
        Self {
            status: ArcSwap::from_pointee(WorkStatus::Pending),
            progress: Progress::default(),
        }
    }

    pub(crate) fn status(&self) -> WorkStatus {
        (**self.status.load()).clone()
    }

    pub(crate) fn set_status(&self, status: WorkStatus) {
        self.status.store(Arc::new(status));
    }

    pub(crate) fn progress(&self) -> &Progress {
        &self.progress
    }
}
