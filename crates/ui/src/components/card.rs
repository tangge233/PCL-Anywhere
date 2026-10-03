//! PCL 卡片（对应 PCL 的 `MyCard` 卡片控件）。
//!
//! 视觉：半透明底 + 细描边 + 双层柔和投影，让卡片浮在页面背景上；标题加粗左对齐、与正文左缘
//! 对齐，可选右上角关闭按钮。标题参与正常流而非浮在内容之上，内容区因此不需要预留标题高度。

use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use std::rc::Rc;

use super::ClickHandler;
use super::icon_button::IconButton;
use crate::theme;

/// 卡片表面：半透明底 + 1px 描边 + 双层柔和投影，[`Card`] 与 [`super::state_card::StateCard`] 共用。
///
/// 卡片底与页面底同为背景色的半透明档，仅靠底色分不出层次，故加一条细描边勾出轮廓；
/// 双层投影里贴紧的一层给接触感、外扩的一层把卡片托离页面，投影色取 Gray1
/// （浅色模式是深灰压出阴影，深色模式是浅灰泛出微光），透明度单独压低。
/// 偏移与模糊按物理像素定义：`BoxShadow` 只接受像素值，不随 rem 缩放。
pub(crate) fn card_surface<T: Styled>(surface: T, cx: &App) -> T {
    let palette = theme::palette(cx);
    surface
        .rounded(cx.theme().radius)
        .bg(palette.transparent_background)
        .border_1()
        .border_color(cx.theme().colors.border)
        .shadow(vec![
            BoxShadow::new(px(0.), px(1.), palette.gray_level(1).opacity(0.06)).blur_radius(px(2.)),
            BoxShadow::new(px(0.), px(6.), palette.gray_level(1).opacity(0.08))
                .blur_radius(px(16.))
                .spread_radius(px(-4.)),
        ])
}

#[derive(IntoElement)]
pub struct Card {
    id: ElementId,
    title: Option<SharedString>,
    close: Option<ClickHandler>,
    children: Vec<AnyElement>,
}

impl Card {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            title: None,
            close: None,
            children: Vec::new(),
        }
    }

    /// 卡片标题，对应 PCL 卡片的标题栏文案。
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// 右上角关闭按钮；仅展示与回调，卡片本身不负责隐藏。
    pub fn on_close(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.close = Some(Rc::new(handler));
        self
    }
}

impl ParentElement for Card {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Card {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let palette = theme::palette(cx);

        card_surface(v_flex().id(self.id), cx)
            .when_some(self.title, |this, title| {
                this.child(
                    h_flex()
                        .items_start()
                        .gap_2()
                        // 标题与正文取同一档左内边距以对齐左缘；上内边距固定，下方留一行呼吸。
                        .px_5()
                        .pt_4()
                        .pb_2()
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .font_weight(FontWeight::BOLD)
                                .text_sm()
                                .text_color(palette.gray_level(1))
                                .child(title),
                        )
                        .when_some(self.close, |this, close| {
                            this.child(
                                // PCL 的卡片关闭按钮为 20px 图标按钮。
                                IconButton::new(
                                    "card-close",
                                    "x",
                                    crate::i18n::lang("Common.Action.Close"),
                                )
                                .size(px(20.))
                                .tooltip(crate::i18n::lang("Common.Action.Close"))
                                .on_click(move |event, window, cx| close(event, window, cx)),
                            )
                        }),
                )
            })
            .child(v_flex().min_w_0().children(self.children))
            .test_support()
    }
}
