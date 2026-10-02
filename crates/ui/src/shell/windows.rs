//! 窗口生命周期：父窗口关闭时，从它开出来的窗口一并关闭。
//!
//! 主窗口是应用根：关掉它之后，从它开出来的弹出窗口不再有意义，随它一起关；最后一个窗口关闭时
//! GPUI 退出进程，于是「关主窗口 = 退出应用」。这条规则只在这里实现一次：页面用 [`open_child`]
//! 开窗即可，不必各自订阅窗口关闭事件。
//!
//! 页面若要知道「我的弹出窗口还在不在」，用 `WindowHandle` 是否还能读来判断（见实例页的
//! `window::is_open`）；窗口关闭后这里会 `refresh_windows`，页面跟着重绘并自行清理记录。

use std::collections::HashMap;

use gpui_kit::*;

/// 父子窗口关系：父窗口 → 从它开出来的窗口。
#[derive(Default)]
struct ChildWindows(HashMap<WindowId, Vec<AnyWindowHandle>>);

impl Global for ChildWindows {}

/// 装上窗口生命周期规则；应用启动时调用一次。
pub(super) fn init(cx: &mut App) {
    cx.set_global(ChildWindows::default());
    cx.on_window_closed(|cx, closed| {
        // 关子窗口会再次触发这里，孙窗口也就跟着关掉了。
        let children = cx
            .global_mut::<ChildWindows>()
            .0
            .remove(&closed)
            .unwrap_or_default();
        for child in children {
            child.update(cx, |_, window, _| window.remove_window()).ok();
        }
        // 顺手把已关闭的窗口从父窗口的列表里去掉。
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
