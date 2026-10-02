//! 页面容器与页面级状态显示。

use gpui_kit::base::v_flex;
use gpui_kit::component::scroll::ScrollableElement as _;
use gpui_kit::*;

use crate::theme;

/// 窗口内容区宽度。
///
/// `Window::bounds()` 是整窗尺寸；Linux 客户端装饰下 gpui-kit 的 `Root` 会在窗外画阴影与边框，
/// 内容区要扣掉 [`gpui_kit::component::window_paddings`] 的内缩（默认每侧 20px）。
/// 按整窗宽度算栏宽会超出内容区，右侧被裁掉。
pub fn content_width(window: &Window) -> Pixels {
    let paddings = gpui_kit::component::window_paddings(window);
    (window.bounds().size.width - paddings.left - paddings.right).max(px(0.))
}

/// 页面内容滚动区（对应 PCL 的 `MyScrollViewer` + 内容外边距）。
///
/// 滚动归这个元素所有，内边距属于它内部的内容层，因此滚动条贴页面边缘而不是贴内容。
#[derive(IntoElement)]
pub struct PageScroll {
    id: ElementId,
    gap: Pixels,
    children: Vec<AnyElement>,
}

impl PageScroll {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            gap: px(16.),
            children: Vec::new(),
        }
    }

    /// 内容块之间的间距；PCL 的卡片间距为 15。
    pub fn gap(mut self, gap: impl Into<Pixels>) -> Self {
        self.gap = gap.into();
        self
    }
}

impl ParentElement for PageScroll {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for PageScroll {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        v_flex()
            .id(self.id)
            .size_full()
            .min_h_0()
            .overflow_y_scrollbar()
            .child(
                v_flex()
                    .w_full()
                    .min_h_0()
                    .p_6()
                    .gap(self.gap)
                    .children(self.children),
            )
    }
}

/// 页面级小标题（PCL 里 `FontSize 12 / Opacity 0.6` 的分组说明）。
#[derive(IntoElement)]
pub struct SectionLabel {
    text: SharedString,
}

impl SectionLabel {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self { text: text.into() }
    }
}

impl RenderOnce for SectionLabel {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .text_xs()
            .text_color(theme::palette(cx).gray_level(2))
            .opacity(0.6)
            .child(self.text)
    }
}

/// 未迁移页面的占位（对应原版壳层的 `TextBlock.pagePlaceholder`）。
#[derive(IntoElement)]
pub struct PagePlaceholder {
    text: SharedString,
}

impl PagePlaceholder {
    /// 按路由名生成「页面 … 尚未迁移」占位。
    pub fn for_route(route: &str) -> Self {
        Self {
            text: SharedString::from(format!("页面 {route} 尚未迁移")),
        }
    }
}

impl RenderOnce for PagePlaceholder {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        v_flex().size_full().items_center().justify_center().child(
            div()
                .max_w(px(420.))
                .opacity(0.6)
                .text_center()
                .child(self.text),
        )
    }
}
