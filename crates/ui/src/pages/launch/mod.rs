//! 启动页（对应 `PCL/Views/Launch/PageLaunchSelector.axaml` 与 `PageLaunchContent.axaml`）。
//!
//! 左栏是登录面板与启动按钮（300px），右栏是社区版提示与启动日志卡片。
//! 这里只实现界面与界面状态：登录面板的几种形态可以切换、启动按钮可以切到「启动中」面板，
//! 但不连接任何账号服务或启动流程。
//!
//! 模块划分：本文件持有页面状态与路由，[`state`] 是示例数据与登录面板枚举，
//! [`login`] 是登录面板（五种形态），[`launching`] 是启动中面板。

mod launching;
mod login;
mod state;

use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::input::InputState;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use self::state::{InstanceHandler, LoginPanel, SAMPLE_AUTH_SERVER, SAMPLE_LOG};
use crate::components::{AppButton, ButtonColor, Card, PageScroll};
use crate::i18n;
use crate::shell::route::InstanceRoute;
use crate::shell::{GroupView, Navigate, Route};
use crate::theme;
use std::rc::Rc;

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
        let palette = theme::palette(cx);
        let launching = self.launching;
        let enter_instance: InstanceHandler = Rc::new(cx.listener(|_, route: &Route, _, cx| {
            cx.emit(Navigate(*route));
        }));

        let selector = v_flex()
            .id("launch-selector")
            .w(px(300.))
            .h_full()
            .flex_shrink_0()
            .justify_end()
            .pb_5()
            .gap_3()
            .when(launching, |this| this.child(self.render_launching(cx)))
            .when(!launching, |this| {
                this.child(
                    div()
                        .flex_1()
                        .flex()
                        .items_center()
                        .child(self.render_login(cx)),
                )
                .child(
                    // 启动按钮：PCL 为 54px 高、占满左栏宽度，版本号压在按钮内侧底部。
                    div()
                        .relative()
                        .mx_4()
                        .child(
                            AppButton::new("launch", i18n::lang("Launch.Home.Button.Launch"))
                                .color(ButtonColor::Highlight)
                                .w_full()
                                .h(px(54.))
                                // PCL 的启动按钮把文案上抬，底部留给版本号。
                                .pt(px(6.))
                                .pb(px(16.))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.launching = true;
                                    cx.notify();
                                })),
                        )
                        .child(
                            // 版本号压在按钮底部（PCL 的 LabVersion）。
                            div()
                                .absolute()
                                .bottom(px(6.))
                                .w_full()
                                .text_center()
                                .text_xs()
                                .text_color(palette.gray_level(3))
                                .child(i18n::lang("Launch.Home.VersionList.Loading")),
                        ),
                )
                .child(
                    h_flex()
                        .px_4()
                        .gap_2()
                        .child(
                            AppButton::new(
                                "select-instance",
                                i18n::lang("Launch.Home.SelectInstance"),
                            )
                            .flex_1()
                            .on_click({
                                let enter_instance = enter_instance.clone();
                                move |_, window, cx| {
                                    enter_instance(
                                        &Route::Instance(InstanceRoute::Select),
                                        window,
                                        cx,
                                    )
                                }
                            }),
                        )
                        .child(
                            AppButton::new(
                                "instance-settings",
                                i18n::lang("Launch.Home.InstanceSettings"),
                            )
                            .flex_1()
                            .on_click({
                                let enter_instance = enter_instance.clone();
                                move |_, window, cx| {
                                    enter_instance(
                                        &Route::Instance(InstanceRoute::Setup),
                                        window,
                                        cx,
                                    )
                                }
                            }),
                        ),
                )
            });

        h_flex().items_stretch().size_full().child(selector).child(
            div().flex_1().min_w_0().child(
                PageScroll::new("launch-content")
                    .gap(px(15.))
                    .child(
                        Card::new("community-hint")
                            .title(i18n::lang("Launch.Right.CommunityHint.Title"))
                            .child(
                                v_flex()
                                    .px_5()
                                    .pb_4()
                                    .gap_1()
                                    .child(
                                        div().text_sm().child(i18n::lang(
                                            "Launch.Right.CommunityHint.Message",
                                        )),
                                    )
                                    .child(div().text_sm().child(i18n::lang(
                                        "Launch.Right.CommunityHint.HidePrompt",
                                    ))),
                            ),
                    )
                    .child(
                        Card::new("launch-log")
                            .title(i18n::lang("Launch.Right.Log.Title"))
                            .child(
                                div()
                                    .px_5()
                                    .pb_4()
                                    .text_sm()
                                    .text_color(palette.gray_level(3))
                                    .child(SAMPLE_LOG),
                            ),
                    ),
            ),
        )
    }
}
