//! 联机页（对应 PCL 的 `ToolsGameLink`）：未进入大厅的「加入 / 创建」两卡，
//! 大厅内的「连接状况 / 大厅操作」两卡，只读写 [`ToolsGroup`] 的界面状态，不连接 `LobbyService`。

use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::input::Input;
use gpui_kit::*;

use super::ToolsGroup;
use crate::components::{AppButton, ButtonColor, Card, PageScroll};
use crate::i18n;
use crate::theme;

/// 示例数据：大厅服务尚未接入，进入大厅后用这份样例填充编号与成员列表。
const SAMPLE_LOBBY_CODE: &str = "U/1A2B-C3D4-E5F6-7890";
const SAMPLE_PLAYER_LIST: &str = "Steve, Alex";

impl ToolsGroup {
    /// 联机页：未进入大厅时的「加入 / 创建」两张卡片。
    fn render_lobby_entry(&self, cx: &mut Context<Self>) -> AnyElement {
        let clear = cx.listener(|this, _, window, cx| {
            this.join_code
                .update(cx, |state, cx| state.set_value("", window, cx));
            cx.notify();
        });

        v_flex()
            .gap_4()
            .child(
                // 标题沿用 PCL 界面的绑定：卡片标题就是操作说明文案。
                Card::new("gamelink-join")
                    .title(i18n::lang("Tools.GameLink.Join.Description"))
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
                                    i18n::lang("Tools.GameLink.Join.Clear"),
                                )
                                .min_width(px(50.))
                                .on_click(clear),
                            )
                            .child(
                                // 剪贴板尚未接入，仅展示按钮（对应 PCL 的 `PasteCodeCommand`）。
                                AppButton::new(
                                    "gamelink-paste",
                                    i18n::lang("Tools.GameLink.Join.Paste"),
                                )
                                .min_width(px(50.)),
                            )
                            .child(
                                AppButton::new(
                                    "gamelink-join",
                                    i18n::lang("Tools.GameLink.Join.Action"),
                                )
                                .color(ButtonColor::Highlight)
                                .min_width(px(50.))
                                .on_click(cx.listener(|this, _, _, cx| this.enter_lobby(cx))),
                            ),
                    ),
            )
            .child(
                Card::new("gamelink-create")
                    .title(i18n::lang("Tools.GameLink.Create.Description"))
                    .child(
                        h_flex().px_5().pb_4().child(
                            AppButton::new(
                                "gamelink-create-action",
                                i18n::lang("Tools.GameLink.Create.Action"),
                            )
                            .min_width(px(120.))
                            .on_click(cx.listener(|this, _, _, cx| this.enter_lobby(cx))),
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
                    .title(i18n::lang("Tools.GameLink.Finish.ConnectionStatus"))
                    .child(
                        v_flex()
                            .px_5()
                            .pb_4()
                            .gap_2()
                            // 状态行（PCL 界面 FontSize 16 / Bold / ColorBrush3）。
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(palette.color_level(3))
                                    .child(i18n::lang("Tools.GameLink.Finish.Connected")),
                            )
                            // 大厅编号（PCL 界面 FontSize 20 / Bold），无大厅服务时为示例值。
                            .child(
                                div()
                                    .text_xl()
                                    .font_weight(FontWeight::BOLD)
                                    .child(SAMPLE_LOBBY_CODE),
                            )
                            // 成员列表（PCL 界面 FontSize 13 / Opacity 0.8 / 自动换行），示例值。
                            .child(div().text_sm().opacity(0.8).child(SAMPLE_PLAYER_LIST)),
                    ),
            )
            .child(
                Card::new("gamelink-actions")
                    .title(i18n::lang("Tools.GameLink.Finish.Actions"))
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
                                    i18n::lang("Tools.GameLink.Finish.CopyLobbyId"),
                                )
                                .min_width(px(120.)),
                            )
                            .child(
                                AppButton::new(
                                    "gamelink-exit",
                                    i18n::lang("Tools.GameLink.Finish.Exit"),
                                )
                                .color(ButtonColor::Red)
                                .min_width(px(120.))
                                .on_click(leave),
                            ),
                    ),
            )
            .into_any_element()
    }

    /// 联机页（对应 PCL 的 `ToolsGameLink`）。
    pub(crate) fn render_game_link(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let content = if self.in_lobby {
            self.render_lobby_inside(cx)
        } else {
            self.render_lobby_entry(cx)
        };

        PageScroll::new("tools-gamelink")
            .child(content)
            .into_any_element()
    }
}
