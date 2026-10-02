//! 三栏合并页（对应 `PageInstanceMerged.axaml`）。
//!
//! 状态 1 = 文件夹 + 实例列表，状态 2 = 实例列表 + 管理栏；两态共用一条收起条。
//! 布局与滑动动画见 [`InstanceGroup::render_merged`]。

use gpui_kit::base::animation::cubic_bezier;
use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::input::Input;
use gpui_kit::component::scroll::ScrollableElement as _;
use gpui_kit::prelude::FluentBuilder as _;

use super::state::{SAMPLE_FOLDERS, SAMPLE_INSTANCES, folder_title};
use super::*;
use crate::components::{EmptyState, IconButton, IconButtonTheme, SelectorItem, content_width};
use crate::i18n;
use crate::theme;

impl InstanceGroup {
    pub(super) fn render_merged(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let palette = theme::palette(cx);
        let state = self.column_state;
        // 实例页占满标题栏以下的内容区，栏宽按内容区宽度算（PCL 同样按视图宽度算栏宽）。
        let viewport = content_width(window);
        let folder_w = px(FOLDER_COLUMN_W);
        let bar_w = px(COLLAPSE_BAR_W);
        let instance_w = px(INSTANCE_COLUMN_W);

        // 两态的取值分别作为动画的起点与终点（0 = 状态 1，1 = 状态 2）。
        let end_slide = -folder_w;
        let end_instances = instance_w;
        let end_manage = (viewport - bar_w - instance_w).max(px(0.));
        let start_instances = (viewport - folder_w - bar_w).max(px(160.));
        // 状态 2 时实例栏左侧要让出窄条的位置，否则窄条会压在列表内容上。
        let start_divider = folder_w;
        let end_divider = instance_w;

        // 两态之间插值：`t` 是动画进度，状态 1 对应 0、状态 2 对应 1。
        // 切换时才用 250ms；进入页面用零时长，直接落在目标状态（PCL 的 `animate=false` 分支）。
        let progress = move |t: f32| if state == 2 { t } else { 1.0 - t };
        let animation = if self.animate_slide {
            Animation::new(SLIDE_DURATION)
                // PCL 用 FluentOut（cubic-bezier(0.1, 0.9, 0.2, 1)）。
                .with_easing(cubic_bezier(0.1, 0.9, 0.2, 1.0))
        } else {
            Animation::new(Duration::ZERO)
        };
        let animation = || animation.clone();
        let lerp = |from: Pixels, to: Pixels, t: f32| {
            px(f32::from(from) + (f32::from(to) - f32::from(from)) * t)
        };

        let strip = h_flex()
            .absolute()
            .top_0()
            .h_full()
            // 状态变化时 id 变化，动画从头播放；相同状态之间重绘不会重放。
            .with_animation(
                SharedString::from(format!("instance-slide-{state}")),
                animation(),
                move |this, t| this.left(lerp(px(0.), end_slide, progress(t))),
            )
            .child(self.render_folder_column(cx))
            .child(self.render_instance_column(window, cx).with_animation(
                SharedString::from(format!("instance-column-{state}")),
                animation(),
                move |this, t| {
                    let t = progress(t);
                    this.w(lerp(start_instances, end_instances, t))
                        .pl(lerp(px(0.), bar_w, t))
                },
            ))
            .child(
                self.render_manage_column(cx)
                    .min_w_0()
                    .overflow_hidden()
                    .with_animation(
                        SharedString::from(format!("manage-column-{state}")),
                        animation(),
                        move |this, t| this.w(lerp(px(0.), end_manage, progress(t))),
                    ),
            );

        div()
            .relative()
            .size_full()
            .overflow_hidden()
            .bg(palette.background)
            .child(strip)
            // 分隔线横跨两栏交界，随状态一起移动（XAML 的 `ColumnDivider`）。
            .child(
                div()
                    .absolute()
                    .top_0()
                    .h_full()
                    .w(px(DIVIDER_W))
                    .bg(divider_color(cx))
                    .with_animation(
                        SharedString::from(format!("instance-divider-{state}")),
                        animation(),
                        move |this, t| this.left(lerp(start_divider, end_divider, progress(t))),
                    ),
            )
            .child(self.render_collapse_bar(cx))
            .into_any_element()
    }

    /// 两栏交界处的窄竖条：整条可点，点击在两种栏位之间切换
    /// （对应 XAML 的 `BarRight` / `BarLeft`：浅色底、悬停加深、居中一个尖角）。
    pub(super) fn render_collapse_bar(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let to_manage = self.column_state != 2;
        let (id, icon, tooltip_key) = if to_manage {
            ("bar-expand", "chevron-right", "Instance.Merged.Expand")
        } else {
            ("bar-collapse", "chevron-left", "Instance.Merged.Collapse")
        };
        let accessible = i18n::lang(tooltip_key);
        // 状态 1 的窄条贴右缘、状态 2 贴左缘；两态都占 18px，不覆盖列表内容。
        let bar = div()
            .absolute()
            .top_0()
            .h_full()
            .w(px(COLLAPSE_BAR_W))
            .flex_shrink_0()
            .flex()
            .items_center()
            .justify_center()
            .when(to_manage, |this| this.right_0())
            .when(!to_manage, |this| this.left_0())
            .bg(bar_background(false))
            .hover(|this| this.bg(bar_background(true)))
            // 整条都是命中区域：按钮撑满窄条，悬停/按压的底色仍由窄条自己表达。
            .child(
                IconButton::new(id, icon, accessible.clone())
                    .theme(IconButtonTheme::Plain)
                    .size(px(COLLAPSE_BAR_W))
                    .stretch()
                    .tooltip(accessible)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.toggle_column_state(to_manage, cx);
                    })),
            );

        bar.into_any_element()
    }

    // ---- 第 1 栏：游戏文件夹 ------------------------------------------------

    pub(super) fn render_folder_column(&self, cx: &mut Context<Self>) -> AnyElement {
        v_flex()
            .w(px(FOLDER_COLUMN_W))
            .h_full()
            .min_h_0()
            .flex_shrink_0()
            // 标题行高度与实例栏一致（30 + 4），保证两栏列表首行对齐。
            .child(
                h_flex()
                    .h(px(30.))
                    .mb(px(4.))
                    .px_3()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_xs()
                            .opacity(0.6)
                            .child(i18n::lang("Instance.Folder.Title")),
                    )
                    .child(
                        IconButton::new(
                            "add-folder",
                            "folder-plus",
                            i18n::lang("Instance.Folder.Add"),
                        )
                        .size(px(24.))
                        .tooltip(i18n::lang("Instance.Folder.Add")),
                    ),
            )
            // 与实例栏的搜索行等高，保持两栏列表首行对齐（本栏没有搜索框）。
            .child(div().h(px(32.)).mb(px(6.)))
            .child(div().flex_1().min_h_0().overflow_y_scrollbar().children(
                SAMPLE_FOLDERS.iter().enumerate().map(|(ix, folder)| {
                    let title = folder_title(folder);
                    // 副文本：该目录下的实例数量（对应 XAML 的 MyListItem.Info）。
                    let count = folder.count.to_string();
                    SelectorItem::new(SharedString::from(format!("folder-{ix}")), title)
                        .info(i18n::lang_with_args(
                            "Instance.Folder.InstanceCount",
                            &[&count],
                        ))
                        .icon("folder")
                        .selected(ix == self.selected_folder)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.selected_folder = ix;
                            cx.notify();
                        }))
                }),
            ))
            .into_any_element()
    }

    // ---- 第 2 栏：实例列表 --------------------------------------------------

    /// `flex` 为真时占满剩余宽度（第一状态的右栏），否则取固定宽（第二状态的左栏）。
    /// 实例列表栏。宽度由合并页的滑动动画决定（状态 1 占满剩余宽度、状态 2 为固定宽），
    /// 因此这里只负责内容，不再自己定宽。
    pub(super) fn render_instance_column(
        &self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Div {
        let column = v_flex().h_full().min_h_0().min_w_0().overflow_hidden();

        let query = self.search_query.to_string().to_lowercase();
        let visible: Vec<usize> = SAMPLE_INSTANCES
            .iter()
            .enumerate()
            .filter(|(_, inst)| query.is_empty() || inst.name.to_lowercase().contains(&query))
            .map(|(ix, _)| ix)
            .collect();

        let mut rows: Vec<AnyElement> = Vec::new();
        if visible.is_empty() {
            rows.push(
                div()
                    .px_3()
                    .pt_2()
                    .child(EmptyState::new(i18n::lang("Instance.Folder.Empty")))
                    .into_any_element(),
            );
        } else {
            for (folder_ix, folder) in SAMPLE_FOLDERS.iter().enumerate() {
                let items: Vec<usize> = visible
                    .iter()
                    .copied()
                    .filter(|ix| SAMPLE_INSTANCES[*ix].folder == folder_ix)
                    .collect();
                if items.is_empty() {
                    continue;
                }
                // 分类标题 + 细分隔线（对应 XAML 的 groupTitle / groupSeparatorLine）。
                rows.push(
                    div()
                        .px_3()
                        .pt_2()
                        .child(
                            v_flex()
                                .child(div().text_xs().opacity(0.6).child(folder_title(folder)))
                                .child(
                                    div()
                                        .mt(px(4.))
                                        .h(px(1.))
                                        .bg(theme::palette(cx).gray_level(6)),
                                ),
                        )
                        .into_any_element(),
                );
                for ix in items {
                    let instance = &SAMPLE_INSTANCES[ix];
                    rows.push(
                        SelectorItem::new(
                            SharedString::from(format!("instance-{ix}")),
                            instance.name,
                        )
                        .icon(instance.icon)
                        .selected(Some(ix) == self.selected_instance)
                        // 单击 = 选中该实例（后续的实例操作都基于这个选择）；
                        // 双击 = 进入实例详情（展开管理栏，对应第二态）。
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.select_instance(ix);
                            cx.notify();
                        }))
                        .on_double_click(cx.listener(move |this, _, _, cx| {
                            this.select_instance(ix);
                            this.manage_tab = TAB_OVERVIEW;
                            this.toggle_column_state(true, cx);
                        }))
                        .into_any_element(),
                    );
                }
            }
        }

        column
            .child(
                h_flex().h(px(30.)).mb(px(4.)).px_3().items_center().child(
                    div()
                        .text_xs()
                        .opacity(0.6)
                        .child(i18n::lang("Instance.List.Title")),
                ),
            )
            .child(
                div()
                    .px_3()
                    .mb(px(6.))
                    .child(Input::new(&self.search).h(px(32.))),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scrollbar()
                    .children(rows),
            )
    }

    // ---- 第 3 栏：实例管理（Tab 布局） -------------------------------------
}

/// 收起条底色：静息 12% 黑、悬停 20% 黑（XAML 的 `collapseBar` 样式）。
pub(super) fn bar_background(hovered: bool) -> Hsla {
    Hsla::black().opacity(if hovered { 0.2 } else { 0.12 })
}

pub(super) fn divider_color(cx: &Context<InstanceGroup>) -> Hsla {
    theme::palette(cx).gray_level(1).opacity(0.15)
}
