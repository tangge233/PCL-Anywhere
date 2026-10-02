//! 管理栏：Tab 与四块内容（概览 / 设置 / 存档 / 资源）。
//!
//! 对应 `PageInstanceManage.axaml`、`PageInstanceOverall.axaml`、`PageInstanceSetup.axaml`
//! 与 `PageInstanceResources.axaml`。

use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::input::Input;
use gpui_kit::component::slider::Slider;
use gpui_kit::component::tab::{Tab, TabBar};

use super::merged::divider_color;
use super::state::SampleInstance;
use super::*;
use crate::components::{AppButton, AppRadio, ButtonColor, Card, EmptyState, PageScroll};
use crate::i18n;
use crate::theme;

/// 管理栏 Tab 文案键（顺序与 XAML 的 TabItem 一致）。
pub(super) const MANAGE_TABS: [&str; 7] = [
    "Instance.Left.Overview",
    "Instance.Left.Settings",
    "Instance.Left.Saves",
    "Instance.Left.Mod",
    "Instance.Left.ResourcePacks",
    "Instance.Left.Shaders",
    "Instance.Left.Schematics",
];

impl InstanceGroup {
    pub(super) fn render_manage_column(&mut self, cx: &mut Context<Self>) -> Div {
        let Some(instance) = self.selected_instance() else {
            return v_flex()
                .size_full()
                .items_center()
                .justify_center()
                .child(EmptyState::new(i18n::lang("Instance.Manage.SelectHint")));
        };

        let tab_bar = TabBar::new("instance-manage-tabs")
            .underline()
            .selected_index(self.manage_tab)
            .children(
                MANAGE_TABS
                    .iter()
                    .map(|key| Tab::new().label(i18n::lang(key))),
            )
            .on_click(cx.listener(|this, ix: &usize, _, cx| {
                this.manage_tab = *ix;
                cx.notify();
            }));

        let content = match self.manage_tab {
            TAB_OVERVIEW => self.render_overview(instance, cx),
            TAB_SETTINGS => self.render_setup(instance, cx),
            TAB_SAVES => self.render_saves_tab(cx),
            TAB_MOD | TAB_RESOURCE_PACKS | TAB_SHADERS | TAB_SCHEMATICS => self.render_resources(),
            _ => div().into_any_element(),
        };

        v_flex()
            .size_full()
            .min_h_0()
            .child(div().px_3().pt_2().child(tab_bar))
            .child(div().flex_1().min_h_0().child(content))
    }

    /// 管理栏 · 概览：信息卡片 + 操作卡片（对应 `PageInstanceOverall.axaml`）。
    pub(super) fn render_overview(
        &mut self,
        instance: &'static SampleInstance,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let palette = theme::palette(cx);
        let launch_count = instance.launch_count.to_string();

        PageScroll::new("instance-overview")
            .gap(px(15.))
            .child(
                Card::new("overview-info")
                    .title(i18n::lang("Instance.Overall.Info.Title"))
                    .child(
                        v_flex()
                            .px_5()
                            .pt(px(30.))
                            .pb_4()
                            .gap_2()
                            .child(
                                div()
                                    .text_xl()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(palette.color_level(3))
                                    .child(instance.name),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .opacity(0.8)
                                    .child(SharedString::from(format!(
                                        // 示例数据：实例状态串。
                                        "{} · {}",
                                        instance.version, instance.loader
                                    ))),
                            )
                            .child(div().text_sm().opacity(0.8).child(i18n::lang_with_args(
                                "Instance.Overall.Info.LaunchCount.Count",
                                &[&launch_count],
                            )))
                            .child(
                                div()
                                    .text_sm()
                                    .opacity(0.8)
                                    .child(SharedString::from(format!(
                                        "{}: {}",
                                        i18n::lang("Instance.Overall.Info.ModpackVersion"),
                                        instance.modpack,
                                    ))),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .opacity(0.6)
                                    .child(SharedString::from(instance.path)),
                            ),
                    ),
            )
            .child(
                Card::new("overview-actions")
                    .title(i18n::lang("Instance.Left.Overview"))
                    .child(
                        h_flex()
                            .px_5()
                            .pt(px(32.))
                            .pb_4()
                            .gap_3()
                            .child(
                                AppButton::new(
                                    "overview-open-folder",
                                    i18n::lang("Common.Action.OpenFolder"),
                                )
                                .min_width(px(120.)),
                            )
                            .child(
                                AppButton::new(
                                    "overview-refresh",
                                    i18n::lang("Common.Action.Refresh"),
                                )
                                .min_width(px(120.)),
                            )
                            .child(
                                AppButton::new(
                                    "overview-delete",
                                    i18n::lang("Common.Action.Delete"),
                                )
                                .color(ButtonColor::Red)
                                .min_width(px(120.)),
                            ),
                    ),
            )
            .into_any_element()
    }

    /// 管理栏 · 设置：内存卡片 + 高级卡片（对应 `PageInstanceSetup.axaml`）。
    pub(super) fn render_setup(
        &mut self,
        instance: &'static SampleInstance,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let palette = theme::palette(cx);
        let manual = self.memory_mode == 1;
        let mode_default = i18n::lang("Common.Option.Default");
        let mode_custom = i18n::lang("Common.Option.Customize");

        PageScroll::new("instance-setup")
            .gap(px(15.))
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
                                            .on_change(cx.listener(|this, _: &bool, _, cx| {
                                                this.memory_mode = 0;
                                                cx.notify();
                                            })),
                                    )
                                    .child(
                                        AppRadio::new("memory-custom")
                                            .label(mode_custom)
                                            .selected(manual)
                                            .on_change(cx.listener(|this, _: &bool, _, cx| {
                                                this.memory_mode = 1;
                                                cx.notify();
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
                                            .child(Slider::new(&self.max_memory).disabled(!manual)),
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
                                        Slider::new(&self.initial_memory).disabled(!manual),
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
                            .child(Input::new(&self.jvm_args))
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
                            .child(Input::new(&self.game_args))
                            .child(
                                div()
                                    .mt_2()
                                    .text_sm()
                                    .child(i18n::lang("Setup.Launch.Options.CustomInfo.Label")),
                            )
                            .child(Input::new(&self.custom_info))
                            .child(
                                div()
                                    .mt_2()
                                    .text_xs()
                                    .opacity(0.6)
                                    .child(i18n::lang("Setup.Launch.Advanced.JvmHead")),
                            )
                            .child(Input::new(&self.classpath))
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

    /// 管理栏 · 存档：150px 存档选择栏 + 存档内容。
    pub(super) fn render_saves_tab(&mut self, cx: &mut Context<Self>) -> AnyElement {
        h_flex()
            .size_full()
            .min_h_0()
            .child(self.render_saves_selector(MANAGE_SAVES_SELECTOR_W, cx))
            .child(div().w(px(DIVIDER_W)).h_full().bg(divider_color(cx)))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .child(self.render_saves_content(cx)),
            )
            .into_any_element()
    }

    /// 管理栏 · 本地资源（模组 / 资源包 / 光影 / 投影）：工具栏 + 空状态。
    pub(super) fn render_resources(&self) -> AnyElement {
        v_flex()
            .size_full()
            .min_h_0()
            .child(
                h_flex()
                    .p_3()
                    .gap_2()
                    .child(AppButton::new(
                        "res-refresh",
                        i18n::lang("Common.Action.Refresh"),
                    ))
                    .child(AppButton::new(
                        "res-open-folder",
                        i18n::lang("Common.Action.OpenFolder"),
                    ))
                    .child(AppButton::new(
                        "res-toggle",
                        i18n::lang("Instance.Resource.Disable"),
                    ))
                    .child(
                        AppButton::new("res-delete", i18n::lang("Common.Action.Delete"))
                            .color(ButtonColor::Red),
                    ),
            )
            .child(
                div().flex_1().items_center().justify_center().child(
                    EmptyState::new(i18n::lang("Instance.Resource.Empty.Title"))
                        .description(i18n::lang("Instance.Resource.Empty.Description")),
                ),
            )
            .into_any_element()
    }

    // ---- 存档选择栏与存档内容 ----------------------------------------------
}
