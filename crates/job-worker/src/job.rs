//! Job 的组装与只读视图。
//!
//! [`JobBuilder`] 收集 work 与依赖，[`JobBuilder::build`] 按依赖深度切层：
//! `depth[i] = 1 + max(depth[deps])`，同深度即同一层。执行器只认这张层表。

use std::sync::Arc;

use arc_swap::ArcSwap;
use tokio_util::sync::CancellationToken;

use crate::data::SharedData;
use crate::state::{JobStatus, Progress, WorkState, WorkStatus};
use crate::work::Work;

/// work 在 Job 内的编号，由 [`JobBuilder::add`] 分配，只在所属 Job 内有效。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WorkId(usize);

impl WorkId {
    /// 在 Job 的 work 列表中的下标。
    pub fn index(self) -> usize {
        self.0
    }
}

/// Job 的编号，由 [`crate::JobWorker::submit`] 分配。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct JobId(pub(crate) u64);

impl JobId {
    /// 原始编号，写日志时用。
    pub fn get(self) -> u64 {
        self.0
    }
}

/// 组装 [`Job`]：登记 work、声明依赖、预置共享数据。
pub struct JobBuilder {
    label: String,
    works: Vec<PendingWork>,
    shared: SharedData,
}

struct PendingWork {
    work: Arc<dyn Work>,
    deps: Vec<WorkId>,
}

impl JobBuilder {
    /// 用展示名创建 builder。
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            works: Vec::new(),
            shared: SharedData::default(),
        }
    }

    /// 登记一个 work 并返回编号，后续 work 用它声明依赖。
    ///
    /// 依赖只能指向已登记的 work，因此依赖图天然无环，不需要环检测。
    pub fn add<W: Work>(&mut self, work: W, deps: &[WorkId]) -> WorkId {
        let id = WorkId(self.works.len());
        self.works.push(PendingWork {
            work: Arc::new(work),
            deps: deps.to_vec(),
        });
        id
    }

    /// 预置共享数据，等价于在第一个 work 里 insert。
    pub fn insert<T: Send + Sync + 'static>(&mut self, value: T) {
        self.shared.insert(value);
    }

    /// 计算分层并冻结成 Job。
    pub fn build(self) -> Job {
        let levels = levels(&self.works);
        Job {
            label: self.label,
            nodes: self.works.into_iter().map(WorkNode::from).collect(),
            levels,
            shared: self.shared,
            token: CancellationToken::new(),
            status: ArcSwap::from_pointee(JobStatus::Pending),
        }
    }
}

/// 按依赖深度分层。
fn levels(works: &[PendingWork]) -> Vec<Vec<WorkId>> {
    let mut depth = Vec::with_capacity(works.len());
    for (index, pending) in works.iter().enumerate() {
        let deepest = pending
            .deps
            .iter()
            .map(|dep| {
                // 编号只能由本 builder 分配，越界说明拿了别的 Job 的 WorkId。
                assert!(
                    dep.index() < index,
                    "work #{index} 依赖了尚未登记的 work #{}",
                    dep.index()
                );
                depth[dep.index()]
            })
            .max();

        depth.push(deepest.map_or(0, |depth| depth + 1));
    }

    let mut layers = vec![Vec::new(); depth.iter().max().map_or(0, |max| max + 1)];
    for (index, &layer) in depth.iter().enumerate() {
        layers[layer].push(WorkId(index));
    }
    layers
}

/// 一次工作流：静态分层图 + 实时状态。
///
/// 以 `Arc<Job>` 在调度器与外部之间共享。读状态的接口全部无锁，可以在 UI 线程
/// 按帧轮询。
pub struct Job {
    label: String,
    nodes: Vec<WorkNode>,
    levels: Vec<Vec<WorkId>>,
    shared: SharedData,
    token: CancellationToken,
    status: ArcSwap<JobStatus>,
}

struct WorkNode {
    work: Arc<dyn Work>,
    state: WorkState,
}

impl From<PendingWork> for WorkNode {
    fn from(pending: PendingWork) -> Self {
        Self {
            work: pending.work,
            state: WorkState::new(),
        }
    }
}

impl Job {
    /// 创建时给的展示名。
    pub fn label(&self) -> &str {
        &self.label
    }

    /// 当前状态。
    pub fn status(&self) -> JobStatus {
        (**self.status.load()).clone()
    }

    /// Job 级进度：已成功的 work 数 / 总 work 数。
    ///
    /// 失败、跳过、取消的 work 不计入完成数，所以失败时进度会停在真实的位置。
    pub fn progress(&self) -> (u64, u64) {
        let succeeded = self
            .nodes
            .iter()
            .filter(|node| matches!(node.state.status(), WorkStatus::Succeeded))
            .count();
        (succeeded as u64, self.nodes.len() as u64)
    }

    /// 单个 work 的快照，编号越界时返回 `None`。
    pub fn work(&self, id: WorkId) -> Option<WorkView> {
        self.nodes
            .get(id.index())
            .map(|node| WorkView::new(id, node))
    }

    /// 按登记顺序遍历所有 work 的快照。
    pub fn works(&self) -> impl Iterator<Item = WorkView> + '_ {
        self.nodes
            .iter()
            .enumerate()
            .map(|(index, node)| WorkView::new(WorkId(index), node))
    }

    pub(crate) fn token(&self) -> &CancellationToken {
        &self.token
    }

    pub(crate) fn is_cancelled(&self) -> bool {
        self.token.is_cancelled()
    }

    pub(crate) fn cancel(&self) {
        self.token.cancel();
    }

    pub(crate) fn shared(&self) -> &SharedData {
        &self.shared
    }

    pub(crate) fn levels(&self) -> &[Vec<WorkId>] {
        &self.levels
    }

    pub(crate) fn work_of(&self, id: WorkId) -> Arc<dyn Work> {
        Arc::clone(&self.nodes[id.index()].work)
    }

    pub(crate) fn state_of(&self, id: WorkId) -> &WorkState {
        &self.nodes[id.index()].state
    }

    pub(crate) fn progress_of(&self, id: WorkId) -> &Progress {
        self.nodes[id.index()].state.progress()
    }

    pub(crate) fn set_status(&self, status: JobStatus) {
        self.status.store(Arc::new(status));
    }

    /// 把尚未启动的 work 一次收尾：失败后标 Skipped，取消后标 Cancelled。
    pub(crate) fn settle_pending(&self, status: WorkStatus) {
        for node in &self.nodes {
            if matches!(node.state.status(), WorkStatus::Pending) {
                node.state.set_status(status.clone());
            }
        }
    }
}

/// 一个 work 的状态快照，供 UI 渲染。
#[derive(Clone, Debug)]
pub struct WorkView {
    id: WorkId,
    label: &'static str,
    status: WorkStatus,
    progress: (u64, u64),
}

impl WorkView {
    fn new(id: WorkId, node: &WorkNode) -> Self {
        Self {
            id,
            label: node.work.label(),
            status: node.state.status(),
            progress: node.state.progress().get(),
        }
    }

    /// work 编号。
    pub fn id(&self) -> WorkId {
        self.id
    }

    /// 展示名。
    pub fn label(&self) -> &'static str {
        self.label
    }

    /// 当前状态。
    pub fn status(&self) -> &WorkStatus {
        &self.status
    }

    /// `(current, total)`，单位由该 work 自己定义。
    pub fn progress(&self) -> (u64, u64) {
        self.progress
    }
}
