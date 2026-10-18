//! 对话框配色与默认标题。

use super::buttons;
use crate::i18n::Text;

/// 对话框配色。警告态（[`DialogTheme::Warning`] 与 [`DialogTheme::Error`]）把标题、主按钮与
/// 遮罩转成红色系。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DialogTheme {
    #[default]
    Info,
    Warning,
    Error,
}

impl DialogTheme {
    pub(crate) fn uses_red(self) -> bool {
        !matches!(self, Self::Info)
    }

    /// 不给标题时的默认标题（对应 PCL `MsgBoxWrapper._ResolveCaption`）。
    pub(crate) fn default_title(self) -> Text {
        match self {
            Self::Info => buttons::DEFAULT_TITLE,
            Self::Warning => buttons::WARNING_TITLE,
            Self::Error => buttons::ERROR_TITLE,
        }
    }
}
