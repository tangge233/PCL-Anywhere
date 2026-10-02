//! 状态卡片：大标题 + 主题色分割线，标题之下可以继续追加内容。

use gpui_kit::base::v_flex;
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::theme;

/// 空状态卡片（PCL 的「大标题 + 主题色分割线」，见 `Containers.axaml` 的 `emptyStateTitle`）。
///
/// 描述之下可以继续 `.child(...)` 追加内容（操作栏、说明卡片等），追加的内容排在卡片内下方。
#[derive(IntoElement)]
pub struct StateCard {
    title: SharedString,
    description: Option<SharedString>,
    min_width: Pixels,
    children: Vec<AnyElement>,
}

impl StateCard {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            description: None,
            min_width: px(150.),
            children: Vec::new(),
        }
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn min_width(mut self, width: impl Into<Pixels>) -> Self {
        self.min_width = width.into();
        self
    }
}

impl ParentElement for StateCard {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for StateCard {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let palette = theme::palette(cx);
        let primary = palette.color_level(3);

        v_flex()
            .min_w(self.min_width)
            .rounded(cx.theme().radius)
            .bg(palette.transparent_background)
            .shadow_sm()
            .p_4()
            .gap_2()
            .items_center()
            .child(div().text_lg().text_color(primary).child(self.title))
            .child(div().w_full().h(px(2.)).bg(primary))
            .when_some(self.description, |this, description| {
                this.child(
                    div()
                        .text_xs()
                        .text_color(palette.gray_level(2))
                        .child(description),
                )
            })
            .children(self.children)
    }
}
