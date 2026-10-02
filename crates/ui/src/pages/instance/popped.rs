//! 弹出窗口登记表：哪些实例的管理栏在独立窗口里，各自停在哪个 Tab。
//!
//! 每个实例至多一个窗口。开窗分两步登记（先占位、拿到句柄再补），这样新窗口第一帧就能读到
//! 自己的 Tab，主窗口也能立刻改显示提示页；开窗失败则撤销登记。

use std::collections::HashMap;

use gpui_kit::*;

use super::data::InstanceId;

/// 一个已弹出的实例。
struct PoppedPage {
    /// 窗口句柄；开窗尚未返回时为 `None`。
    window: Option<AnyWindowHandle>,
    tab: usize,
}

#[derive(Default)]
pub(super) struct PoppedPages(HashMap<InstanceId, PoppedPage>);

impl PoppedPages {
    /// 开始弹出：先占位登记 Tab，句柄等窗口开出来再补。
    pub(super) fn begin(&mut self, id: InstanceId, tab: usize) {
        self.0.insert(id, PoppedPage { window: None, tab });
    }

    /// 窗口开出来了，补上句柄。
    pub(super) fn finish(&mut self, id: &InstanceId, window: AnyWindowHandle) {
        if let Some(page) = self.0.get_mut(id) {
            page.window = Some(window);
        }
    }

    /// 某个实例是否已弹出（含正在开窗）。
    pub(super) fn is_popped(&self, id: &InstanceId) -> bool {
        self.0.contains_key(id)
    }

    /// 某个实例的窗口句柄。
    pub(super) fn window(&self, id: &InstanceId) -> Option<AnyWindowHandle> {
        self.0.get(id).and_then(|page| page.window)
    }

    /// 某个实例的窗口停在哪个 Tab。
    pub(super) fn tab(&self, id: &InstanceId) -> Option<usize> {
        self.0.get(id).map(|page| page.tab)
    }

    /// 切换某个实例窗口的 Tab。
    pub(super) fn set_tab(&mut self, id: &InstanceId, tab: usize) {
        if let Some(page) = self.0.get_mut(id) {
            page.tab = tab;
        }
    }

    /// 撤回登记，返回窗口句柄（调用方负责关窗）。
    pub(super) fn remove(&mut self, id: &InstanceId) -> Option<AnyWindowHandle> {
        self.0.remove(id).and_then(|page| page.window)
    }

    /// 按窗口句柄找到对应实例：窗口被系统关掉时用来清理。
    pub(super) fn take_by_window(&mut self, window_id: WindowId) -> Option<InstanceId> {
        let id = self
            .0
            .iter()
            .find(|(_, page)| {
                page.window
                    .is_some_and(|handle| handle.window_id() == window_id)
            })
            .map(|(id, _)| id.clone())?;
        self.0.remove(&id);
        Some(id)
    }
}
