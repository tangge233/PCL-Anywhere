//! 管理栏 · 概览 Tab（对应 PCL 的 `PageInstanceOverall` 界面）。

use gpui_kit::base::{h_flex, v_flex};

use super::super::data::InstanceId;
use super::super::*;
use crate::components::{AppButton, ButtonColor, Card, PageScroll};
use crate::i18n;
use crate::theme;

impl InstanceGroup {
    /// 管理栏 · 概览：信息卡片 + 操作卡片（对应 PCL 的 `PageInstanceOverall` 界面）。
    pub(super) fn render_overview(
        &mut self,
        id: &InstanceId,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Some(instance) = self.instance_by_id(id) else {
            return div().into_any_element();
        };
        let palette = theme::palette(cx);
        let launch_count = instance.launch_count.to_string();

        PageScroll::new("instance-overview")
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
                            .child(detail_line(
                                "Instance.Overall.Info.ModpackVersion",
                                instance.modpack,
                            ))
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
}
