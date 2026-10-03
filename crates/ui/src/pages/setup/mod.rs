//! 设置页分组（对应 PCL 的设置页视图、页面逻辑与导航分类表）。
//!
//! 11 个分类各对应 gpui-kit `Settings` 的一个 [`SettingPage`]，顺序与 `SetupRoute` 的
//! 声明顺序一致，因此外壳路由 `Route::Setup(..)` 可以直接映射为 `Settings` 的页面下标，
//! 侧栏（分类 + 搜索）等价于 PCL 左栏的设置分类列表；分类标题与图标由各页模块的
//! `SettingPage::new(..).icon(..)` 提供（文案键与图标名同路由目录
//! [`crate::shell::route::SETUP_ENTRIES`]，本组不读取该目录）。
//!
//! 界面状态全部留在本组内：受控值 + `on_change` 写状态 + `cx.notify()`，不落盘、不接业务。
//! 滑块（`SliderState`）与多行文本（`TextareaState`）的值由 gpui-kit 的实体承载，
//! 其余受控值集中在 [`state::SetupState`]（由页面里的字段闭包通过 `Rc` 共享持有）。
//! 无法从 PCL 直接取得的示例数据与默认值均逐处注明来源。

mod about;
mod fields;
mod game_link;
mod game_manage;
mod java;
mod language;
mod launch;
mod log;
mod misc;
mod state;
mod ui;

use std::cell::RefCell;
use std::rc::Rc;

use gpui_kit::component::setting::{SelectIndex, SettingGroup, SettingItem, SettingPage, Settings};
use gpui_kit::*;

use crate::components::{PagePlaceholder, lucide};

use crate::i18n;
use crate::shell::route::SetupRoute;
use crate::shell::{GroupView, Navigate, Route};

// 页面子模块通过 `super::{Fields, group, …}` 使用这些构建器（私有导入对后代模块可见）。
use fields::{Fields, NO_ENABLE, choices, group, item, slider_state, tip};
use state::SetupState;

/// 设置分组视图。
///
/// `pages` 在构造时构建一次（闭包持有 [`Fields`]，读写在 `SetupGroup` 的状态上），
/// 每帧克隆一份交给 `Settings`：`SettingPage`/`SettingGroup`/`SettingItem` 内部是 `Rc`，
/// 克隆只增加引用计数。
pub struct SetupGroup {
    route: Route,
    pages: Vec<SettingPage>,
}

impl SetupGroup {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        // `cx.weak_entity()` 在实体插入前就可用；闭包在实体存活后才会被调用。
        let weak = cx.weak_entity();
        let fields = Fields {
            state: Rc::new(RefCell::new(SetupState::new())),
            group: weak.clone(),
        };

        let pages = vec![
            launch::page(&fields, window, cx),
            java::page(&fields),
            game_manage::page(&fields, cx),
            game_link::page(&fields),
            ui::page(&fields, cx),
            language::page(&fields),
            misc::page(&fields, window, cx),
            about::page(&fields),
            placeholder_page(SetupRoute::Update),
            placeholder_page(SetupRoute::Feedback),
            log::page(&fields),
        ];

        Self {
            route: Route::Setup(SetupRoute::Launch),
            pages,
        }
    }
}

impl GroupView for SetupGroup {
    fn set_route(&mut self, route: Route, _: &mut Window, cx: &mut Context<Self>) {
        self.route = route;
        cx.notify();
    }
}

impl EventEmitter<Navigate> for SetupGroup {}

impl Render for SetupGroup {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let page_ix = page_index(self.route);
        div().size_full().child(
            Settings::new(SharedString::from(format!("setup-{page_ix}")))
                .pages(self.pages.clone())
                // 侧栏选中项跟随外壳路由；换页即换一个 keyed state，与 PCL 的「进入某分类」一致。
                .default_selected_index(SelectIndex {
                    page_ix,
                    group_ix: None,
                }),
        )
    }
}

/// `Route` 对应的 `Settings` 页面下标。
///
/// 非设置页（外壳只会把设置页路由交给本组）落到启动页。
fn page_index(route: Route) -> usize {
    match route {
        Route::Setup(route) => match route {
            SetupRoute::Launch => 0,
            SetupRoute::Java => 1,
            SetupRoute::GameManage => 2,
            SetupRoute::GameLink => 3,
            SetupRoute::Ui => 4,
            SetupRoute::Language => 5,
            SetupRoute::Misc => 6,
            SetupRoute::About => 7,
            SetupRoute::Update => 8,
            SetupRoute::Feedback => 9,
            SetupRoute::Log => 10,
        },
        _ => 0,
    }
}

/// 尚未实现的页面（`SetupRoute::Update` / `SetupRoute::Feedback`）：路由目录里登记了分类，
/// 但 PCL 没有对应的设置页，因此只给出占位。
fn placeholder_page(route: SetupRoute) -> SettingPage {
    let (label, icon) = match route {
        SetupRoute::Update => (i18n::lang("Setup.Left.Item.Update"), "refresh-cw"),
        _ => (i18n::lang("Setup.Left.Item.Feedback"), "message-circle"),
    };
    // `SettingItem::Element` 的搜索只看 keywords（标题不参与），关键词必须与本页标签一致。
    let keyword = label.clone();
    let title = label.clone();
    SettingPage::new(title).icon(lucide(icon)).group(
        SettingGroup::new().item(
            SettingItem::render(move |_, _, _| {
                div()
                    .h_80()
                    .child(PagePlaceholder::for_route(label.as_ref()))
            })
            .keywords([keyword]),
        ),
    )
}
