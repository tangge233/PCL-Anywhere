//! 启动页（对应 PCL 的选择器栏 `PageLaunchSelector` 与内容区 `PageLaunchContent` 界面）。
//!
//! 左栏是登录面板与启动按钮（300px），右栏是 PCL-Anywhere 提示卡。
//! 这里只实现界面与界面状态：登录面板的几种形态可以切换、启动按钮可以切到「启动中」面板，
//! 但不连接任何账号服务或启动流程。
//!
//! 模块划分：本文件持有页面状态与路由，[`home`] 是左右两栏的布局，[`state`] 是示例数据与登录面板枚举，
//! [`login`] 是登录面板（五种形态），[`launching`] 是启动中面板。

mod home;
mod launching;
mod login;
mod state;

use gpui_kit::component::input::InputState;
use gpui_kit::*;

use self::state::{LoginPanel, SAMPLE_AUTH_SERVER};
use crate::i18n;
use crate::shell::{GroupView, Navigate, Route};

pub struct LaunchGroup {
    route: Route,
    login: LoginPanel,
    launching: bool,
    auth_server: Entity<InputState>,
    auth_account: Entity<InputState>,
    auth_password: Entity<InputState>,
    offline_name: Entity<InputState>,
    offline_uuid: Entity<InputState>,
}

impl LaunchGroup {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = |window: &mut Window, cx: &mut Context<Self>, placeholder: SharedString| {
            cx.new(|cx| InputState::new(window, cx).placeholder(placeholder))
        };

        Self {
            route: Route::Launch,
            login: LoginPanel::Select,
            launching: false,
            auth_server: input(
                window,
                cx,
                i18n::lang_with_args("Launch.Account.Auth.ServerLabel", &[SAMPLE_AUTH_SERVER]),
            ),
            auth_account: input(window, cx, i18n::lang("Launch.Account.Auth.Email")),
            auth_password: input(window, cx, i18n::lang("Launch.Account.Auth.Password")),
            offline_name: input(window, cx, i18n::lang("Launch.Account.Offline.PlayerId")),
            offline_uuid: input(
                window,
                cx,
                i18n::lang("Launch.Account.Offline.UuidStandard"),
            ),
        }
    }
}

impl GroupView for LaunchGroup {
    fn set_route(&mut self, route: Route, _: &mut Window, cx: &mut Context<Self>) {
        self.route = route;
        cx.notify();
    }
}

impl EventEmitter<Navigate> for LaunchGroup {}

impl Render for LaunchGroup {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.render_home(cx)
    }
}
