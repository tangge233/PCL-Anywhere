//! 管理栏的视图与视图状态：主窗口一份，每个独立窗口各一份。

use super::data::InstanceId;

/// 管理栏的视图：主窗口的三栏布局，或某个实例的独立窗口。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ManageView {
    /// 主窗口里的管理栏。
    Main,
    /// 某个实例的独立窗口。
    Popped(InstanceId),
}

/// 一个视图的界面状态：看哪个实例、停在哪个 Tab。
#[derive(Clone, Debug, Default)]
pub(super) struct ManageViewState {
    pub(super) instance: Option<InstanceId>,
    pub(super) tab: usize,
}
