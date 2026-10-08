//! PCL-Anywhere 的 GPUI 界面层。
//!
//! 依赖关系：应用外壳（`app`）→ 本 crate（界面）→ gpui-kit。
//! 界面层不依赖任何具体业务实现；模型与服务将来放在独立的 crate 中。

pub mod assets;
pub mod components;
pub mod dialog;
pub mod i18n;
pub mod pages;
pub mod shell;
pub mod theme;

pub use shell::{init, open_main_window};
