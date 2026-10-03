//! 管理栏 · 设置 Tab（对应 PCL 的 `PageInstanceSetup` 界面）。

use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::input::Input;
use gpui_kit::component::slider::Slider;

use super::super::data::{InstanceData, InstanceId};
use super::super::*;
use crate::components::{AppButton, AppRadio, ButtonColor, Card, PageScroll};
use crate::i18n;
use crate::theme;

impl InstanceGroup {
    /// 管理栏 · 设置：内存卡片 + 高级卡片（对应 PCL 的 `PageInstanceSetup` 界面）。
    pub(super) fn render_setup(
        &mut self,
        id: &InstanceId,
        data: &InstanceData,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Some(instance) = self.instance_by_id(id) else {
            return div().into_any_element();
        };
        let palette = theme::palette(cx);
        let manual = data.memory_mode == 1;
        let data_id = id.clone();
        let mode_default = i18n::lang("Common.Option.Default");
        let mode_custom = i18n::lang("Common.Option.Customize");

        PageScroll::new("instance-setup")
            .child(
                Card::new("setup-memory")
                    .title(i18n::lang("Setup.Launch.Memory.Title"))
                    .child(
                        v_flex()
                            .px_5()
                            .pt(px(30.))
                            .pb_4()
                            .gap_3()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(palette.color_level(3))
                                    .child(instance.name),
                            )
                            .child(
                                h_flex()
                                    .gap_5()
                                    .child(
                                        AppRadio::new("memory-default")
                                            .label(mode_default)
                                            .selected(!manual)
                                            .on_change(cx.listener({
                                                let id = data_id.clone();
                                                move |this, _: &bool, window, cx| {
                                                    this.data
                                                        .get_or_create(&id, window, cx)
                                                        .memory_mode = 0;
                                                    cx.notify();
                                                }
                                            })),
                                    )
                                    .child(
                                        AppRadio::new("memory-custom")
                                            .label(mode_custom)
                                            .selected(manual)
                                            .on_change(cx.listener({
                                                let id = data_id.clone();
                                                move |this, _: &bool, window, cx| {
                                                    this.data
                                                        .get_or_create(&id, window, cx)
                                                        .memory_mode = 1;
                                                    cx.notify();
                                                }
                                            })),
                                    ),
                            )
                            .child(
                                v_flex()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_sm()
                                            .child(i18n::lang("Setup.Launch.Memory.Max")),
                                    )
                                    .child(
                                        div()
                                            .w_full()
                                            .child(Slider::new(&data.max_memory).disabled(!manual)),
                                    ),
                            )
                            .child(
                                v_flex()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_sm()
                                            .child(i18n::lang("Setup.Launch.Memory.Initial")),
                                    )
                                    .child(div().w_full().child(
                                        Slider::new(&data.initial_memory).disabled(!manual),
                                    )),
                            ),
                    ),
            )
            .child(
                Card::new("setup-advanced")
                    .title(i18n::lang("Setup.Launch.Advanced.Title"))
                    .child(
                        v_flex()
                            .px_5()
                            .pt(px(30.))
                            .pb_4()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .child(i18n::lang("Setup.Launch.Advanced.JvmHead")),
                            )
                            .child(Input::new(&data.jvm_args))
                            .child(
                                AppButton::new(
                                    "setup-restore-jvm",
                                    i18n::lang("Setup.Launch.Advanced.JvmHead.Restore"),
                                )
                                .min_width(px(120.)),
                            )
                            .child(
                                div()
                                    .mt_2()
                                    .text_sm()
                                    .child(i18n::lang("Setup.Launch.Advanced.GameTail")),
                            )
                            .child(Input::new(&data.game_args))
                            .child(
                                div()
                                    .mt_2()
                                    .text_sm()
                                    .child(i18n::lang("Setup.Launch.Options.CustomInfo.Label")),
                            )
                            .child(Input::new(&data.custom_info))
                            .child(
                                div()
                                    .mt_2()
                                    .text_xs()
                                    .opacity(0.6)
                                    // PCL 沿用 JVM 参数键属其笔误，
                                    // 本仓库改用专用 Classpath 键。
                                    .child(i18n::lang("Instance.Setup.Advanced.Classpath")),
                            )
                            .child(Input::new(&data.classpath))
                            .child(
                                h_flex()
                                    .mt_3()
                                    .gap_3()
                                    .child(
                                        AppButton::new(
                                            "setup-open-folder",
                                            i18n::lang("Common.Action.OpenFolder"),
                                        )
                                        .min_width(px(120.)),
                                    )
                                    .child(
                                        AppButton::new(
                                            "setup-reset",
                                            i18n::lang("Common.Action.Reset"),
                                        )
                                        .color(ButtonColor::Red)
                                        .min_width(px(120.)),
                                    ),
                            ),
                    ),
            )
            .into_any_element()
    }
}
