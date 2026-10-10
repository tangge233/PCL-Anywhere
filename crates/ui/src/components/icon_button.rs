//! 圆形图标按钮（对应 PCL 的 `MyIconButton`）。
//!
//! 圆形底 + 居中图标，配色由 [`IconButtonTheme`] 表达；悬停加深、按下变色，
//! 键盘可达（Tab 停靠 + Enter/Space 触发）并带可访问名称。

use gpui_kit::base::{StyledExt as _, TestSupportExt as _};
use gpui_kit::component::ThemeStyled as _;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use std::rc::Rc;

use super::{ClickHandler, lucide, on_enter_space, register_focus};
use crate::theme;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IconButtonTheme {
    /// 跟随主题色：静息为主题浅色，悬停转深。
    #[default]
    Accent,
    /// 画在深色底上（标题栏、自绘弹窗标题）：图标取该主题的“白”，在暗色主题下即浅字。
    OnDark,
    /// 跟随正文色（浅底上的普通图标）。
    Foreground,
    /// 红色：用于删除等操作。
    Red,
    /// 不画自己的底色，图标跟随外层文字色；用于外层容器已经提供悬停反馈的场合
    /// （例如实例页的收起条，悬停由窄条底色表达）。
    Plain,
}

/// 圆形图标按钮，PCL 的 `MyIconButton`。
#[derive(IntoElement)]
pub struct IconButton {
    id: ElementId,
    icon: SharedString,
    theme: IconButtonTheme,
    size: Pixels,
    disabled: bool,
    accessibility_label: SharedString,
    tooltip: Option<SharedString>,
    on_click: Option<ClickHandler>,
    consume_mouse_down: bool,
    /// 命中区域撑满外层容器（图标仍按 `size` 居中）：用于整条可点的窄条这类控件。
    stretch: bool,
    style: StyleRefinement,
}

impl IconButton {
    /// `accessibility_label` 是图标按钮必需的可访问名称。
    pub fn new(
        id: impl Into<ElementId>,
        icon: impl Into<SharedString>,
        accessibility_label: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            icon: icon.into(),
            theme: IconButtonTheme::Accent,
            size: px(28.),
            disabled: false,
            accessibility_label: accessibility_label.into(),
            tooltip: None,
            on_click: None,
            consume_mouse_down: false,
            stretch: false,
            style: StyleRefinement::default(),
        }
    }

    /// 命中区域撑满外层容器，图标保持 [`Self::size`] 的大小居中。
    ///
    /// 用于「整条可点」的控件：例如实例页两栏之间的收起条，整条 18px 宽、通高都要能点，
    /// 但尖角仍是 8px。取值为 `true` 时不要再额外给高度，元素会 `size_full()`。
    pub fn stretch(mut self) -> Self {
        self.stretch = true;
        self
    }

    /// 按下时先消费事件，避免自绘标题栏里的按钮同时触发窗口拖动。
    pub fn consume_mouse_down(mut self) -> Self {
        self.consume_mouse_down = true;
        self
    }

    pub fn theme(mut self, theme: IconButtonTheme) -> Self {
        self.theme = theme;
        self
    }

    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = size.into();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn tooltip(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }
}

impl Styled for IconButton {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for IconButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let palette = theme::palette(cx);
        let (color, hover_color, hover_bg, active_bg) = match self.theme {
            IconButtonTheme::Accent => (
                palette.color_level(4),
                palette.color_level(2),
                palette.color_level(7),
                palette.color_level(6),
            ),
            IconButtonTheme::OnDark => (
                palette.white,
                palette.white,
                palette.half_white,
                palette.semi_white,
            ),
            IconButtonTheme::Foreground => (
                palette.gray_level(1),
                palette.color_level(2),
                palette.color_level(7),
                palette.color_level(6),
            ),
            IconButtonTheme::Red => (
                palette.red_light.opacity(0.627),
                palette.red_light,
                palette.red_back,
                palette.red_back,
            ),
            IconButtonTheme::Plain => (
                palette.gray_level(1),
                palette.gray_level(1),
                gpui::transparent_black(),
                gpui::transparent_black(),
            ),
        };

        let disabled = self.disabled;
        let on_click = self.on_click;
        let (focus_handle, is_focused) = register_focus(window, self.id.clone(), cx);

        div()
            .id(self.id)
            .test_support()
            .role(Role::Button)
            .aria_label(self.accessibility_label.clone())
            .flex()
            .flex_shrink_0()
            .items_center()
            .justify_center()
            .size(self.size)
            .rounded_full_style(cx)
            .when(self.stretch, |this| this.size_full())
            // 图标与圆形底之间留 5px，与 PCL 的 PART_IconHost Margin 一致。
            .p(px(5.))
            .text_color(if disabled {
                palette.gray_level(4)
            } else {
                color
            })
            .tab_stop(true)
            .tab_index(0)
            .track_focus(&focus_handle)
            .when(!disabled, |this| {
                this.hover(|this| this.bg(hover_bg).text_color(hover_color))
                    .active(|this| this.bg(active_bg).text_color(hover_color))
            })
            .when(is_focused, |this| this.focus_ring_style(window, cx))
            .when_some(self.tooltip.clone(), |this, tooltip| {
                this.tooltip(move |window, cx| Tooltip::new(tooltip.clone()).build(window, cx))
            })
            .when(self.consume_mouse_down, |this| {
                this.on_mouse_down(MouseButton::Left, |_, window, cx| {
                    window.prevent_default();
                    cx.stop_propagation();
                })
            })
            .when_some(on_click.filter(|_| !disabled), |this, handler| {
                this.on_key_down(on_enter_space(handler.clone(), ClickEvent::default))
                    .on_click(move |event, window, cx| handler(event, window, cx))
            })
            .refine_style(&self.style)
            // 图标始终按 size 计算，撑满容器时不会被拉大。
            .child(lucide(&self.icon).size(self.size - px(10.)))
    }
}
