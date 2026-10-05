//! 启动页两栏布局（对应 PCL 的选择器栏 `PageLaunchSelector` 与内容区 `PageLaunchContent`）：
//! 左栏登录面板与启动按钮，右栏 PCL-Anywhere 提示卡；只拼装界面，不持有页面状态。

use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::LaunchGroup;
use super::state::InstanceHandler;
use crate::components::{AppButton, ButtonColor, Card, PageScroll};
use crate::i18n;
use crate::shell::route::InstanceRoute;
use crate::shell::{Navigate, Route};
use crate::theme;
use std::rc::Rc;

impl LaunchGroup {
    /// 左栏（登录 / 启动中面板 + 启动按钮 + 实例跳转）与右栏（PCL-Anywhere 提示卡）。
    pub(super) fn render_home(&self, cx: &mut Context<Self>) -> impl IntoElement {
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
                                    logger::log::info!(target: "Launch", "开始启动游戏");
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
                PageScroll::new("launch-content").child(
                    Card::new("community-hint")
                        .title(i18n::lang("Launch.Right.CommunityHint.Title"))
                        .child(
                            v_flex()
                                .px_5()
                                .pb_4()
                                .gap_1()
                                .child(
                                    div()
                                        .text_sm()
                                        .child(i18n::lang("Launch.Right.CommunityHint.Message")),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .child(i18n::lang("Launch.Right.CommunityHint.HidePrompt")),
                                ),
                        ),
                ),
            ),
        )
    }
}
