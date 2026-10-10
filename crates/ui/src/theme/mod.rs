//! PCL 配色主题：把调色板写进 gpui-kit 主题。
//!
//! 颜色数值与 PCL 启动器同源（见 [`Palette`]，字段对应 PCL 的 `ColorBrush*` 资源键），
//! 这里只负责把它们映射到语义角色。

mod palette;

use palette::oklch;
pub use palette::{ColorTheme, Palette, ToneProfile};

use gpui_kit::component::{ActiveTheme as _, Theme, ThemeMode};
use gpui_kit::{App, Hsla, Rgba, px, transparent_black};

/// 状态色的三档亮度（同色相）：静息 / 悬停 / 按下。
///
/// PCL 未给状态色提供固定色值，用主色同一条 OKLCH 曲线按色相角配色；
/// 调用处依次传入红 30°、绿 145°、黄 75°、蓝 240°。
struct StatusShades {
    rest: Hsla,
    hover: Hsla,
    active: Hsla,
}

fn status_shades(hue: f64) -> StatusShades {
    StatusShades {
        rest: oklch(0.62, 0.19, hue, 1.0),
        hover: oklch(0.70, 0.16, hue, 1.0),
        active: oklch(0.55, 0.20, hue, 1.0),
    }
}

/// 把 PCL 调色板写入 gpui-kit 主题。`Theme::update` 会同步 `tokens` 与 base 层的投影。
pub fn install(cx: &mut App, mode: ThemeMode) {
    let palette = Palette::resolve(mode, ColorTheme::CAT_BLUE);
    // 先切到目标模式（这一步会载入该模式的默认主题），再写入 PCL 调色板；
    // 顺序反过来的话 change 会把刚写入的颜色重新盖回默认值。
    Theme::change(mode, None, cx);

    // 红 / 绿 / 黄 / 蓝四组状态色（档位含义与色相角来源见 `status_shades`）。
    let danger = status_shades(30.0);
    let success = status_shades(145.0);
    let warning = status_shades(75.0);
    let info = status_shades(240.0);

    Theme::update(cx, |theme| {
        theme.mode = mode;
        theme.colors.background = palette.background;
        theme.colors.foreground = palette.gray_level(1);
        theme.colors.border = palette.gray_level(6);
        theme.colors.input = palette.gray_level(5);
        theme.colors.caret = palette.gray_level(1);
        theme.colors.ring = palette.color_level(4);
        theme.colors.selection = palette.color_level(6);
        theme.colors.overlay = Hsla::from(Rgba {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.45,
        });

        theme.colors.primary = palette.color_level(3);
        theme.colors.primary_hover = palette.color_level(4);
        theme.colors.primary_active = palette.color_level(2);
        theme.colors.primary_foreground = palette.white;

        theme.colors.secondary = palette.gray_level(7);
        theme.colors.secondary_hover = palette.gray_level(6);
        theme.colors.secondary_active = palette.gray_level(5);
        theme.colors.secondary_foreground = palette.gray_level(1);

        theme.colors.muted = palette.gray_level(7);
        theme.colors.muted_foreground = palette.gray_level(3);
        theme.colors.accent = palette.color_level(7);
        theme.colors.accent_foreground = palette.gray_level(1);
        theme.colors.popover = palette.background;
        theme.colors.popover_foreground = palette.gray_level(1);
        theme.colors.link = palette.color_level(3);
        theme.colors.link_hover = palette.color_level(4);
        theme.colors.link_active = palette.color_level(2);

        theme.colors.danger = danger.rest;
        theme.colors.danger_hover = danger.hover;
        theme.colors.danger_active = danger.active;
        theme.colors.danger_foreground = palette.white;
        theme.colors.success = success.rest;
        theme.colors.success_hover = success.hover;
        theme.colors.success_active = success.active;
        theme.colors.success_foreground = palette.white;
        theme.colors.warning = warning.rest;
        theme.colors.warning_hover = warning.hover;
        theme.colors.warning_active = warning.active;
        theme.colors.warning_foreground = palette.white;
        theme.colors.info = info.rest;
        theme.colors.info_hover = info.hover;
        theme.colors.info_active = info.active;
        theme.colors.info_foreground = palette.white;

        // 按钮：默认按钮是灰底白面，主按钮走主题色，危险按钮走红色。
        theme.colors.button = palette.gray_level(7);
        theme.colors.button_hover = palette.gray_level(6);
        theme.colors.button_active = palette.gray_level(5);
        theme.colors.button_foreground = palette.gray_level(1);
        theme.colors.button_primary = palette.color_level(3);
        theme.colors.button_primary_hover = palette.color_level(4);
        theme.colors.button_primary_active = palette.color_level(2);
        theme.colors.button_primary_foreground = palette.white;
        theme.colors.button_secondary = palette.gray_level(7);
        theme.colors.button_secondary_hover = palette.gray_level(6);
        theme.colors.button_secondary_active = palette.gray_level(5);
        theme.colors.button_secondary_foreground = palette.gray_level(1);
        theme.colors.button_danger = danger.rest;
        theme.colors.button_danger_hover = danger.hover;
        theme.colors.button_danger_active = danger.active;
        theme.colors.button_danger_foreground = palette.white;
        theme.colors.button_info = info.rest;
        theme.colors.button_info_hover = info.hover;
        theme.colors.button_info_active = info.active;
        theme.colors.button_info_foreground = palette.white;
        theme.colors.button_success = success.rest;
        theme.colors.button_success_hover = success.hover;
        theme.colors.button_success_active = success.active;
        theme.colors.button_success_foreground = palette.white;
        theme.colors.button_warning = warning.rest;
        theme.colors.button_warning_hover = warning.hover;
        theme.colors.button_warning_active = warning.active;
        theme.colors.button_warning_foreground = palette.white;

        theme.colors.group_box = palette.transparent_background;
        theme.colors.group_box_foreground = palette.gray_level(1);
        theme.colors.accordion = palette.transparent_background;

        theme.colors.sidebar = palette.background;
        theme.colors.sidebar_foreground = palette.gray_level(1);
        theme.colors.sidebar_accent = palette.color_level(7);
        theme.colors.sidebar_accent_foreground = palette.color_level(2);
        theme.colors.sidebar_border = palette.gray_level(6);
        theme.colors.sidebar_primary = palette.color_level(3);
        theme.colors.sidebar_primary_foreground = palette.white;

        theme.colors.tab_bar = palette.background;
        theme.colors.tab = palette.background;
        theme.colors.tab_foreground = palette.gray_level(1);
        theme.colors.tab_active = palette.color_level(6);
        theme.colors.tab_active_foreground = palette.color_level(2);
        theme.colors.tab_bar_segmented = palette.gray_level(7);

        theme.colors.table = palette.background;
        theme.colors.table_head = palette.gray_level(7);
        theme.colors.table_head_foreground = palette.gray_level(1);
        theme.colors.table_foot = palette.gray_level(7);
        theme.colors.table_foot_foreground = palette.gray_level(1);
        theme.colors.table_hover = palette.color_level(7);
        theme.colors.table_active = palette.color_level(6);
        theme.colors.table_active_border = palette.color_level(3);
        theme.colors.table_even = palette.semi_transparent;
        theme.colors.table_row_border = palette.gray_level(6);

        theme.colors.list = palette.background;
        theme.colors.list_hover = palette.color_level(7);
        theme.colors.list_active = palette.color_level(6);
        theme.colors.list_active_border = palette.color_level(3);
        theme.colors.list_even = palette.semi_transparent;
        theme.colors.list_head = palette.gray_level(7);

        theme.colors.title_bar = palette.color_level(3);
        theme.colors.title_bar_border = palette.color_level(3);
        theme.colors.status_bar = palette.gray_level(7);
        theme.colors.status_bar_border = palette.gray_level(6);
        theme.colors.window_border = palette.gray_level(5);
        theme.colors.drag_border = palette.color_level(4);
        theme.colors.drop_target = palette.color_level(6);

        theme.colors.progress_bar = palette.color_level(3);
        theme.colors.slider_bar = palette.color_level(3);
        theme.colors.slider_thumb = palette.white;
        theme.colors.switch = palette.color_level(3);
        theme.colors.switch_thumb = palette.white;
        theme.colors.skeleton = palette.gray_level(6);
        // PCL 的滚动条轨道是透明的（MyScrollBar 的 Background=Transparent）：
        // 轨道一旦不透明，滚动条出现时就会盖住下面一列内容。
        theme.colors.scrollbar = transparent_black();
        theme.colors.scrollbar_thumb = palette.gray_level(4);
        theme.colors.scrollbar_thumb_hover = palette.gray_level(3);

        theme.colors.chart_1 = palette.color_level(3);
        theme.colors.chart_2 = palette.color_level(4);
        theme.colors.chart_3 = palette.color_level(5);
        theme.colors.chart_4 = palette.color_level(2);
        theme.colors.chart_5 = palette.color_level(6);
        theme.colors.chart_grid = palette.gray_level(6);

        theme.colors.red = danger.rest;
        theme.colors.red_light = danger.hover;
        theme.colors.green = success.rest;
        theme.colors.green_light = success.hover;
        theme.colors.blue = palette.color_level(3);
        theme.colors.blue_light = palette.color_level(6);
        theme.colors.yellow = warning.rest;
        theme.colors.yellow_light = warning.hover;

        theme.radius = px(6.);
        theme.radius_lg = px(8.);
    });
}

/// 当前调色板，供界面代码取用 PCL 档位色（Gray1~8 / Color1~8 等）。
pub fn palette(cx: &App) -> Palette {
    Palette::resolve(cx.theme().mode, ColorTheme::CAT_BLUE)
}
