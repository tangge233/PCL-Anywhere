//! 窗口生命周期：父窗口关闭时，由它创建的子窗口一并关闭。
//!
//! 主窗口是应用根：关闭主窗口后，由它弹出的子窗口不再有意义，随之关闭；最后一个窗口关闭时
//! GPUI 退出进程，于是「关主窗口 = 退出应用」。这条规则只在这里实现一次：页面用 [`open_child`]
//! 创建窗口即可，不必各自订阅窗口关闭事件。
//!
//! 页面若要判断弹出窗口是否仍存在，用 `WindowHandle` 是否还能读来判断（见实例页的
//! `window::is_open`）；窗口关闭后这里会 `refresh_windows`，页面跟着重绘并自行清理记录。

use std::collections::HashMap;

use gpui_kit::*;

/// 父子窗口关系：父窗口 → 从它创建的子窗口。
#[derive(Default)]
struct ChildWindows(HashMap<WindowId, Vec<AnyWindowHandle>>);

impl Global for ChildWindows {}

/// 装上窗口生命周期规则；应用启动时调用一次。
pub(super) fn init(cx: &mut App) {
    cx.set_global(ChildWindows::default());
    cx.on_window_closed(|cx, closed| {
        // 关闭子窗口会再次触发本回调，孙窗口随之关闭。
        let children = cx
            .global_mut::<ChildWindows>()
            .0
            .remove(&closed)
            .unwrap_or_default();
        for child in children {
            child.update(cx, |_, window, _| window.remove_window()).ok();
        }
        // 同时从其余父窗口的子窗口列表中移除该已关闭窗口。
        for children in cx.global_mut::<ChildWindows>().0.values_mut() {
            children.retain(|child| child.window_id() != closed);
        }
        cx.refresh_windows();
    })
    .detach();
}

/// 打开一个跟随 `parent` 生命周期的子窗口。
pub(crate) fn open_child<V: Render>(
    parent: AnyWindowHandle,
    options: WindowOptions,
    cx: &mut App,
    build: impl FnOnce(&mut Window, &mut App) -> Entity<V>,
) -> Result<AnyWindowHandle> {
    let (handle, _) = gpui_kit::open_window(options, cx, build)?;
    cx.global_mut::<ChildWindows>()
        .0
        .entry(parent.window_id())
        .or_default()
        .push(handle);
    Ok(handle)
}
