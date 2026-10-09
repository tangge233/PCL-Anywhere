//! 测试页（对应 PCL 的 `ToolsTest`）：百宝箱的四个操作卡片，行为未接线。

use gpui_kit::base::h_flex;
use gpui_kit::*;

use super::ToolsGroup;
use crate::components::{AppButton, Card, PageScroll};
use crate::i18n;

impl ToolsGroup {
    /// 测试页（对应 PCL 的 `ToolsTest`）：百宝箱的四个操作按钮，行为未接线。
    pub(crate) fn render_test(
        &mut self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> AnyElement {
        PageScroll::new("tools-test")
            .child(
                Card::new("tools-test-card")
                    .title(i18n::lang("Tools.Test.Title"))
                    .child(
                        h_flex()
                            .flex_wrap()
                            .px_5()
                            .pb_4()
                            .gap_4()
                            .child(
                                AppButton::new("test-clean", i18n::lang("Tools.Test.Clean.Title"))
                                    .min_width(px(120.))
                                    .tooltip(i18n::lang("Tools.Test.Clean.ToolTip")),
                            )
                            .child(
                                AppButton::new("test-luck", i18n::lang("Tools.Test.Luck.Title"))
                                    .min_width(px(100.)),
                            )
                            .child(
                                AppButton::new(
                                    "test-shortcut",
                                    i18n::lang("Tools.Test.Shortcut.Title"),
                                )
                                .min_width(px(120.))
                                .tooltip(i18n::lang("Tools.Test.Shortcut.ToolTip")),
                            )
                            .child(
                                AppButton::new(
                                    "test-launch-count",
                                    i18n::lang("Tools.Test.LaunchCount.Title"),
                                )
                                .min_width(px(120.)),
                            ),
                    ),
            )
            .into_any_element()
    }
}
