//! 工具页分组（对应 PCL 的工具页）。
//!
//! 选择栏取自路由目录（联机 / 百宝箱 / UI 测试三项），内容区按 [`ToolsRoute`] 分发：
//! - 联机页还原「加入 / 创建 / 大厅信息」三段（PCL 的 `ToolsGameLink` 界面）；
//! - 测试页还原百宝箱的操作卡片（PCL 的 `ToolsTest` 界面）；
//! - UI 测试页是本项目自加的：把对话框的每条路径摆成按钮，作答写进页面记录。
//!
//! 这里只做界面与界面状态：加入 / 创建 / 退出只切换页面内的「是否在大厅」状态，
//! 不连接 `LobbyService`，也不执行清理缓存、人品、快捷方式等真实动作。
//!
//! 模块划分：本文件持有页面状态与路由分发，[`game_link`] 是联机页，[`test`] 是测试页，
//! [`ui_test`] 是 UI 测试页。

mod game_link;
mod test;
mod ui_test;

use gpui_kit::base::h_flex;
use gpui_kit::component::input::InputState;
use gpui_kit::*;

use crate::i18n;
use crate::shell::{GroupView, Navigate, Route, route::ToolsRoute};

use super::route_selector;

pub struct ToolsGroup {
    route: Route,
    /// 联机页：大厅编号输入框。
    join_code: Entity<InputState>,
    /// 联机页是否已进入大厅（纯界面状态，对应 PCL 联机页的「是否在大厅」标志）。
    in_lobby: bool,
    /// UI 测试页的结果记录，最新的在最前面。
    ui_results: Vec<SharedString>,
}

impl ToolsGroup {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            route: Route::Tools(ToolsRoute::GameLink),
            join_code: cx.new(|cx| {
                InputState::new(window, cx).placeholder(i18n::lang("Tools.GameLink.Join.IdHint"))
            }),
            in_lobby: false,
            ui_results: Vec::new(),
        }
    }

    /// 「加入」与「创建」共用同一状态切换（见 `in_lobby` 字段注释）。
    fn enter_lobby(&mut self, cx: &mut Context<Self>) {
        self.in_lobby = true;
        cx.notify();
    }
}

impl GroupView for ToolsGroup {
    fn set_route(&mut self, route: Route, _: &mut Window, cx: &mut Context<Self>) {
        self.route = route;
        cx.notify();
    }
}

impl EventEmitter<Navigate> for ToolsGroup {}

impl Render for ToolsGroup {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let route = self.route;
        let navigate = cx.listener(|_, route: &Route, _, cx| cx.emit(Navigate(*route)));
        // 目录中联机与百宝箱都没有右侧动作按钮，这里的回调不会触发。
        let action = cx.listener(|_, _: &Route, _, _| {});

        let content = match route {
            Route::Tools(ToolsRoute::GameLink) => self.render_game_link(cx),
            Route::Tools(ToolsRoute::Test) => self.render_test(cx),
            Route::Tools(ToolsRoute::UiTest) => self.render_ui_test(cx),
            _ => self.render_game_link(cx),
        };

        h_flex()
            .items_stretch()
            .size_full()
            .child(route_selector("tools", route, px(255.), navigate, action))
            .child(div().flex_1().min_w_0().child(content))
    }
}
