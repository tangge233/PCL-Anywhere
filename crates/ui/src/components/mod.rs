//! 应用自有的界面组件。
//!
//! 这里只放为还原「PCL 的界面语言」而自实现的控件：按钮与图标按钮、勾选与单选、
//! 卡片、选择栏、页面滚动容器与状态卡片；其余通用控件（输入框、滑条、设置项、
//! 对话框等）直接用 `gpui_kit::component` 的组件，不要在这里另做一份。

pub mod button;
pub mod card;
pub mod check;
pub mod group_button;
pub mod hint;
pub mod icon_button;
pub mod page;
pub mod selector;
pub mod state_card;

use gpui_kit::component::Icon;
use gpui_kit::*;
use std::rc::Rc;

/// 带载荷的回调（`T` 为载荷类型）：按钮类控件的点击与键盘激活共用。
pub(crate) type Callback<T> = Rc<dyn Fn(&T, &mut Window, &mut App)>;

/// 点击回调：与 gpui-kit 的组件一致，带回点击事件。
pub(crate) type ClickHandler = Callback<ClickEvent>;

pub use button::{AppButton, ButtonColor};
pub use card::Card;
pub use check::{AppCheckBox, AppRadio};
pub use group_button::{GroupButton, GroupButtonItem};
pub use hint::{HintLevel, hint_row};
pub use icon_button::{IconButton, IconButtonTheme};
pub use page::{PagePlaceholder, PageScroll, SectionLabel, content_width};
pub use selector::{Selector, SelectorItem};
pub use state_card::StateCard;

/// 按 lucide 文件名取图标，名称与对应 PCL 界面里 `lucide/xxx` 的图标名一一对应。
///
/// 应用注册的是 `gpui_kit::assets::AllAssets`（完整 lucide 图标集），因此 PCL 用到的图标都能按名取到。
pub fn lucide(name: &str) -> Icon {
    Icon::default().path(SharedString::from(format!("icons/{name}.svg")))
}

/// 键盘激活：Enter / Space 走与点击相同的回调（指针事件不覆盖键盘路径，按钮类控件统一接线）。
///
/// `value` 生成回调载荷：按钮与图标按钮传 `ClickEvent::default`，
/// 复选框传取反后的目标值，单选框传 `true`。
pub(super) fn on_enter_space<T: 'static>(
    handler: Callback<T>,
    value: impl Fn() -> T + 'static,
) -> impl Fn(&KeyDownEvent, &mut Window, &mut App) + 'static {
    move |event, window, cx| {
        if matches!(event.keystroke.key.as_str(), "enter" | "space") {
            handler(&value(), window, cx);
        }
    }
}

/// 取元素的键控焦点句柄与当前是否聚焦；键即元素标识，句柄注册为 Tab 停靠点。
pub(super) fn focus_state(window: &mut Window, id: ElementId, cx: &mut App) -> (FocusHandle, bool) {
    let handle = window
        .use_keyed_state(id, cx, |_, cx| cx.focus_handle().tab_stop(true))
        .read(cx)
        .clone();
    let focused = handle.is_focused(window);
    (handle, focused)
}
