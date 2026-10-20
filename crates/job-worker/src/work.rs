//! work 抽象与执行上下文。

use std::sync::Arc;

use async_trait::async_trait;
use tokio_util::sync::CancellationToken;

use crate::data::SharedData;
use crate::job::{Job, WorkId};

/// work 的失败原因，错误文本会原样进入 [`crate::WorkStatus::Failed`]。
pub type WorkError = Box<dyn std::error::Error + Send + Sync>;

/// 一个可被调度的工作单元。
///
/// work 之间不互相引用：依赖在 [`crate::JobBuilder::add`] 时声明，需要传递的数据
/// 走 [`WorkContext::shared`]。
#[async_trait]
pub trait Work: Send + Sync + 'static {
    /// 展示名，默认取类型名；UI 需要更友好的文案时覆写。
    fn label(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    /// 执行体。返回 `Err` 会触发整个 Job 的 fail-fast 取消。
    async fn run(&self, ctx: WorkContext) -> Result<(), WorkError>;
}

/// 传给 work 的上下文：进度、取消与共享数据。
///
/// 内部只持有一个 `Arc<Job>`，克隆开销极小，可以跨 `await` 使用。
pub struct WorkContext {
    job: Arc<Job>,
    id: WorkId,
}

impl WorkContext {
    pub(crate) fn new(job: Arc<Job>, id: WorkId) -> Self {
        Self { job, id }
    }

    /// 本 work 在 Job 内的编号。
    pub fn id(&self) -> WorkId {
        self.id
    }

    /// 上报 `(current, total)`，单位由 work 自己决定：字节、文件数、百分比都行。
    pub fn report(&self, current: u64, total: u64) {
        self.job.progress_of(self.id).set(current, total);
    }

    /// 在当前值上累加，`total` 沿用上次 [`Self::report`] 的取值。
    pub fn advance(&self, delta: u64) {
        self.job.progress_of(self.id).advance(delta);
    }

    /// Job 级共享数据。分层调度保证上游写入对下游可见。
    pub fn shared(&self) -> &SharedData {
        self.job.shared()
    }

    /// 取消令牌；需要派生可独立取消的子任务时用 `child_token`。
    pub fn token(&self) -> &CancellationToken {
        self.job.token()
    }

    /// 是否已被取消。
    pub fn is_cancelled(&self) -> bool {
        self.job.is_cancelled()
    }

    /// 等到 Job 被取消，常配合 `tokio::select!` 打断长任务。
    pub async fn cancelled(&self) {
        self.job.token().cancelled().await
    }
}
