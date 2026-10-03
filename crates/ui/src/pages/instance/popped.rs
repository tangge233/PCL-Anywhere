//! 弹出窗口登记表：哪些实例的管理栏在独立窗口里，各自停在哪个 Tab。
//!
//! 每个实例至多一个窗口。开窗分两步登记（先占位、取得句柄后补记），这样新窗口第一帧就能读到
//! 自己的 Tab，主窗口也能立即显示提示页；开窗失败则撤销登记。

use std::collections::HashMap;

use gpui_kit::*;

use super::data::InstanceId;
use super::window;

/// 一个已弹出的实例。
struct PoppedPage {
    /// 窗口句柄；开窗尚未返回时为 `None`。
    window: Option<AnyWindowHandle>,
    tab: usize,
}

#[derive(Default)]
pub(super) struct PoppedPages(HashMap<InstanceId, PoppedPage>);

impl PoppedPages {
    /// 开始弹出：先占位登记 Tab，窗口句柄待开窗返回后补记。
    pub(super) fn begin(&mut self, id: InstanceId, tab: usize) {
        self.0.insert(id, PoppedPage { window: None, tab });
    }

    /// 开窗完成，补记窗口句柄。
    pub(super) fn finish(&mut self, id: &InstanceId, window: AnyWindowHandle) {
        if let Some(page) = self.0.get_mut(id) {
            page.window = Some(window);
        }
    }

    /// 某个实例是否已弹出（含正在开窗）。窗口已被关掉的记录在这里清掉。
    pub(super) fn is_popped(&mut self, id: &InstanceId, cx: &App) -> bool {
        match self.0.get(id) {
            Some(page) => match page.window {
                Some(handle) if !window::is_open(handle, cx) => {
                    self.0.remove(id);
                    false
                }
                _ => true,
            },
            None => false,
        }
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
}
