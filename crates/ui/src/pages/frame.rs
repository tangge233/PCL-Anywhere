//! 分组页外壳：左栏（由目录表生成）+ 一条可拖动的分割栏 + 内容区。
//!
//! 条目只来自 [`PageRoute::ALL`]：标题、图标、分组标题一处定义，分组的 `Render` 只剩一行。
//! 条目不带动作按钮——页面的动作（例如下载页的刷新）落在内容区里。

use std::rc::Rc;

use gpui_kit::base::v_flex;
use gpui_kit::component::sidebar::{Sidebar, SidebarItem, SidebarMenuItem};
use gpui_kit::component::{ActiveTheme as _, Collapsible, h_resizable, resizable_panel};
use gpui_kit::*;

use crate::components::{PagePlaceholder, lucide, placeholder_text};
use crate::i18n;
use crate::shell::route::page::{PageHost, PageMeta, PageRoute};
use crate::shell::{GroupView, Navigate, Route};

use super::NAV_WIDTH;

/// 按路由分发的导航回调。
type NavHandler = Rc<dyn Fn(&Route, &mut Window, &mut App)>;

/// 渲染一个分组页：左栏来自 `R::ALL`，内容由分组按当前路由给出。
pub(crate) fn page_frame<R, G>(
    id: &'static str,
    route: R,
    group: &mut G,
    window: &mut Window,
    cx: &mut Context<G>,
) -> AnyElement
where
    R: PageRoute,
    G: GroupView + PageHost<R> + EventEmitter<Navigate>,
{
    let content = group.render_page(route, window, cx);
    let on_navigate: NavHandler =
        Rc::new(cx.listener(|_, route: &Route, _, cx| cx.emit(Navigate(*route))));
    let nav = nav_groups(id, R::ALL, route, &on_navigate);

    let mut sidebar = Sidebar::new(SharedString::from(format!("{id}-nav-sidebar")))
        .w(relative(1.))
        .collapsible(false)
        .collapsed(false);
    for group in nav {
        sidebar = sidebar.child(group);
    }

    h_resizable(SharedString::from(format!("{id}-nav")))
        .child(
            resizable_panel()
                .size(NAV_WIDTH)
                .size_range(super::NAV_SIZE_RANGE)
                .child(sidebar),
        )
        .child(resizable_panel().child(div().size_full().min_w_0().child(content)))
        .into_any_element()
}

/// 左栏的一组条目：目录登记了分组标题就另起一组，无标题的一组只有条目。
/// 一个 `Sidebar` 的条目类型必须唯一，分组与条目只能由本类型一并承载。
///
/// 不复用 gpui-component 的 `SidebarGroup`：它按组内下标给条目命名，而这里要的是
/// 「页面 + 路由」这种稳定标识（测试与可访问性都靠它）。
#[derive(Clone)]
struct NavGroup {
    title: Option<SharedString>,
    collapsed: bool,
    /// 条目与它的元素标识：标识取自路由（目录里唯一），不随组内顺序或文案变化。
    items: Vec<(SharedString, SidebarMenuItem)>,
}

impl Collapsible for NavGroup {
    fn is_collapsed(&self) -> bool {
        self.collapsed
    }

    fn collapsed(self, collapsed: bool) -> Self {
        Self {
            title: self.title,
            collapsed,
            items: self
                .items
                .into_iter()
                .map(|(id, item)| (id, item.collapsed(collapsed)))
                .collect(),
        }
    }
}

impl SidebarItem for NavGroup {
    fn render(
        self,
        id: impl Into<ElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> impl IntoElement {
        let mut column = v_flex().id(id.into()).gap_2();
        if let Some(title) = self.title {
            column = column.child(
                div()
                    .pt_1()
                    .text_xs()
                    .text_color(cx.theme().sidebar_foreground.opacity(0.6))
                    .child(title),
            );
        }
        column.children(
            self.items
                .into_iter()
                .map(|(id, item)| item.render(id, window, cx).into_any_element()),
        )
    }
}

/// 按目录表把条目切成组：`section` 出现即另起一组，组标题之后的条目归该组。
fn nav_groups<R: PageRoute>(
    id: &'static str,
    all: &'static [PageMeta<R>],
    current: R,
    on_navigate: &NavHandler,
) -> Vec<NavGroup> {
    let mut groups: Vec<NavGroup> = Vec::new();

    for meta in all {
        if meta.section.is_some() || groups.is_empty() {
            groups.push(NavGroup {
                title: meta.section.map(i18n::lang),
                collapsed: false,
                items: Vec::new(),
            });
        }

        // 标识用顶层路由（含分组名），不随分组内路由类型变化。
        let nav_id = SharedString::from(format!("{id}-nav-{:?}", meta.route.to_route()));
        let item = nav_item(meta, current, on_navigate);
        if let Some(group) = groups.last_mut() {
            group.items.push((nav_id, item));
        }
    }

    groups
}

/// 尚未迁移的页：按该页标题渲染占位，元素带稳定标识与可访问名称（测试靠它定位）。
pub(crate) fn page_placeholder<R: PageRoute>(route: R) -> AnyElement {
    let title = route.title();
    div()
        .id(SharedString::from(format!(
            "placeholder-{:?}",
            route.to_route()
        )))
        .test_support()
        .aria_label(placeholder_text(title.as_ref()))
        .size_full()
        .child(PagePlaceholder::for_route(title.as_ref()))
        .into_any_element()
}

fn nav_item<R: PageRoute>(
    meta: &PageMeta<R>,
    current: R,
    on_navigate: &NavHandler,
) -> SidebarMenuItem {
    let route = meta.route.to_route();
    SidebarMenuItem::new(meta.route.title())
        .icon(lucide(meta.icon))
        .active(meta.route == current)
        .on_click({
            let on_navigate = on_navigate.clone();
            move |_, window, cx| on_navigate(&route, window, cx)
        })
}
