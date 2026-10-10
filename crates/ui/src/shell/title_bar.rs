//! 自绘标题栏（对应 PCL 启动器标题栏的 `PanTitle` 控件，48px 高）。
//!
//! 窗口本身不要求服务端装饰（`WindowDecorations::Client`），因此拖动、双击最大化、
//! 最小化与关闭都由这里提供；PCL 启动器同样是「无边框窗口 + 自绘标题栏」的形态。
//!
//! macOS 例外：最小化与关闭用 AppKit 原生红绿灯，本模块只负责让位（见 `TRAFFIC_LIGHT_*`）。

use gpui_kit::base::InteractiveElementExt as _;
use gpui_kit::base::h_flex;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use std::rc::Rc;

use super::route::{NAV_ITEMS, Route};
use crate::components::{IconButton, IconButtonTheme, lucide};
use crate::i18n;
use crate::theme;

/// 标题栏高度（PCL 为 48px）。红绿灯落点要在 `const` 里算，所以原值留一份 `f32`。
const TITLE_BAR_HEIGHT_PX: f32 = 48.;
const TITLE_BAR_HEIGHT: Pixels = px(TITLE_BAR_HEIGHT_PX);
/// 左上角字标（PCL 的字标是图形，这里用文字排版）。
const WORDMARK: &str = "PCL";

/// macOS 原生红绿灯按钮的高度（AppKit 实测 16px）。
const TRAFFIC_LIGHT_BUTTON_PX: f32 = 16.;

/// macOS 红绿灯落点：`x` 是关闭按钮左边缘距窗口左边缘，`y` 是按钮上下的留白
/// （gpui 把标题栏容器设成「按钮高 + 2y」，`y` 不是中心）。
pub(super) const TRAFFIC_LIGHT_INSET_X: Pixels = px(9.);
pub(super) const TRAFFIC_LIGHT_INSET_Y: Pixels =
    px((TITLE_BAR_HEIGHT_PX - TRAFFIC_LIGHT_BUTTON_PX) / 2.);

/// 红绿灯占掉的左侧宽度（Zed：macOS 26 起 78，更早 71）；取大值，新系统按钮更宽。
const TRAFFIC_LIGHT_PADDING: Pixels = px(78.);

type NavHandler = Rc<dyn Fn(&Route, &mut Window, &mut App)>;
type BackHandler = Rc<dyn Fn(&mut Window, &mut App)>;

#[derive(Default)]
struct DragState {
    dragging: bool,
}

#[derive(IntoElement)]
pub struct TitleBar {
    route: Route,
    /// 副页面标题（由外壳按当前栏位解析后传入）。
    title: Option<SharedString>,
    on_navigate: NavHandler,
    on_back: BackHandler,
}

impl TitleBar {
    pub fn new(
        route: Route,
        title: Option<SharedString>,
        on_navigate: impl Fn(&Route, &mut Window, &mut App) + 'static,
        on_back: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            route,
            title,
            on_navigate: Rc::new(on_navigate),
            on_back: Rc::new(on_back),
        }
    }

    fn render_nav(&self, cx: &mut App) -> AnyElement {
        let palette = theme::palette(cx);
        let active_group = self.route.group();
        let on_navigate = self.on_navigate.clone();

        h_flex()
            .gap_1()
            .children(NAV_ITEMS.iter().map(|item| {
                let selected = item.route.group() == active_group;
                let route = item.route;
                let on_navigate = on_navigate.clone();

                h_flex()
                    .id(SharedString::from(format!("nav-{}", item.title_key)))
                    // 主导航胶囊高度与 PCL 的 `PanTitleSelect` 控件一致（Height=27）。
                    .h(px(27.))
                    .px_3()
                    .gap_2()
                    .rounded_full()
                    .bg(if selected {
                        palette.white
                    } else {
                        palette.semi_transparent
                    })
                    .text_color(if selected {
                        palette.color_level(3)
                    } else {
                        palette.white
                    })
                    // 悬停与按下改用「选中后不再变色」的次序，与 PCL 的按钮一致。
                    .when(!selected, |this| {
                        this.hover(|this| this.bg(palette.half_white))
                            .active(|this| this.bg(palette.semi_white))
                    })
                    .child(lucide(item.icon).size_4())
                    .child(div().text_base().child(i18n::lang(item.title_key)))
                    .on_click(move |_, window, cx| on_navigate(&route, window, cx))
                    .into_any_element()
            }))
            .into_any_element()
    }

    fn render_sub_title(&self) -> AnyElement {
        let on_back = self.on_back.clone();
        let title = self.title.clone().unwrap_or_default();

        h_flex()
            .gap_3()
            .child(
                IconButton::new("nav-back", "arrow-left", i18n::lang("Common.Action.Back"))
                    .theme(IconButtonTheme::OnDark)
                    .consume_mouse_down()
                    .on_click(move |_, window, cx| on_back(window, cx)),
            )
            .child(div().max_w(px(420.)).truncate().text_lg().child(title))
            .into_any_element()
    }

    fn render_window_buttons(&self) -> AnyElement {
        // macOS 用系统红绿灯，不画第二套。
        if cfg!(target_os = "macos") {
            return div().into_any_element();
        }

        h_flex()
            .gap_2()
            .child(
                IconButton::new("title-min", "minus", i18n::lang("Common.Action.Minimize"))
                    .theme(IconButtonTheme::OnDark)
                    .consume_mouse_down()
                    .tooltip(i18n::lang("Common.Action.Minimize"))
                    .on_click(|_, window, _| window.minimize_window()),
            )
            .child(
                IconButton::new("title-close", "x", i18n::lang("Common.Action.Close"))
                    .theme(IconButtonTheme::OnDark)
                    .consume_mouse_down()
                    .tooltip(i18n::lang("Common.Action.Close"))
                    .on_click(|_, window, _| window.remove_window()),
            )
            .into_any_element()
    }
}

impl RenderOnce for TitleBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let palette = theme::palette(cx);
        let state = window.use_state(cx, |_, _| DragState::default());

        let left = if self.route.is_sub() {
            self.render_sub_title()
        } else if cfg!(target_os = "macos") {
            // 左侧让给红绿灯，字标放不下。
            div().into_any_element()
        } else {
            // 字标用文字绘制：PCL 的字标是矢量字形，这里按同样的位置与大小排版，
            // 避免矢量路径在窗口缩放/切边时被裁掉。
            h_flex()
                .flex_shrink_0()
                .text_xl()
                .font_weight(FontWeight::BOLD)
                .text_color(palette.white)
                .child(WORDMARK)
                .into_any_element()
        };

        let center = if self.route.is_sub() {
            div().flex_1().into_any_element()
        } else {
            self.render_nav(cx)
        };

        h_flex()
            .id("title-bar")
            .h(TITLE_BAR_HEIGHT)
            .w_full()
            .flex_shrink_0()
            .px_3()
            // macOS 左侧让给红绿灯、右侧留同样宽度，主导航才落在窗口正中；
            // 全屏时标题栏归系统、灯不在自绘栏里，留白也跟着去掉。
            .when(
                cfg!(target_os = "macos") && !window.is_fullscreen(),
                |this| this.px(TRAFFIC_LIGHT_PADDING),
            )
            .gap_3()
            .items_center()
            .bg(palette.color_level(3))
            .text_color(palette.white)
            // macOS 交给系统偏好（`AppleActionOnDoubleClick`），其余平台一律最大化。
            .on_double_click(|_, window, _| {
                #[cfg(target_os = "macos")]
                window.titlebar_double_click();
                #[cfg(not(target_os = "macos"))]
                window.zoom_window();
            })
            .on_mouse_down_out(window.listener_for(&state, |state, _, _, _| state.dragging = false))
            .on_mouse_down(
                MouseButton::Left,
                window.listener_for(&state, |state, _, _, _| state.dragging = true),
            )
            .on_mouse_up(
                MouseButton::Left,
                window.listener_for(&state, |state, _, _, _| state.dragging = false),
            )
            .on_mouse_move(window.listener_for(&state, |state, _, window, _| {
                if state.dragging {
                    state.dragging = false;
                    window.start_window_move();
                }
            }))
            .child(left)
            // 左右等宽的占位让主导航居中，与 PCL 的 `1*,Auto,1*` 栅格一致。
            .child(div().flex_1())
            .child(center)
            .child(div().flex_1())
            .child(self.render_window_buttons())
    }
}

#[cfg(test)]
mod tests {
    // 按名字导入：`use super::*` 会连带 `gpui_kit::*` 里的 `test` 属性宏，遮住内置 `#[test]`。
    use super::{TITLE_BAR_HEIGHT, TRAFFIC_LIGHT_INSET_X, TRAFFIC_LIGHT_INSET_Y};
    use gpui_kit::px;

    /// macOS 上没有自动化检查，这三个数字就是红绿灯位置的全部依据。
    #[test]
    fn traffic_lights_sit_inside_the_title_bar() {
        assert_eq!(TITLE_BAR_HEIGHT, px(48.));
        assert_eq!(TRAFFIC_LIGHT_INSET_Y, px(16.));
        assert_eq!(TRAFFIC_LIGHT_INSET_X, px(9.));
    }
}
