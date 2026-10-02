//! 页面模块。每个分组一个模块，自己拥有选择栏与内容区。

pub mod download;
pub mod instance;
pub mod launch;
pub mod setup;
pub mod tools;

use gpui_kit::*;

use crate::components::{Selector, SelectorItem};
use crate::shell::Route;

/// 按路由目录生成选择栏：分组标题、条目、选中态与右侧动作按钮都取自 [`Route::selector_entries`]。
///
/// `on_action` 为空表示该条目暂未接入动作，按钮仅展示（与 .NET 版本当前的刷新 / 初始化按钮一致）。
pub(crate) fn route_selector(
    id: &'static str,
    current: Route,
    width: impl Into<Pixels>,
    on_navigate: impl Fn(&Route, &mut Window, &mut App) + 'static,
    on_action: impl Fn(&Route, &mut Window, &mut App) + 'static,
) -> Selector {
    let on_navigate = std::rc::Rc::new(on_navigate);
    let on_action = std::rc::Rc::new(on_action);
    let entries = current.selector_entries();
    let mut selector = Selector::new(id).width(width);

    for entry in entries {
        if let Some((section_key, top_margin)) = entry.section {
            selector = selector.section(entry_title(section_key), px(top_margin));
        }

        let route = entry.route;
        let mut item = SelectorItem::new(
            SharedString::from(format!("{id}-{}", entry.title_key)),
            entry_title(entry.title_key),
        )
        .icon(entry.icon)
        .selected(route == current)
        .on_click({
            let on_navigate = on_navigate.clone();
            move |_, window, cx| on_navigate(&route, window, cx)
        });

        if let Some(action) = entry.action {
            item = item.action(action.icon(), action.tooltip(), {
                let on_action = on_action.clone();
                move |_, window, cx| on_action(&route, window, cx)
            });
        }

        selector = selector.child(item);
    }

    selector
}

fn entry_title(key: &str) -> SharedString {
    crate::i18n::lang(key)
}
