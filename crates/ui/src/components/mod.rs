//! 应用自有的界面组件。
//!
//! 这里只放「PCL 的界面语言」需要、而 gpui-kit 通用组件不提供的构件：卡片、选择栏、
//! 页面滚动容器与空状态。通用控件（按钮、输入框、下拉、开关、对话框等）一律直接用
//! `gpui_kit::component` 的组件，不要在这里复制一份。

pub mod button;
pub mod card;
pub mod check;
pub mod icon_button;
pub mod page;
pub mod selector;

pub use button::{AppButton, ButtonColor};
pub use icon_button::{IconButton, IconButtonTheme};

use gpui_kit::{App, ClickEvent, Window};
use std::rc::Rc;

/// 点击回调：与 gpui-kit 的组件一致，带回点击事件。
pub(crate) type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;
pub use card::Card;
pub use check::{AppCheckBox, AppRadio};
pub use page::{EmptyState, PagePlaceholder, PageScroll, SectionLabel, content_width};
pub use selector::{Selector, SelectorItem};

use gpui_kit::component::Icon;
use gpui_kit::*;

/// 按 lucide 文件名取图标，与 .NET 版本中的 `lucide/xxx` 图标名一一对应。
///
/// 应用注册的是 `gpui_kit::assets::AllAssets`（完整 lucide 图标集），因此 PCL 用到的图标都能按名取到。
pub fn lucide(name: &str) -> Icon {
    Icon::default().path(SharedString::from(format!("icons/{name}.svg")))
}
