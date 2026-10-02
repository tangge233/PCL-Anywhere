//! 启动中面板（对应 `PageLaunchSelector.axaml` 的 `PanLaunching`：进度条、阶段信息与小贴士）。
use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::Sizable as _;
use gpui_kit::component::spinner::Spinner;
use gpui_kit::*;

use super::*;
use crate::components::AppButton;
use crate::i18n;
use crate::theme;

impl LaunchGroup {
    pub(super) fn render_launching(&self, cx: &mut Context<Self>) -> AnyElement {
        let palette = theme::palette(cx);
        let cancel = cx.listener(|this, _: &ClickEvent, _, cx| {
            this.launching = false;
            cx.notify();
        });

        v_flex()
            .size_full()
            .justify_end()
            .gap_4()
            .pb_5()
            .child(
                v_flex()
                    .items_center()
                    .gap_2()
                    .child(Spinner::new().large())
                    .child(
                        div()
                            .text_xl()
                            .text_color(palette.color_level(3))
                            .child(i18n::lang("Launch.Status.Title.Launching")),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(palette.color_level(3))
                            .child(i18n::lang("Common.State.Unknown")),
                    ),
            )
            .child(
                // 进度条：PCL 用两段横条表示已下载与剩余部分。
                h_flex()
                    .h(px(4.))
                    .mx_4()
                    .child(div().w(px(120.)).bg(palette.color_level(4)))
                    .child(div().flex_1().bg(palette.color_level(6)).opacity(0.6)),
            )
            .child(
                v_flex()
                    .items_center()
                    .gap_1()
                    .child(status_row(
                        cx,
                        "Launch.Status.CurrentStep",
                        i18n::lang("Common.Action.Initialize"),
                    ))
                    .child(status_row(
                        cx,
                        "Launch.Status.LoginMethod",
                        i18n::lang("Common.State.Unknown"),
                    ))
                    .child(status_row(
                        cx,
                        "Launch.Status.LaunchProgress",
                        i18n::lang("Common.Action.Initialize"),
                    )),
            )
            .child(
                AppButton::new("cancel-launch", i18n::lang("Common.Action.Cancel"))
                    .mx_4()
                    .on_click(move |event, window, cx| cancel(event, window, cx)),
            )
            .into_any_element()
    }
}

/// 「标签：值」一行；标签弱化右对齐、值左对齐，与 PCL 的启动信息表一致。
pub(super) fn status_row(
    cx: &Context<LaunchGroup>,
    label_key: &str,
    value: SharedString,
) -> AnyElement {
    let palette = theme::palette(cx);
    h_flex()
        .gap_3()
        .child(div().text_xs().opacity(0.5).child(i18n::lang(label_key)))
        .child(
            div()
                .max_w(px(160.))
                .text_xs()
                .text_color(palette.gray_level(1))
                .child(value),
        )
        .into_any_element()
}
