//! 管理栏：Tab 栏、各 Tab 的内容分派，以及管理栏弹出去后主窗口的提示页。
//!
//! 对应 `PageInstanceManage.axaml`。概览、设置、本地资源三个 Tab 的内容在 [`overview`]、
//! [`settings`]、[`resources`]；存档 Tab 复用 [`super::saves`] 的选择栏与内容（与存档页共用）。

use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::tab::{Tab, TabBar};
use gpui_kit::prelude::FluentBuilder as _;

use super::data::InstanceData;
use super::merged::divider_color;
use super::view::ManageView;
use super::*;
use crate::components::{AppButton, Card, IconButton, StateCard};
use crate::i18n;

/// 管理栏 Tab 文案键（顺序与 XAML 的 TabItem 一致）。
pub(super) const MANAGE_TABS: [&str; 7] = [
    "Instance.Left.Overview",
    "Instance.Left.Settings",
    "Instance.Left.Saves",
    "Instance.Left.Mod",
    "Instance.Left.ResourcePacks",
    "Instance.Left.Shaders",
    "Instance.Left.Schematics",
];

mod overview;
mod resources;
mod settings;

impl InstanceGroup {
    /// 管理栏：Tab 栏 + 当前 Tab 的内容。主窗口与独立窗口各传各的视图。
    pub(super) fn render_manage_column(
        &mut self,
        view: ManageView,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Div {
        let Some(id) = self.view_instance(&view) else {
            return v_flex()
                .size_full()
                .items_center()
                .justify_center()
                .child(StateCard::new(i18n::lang("Instance.Manage.SelectHint")));
        };
        let tab = self.view_tab(&view);
        // 克隆一份出来用：分片锁不能跨 GPUI 调用持有（见 `data` 模块说明）。
        let data = self.data.get_or_create(&id, window, cx).clone();

        let tab_bar = TabBar::new("instance-manage-tabs")
            .underline()
            .selected_index(tab)
            .children(
                MANAGE_TABS
                    .iter()
                    .map(|key| Tab::new().label(i18n::lang(key))),
            )
            // 弹出按钮只在主窗口：独立窗口本身就是被弹出去的那一份，关掉窗口即可收回。
            .when(matches!(view, ManageView::Main), |this| {
                this.suffix(
                    IconButton::new(
                        "manage-pop-out",
                        "square-arrow-out-up-right",
                        i18n::lang("Instance.Manage.PopOut"),
                    )
                    .size(px(24.))
                    .tooltip(i18n::lang("Instance.Manage.PopOut"))
                    .on_click(
                        cx.listener(|this, _, window, cx| this.pop_out_manage_page(window, cx)),
                    ),
                )
            })
            .on_click(
                cx.listener(move |this, ix: &usize, _, cx| this.set_view_tab(&view, *ix, cx)),
            );

        let content = match tab {
            TAB_OVERVIEW => self.render_overview(&id, cx),
            TAB_SETTINGS => self.render_setup(&id, &data, cx),
            TAB_SAVES => self.render_saves_tab(&id, &data, cx),
            TAB_MOD | TAB_RESOURCE_PACKS | TAB_SHADERS | TAB_SCHEMATICS => self.render_resources(),
            _ => div().into_any_element(),
        };

        v_flex()
            .size_full()
            .min_h_0()
            .child(div().px_3().pt_2().child(tab_bar))
            .child(div().flex_1().min_h_0().child(content))
    }

    /// 某个实例的管理栏弹到独立窗口后，主窗口切到它时显示这个提示页。
    pub(super) fn render_manage_popped_hint(
        &mut self,
        id: &InstanceId,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        v_flex()
            .size_full()
            .items_center()
            .justify_center()
            .p_5()
            .child(
                StateCard::new(i18n::lang("Instance.Manage.Popped.Title"))
                    .description(i18n::lang("Instance.Manage.Popped.Description"))
                    .child(
                        h_flex()
                            .p_4()
                            .gap_3()
                            .child(
                                AppButton::new(
                                    "manage-restore",
                                    i18n::lang("Instance.Manage.Popped.Restore"),
                                )
                                .icon("square-arrow-out-down-left")
                                .on_click(cx.listener({
                                    let id = id.clone();
                                    move |this, _, _, cx| this.retract_manage_page(&id, cx)
                                })),
                            )
                            .child(
                                AppButton::new(
                                    "manage-focus",
                                    i18n::lang("Instance.Manage.Popped.Focus"),
                                )
                                .on_click(cx.listener({
                                    let id = id.clone();
                                    move |this, _, _, cx| this.focus_manage_window(&id, cx)
                                })),
                            ),
                    ),
            )
            .into_any_element()
    }

    /// 管理栏 · 存档：150px 存档选择栏 + 存档内容。
    pub(super) fn render_saves_tab(
        &mut self,
        id: &InstanceId,
        data: &InstanceData,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        h_flex()
            .size_full()
            .min_h_0()
            .child(self.render_saves_selector(id, data, MANAGE_SAVES_SELECTOR_W, cx))
            .child(div().w(px(DIVIDER_W)).h_full().bg(divider_color(cx)))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .child(self.render_saves_content(id, data, cx)),
            )
            .into_any_element()
    }
}
