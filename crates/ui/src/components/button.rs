//! PCL 的按钮控件（对应 `PCL.Ui/Controls/MyButton.cs` 与 `MyIconButton.cs`）。
//!
//! PCL 的按钮不是实心填充，而是「半透明白底 + 1px 彩色描边 + 与描边同色的文字」：
//! 普通按钮描边取正文色，强调按钮取主题深色，危险按钮取红色；悬停时描边转为主题色、
//! 底色转为主题浅色（红色按钮则转成红色系）。这里把这套状态做成通用组件，页面只声明语义。

use super::ClickHandler;
use super::lucide;
use crate::theme;
use gpui_kit::base::{StyledExt as _, TestSupportExt as _, h_flex};
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::component::{Sizable as _, ThemeStyled as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use std::rc::Rc;

/// 按钮配色（对应 `MyButton.ColorState`）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonColor {
    /// 正文色描边，用于普通操作。
    #[default]
    Normal,
    /// 主题色描边，用于面板里的主要操作。
    Highlight,
    /// 红色描边，用于退出登录等破坏性操作。
    Red,
}

/// 按钮默认高度；PCL 的普通按钮取 30，调用方可以按 PCL 的原值覆盖（如 35、54）。
const DEFAULT_HEIGHT: Pixels = px(30.);
/// PCL 的按钮圆角为 3。
const RADIUS: Pixels = px(3.);
/// PCL 的按钮文字为 13px（16px 基准下的 0.8125rem）。
const LABEL_SIZE: Rems = rems(0.8125);

/// 文本按钮。
#[derive(IntoElement)]
pub struct AppButton {
    id: ElementId,
    label: SharedString,
    color: ButtonColor,
    icon: Option<SharedString>,
    min_width: Option<Pixels>,
    height: Pixels,
    disabled: bool,
    loading: bool,
    tooltip: Option<SharedString>,
    on_click: Option<ClickHandler>,
    style: StyleRefinement,
}

impl AppButton {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            color: ButtonColor::Normal,
            icon: None,
            min_width: None,
            height: DEFAULT_HEIGHT,
            disabled: false,
            loading: false,
            tooltip: None,
            on_click: None,
            style: StyleRefinement::default(),
        }
    }

    pub fn color(mut self, color: ButtonColor) -> Self {
        self.color = color;
        self
    }

    /// 文字前的图标（lucide 文件名）。
    pub fn icon(mut self, name: &'static str) -> Self {
        self.icon = Some(SharedString::from(name));
        self
    }

    pub fn min_width(mut self, width: impl Into<Pixels>) -> Self {
        self.min_width = Some(width.into());
        self
    }

    pub fn height(mut self, height: impl Into<Pixels>) -> Self {
        self.height = height.into();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 等待中：保留配色，屏蔽重复点击。
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
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

impl Styled for AppButton {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for AppButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let palette = theme::palette(cx);
        let (border, hover_border, hover_fill, active_fill) = match self.color {
            ButtonColor::Normal => (
                palette.gray_level(1),
                palette.color_level(3),
                palette.color_level(7),
                palette.color_level(6),
            ),
            ButtonColor::Highlight => (
                palette.color_level(2),
                palette.color_level(3),
                palette.color_level(7),
                palette.color_level(6),
            ),
            ButtonColor::Red => (
                palette.red_dark,
                palette.red_light,
                palette.red_back,
                palette.red_back,
            ),
        };

        let disabled = self.disabled;
        let loading = self.loading;
        let enabled = !disabled && !loading;
        let on_click = self.on_click.clone();
        let focus_handle = window
            .use_keyed_state(self.id.clone(), cx, |_, cx| {
                cx.focus_handle().tab_stop(true)
            })
            .read(cx)
            .clone();
        let is_focused = focus_handle.is_focused(window);

        let label = if loading {
            h_flex()
                .gap_2()
                .child(Spinner::new().small())
                .child(self.label.clone())
                .into_any_element()
        } else {
            h_flex()
                .gap_2()
                .when_some(self.icon.clone(), |this, icon| {
                    this.child(lucide(&icon).size_4())
                })
                .child(self.label.clone())
                .into_any_element()
        };

        div()
            .id(self.id)
            .test_support()
            .role(Role::Button)
            .aria_label(self.label.clone())
            .flex()
            .flex_shrink_0()
            .items_center()
            .justify_center()
            .h(self.height)
            .px_3()
            .rounded(RADIUS)
            .border_1()
            .border_color(if disabled {
                palette.gray_level(4)
            } else {
                border
            })
            .bg(palette.half_white)
            .text_color(if disabled {
                palette.gray_level(4)
            } else {
                border
            })
            .text_size(LABEL_SIZE)
            .tab_stop(true)
            .tab_index(0)
            .track_focus(&focus_handle)
            .when_some(self.min_width, |this, width| this.min_w(width))
            .when(enabled, |this| {
                this.hover(|this| this.bg(hover_fill).border_color(hover_border))
                    .active(|this| this.bg(active_fill).border_color(hover_border))
            })
            .when(is_focused, |this| this.focus_ring_style(window, cx))
            .when_some(self.tooltip.clone(), |this, tooltip| {
                this.tooltip(move |window, cx| Tooltip::new(tooltip.clone()).build(window, cx))
            })
            .when_some(on_click.filter(|_| enabled), |this, handler| {
                let on_key = handler.clone();
                this.on_key_down(move |event, window, cx| {
                    if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                        on_key(&ClickEvent::default(), window, cx);
                    }
                })
                .on_click(move |event, window, cx| handler(event, window, cx))
            })
            .refine_style(&self.style)
            .child(label)
    }
}
