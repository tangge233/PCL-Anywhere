//! 自绘标题栏（对应 `FormMain.axaml` 的 `PanTitle`，48px 高）。
//!
//! 窗口本身不要求服务端装饰（`WindowDecorations::Client`），因此拖动、双击最大化、
//! 最小化与关闭都由这里提供；这也和 .NET 版本「无边框窗口 + 自绘标题栏」的形态一致。

use gpui_kit::base::InteractiveElementExt as _;
use gpui_kit::base::h_flex;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use std::rc::Rc;

use super::route::{NAV_ITEMS, Route};
use crate::components::{IconButton, IconButtonTheme, lucide};
use crate::i18n;
use crate::theme;

/// 标题栏高度（PCL 为 48px）。
const TITLE_BAR_HEIGHT: Pixels = px(48.);
/// 左上角字标（PCL 的字标是图形，这里用文字排版）。
const WORDMARK: &str = "PCL";

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
                    .child(div().text_base().child(i18n::text(item.title_key)))
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
                IconButton::new("nav-back", "arrow-left", i18n::text("Common.Action.Back"))
                    .theme(IconButtonTheme::White)
                    .consume_mouse_down()
                    .on_click(move |_, window, cx| on_back(window, cx)),
            )
            .child(div().max_w(px(420.)).truncate().text_lg().child(title))
            .into_any_element()
    }

    fn render_window_buttons(&self) -> AnyElement {
        h_flex()
            .gap_2()
            .child(
                IconButton::new("title-min", "minus", SharedString::from("最小化"))
                    .theme(IconButtonTheme::White)
                    .consume_mouse_down()
                    .tooltip(SharedString::from("最小化"))
                    .on_click(|_, window, _| window.minimize_window()),
            )
            .child(
                IconButton::new("title-close", "x", i18n::text("Common.Action.Close"))
                    .theme(IconButtonTheme::White)
                    .consume_mouse_down()
                    .tooltip(i18n::text("Common.Action.Close"))
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
        } else {
            // 字标用文字绘制：PCL 原版是矢量字形，这里按同样的位置与大小排版，
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
            .gap_3()
            .items_center()
            .bg(palette.color_level(3))
            .text_color(palette.white)
            // 双击最大化：与桌面惯例一致（PCL 的标题栏同样允许双击）。
            .on_double_click(|_, window, _| window.zoom_window())
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
