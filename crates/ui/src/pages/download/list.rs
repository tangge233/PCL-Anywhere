//! 版本清单页（对应 `PageDownloadInstall.axaml` 的 `PanMinecraft`：搜索栏、最新版卡片与分类卡片）。
use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::Sizable as _;
use gpui_kit::component::input::Input;
use gpui_kit::component::spinner::Spinner;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use std::rc::Rc;

use super::state::{CATEGORIES, CardSpec, LoadPhase, VERSION_SAMPLES, VersionKind};
use super::*;
use crate::components::{Card, PageScroll, SectionLabel, StateCard, lucide};
use crate::i18n;
use crate::theme;

impl DownloadGroup {
    pub(super) fn render_minecraft(&self, cx: &mut Context<Self>) -> AnyElement {
        match self.load {
            LoadPhase::Loading => self.render_loading(cx),
            LoadPhase::Ready => match self.panel {
                InstallPanel::List => self.render_version_list(cx),
                InstallPanel::Select => self.render_install_panel(cx),
            },
        }
    }

    /// `PanLoad`：居中的加载卡片（原版 `MyLoading` + `Download.Version.LoadingList`）。
    pub(super) fn render_loading(&self, cx: &Context<Self>) -> AnyElement {
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

    /// `PanMinecraft`：顶部的清单筛选 + 「最新版本」卡片 + 三个版本分类卡片。
    pub(super) fn render_version_list(&self, cx: &mut Context<Self>) -> AnyElement {
        let palette = theme::palette(cx);
        // 筛选：.NET 版的清单区没有筛选栏，这里用搜索框按版本号过滤分类卡片（示例数据）。
        let query = self.search.read(cx).value().as_str().to_lowercase();
        let matching = |kind: VersionKind| -> Vec<usize> {
            VERSION_SAMPLES
                .iter()
                .enumerate()
                .filter(|(_, sample)| sample.kind == kind)
                .filter(|(_, sample)| query.is_empty() || sample.id.to_lowercase().contains(&query))
                .map(|(index, _)| index)
                .collect()
        };

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
            .child(div().w(px(220.)).child(Input::new(&self.search).w_full()))
            .into_any_element();

        let mut cards: Vec<AnyElement> = Vec::new();

        // 最新版本卡片（`_AddLatestVersionCard`）：固定显示最新正式版与最新预览版，搜索时隐藏。
        if query.is_empty() {
            let mut rows = Vec::new();
            if let Some(index) = matching(VersionKind::Release).first().copied() {
                rows.push(self.version_row(
                    "latest-release",
                    index,
                    i18n::lang_with_args(
                        "Download.Version.Latest.Release",
                        &[VERSION_SAMPLES[index].released],
                    ),
                    cx,
                ));
            }
            if let Some(index) = matching(VersionKind::Snapshot).first().copied() {
                rows.push(self.version_row(
                    "latest-snapshot",
                    index,
                    i18n::lang_with_args(
                        "Download.Version.Latest.Development",
                        &[VERSION_SAMPLES[index].released],
                    ),
                    cx,
                ));
            }
            if !rows.is_empty() {
                cards.push(self.version_card(
                    CardSpec {
                        id: SharedString::from("download-card-latest"),
                        title: i18n::lang("Download.Version.Latest.Title"),
                        icon: None,
                        expanded: true,
                        toggle: None,
                    },
                    rows,
                    cx,
                ));
            }
        }

        // 分类卡片（`_AddCategoryCard`）：标题带版本数，折叠态只显示标题。
        for (index, (title_key, kind)) in CATEGORIES.iter().enumerate() {
            let versions = matching(*kind);
            if versions.is_empty() {
                continue;
            }
            let title = format!("{} ({})", i18n::lang(title_key).as_str(), versions.len());
            let rows = versions
                .iter()
                .enumerate()
                .map(|(row, version)| {
                    self.version_row(
                        &format!("cat-{index}-{row}"),
                        *version,
                        i18n::lang_with_args(
                            "Download.Version.ReleaseDate",
                            &[VERSION_SAMPLES[*version].released],
                        ),
                        cx,
                    )
                })
                .collect::<Vec<_>>();
            cards.push(self.version_card(
                CardSpec {
                    id: SharedString::from(format!("download-card-{index}")),
                    title: SharedString::from(title),
                    icon: None,
                    expanded: self.expanded[index],
                    toggle: Some(Rc::new(cx.listener(move |this, _, _, cx| {
                        this.expanded[index] = !this.expanded[index];
                        cx.notify();
                    }))),
                },
                rows,
                cx,
            ));
        }

        let mut children = vec![header];
        if cards.is_empty() {
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
            children.extend(cards);
        }

        PageScroll::new("download-versions")
            .gap(px(12.)) // PCL 的卡片间距为 15，卡片自带标题内边距，这里取 12。
            .children(children)
            .into_any_element()
    }

    /// PCL 的分类卡片：标题行可点击折叠（`MyCard` 的 `SwapControl` / `IsSwapped`）。
    /// 「最新版本」卡片不折叠，`toggle` 传 `None`。
    /// 版本/加载器卡片（对应 `MyCard`）。
    ///
    /// 展开内容由 `rows` 给出；[`CardSpec::icon`] 是标题左侧的 18px 方块图
    /// （XAML 里每张加载器卡片的 `MyImage`），路径与 .NET 版本的 `avares://PCL.Ui/Assets/…` 同名。
    pub(super) fn version_card(
        &self,
        spec: CardSpec<'_>,
        rows: Vec<AnyElement>,
        cx: &Context<Self>,
    ) -> AnyElement {
        let CardSpec {
            id,
            title,
            icon,
            expanded,
            toggle,
        } = spec;
        let palette = theme::palette(cx);

        let head = h_flex()
            .id(SharedString::from(format!("{id}-head")))
            .items_center()
            .gap_2()
            .px_4()
            .pt_3()
            .pb_2()
            .when_some(icon, |this, icon| {
                this.child(img(icon).w(px(18.)).h(px(18.)).flex_shrink_0())
            })
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .font_weight(FontWeight::BOLD)
                    .text_sm()
                    .text_color(palette.gray_level(1))
                    .child(title),
            )
            .when_some(toggle, |this, toggle| {
                this.cursor_pointer()
                    .child(
                        lucide(if expanded {
                            "chevron-down"
                        } else {
                            "chevron-right"
                        })
                        .size_4()
                        .text_color(palette.gray_level(2)),
                    )
                    .on_click(move |event, window, cx| toggle(event, window, cx))
            });

        v_flex()
            .rounded(cx.theme().radius)
            .bg(palette.transparent_background)
            .shadow_sm()
            .child(head)
            .when(expanded, |this| {
                this.child(v_flex().px_2().pb_2().children(rows))
            })
            .into_any_element()
    }

    /// 版本列表项（对应 `MyListItem`：高度 42、图标 + 版本号 + 副标题、整行可点击）。
    /// `slot` 用来区分同一条数据在不同卡片里的行（例如最新版卡片与稳定版分类都含最新正式版）。
    pub(super) fn version_row(
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
            .h(px(42.)) // 原版 McDownloadListItem 的高度 42
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

pub(super) fn hint_row(key: &str, strong: bool, cx: &App) -> AnyElement {
    let palette = theme::palette(cx);
    let color = if strong {
        // `ColorBrushRed*`。
        palette.red_light
    } else {
        // `ColorBrushYellow*`（主题里 yellow 与 warning 同值）。
        cx.theme().yellow
    };

    h_flex()
        .items_start()
        .gap_2()
        .pt_1()
        .child(
            lucide(if strong { "triangle-alert" } else { "info" })
                .size_4()
                .flex_shrink_0()
                .text_color(color),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .text_xs()
                .text_color(color)
                .child(i18n::lang(key)),
        )
        .into_any_element()
}

/// 未迁移页面的名称：取选择栏条目的文案。
/// （`Route::label` 对 17 个下载子页都是「下载」，区分不开，所以这里查目录。）
pub(super) fn sub_page_name(route: Route) -> SharedString {
    route
        .selector_entries()
        .iter()
        .find(|entry| entry.route == route)
        .map(|entry| i18n::lang(entry.title_key))
        .unwrap_or_else(|| route.label())
}
