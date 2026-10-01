//! PCL 的勾选控件（对应 `PCL.Ui/Controls/MyCheckBox.axaml` 与 `MyRadioBox.axaml`）。
//!
//! 两者共用同一套配色规则：静息用正文色描边，悬停转主题色，勾选后用主题深色，
//! 禁用转灰；文字 13px，与控件之间留 8px。控件是受控的：值由持有方保存，
//! 交互通过 `on_change` 请求新值。

use gpui_kit::base::TestSupportExt as _;
use gpui_kit::component::ThemeStyled as _;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use std::rc::Rc;

use super::lucide;
use crate::theme;

/// 勾选框边长（PCL 为 18 × 18）。
const BOX_SIZE: Pixels = px(18.);
/// 单选框直径（PCL 为 20 × 20）。
const RADIO_SIZE: Pixels = px(20.);
const RADIUS: Pixels = px(3.);
/// PCL 的勾选控件文字为 13px（16px 基准下的 0.8125rem）。
const LABEL_SIZE: Rems = rems(0.8125);

type ChangeHandler = Rc<dyn Fn(&bool, &mut Window, &mut App)>;

/// 复选框。
#[derive(IntoElement)]
pub struct AppCheckBox {
    id: ElementId,
    label: Option<SharedString>,
    checked: bool,
    disabled: bool,
    tooltip: Option<SharedString>,
    on_change: Option<ChangeHandler>,
}

impl AppCheckBox {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            checked: false,
            disabled: false,
            tooltip: None,
            on_change: None,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 悬停提示（PCL 的 `ToolTip.Tip`）。
    pub fn tooltip(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    pub fn on_change(mut self, handler: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for AppCheckBox {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let palette = theme::palette(cx);
        let checked = self.checked;
        let disabled = self.disabled;
        let on_change = self.on_change.clone();
        let focus_handle = window
            .use_keyed_state(self.id.clone(), cx, |_, cx| {
                cx.focus_handle().tab_stop(true)
            })
            .read(cx)
            .clone();
        let is_focused = focus_handle.is_focused(window);

        let (border, label_color) = match (disabled, checked) {
            (true, _) => (palette.gray_level(4), palette.gray_level(4)),
            (false, true) => (palette.color_level(2), palette.color_level(1)),
            (false, false) => (palette.gray_level(1), palette.gray_level(1)),
        };

        div()
            .id(self.id)
            .test_support()
            .role(Role::CheckBox)
            .when_some(self.label.clone(), |this, label| this.aria_label(label))
            .aria_toggled(if checked {
                accesskit::Toggled::True
            } else {
                accesskit::Toggled::False
            })
            .flex()
            .items_center()
            .gap_2()
            .h(BOX_SIZE)
            .tab_stop(true)
            .tab_index(0)
            .track_focus(&focus_handle)
            .when(!disabled, |this| {
                this.hover(|this| this.text_color(palette.color_level(3)))
            })
            .when(is_focused, |this| this.focus_ring_style(window, cx))
            .when_some(self.tooltip.clone(), |this, tooltip| {
                this.tooltip(move |window, cx| Tooltip::new(tooltip.clone()).build(window, cx))
            })
            .when_some(on_change.filter(|_| !disabled), |this, handler| {
                let on_key = handler.clone();
                this.on_key_down(move |event, window, cx| {
                    if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                        on_key(&!checked, window, cx);
                    }
                })
                .on_click(move |_, window, cx| handler(&!checked, window, cx))
            })
            .child(
                div()
                    .flex()
                    .flex_shrink_0()
                    .items_center()
                    .justify_center()
                    .size(BOX_SIZE)
                    .rounded(RADIUS)
                    .border_1()
                    .border_color(border)
                    .bg(palette.half_white)
                    .when(checked, |this| {
                        this.child(lucide("check").size_3().text_color(if disabled {
                            palette.gray_level(4)
                        } else {
                            palette.color_level(2)
                        }))
                    }),
            )
            .when_some(self.label.clone(), |this, label| {
                this.child(
                    div()
                        .text_size(LABEL_SIZE)
                        .text_color(label_color)
                        .when(!disabled, |this| {
                            this.hover(|this| this.text_color(palette.color_level(3)))
                        })
                        .child(label),
                )
            })
    }
}

/// 单选框；一组单选框由持有方保存当前选中项。
#[derive(IntoElement)]
pub struct AppRadio {
    id: ElementId,
    label: Option<SharedString>,
    selected: bool,
    disabled: bool,
    tooltip: Option<SharedString>,
    on_change: Option<ChangeHandler>,
}

impl AppRadio {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            selected: false,
            disabled: false,
            tooltip: None,
            on_change: None,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 悬停提示（PCL 的 `ToolTip.Tip`）。
    pub fn tooltip(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    pub fn on_change(mut self, handler: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for AppRadio {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let palette = theme::palette(cx);
        let selected = self.selected;
        let disabled = self.disabled;
        let on_change = self.on_change.clone();
        let focus_handle = window
            .use_keyed_state(self.id.clone(), cx, |_, cx| {
                cx.focus_handle().tab_stop(true)
            })
            .read(cx)
            .clone();
        let is_focused = focus_handle.is_focused(window);

        let (color, label_color) = match (disabled, selected) {
            (true, _) => (palette.gray_level(4), palette.gray_level(4)),
            (false, true) => (palette.color_level(3), palette.color_level(1)),
            (false, false) => (palette.gray_level(1), palette.gray_level(1)),
        };

        div()
            .id(self.id)
            .test_support()
            .role(Role::RadioButton)
            .when_some(self.label.clone(), |this, label| this.aria_label(label))
            .aria_toggled(if selected {
                accesskit::Toggled::True
            } else {
                accesskit::Toggled::False
            })
            .flex()
            .items_center()
            .gap_2()
            .h(RADIO_SIZE)
            .tab_stop(true)
            .tab_index(0)
            .track_focus(&focus_handle)
            .when(!disabled, |this| {
                this.hover(|this| this.text_color(palette.color_level(3)))
            })
            .when(is_focused, |this| this.focus_ring_style(window, cx))
            .when_some(self.tooltip.clone(), |this, tooltip| {
                this.tooltip(move |window, cx| Tooltip::new(tooltip.clone()).build(window, cx))
            })
            .when_some(on_change.filter(|_| !disabled), |this, handler| {
                let on_key = handler.clone();
                this.on_key_down(move |event, window, cx| {
                    if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                        on_key(&true, window, cx);
                    }
                })
                .on_click(move |_, window, cx| handler(&true, window, cx))
            })
            .child(
                div()
                    .flex()
                    .flex_shrink_0()
                    .items_center()
                    .justify_center()
                    .size(RADIO_SIZE)
                    .rounded_full_style(cx)
                    .border_1()
                    .border_color(color)
                    .when(selected, |this| {
                        this.child(div().size_2p5().rounded_full_style(cx).bg(color))
                    }),
            )
            .when_some(self.label.clone(), |this, label| {
                this.child(
                    div()
                        .text_size(LABEL_SIZE)
                        .text_color(label_color)
                        .when(!disabled, |this| {
                            this.hover(|this| this.text_color(palette.color_level(3)))
                        })
                        .child(label),
                )
            })
    }
}
