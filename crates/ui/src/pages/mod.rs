//! 页面模块。每个分组一个模块，自己拥有左栏与内容区。
//!
//! 左栏与内容的外壳在 [`frame`]：左栏按目录表生成，内容由分组按当前路由给出。
//! 左栏宽度是这一层的常量，设置页（`Settings` 渲染的侧栏）也用它。

use std::ops::Range;

use gpui_kit::{Pixels, px};

pub mod download;
pub mod instance;
pub mod launch;
pub mod setup;
pub mod tools;

pub(crate) mod frame;

pub(crate) use frame::page_frame;

/// 左栏宽度与可拖动范围：三个分组页共用（设置页的 `Settings` 侧栏要显式传）。
pub(crate) const NAV_WIDTH: Pixels = px(220.);
pub(crate) const NAV_SIZE_RANGE: Range<Pixels> = px(160.)..px(360.);
