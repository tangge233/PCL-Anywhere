//! 设置页分组（对应 PCL 的设置页视图与导航分类表）。
//!
//! 11 个分类登记在 `crate::shell::route::setup` 的 `setting_pages!` 表里：顺序即 gpui-kit
//! `Settings` 的页序（下标由 `PageRoute::index` 给出），标题与图标也取自那张表，
//! 各页构造器只提供内容分组。本组持有界面状态（[`Fields`]）并把它交给 `Settings` 渲染。
//!
//! 受控值 + `on_change` 写状态 + `cx.notify()`，不落盘、不接业务；滑块（`SliderState`）与
//! 多行文本（`TextareaState`）由 gpui-kit 的实体承载，其余集中在 [`state::SetupState`]。
//!
//! 已知缺口（受 gpui-component 限制，不是本组能解决的）：
//! - `Settings` 没有选择回调，侧栏里换分类不会回到外壳路由，因此 `Route::Setup(Java)`
//!   这类路由目前只有初值可达；
//! - 分类标题（游戏 / 工具 / 启动器 / 关于）与侧栏动作按钮渲染不出来，表里登记的
//!   `section` 暂时只是数据。
//!
//! 无法从 PCL 直接取得的示例数据与默认值均逐处注明来源。

pub(crate) mod about;
mod fields;
pub(crate) mod game_link;
pub(crate) mod game_manage;
pub(crate) mod java;
pub(crate) mod language;
pub(crate) mod launch;
pub(crate) mod log;
pub(crate) mod misc;
pub(crate) mod placeholder;
mod state;
pub(crate) mod ui;

use std::cell::RefCell;
use std::rc::Rc;

use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::setting::{SelectIndex, SettingPage, Settings};
use gpui_kit::*;

// 各页构造器通过 `super::{Fields, group, …}` 取这些构建器（私有导入对后代模块可见）。
use fields::{Fields, NO_ENABLE, choices, group, item, slider_state, tip};
use state::SetupState;

use crate::shell::route::SetupRoute;
use crate::shell::route::page::PageRoute;
use crate::shell::{GroupView, Navigate, Route};

/// 设置分组视图。
///
/// 页面（`SettingPage`）在第一次渲染时按目录表建一次并缓存——`SettingPage` 内部是 `Rc`，
/// 每帧克隆只增加引用计数；各页的受控值由闭包在渲染时读状态，缓存不会读到旧值。
pub struct SetupGroup {
    /// 当前分类（外壳广播的子路由）。
    route: SetupRoute,
    /// 各设置页共用的受控值入口（页面构造器按 `&mut SetupGroup` 取用）。
    fields: Fields,
    /// `build_setting_pages` 的结果，建一次后复用。
    pages: Option<Vec<SettingPage>>,
}

impl SetupGroup {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        // `cx.weak_entity()` 在实体插入前就可用；闭包在实体存活后才会被调用。
        let weak = cx.weak_entity();
        Self {
            route: SetupRoute::Launch,
            fields: Fields {
                state: Rc::new(RefCell::new(SetupState::new())),
                group: weak,
            },
            pages: None,
        }
    }
}

impl GroupView for SetupGroup {
    fn set_route(&mut self, route: Route, _: &mut Window, cx: &mut Context<Self>) {
        if let Route::Setup(route) = route {
            self.route = route;
        }
        cx.notify();
    }
}

impl EventEmitter<Navigate> for SetupGroup {}

impl Render for SetupGroup {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let page_ix = self.route.index();
        if self.pages.is_none() {
            let pages = self.build_setting_pages(window, cx);
            self.pages = Some(pages);
        }
        let pages = self.pages.clone().unwrap_or_default();
        // `Settings` 把侧栏右描边走 0（只剩可拖动把手的发丝线），这里补回 1px：
        // 与下载 / 工具页左栏是同一条分割线。
        let sidebar_style = StyleRefinement::default()
            .border_r_1()
            .border_color(cx.theme().sidebar_border);

        div().size_full().child(
            Settings::new(SharedString::from(format!("setup-{page_ix}")))
                .pages(pages)
                // `Settings` 的侧栏默认 250 宽，要显式跟上另外两页。
                .sidebar_width(super::NAV_WIDTH)
                .sidebar_size_range(super::NAV_SIZE_RANGE)
                .sidebar_style(&sidebar_style)
                // 侧栏选中项跟随外壳路由；id 带下标，换页即换一份 keyed state。
                .default_selected_index(SelectIndex {
                    page_ix,
                    group_ix: None,
                }),
        )
    }
}
