//! 登录面板（对应 `PageLaunchSelector.axaml` 的 `PanLogin`：已登录 / 选择方式 / 微软 / 第三方 / 离线五种形态）。
use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::input::{Input, InputContentType};
use gpui_kit::*;

use super::state::{LoginPanel, MICROSOFT_LINK_URL, SAMPLE_DEVICE_CODE, SAMPLE_PROFILE_NAME};
use super::*;
use crate::components::{AppButton, ButtonColor};
use crate::i18n;

impl LaunchGroup {
    pub(super) fn render_login(&self, cx: &mut Context<Self>) -> AnyElement {
        match self.login {
            LoginPanel::Profile => v_flex()
                .items_center()
                .gap_2()
                // 示例档案：无账号服务时展示一份样例数据。
                .child(
                    div()
                        .text_lg()
                        .font_weight(FontWeight::BOLD)
                        .child(SAMPLE_PROFILE_NAME),
                )
                .child(
                    div()
                        .text_xs()
                        .opacity(0.7)
                        .child(i18n::text("Launch.Account.Type.Microsoft")),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .child(
                            AppButton::new(
                                "switch-profile",
                                i18n::text("Launch.Account.Auth.ChangeProfile"),
                            )
                            .min_width(px(90.))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.login = LoginPanel::Select;
                                cx.notify();
                            }))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.login = LoginPanel::Select;
                                cx.notify();
                            })),
                        )
                        .child(
                            AppButton::new("logout", i18n::text("Launch.Account.Auth.Logout"))
                                .color(ButtonColor::Red)
                                .min_width(px(90.))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.login = LoginPanel::Select;
                                    cx.notify();
                                })),
                        ),
                )
                .into_any_element(),
            LoginPanel::Select => v_flex()
                .items_center()
                .gap_2()
                .child(
                    AppButton::new(
                        "login-microsoft",
                        i18n::text("Launch.Account.Microsoft.Start"),
                    )
                    .color(ButtonColor::Highlight)
                    .min_width(px(150.))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.login = LoginPanel::Microsoft;
                        cx.notify();
                    })),
                )
                .child(
                    AppButton::new("login-authlib", i18n::text("Launch.Account.Auth.Login"))
                        .min_width(px(150.))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.login = LoginPanel::Authlib;
                            cx.notify();
                        })),
                )
                .child(
                    AppButton::new("login-offline", i18n::text("Launch.Account.Offline.Create"))
                        .min_width(px(150.))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.login = LoginPanel::Offline;
                            cx.notify();
                        })),
                )
                .into_any_element(),
            LoginPanel::Microsoft => v_flex()
                .gap_2()
                .child(div().text_xs().opacity(0.8).child(i18n::text_args(
                    "Launch.Account.LoginDialog.MicrosoftInstructions",
                    &[SAMPLE_DEVICE_CODE, MICROSOFT_LINK_URL],
                )))
                // 授权码：无登录流程时为示例值，接入设备代码流程后由服务返回。
                .child(
                    div()
                        .text_2xl()
                        .font_weight(FontWeight::BOLD)
                        .text_center()
                        .child(SAMPLE_DEVICE_CODE),
                )
                .child(
                    h_flex()
                        .justify_center()
                        .gap_2()
                        .child(
                            AppButton::new(
                                "microsoft-website",
                                i18n::text("Launch.Account.Microsoft.Website"),
                            )
                            .color(ButtonColor::Highlight)
                            .min_width(px(100.)),
                        )
                        .child(
                            AppButton::new(
                                "copy-code",
                                i18n::text("Launch.Account.LoginDialog.CopyCode"),
                            )
                            .min_width(px(100.)),
                        )
                        .child(
                            AppButton::new("microsoft-back", i18n::text("Common.Action.Cancel"))
                                .min_width(px(80.))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.login = LoginPanel::Select;
                                    cx.notify();
                                })),
                        ),
                )
                .into_any_element(),
            LoginPanel::Authlib => v_flex()
                .gap_2()
                .child(Input::new(&self.auth_server))
                .child(Input::new(&self.auth_account))
                .child(Input::new(&self.auth_password).content_type(InputContentType::Password))
                .child(
                    h_flex()
                        .justify_center()
                        .gap_2()
                        .child(
                            AppButton::new(
                                "authlib-login",
                                i18n::text("Launch.Account.Auth.Login"),
                            )
                            .color(ButtonColor::Highlight)
                            .min_width(px(100.)),
                        )
                        .child(
                            AppButton::new("authlib-back", i18n::text("Launch.Account.Back"))
                                .min_width(px(80.))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.login = LoginPanel::Select;
                                    cx.notify();
                                })),
                        ),
                )
                .into_any_element(),
            LoginPanel::Offline => v_flex()
                .gap_2()
                .child(Input::new(&self.offline_name))
                .child(Input::new(&self.offline_uuid))
                .child(
                    h_flex()
                        .justify_center()
                        .gap_2()
                        .child(
                            AppButton::new(
                                "offline-create",
                                i18n::text("Launch.Account.Offline.Create"),
                            )
                            .color(ButtonColor::Highlight)
                            .min_width(px(100.))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.login = LoginPanel::Profile;
                                cx.notify();
                            })),
                        )
                        .child(
                            AppButton::new(
                                "offline-back",
                                i18n::text("Launch.Account.Offline.Back"),
                            )
                            .min_width(px(80.))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.login = LoginPanel::Select;
                                cx.notify();
                            })),
                        ),
                )
                .into_any_element(),
        }
    }
}
