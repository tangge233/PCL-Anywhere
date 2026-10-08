//! 常用按钮与按钮组：文案 + 配色。
//!
//! 对话框核心不设默认按钮，这里给的是「常用外观」而不是默认值：调用方仍要显式把按钮交给
//! 请求，并自己决定它返回什么。
//!
//! `Text` 与 `ButtonColor` 都是编译期常量，所以单按钮与按钮组是 `const`。整套 `DialogButton`
//! 做不了 `static`：它持有作答闭包（`Rc`），而 `static` 要求 `Sync`；组装是一行 `.value(..)`。

use super::request::ButtonStyle;
use crate::components::ButtonColor;
use crate::i18n::Text;

/// 默认标题。
pub const DEFAULT_TITLE: Text = Text::key("Common.Dialog.Title");
/// 警告标题。
pub const WARNING_TITLE: Text = Text::key("Common.Dialog.Warning");
/// 错误标题。
pub const ERROR_TITLE: Text = Text::key("SystemDialog.Error.Title");

pub const CONFIRM: ButtonStyle = ButtonStyle::new(Text::key("Common.Action.Confirm"));
pub const CANCEL: ButtonStyle = ButtonStyle::new(Text::key("Common.Action.Cancel"));
pub const YES: ButtonStyle = ButtonStyle::new(Text::key("Common.Option.Yes"));
pub const NO: ButtonStyle = ButtonStyle::new(Text::key("Common.Option.No"));
pub const CONTINUE: ButtonStyle = ButtonStyle::new(Text::key("Common.Action.Continue"));
pub const CLOSE: ButtonStyle = ButtonStyle::new(Text::key("Common.Action.Close"));
pub const GOT_IT: ButtonStyle = ButtonStyle::new(Text::key("Common.Action.GotIt"));
pub const COPY: ButtonStyle = ButtonStyle::new(Text::key("Common.Action.Copy"));
pub const OPEN: ButtonStyle = ButtonStyle::new(Text::key("Common.Action.Open"));
pub const OPEN_FOLDER: ButtonStyle = ButtonStyle::new(Text::key("Common.Action.OpenFolder"));
pub const OVERWRITE: ButtonStyle = ButtonStyle::new(Text::key("Common.Action.Overwrite"));
pub const REFRESH: ButtonStyle = ButtonStyle::new(Text::key("Common.Action.Refresh"));
pub const RESET: ButtonStyle = ButtonStyle::new(Text::key("Common.Action.Reset"));
pub const INSTALL: ButtonStyle = ButtonStyle::new(Text::key("Common.Action.Install"));
pub const DOWNLOAD: ButtonStyle = ButtonStyle::new(Text::key("Common.Action.Download"));
pub const DO_NOT_SHOW_AGAIN: ButtonStyle =
    ButtonStyle::new(Text::key("Common.Hint.DoNotShowAgain"));
/// 破坏性操作：红色描边（PCL 的 `MyButton.ColorState.Red`）。
pub const DELETE: ButtonStyle =
    ButtonStyle::with_color(Text::key("Common.Action.Delete"), ButtonColor::Red);

/// 确定 / 取消。
pub const OK_CANCEL: [ButtonStyle; 2] = [CONFIRM, CANCEL];
/// 是 / 否。
pub const YES_NO: [ButtonStyle; 2] = [YES, NO];
/// 是 / 否 / 取消。
pub const YES_NO_CANCEL: [ButtonStyle; 3] = [YES, NO, CANCEL];
