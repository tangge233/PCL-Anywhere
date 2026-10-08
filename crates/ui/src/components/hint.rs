//! 兼容性提示行构件：一行带档位色图标与文字的说明，用于逐条列出兼容性问题。
//!
//! 只负责单行的渲染（配色、图标、按文案键取文案），不持有状态、不响应交互；
//! 提示条目由调用方给定，分组场景见下载页安装面板的 `HINTS`。

use gpui_kit::base::h_flex;
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::*;

use super::lucide;
use crate::i18n;
use crate::theme;

/// 提示行的档位色：红、黄两档，决定图标与文字的颜色。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HintLevel {
    /// 红档（强提醒）：红色图标与文字，提示必须处理的兼容性问题。
    Red,
    /// 黄档（提示）：黄色图标与文字，提示需要注意但不阻断安装的问题。
    Yellow,
}

/// 按文案键渲染一行提示；`level` 决定档位色，文案在渲染时查表。
pub fn hint_row(key: &str, level: HintLevel, cx: &App) -> AnyElement {
    hint_row_text(i18n::lang(key), level, cx)
}

/// 按已解析的文案渲染一行提示；文案来自运行期（校验结果、错误详情）时用这个。
pub fn hint_row_text(text: SharedString, level: HintLevel, cx: &App) -> AnyElement {
    let palette = theme::palette(cx);
    let (color, icon) = match level {
        HintLevel::Red => (palette.red_light, "triangle-alert"),
        // 黄档取主题黄色，主题里它与警告色同值。
        HintLevel::Yellow => (cx.theme().yellow, "info"),
    };

    h_flex()
        .items_start()
        .gap_2()
        .pt_1()
        .child(lucide(icon).size_4().flex_shrink_0().text_color(color))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .text_xs()
                .text_color(color)
                .child(text),
        )
        .into_any_element()
}
