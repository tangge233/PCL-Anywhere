//! 对话框按钮：外观与点击时的取值。

use std::rc::Rc;

use gpui_kit::*;

use crate::components::{ButtonColor, ClickHandler};
use crate::i18n::Text;

/// 按钮外观：文案与配色。配色为空时由框架按位置与主题定。
#[derive(Clone, Debug)]
pub struct ButtonStyle {
    pub label: Text,
    pub color: Option<ButtonColor>,
}

impl ButtonStyle {
    pub const fn new(label: Text) -> Self {
        Self { label, color: None }
    }

    /// 带配色的外观；`const`，供 [`super::buttons`] 的常量表使用。
    pub const fn with_color(label: Text, color: ButtonColor) -> Self {
        Self {
            label,
            color: Some(color),
        }
    }
}

impl From<Text> for ButtonStyle {
    fn from(label: Text) -> Self {
        Self::new(label)
    }
}

impl From<SharedString> for ButtonStyle {
    fn from(label: SharedString) -> Self {
        Self::new(Text::Literal(label))
    }
}

pub(crate) type ValueFn<R> = Rc<dyn Fn(&mut Window, &mut App) -> Option<R>>;
pub(crate) type DisabledFn = Rc<dyn Fn(&mut Window, &mut App) -> bool>;

/// 对话框按钮。
pub struct DialogButton<R> {
    pub(crate) style: ButtonStyle,
    pub(crate) value: Option<ValueFn<R>>,
    pub(crate) on_click: Option<ClickHandler>,
    pub(crate) disabled: Option<DisabledFn>,
}

impl<R: 'static> DialogButton<R> {
    /// 用常用按钮（[`super::buttons`]）或一段文案建按钮。
    pub fn new(style: impl Into<ButtonStyle>) -> Self {
        Self {
            style: style.into(),
            value: None,
            on_click: None,
            disabled: None,
        }
    }

    /// 点击时固定返回 `value`。
    pub fn value(self, value: R) -> Self
    where
        R: Clone,
    {
        self.value_with(move |_, _| Some(value.clone()))
    }

    /// 点击时求值：`None` 表示这次点击不作答，弹窗保持打开（例如内容还没准备好）。
    pub fn value_with(self, value: impl Fn(&mut Window, &mut App) -> Option<R> + 'static) -> Self {
        Self {
            value: Some(Rc::new(value)),
            ..self
        }
    }

    /// 先执行的副作用，不影响是否作答（「复制详情」这类按钮：做完事继续留着）。
    pub fn on_click(self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        Self {
            on_click: Some(Rc::new(handler)),
            ..self
        }
    }

    /// 覆盖常用按钮的配色。
    pub fn color(mut self, color: ButtonColor) -> Self {
        self.style.color = Some(color);
        self
    }

    /// 始终禁用。
    pub fn disabled(self, disabled: bool) -> Self {
        self.disabled_when(move |_, _| disabled)
    }

    /// 每帧求值的禁用条件：随内容状态变化（例如选择列表还没选中）。
    pub fn disabled_when(self, disabled: impl Fn(&mut Window, &mut App) -> bool + 'static) -> Self {
        Self {
            disabled: Some(Rc::new(disabled)),
            ..self
        }
    }
}
