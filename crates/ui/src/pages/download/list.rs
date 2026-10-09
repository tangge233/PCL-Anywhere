//! 版本清单页（对应 PCL 的版本列表：顶部标题行、分类多选按钮组与版本行列表）。
use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::Sizable as _;
use gpui_kit::component::input::Input;
use gpui_kit::component::spinner::Spinner;
use gpui_kit::*;

use super::state::{CATEGORIES, LoadPhase, VERSION_SAMPLES, VersionKind};
use super::*;
use crate::components::{
    Card, GroupButton, GroupButtonItem, IconButton, PageScroll, SectionLabel, StateCard, lucide,
};
use crate::i18n;
use crate::theme;

impl DownloadGroup {
    pub(crate) fn render_minecraft(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        match self.load {
            LoadPhase::Loading => self.render_loading(cx),
            LoadPhase::Ready => match self.panel {
                InstallPanel::List => self.render_version_list(cx),
                InstallPanel::Select => self.render_install_panel(cx),
            },
        }
    }

    /// 居中的加载卡片：加载环 + `Download.Version.LoadingList`。
    fn render_loading(&self, cx: &Context<Self>) -> AnyElement {
        let palette = theme::palette(cx);

        v_flex()
            .size_full()
            .items_center()
            .justify_center()
            .child(
                Card::new("download-loading").child(
                    v_flex()
                        .items_center()
                        .gap_3()
                        .px_8()
                        .py_6()
                        .child(Spinner::new().large())
                        .child(
                            div()
                                .text_sm()
                                .text_color(palette.gray_level(2))
                                .child(i18n::lang("Download.Version.LoadingList")),
                        ),
                ),
            )
            .into_any_element()
    }

    /// 顶部标题行（标题 + 数量 + 搜索框）、版本分类多选按钮组与版本行列表。
    ///
    /// 搜索与分类开关都只作用于显示：被过滤掉的行不渲染，但分类开关的受控状态保留。
    fn render_version_list(&self, cx: &mut Context<Self>) -> AnyElement {
        let palette = theme::palette(cx);
        let query = self.search.read(cx).value().as_str().to_lowercase();
        let enabled = CATEGORIES
            .iter()
            .enumerate()
            .filter(|(index, _)| self.category_on[*index])
            .map(|(_, (_, kind))| *kind)
            .collect::<Vec<VersionKind>>();
        let visible = VERSION_SAMPLES
            .iter()
            .enumerate()
            .filter(|(_, sample)| enabled.contains(&sample.kind))
            .filter(|(_, sample)| query.is_empty() || sample.id.to_lowercase().contains(&query))
            .map(|(index, _)| index)
            .collect::<Vec<usize>>();

        let total = VERSION_SAMPLES.len().to_string();
        let header = h_flex()
            .items_center()
            .justify_between()
            .gap_4()
            .child(
                h_flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::BOLD)
                            .text_color(palette.gray_level(1))
                            .child(i18n::lang("Main.PageDownload.VersionList")),
                    )
                    .child(SectionLabel::new(i18n::lang_with_args(
                        "Download.Version.VersionListCount",
                        &[total.as_str()],
                    ))),
            )
            .child(
                h_flex()
                    .items_center()
                    .gap_2()
                    // 搜索框固定 220px 宽，只过滤下方版本行的显示，不动分类开关与选中集合。
                    .child(div().w(px(220.)).child(Input::new(&self.search).w_full()))
                    .child(self.refresh_button(cx)),
            )
            .into_any_element();

        // 按钮组常驻：分类全关时列表为空，但仍要能点回来。
        let mut children = vec![header, self.category_filters(cx)];

        if visible.is_empty() {
            children.push(
                h_flex()
                    .py_8()
                    .justify_center()
                    .child(StateCard::new(i18n::lang(
                        "Download.Comp.List.NoResultsSimple",
                    )))
                    .into_any_element(),
            );
        } else {
            let rows = visible
                .iter()
                .map(|&index| {
                    let sample = &VERSION_SAMPLES[index];
                    self.version_row(
                        sample.id,
                        index,
                        i18n::lang_with_args("Download.Version.ReleaseDate", &[sample.released]),
                        cx,
                    )
                })
                .collect::<Vec<_>>();
            children.push(
                Card::new("download-version-list")
                    .child(v_flex().px_2().pt_1().pb_2().children(rows))
                    .into_any_element(),
            );
        }

        PageScroll::new("download-versions")
            .gap(px(12.)) // 卡片间距取 12，卡片自带内边距。
            .children(children)
            .into_any_element()
    }

    /// 刷新清单。PCL 把它挂在左栏条目上，这里放内容区（左栏条目不带动作）。
    fn refresh_button(&self, cx: &mut Context<Self>) -> AnyElement {
        let label = i18n::lang("Common.Action.Refresh");
        IconButton::new("download-refresh", "refresh-cw", label.clone())
            .tooltip(label)
            .on_click(cx.listener(|this, _, _, cx| this.reload(cx)))
            .into_any_element()
    }

    /// 版本分类多选按钮组；选中态由 `self.category_on` 持有，关掉的分类不进列表。
    fn category_filters(&self, cx: &mut Context<Self>) -> AnyElement {
        let items = CATEGORIES
            .iter()
            .enumerate()
            .map(|(index, (title_key, _))| {
                GroupButtonItem::new(
                    SharedString::from(format!("download-kind-{index}")),
                    i18n::lang(title_key),
                )
                .selected(self.category_on[index])
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.category_on[index] = !this.category_on[index];
                    cx.notify();
                }))
            })
            .collect::<Vec<_>>();

        Card::new("download-version-filters")
            .child(
                h_flex()
                    .items_center()
                    .px_4()
                    .py_3()
                    .child(GroupButton::new("download-kind-group").children(items)),
            )
            .into_any_element()
    }

    /// 版本列表行：高度 42，图标 + 版本号 + 发布日期，整行可点击进入安装面板。
    /// `slot` 是行的稳定标识：搜索与分类过滤会改变行序，标识必须保持稳定。
    fn version_row(
        &self,
        slot: &str,
        index: usize,
        info: SharedString,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let sample = &VERSION_SAMPLES[index];
        let palette = theme::palette(cx);

        h_flex()
            .id(SharedString::from(format!("download-version-{slot}")))
            .test_support()
            .h(px(42.)) // 版本列表行高 42。
            .px_3()
            .rounded(px(4.))
            .gap_3()
            .cursor_pointer()
            .hover(|this| this.bg(palette.color_level(7)))
            .child(
                lucide(sample.kind.icon())
                    .size_5()
                    .text_color(palette.gray_level(2)),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .child(
                        div()
                            .text_sm()
                            .text_color(palette.gray_level(1))
                            .child(sample.id),
                    )
                    .child(div().text_xs().opacity(0.6).child(info)),
            )
            .on_click(
                cx.listener(move |this, _, window, cx| this.select_version(index, window, cx)),
            )
            .into_any_element()
    }
}
