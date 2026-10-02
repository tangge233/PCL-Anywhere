//! 存档管理页（对应 `PageInstanceSaves.axaml` 与 `PageInstanceSavesSelector.axaml`）。
//!
//! 「300px 存档选择栏 + 存档内容」两栏；内容里的存档设置与管理栏的「存档」Tab 共用同一批控件状态。

use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::select::Select;

use super::data::{InstanceData, InstanceId};
use super::merged::divider_color;
use super::*;
use crate::components::{
    AppButton, AppCheckBox, ButtonColor, Card, EmptyState, PageScroll, Selector, SelectorItem,
};
use crate::i18n;
use crate::theme;
impl InstanceGroup {
    /// 存档管理页：显示主窗口当前实例的存档。
    pub(super) fn render_saves(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Some(id) = self.main_view.instance.clone() else {
            return div().into_any_element();
        };
        // 克隆一份出来用：分片锁不能跨 GPUI 调用持有（见 `data` 模块说明）。
        let data = self.data.get_or_create(&id, window, cx).clone();

        h_flex()
            .items_stretch()
            .size_full()
            .child(self.render_saves_selector(&id, &data, SELECTOR_W, cx))
            .child(div().w(px(DIVIDER_W)).h_full().bg(divider_color(cx)))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .child(self.render_saves_content(&id, &data, cx)),
            )
            .into_any_element()
    }

    /// 存档选择栏：标题 + 存档条目（对应 `PageInstanceSavesSelector.axaml`）。
    pub(super) fn render_saves_selector(
        &self,
        id: &InstanceId,
        data: &InstanceData,
        width: f32,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let mut selector = Selector::new("saves-selector")
            .width(px(width))
            .top_inset(px(0.));

        if SAMPLE_SAVES.is_empty() {
            selector = selector.child(EmptyState::new(i18n::lang("Instance.Saves.Empty.Title")));
        } else {
            for (ix, save) in SAMPLE_SAVES.iter().enumerate() {
                selector = selector.child(
                    SelectorItem::new(SharedString::from(format!("save-{ix}")), save.name)
                        .icon("map")
                        .selected(Some(ix) == data.selected_save)
                        .on_click(cx.listener({
                            let id = id.clone();
                            move |this, _, window, cx| {
                                this.data.get_or_create(&id, window, cx).selected_save = Some(ix);
                                cx.notify();
                            }
                        })),
                );
            }
        }

        v_flex()
            .w(px(width))
            .h_full()
            .min_h_0()
            .flex_shrink_0()
            .child(
                div()
                    .px_3()
                    .pt_3()
                    .pb_1()
                    .text_xs()
                    .opacity(0.6)
                    .child(i18n::lang("Instance.Left.Saves")),
            )
            .child(div().flex_1().min_h_0().child(selector))
            .into_any_element()
    }

    pub(super) fn render_saves_content(
        &mut self,
        id: &InstanceId,
        data: &InstanceData,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Some(save) = data.selected_save.and_then(|i| SAMPLE_SAVES.get(i)) else {
            return v_flex()
                .size_full()
                .items_center()
                .justify_center()
                .child(EmptyState::new(i18n::lang("Instance.Saves.Empty.Title")))
                .into_any_element();
        };

        let palette = theme::palette(cx);
        let difficulty = data.difficulty.clone();

        PageScroll::new("instance-saves")
            .gap(px(15.))
            .child(
                Card::new("saves-details")
                    .title(i18n::lang("Instance.Saves.Info.Details.Title"))
                    .child(
                        v_flex()
                            .px_5()
                            .pt(px(30.))
                            .pb_4()
                            .gap_1()
                            .child(
                                div()
                                    .text_xl()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(palette.color_level(3))
                                    .child(save.name),
                            )
                            .child(save_detail_line(
                                "Instance.Saves.Info.Version",
                                save.version,
                            ))
                            .child(save_detail_line(
                                "Instance.Saves.Info.GameMode",
                                i18n::lang(save.game_mode_key).to_string(),
                            ))
                            .child(save_detail_line(
                                "Instance.Saves.Info.PlayTime",
                                save.play_time,
                            ))
                            .child(save_detail_line(
                                "Instance.Saves.Info.LastPlayed",
                                save.last_played,
                            ))
                            .child(save_detail_line("Instance.Saves.Info.Seed", save.seed))
                            .child(save_detail_line(
                                "Instance.Saves.Info.SpawnPoint",
                                save.spawn,
                            )),
                    ),
            )
            .child(
                Card::new("saves-settings")
                    .title(i18n::lang("Instance.Saves.Info.Settings.Title"))
                    .child(
                        v_flex()
                            .px_5()
                            .pt(px(30.))
                            .pb_4()
                            .gap_3()
                            .child(
                                AppCheckBox::new("saves-allow-commands")
                                    .label(i18n::lang("Instance.Saves.Info.AllowCommands"))
                                    .checked(data.allow_commands)
                                    .on_change(cx.listener({
                                        let id = id.clone();
                                        move |this, checked: &bool, window, cx| {
                                            this.data
                                                .get_or_create(&id, window, cx)
                                                .allow_commands = *checked;
                                            cx.notify();
                                        }
                                    })),
                            )
                            .child(
                                h_flex()
                                    .gap_4()
                                    .items_center()
                                    .child(div().text_sm().child(i18n::lang(
                                        "Instance.Saves.Info.GameDifficultyLabel",
                                    )))
                                    .child(div().w(px(160.)).child(Select::new(&difficulty))),
                            )
                            .child(
                                AppCheckBox::new("saves-lock-difficulty")
                                    .label(i18n::lang("Instance.Saves.Info.LockDifficulty"))
                                    .tooltip(i18n::lang(
                                        "Instance.Saves.Info.LockDifficulty.ToolTip",
                                    ))
                                    .checked(data.lock_difficulty)
                                    .on_change(cx.listener({
                                        let id = id.clone();
                                        move |this, checked: &bool, window, cx| {
                                            this.data
                                                .get_or_create(&id, window, cx)
                                                .lock_difficulty = *checked;
                                            cx.notify();
                                        }
                                    })),
                            ),
                    ),
            )
            .child(
                Card::new("saves-actions")
                    .title(i18n::lang("Instance.Overall.Info.Title"))
                    .child(
                        h_flex()
                            .px_5()
                            .pt(px(32.))
                            .pb_4()
                            .gap_3()
                            .child(
                                AppButton::new(
                                    "saves-open-folder",
                                    i18n::lang("Common.Action.OpenFolder"),
                                )
                                .min_width(px(120.)),
                            )
                            .child(
                                AppButton::new(
                                    "saves-apply",
                                    i18n::lang("Instance.Saves.Info.Modify.BeforeSave"),
                                )
                                .color(ButtonColor::Highlight)
                                .min_width(px(120.)),
                            )
                            .child(
                                AppButton::new(
                                    "saves-refresh",
                                    i18n::lang("Common.Action.Refresh"),
                                )
                                .min_width(px(120.)),
                            )
                            .child(
                                AppButton::new("saves-delete", i18n::lang("Common.Action.Delete"))
                                    .color(ButtonColor::Red)
                                    .min_width(px(120.)),
                            ),
                    ),
            )
            .into_any_element()
    }
}

/// 存档详情的一行：「标签：值」。
pub(super) fn save_detail_line(label_key: &str, value: impl Into<SharedString>) -> AnyElement {
    div()
        .text_sm()
        .opacity(0.8)
        .child(SharedString::from(format!(
            "{}: {}",
            i18n::lang(label_key),
            value.into()
        )))
        .into_any_element()
}
