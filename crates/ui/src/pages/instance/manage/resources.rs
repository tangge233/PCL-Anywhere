//! 管理栏 · 本地资源 Tab（模组 / 资源包 / 光影 / 投影，对应 `PageInstanceResources.axaml`）。

use gpui_kit::base::{h_flex, v_flex};

use super::super::*;
use crate::components::{AppButton, ButtonColor, StateCard};
use crate::i18n;

impl InstanceGroup {
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
                    StateCard::new(i18n::lang("Instance.Resource.Empty.Title"))
                        .description(i18n::lang("Instance.Resource.Empty.Description")),
                ),
            )
            .into_any_element()
    }
}
