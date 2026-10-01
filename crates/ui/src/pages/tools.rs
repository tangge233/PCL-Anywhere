//! 工具页分组（对应 `PCL/Views/Tools/*` 与 `PCL/ViewModels/Tools/*`）。
//!
//! 选择栏取自路由目录（联机 / 百宝箱两项），内容区按 [`ToolsRoute`] 分发：
//! - 联机页还原「加入 / 创建 / 大厅信息」三段（`ToolsGameLink.axaml`）；
//! - 测试页还原百宝箱的操作卡片（`ToolsTest.axaml`）。
//!
//! 这里只做界面与界面状态：加入 / 创建 / 退出只切换页面内的「是否在大厅」状态，
//! 不连接 `LobbyService`，也不执行清理缓存、人品、快捷方式等真实动作。

use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::*;

use crate::components::{AppButton, ButtonColor, Card, PageScroll};
use crate::i18n;
use crate::shell::{GroupView, Navigate, Route, route::ToolsRoute};
use crate::theme;

use super::route_selector;

/// 示例数据：大厅服务尚未接入，进入大厅后用这份样例填充编号与成员列表。
const SAMPLE_LOBBY_CODE: &str = "U/1A2B-C3D4-E5F6-7890";
const SAMPLE_PLAYER_LIST: &str = "Steve, Alex";

pub struct ToolsGroup {
    route: Route,
    /// 联机页：大厅编号输入框。
    join_code: Entity<InputState>,
    /// 联机页是否已进入大厅（纯界面状态，对应 `ToolsGameLinkViewModel.IsInLobby`）。
    in_lobby: bool,
}

impl ToolsGroup {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            route: Route::Tools(ToolsRoute::GameLink),
            join_code: cx.new(|cx| {
                InputState::new(window, cx).placeholder(i18n::text("Tools.GameLink.Join.IdHint"))
            }),
            in_lobby: false,
        }
    }

    /// 联机页：未进入大厅时的「加入 / 创建」两张卡片。
    fn render_lobby_entry(&self, cx: &mut Context<Self>) -> AnyElement {
        let clear = cx.listener(|this, _, window, cx| {
            this.join_code
                .update(cx, |state, cx| state.set_value("", window, cx));
            cx.notify();
        });
        let enter = cx.listener(|this, _, _, cx| {
            this.in_lobby = true;
            cx.notify();
        });
        let create = cx.listener(|this, _, _, cx| {
            this.in_lobby = true;
            cx.notify();
        });

        v_flex()
            .gap_4()
            .child(
                // 标题沿用 XAML 的绑定：卡片标题就是操作说明文案。
                Card::new("gamelink-join")
                    .title(i18n::text("Tools.GameLink.Join.Description"))
                    .child(
                        h_flex()
                            .px_5()
                            .pb_4()
                            .gap_2()
                            .items_center()
                            .child(div().flex_1().min_w_0().child(Input::new(&self.join_code)))
                            // MinWidth 50 为控件最小可点宽度，非 rem 档位。
                            .child(
                                AppButton::new(
                                    "gamelink-clear",
                                    i18n::text("Tools.GameLink.Join.Clear"),
                                )
                                .min_width(px(50.))
                                .on_click(clear),
                            )
                            .child(
                                // 剪贴板尚未接入，仅展示按钮（对应原版 PasteCodeCommand）。
                                AppButton::new(
                                    "gamelink-paste",
                                    i18n::text("Tools.GameLink.Join.Paste"),
                                )
                                .min_width(px(50.)),
                            )
                            .child(
                                AppButton::new(
                                    "gamelink-join",
                                    i18n::text("Tools.GameLink.Join.Action"),
                                )
                                .color(ButtonColor::Highlight)
                                .min_width(px(50.))
                                .on_click(enter),
                            ),
                    ),
            )
            .child(
                Card::new("gamelink-create")
                    .title(i18n::text("Tools.GameLink.Create.Description"))
                    .child(
                        h_flex().px_5().pb_4().child(
                            AppButton::new(
                                "gamelink-create-action",
                                i18n::text("Tools.GameLink.Create.Action"),
                            )
                            .min_width(px(120.))
                            .on_click(create),
                        ),
                    ),
            )
            .into_any_element()
    }

    /// 联机页：进入大厅后的「连接状况 / 大厅操作」两张卡片。
    fn render_lobby_inside(&self, cx: &mut Context<Self>) -> AnyElement {
        let palette = theme::palette(cx);
        let leave = cx.listener(|this, _, _, cx| {
            this.in_lobby = false;
            cx.notify();
        });

        v_flex()
            .gap_4()
            .child(
                Card::new("gamelink-status")
                    .title(i18n::text("Tools.GameLink.Finish.ConnectionStatus"))
                    .child(
                        v_flex()
                            .px_5()
                            .pb_4()
                            .gap_2()
                            // 状态行（XAML FontSize 16 / Bold / ColorBrush3）。
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(palette.color_level(3))
                                    .child(i18n::text("Tools.GameLink.Finish.Connected")),
                            )
                            // 大厅编号（XAML FontSize 20 / Bold），无大厅服务时为示例值。
                            .child(
                                div()
                                    .text_xl()
                                    .font_weight(FontWeight::BOLD)
                                    .child(SAMPLE_LOBBY_CODE),
                            )
                            // 成员列表（XAML FontSize 13 / Opacity 0.8 / 自动换行），示例值。
                            .child(div().text_sm().opacity(0.8).child(SAMPLE_PLAYER_LIST)),
                    ),
            )
            .child(
                Card::new("gamelink-actions")
                    .title(i18n::text("Tools.GameLink.Finish.Actions"))
                    .child(
                        h_flex()
                            .flex_wrap()
                            .px_5()
                            .pb_4()
                            .gap_4()
                            // MinWidth 120 为按钮最小宽度。
                            .child(
                                AppButton::new(
                                    "gamelink-copy-id",
                                    i18n::text("Tools.GameLink.Finish.CopyLobbyId"),
                                )
                                .min_width(px(120.)),
                            )
                            .child(
                                AppButton::new(
                                    "gamelink-exit",
                                    i18n::text("Tools.GameLink.Finish.Exit"),
                                )
                                .color(ButtonColor::Red)
                                .min_width(px(120.))
                                .on_click(leave),
                            ),
                    ),
            )
            .into_any_element()
    }

    /// 联机页（`ToolsGameLink.axaml`）。
    fn render_game_link(&self, cx: &mut Context<Self>) -> AnyElement {
        let content = if self.in_lobby {
            self.render_lobby_inside(cx)
        } else {
            self.render_lobby_entry(cx)
        };

        PageScroll::new("tools-gamelink")
            .gap(px(15.))
            .child(content)
            .into_any_element()
    }

    /// 测试页（`ToolsTest.axaml`）：百宝箱的四个操作按钮，行为未接线。
    fn render_test(&self, _: &mut Context<Self>) -> AnyElement {
        PageScroll::new("tools-test")
            .gap(px(15.))
            .child(
                Card::new("tools-test-card")
                    .title(i18n::text("Tools.Test.Title"))
                    .child(
                        h_flex()
                            .flex_wrap()
                            .px_5()
                            .pb_4()
                            .gap_4()
                            .child(
                                AppButton::new("test-clean", i18n::text("Tools.Test.Clean.Title"))
                                    .min_width(px(120.))
                                    .tooltip(i18n::text("Tools.Test.Clean.ToolTip")),
                            )
                            .child(
                                AppButton::new("test-luck", i18n::text("Tools.Test.Luck.Title"))
                                    .min_width(px(100.)),
                            )
                            .child(
                                AppButton::new(
                                    "test-shortcut",
                                    i18n::text("Tools.Test.Shortcut.Title"),
                                )
                                .min_width(px(120.))
                                .tooltip(i18n::text("Tools.Test.Shortcut.ToolTip")),
                            )
                            .child(
                                AppButton::new(
                                    "test-launch-count",
                                    i18n::text("Tools.Test.LaunchCount.Title"),
                                )
                                .min_width(px(120.)),
                            ),
                    ),
            )
            .into_any_element()
    }
}

impl GroupView for ToolsGroup {
    fn set_route(&mut self, route: Route, _: &mut Window, cx: &mut Context<Self>) {
        self.route = route;
        cx.notify();
    }
}

impl EventEmitter<Navigate> for ToolsGroup {}

impl Render for ToolsGroup {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let route = self.route;
        let navigate = cx.listener(|_, route: &Route, _, cx| cx.emit(Navigate(*route)));
        // 目录中联机与百宝箱都没有右侧动作按钮，这里的回调不会触发。
        let action = cx.listener(|_, _: &Route, _, _| {});

        let content = match route {
            Route::Tools(ToolsRoute::GameLink) => self.render_game_link(cx),
            Route::Tools(ToolsRoute::Test) => self.render_test(cx),
            _ => self.render_game_link(cx),
        };

        h_flex()
            .items_stretch()
            .size_full()
            .child(route_selector("tools", route, px(255.), navigate, action))
            .child(div().flex_1().min_w_0().child(content))
    }
}
