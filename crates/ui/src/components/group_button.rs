//! 分段按钮组（对应 PCL 的连体分类按钮）：多个分段连成一条，段间以 1px 竖线分隔，
//! 每段独立选中，用于「版本分类筛选」这类多选场景。受控控件——选中态由持有方保存，
//! 交互通过每段的 `on_click` 回报。
//!
//! ```ignore
//! GroupButton::new("kind").children(CATEGORIES.iter().enumerate().map(|(i, (key, _))| {
//!     GroupButtonItem::new(format!("kind-{i}"), lang(key))
//!         .selected(on[i])
//!         .on_click(cx.listener(move |this, _, _, cx| { this.on[i] = !this.on[i]; cx.notify(); }))
//! }))
//! ```

use gpui_kit::base::TestSupportExt as _;
use gpui_kit::component::ThemeStyled as _;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use std::rc::Rc;

use super::{ClickHandler, focus_state, on_enter_space};
use crate::theme;

/// 分段圆角，与按钮控件一致。
const RADIUS: Pixels = px(3.);
/// 分段默认高度，与按钮控件一致。
const DEFAULT_HEIGHT: Pixels = px(30.);
/// 分段文字 13px，与按钮、勾选框一致。
const LABEL_SIZE: Rems = rems(0.8125);

/// 横向连体按钮组；子项为 [`GroupButtonItem`]。
#[derive(IntoElement)]
pub struct GroupButton {
    id: ElementId,
    height: Pixels,
    items: Vec<GroupButtonItem>,
}

impl GroupButton {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            height: DEFAULT_HEIGHT,
            items: Vec::new(),
        }
    }

    /// 分段高度；默认 30，可按 PCL 原值覆盖。
    pub fn height(mut self, height: impl Into<Pixels>) -> Self {
        self.height = height.into();
        self
    }

    pub fn child(mut self, item: GroupButtonItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn children(mut self, items: impl IntoIterator<Item = GroupButtonItem>) -> Self {
        self.items.extend(items);
        self
    }
}

impl RenderOnce for GroupButton {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let palette = theme::palette(cx);
        let border = palette.gray_level(1);
        let last = self.items.len().saturating_sub(1);

        div()
            .id(self.id)
            .flex()
            .items_center()
            .rounded(RADIUS)
            .border_1()
            .border_color(border)
            // 圆角外框内裁剪，避免两端分段的底色溢出到圆角之外。
            .overflow_hidden()
            .children(self.items.into_iter().enumerate().map(|(index, item)| {
                item.height(self.height)
                    .divider(index > 0)
                    .round_left(index == 0)
                    .round_right(index == last)
            }))
    }
}

/// 分段按钮组中的单个分段。
#[derive(IntoElement)]
pub struct GroupButtonItem {
    id: ElementId,
    label: SharedString,
    selected: bool,
    disabled: bool,
    /// 是否在左侧画 1px 分隔竖线（非首段由组统一设置）。
    divider: bool,
    round_left: bool,
    round_right: bool,
    height: Pixels,
    on_click: Option<ClickHandler>,
}

impl GroupButtonItem {
    /// `id` 在组内必须唯一：它是分段的聚焦停靠点的键，重复会互相顶掉。
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            selected: false,
            disabled: false,
            divider: false,
            round_left: false,
            round_right: false,
            height: DEFAULT_HEIGHT,
            on_click: None,
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    fn height(mut self, height: Pixels) -> Self {
        self.height = height;
        self
    }

    fn divider(mut self, divider: bool) -> Self {
        self.divider = divider;
        self
    }

    fn round_left(mut self, round_left: bool) -> Self {
        self.round_left = round_left;
        self
    }

    fn round_right(mut self, round_right: bool) -> Self {
        self.round_right = round_right;
        self
    }
}

impl RenderOnce for GroupButtonItem {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let palette = theme::palette(cx);
        let enabled = !self.disabled;
        let border = palette.gray_level(1);
        // 选中态：主题浅色底 + 主题深色文字；静息态：半透明白底 + 正文色文字。
        let (fill, text) = if self.selected {
            (palette.color_level(7), palette.color_level(2))
        } else {
            (palette.half_white, border)
        };
        let (focus_handle, is_focused) = focus_state(window, self.id.clone(), cx);
        let on_click = self.on_click;

        div()
            .id(self.id)
            .test_support()
            .role(Role::Button)
            .aria_label(self.label.clone())
            .aria_toggled(if self.selected {
                accesskit::Toggled::True
            } else {
                accesskit::Toggled::False
            })
            .flex()
            .items_center()
            .justify_center()
            .h(self.height)
            .px_3()
            .border_color(border)
            .when(self.divider, |this| this.border_l_1())
            .when(self.round_left, |this| this.rounded_l(RADIUS))
            .when(self.round_right, |this| this.rounded_r(RADIUS))
            .bg(fill)
            .text_color(if enabled { text } else { palette.gray_level(4) })
            .text_size(LABEL_SIZE)
            .tab_stop(true)
            .tab_index(0)
            .track_focus(&focus_handle)
            .when(enabled, |this| this.cursor_pointer())
            .when(enabled && !self.selected, |this| {
                this.hover(|this| this.bg(palette.color_level(8)))
            })
            .when(is_focused, |this| this.focus_ring_style(window, cx))
            .when_some(on_click.filter(|_| enabled), |this, handler| {
                this.on_key_down(on_enter_space(handler.clone(), ClickEvent::default))
                    .on_click(move |event, window, cx| handler(event, window, cx))
            })
            .child(self.label.clone())
    }
}
