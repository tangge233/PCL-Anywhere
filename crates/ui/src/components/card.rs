//! PCL 卡片（对应迁移前实现的 `MyCard` 控件）。
//!
//! 视觉：半透明背景 + 6px 圆角 + 轻阴影，标题加粗左对齐，可选右上角关闭按钮。
//! 与 .NET 版本的差别：标题参与正常流而非浮在内容之上，内容区因此不需要预留标题高度。

use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use std::rc::Rc;

use super::icon_button::IconButton;
use crate::theme;

type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

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

        v_flex()
            .id(self.id)
            .rounded(cx.theme().radius)
            .bg(palette.transparent_background)
            .shadow_sm()
            .when_some(self.title, |this, title| {
                let close = self.close.clone();
                this.child(
                    h_flex()
                        .items_start()
                        .gap_2()
                        .px_4()
                        .pt_3()
                        .pb_1()
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .font_weight(FontWeight::BOLD)
                                .text_sm()
                                .text_color(palette.gray_level(1))
                                .child(title),
                        )
                        .when_some(close, |this, close| {
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
    }
}
