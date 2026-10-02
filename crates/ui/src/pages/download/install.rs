//! 安装面板（对应 `PageDownloadInstall.axaml` 的 `PanSelect`：返回栏、兼容性提示、加载器卡片与安装信息）。
use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::input::Input;
use gpui_kit::component::progress::Progress;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use std::rc::Rc;

use super::list::hint_row;
use super::state::{
    CardSpec, HINTS, InstallState, LOADERS, SAMPLE_INSTALL_PROGRESS, VERSION_SAMPLES, VersionSample,
};
use super::*;
use crate::components::{
    AppButton, ButtonColor, Card, IconButton, IconButtonTheme, PageScroll, lucide,
};
use crate::i18n;
use crate::theme;

impl DownloadGroup {
    pub(super) fn render_install_panel(&self, cx: &mut Context<Self>) -> AnyElement {
        let selected = self.selected.map(|index| &VERSION_SAMPLES[index]);

        let mut children: Vec<AnyElement> = Vec::new();
        children.push(self.select_header_card(selected, cx));
        for (key, strong) in HINTS {
            children.push(hint_row(key, *strong, cx));
        }
        for (index, (title_key, block_image)) in LOADERS.iter().enumerate() {
            children.push(self.loader_card(index, title_key, block_image, cx));
        }
        children.push(self.install_info_card(selected, cx));
        // 为底部浮动的「开始下载」留出空间（原版安装面板的下外边距为 45）。
        children.push(div().h(px(48.)).into_any_element());

        let content = PageScroll::new("download-select")
            .gap(px(12.)) // 与清单区一致；提示行自带间距。
            .children(children);

        // 底部居中的「开始下载」（XAML 的 `BtnInstall` 浮在内容之上）。
        div()
            .relative()
            .size_full()
            .child(content)
            .child(
                h_flex()
                    .absolute()
                    .bottom(px(20.))
                    .left(px(0.))
                    .w_full()
                    .justify_center()
                    .child(
                        AppButton::new(
                            "download-start",
                            i18n::lang("Download.Install.StartDownload"),
                        )
                        .color(ButtonColor::Highlight)
                        .icon("download")
                        .min_width(px(160.))
                        .loading(self.install == InstallState::Running)
                        .disabled(selected.is_none() || self.install == InstallState::Running)
                        .on_click(cx.listener(|this, _, _, cx| this.start_install(cx))),
                    ),
            )
            .into_any_element()
    }

    /// 安装面板顶部的返回栏（对应 `MyCard Height=62`：返回按钮 + 版本图标 + 实例名输入框）。
    pub(super) fn select_header_card(
        &self,
        selected: Option<&VersionSample>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let palette = theme::palette(cx);
        let icon = selected.map(|sample| sample.kind.icon()).unwrap_or("boxes");

        Card::new("download-select-head")
            .child(
                h_flex()
                    .items_center()
                    .gap_3()
                    .px_4()
                    .py_3()
                    .child(
                        IconButton::new(
                            "download-back",
                            "arrow-left",
                            i18n::lang("Common.Action.Back"),
                        )
                        .theme(IconButtonTheme::Black)
                        .size(px(26.)) // 原版 BtnBack 为 26 × 26
                        .tooltip(i18n::lang("Common.Action.Back"))
                        .on_click(cx.listener(|this, _, _, cx| this.exit_select(cx))),
                    )
                    // 原版 ImgLogo 高 32。
                    .child(lucide(icon).size_8().text_color(palette.gray_level(2)))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .child(Input::new(&self.instance_name)),
                    ),
            )
            .into_any_element()
    }

    /// 加载器卡片（XAML 里 11 张 `Height=40`、`IsSwapped=True` 的折叠卡片）。
    /// .NET 版的卡片内容（`PanForge` 等）由尚未迁移的加载器清单填充，
    /// 所以这里展开后只显示列表服务的空状态文案。
    pub(super) fn loader_card(
        &self,
        index: usize,
        title_key: &'static str,
        block_image: &str,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let rows = vec![
            h_flex()
                .items_center()
                .px_3()
                .py_2()
                .child(
                    div()
                        .text_xs()
                        .opacity(0.7)
                        .child(i18n::lang("Download.Install.State.NoVersion")),
                )
                .into_any_element(),
        ];

        self.version_card(
            CardSpec {
                id: SharedString::from(format!("download-loader-{index}")),
                title: i18n::lang(title_key),
                icon: Some(block_image),
                expanded: self.loader_expanded[index],
                toggle: Some(Rc::new(cx.listener(move |this, _, _, cx| {
                    this.loader_expanded[index] = !this.loader_expanded[index];
                    cx.notify();
                }))),
            },
            rows,
            cx,
        )
    }

    /// 安装信息卡片（`Main.PageDownload.InstallCard`）：版本号、详情、进度与取消按钮。
    pub(super) fn install_info_card(
        &self,
        selected: Option<&VersionSample>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let palette = theme::palette(cx);
        let id = selected
            .map(|sample| SharedString::from(sample.id))
            .unwrap_or_else(|| i18n::lang("Main.PageDownload.NoSelection"));
        let detail = selected
            .map(|sample| {
                let label = sample.kind.label();
                i18n::lang_with_args(
                    "Main.PageDownload.VersionDetail",
                    &[sample.released, label.as_str()],
                )
            })
            .unwrap_or_default();
        let progress_text = if self.install == InstallState::Cancelled {
            i18n::lang("Main.PageDownload.InstallCancelled")
        } else {
            i18n::lang("Main.PageDownload.InstallHint")
        };
        let running = self.install == InstallState::Running;

        Card::new("download-install-card")
            .title(i18n::lang("Main.PageDownload.InstallCard"))
            .child(
                v_flex()
                    .px_5()
                    .pb_4()
                    .gap_3()
                    .child(
                        div()
                            .text_2xl()
                            .font_weight(FontWeight::BOLD)
                            .text_color(palette.color_level(3))
                            .child(id),
                    )
                    .child(div().text_sm().opacity(0.8).child(detail))
                    .child(div().text_xs().opacity(0.75).child(progress_text))
                    .when(running, |this| {
                        this.child(
                            Progress::new("download-progress")
                                .value(SAMPLE_INSTALL_PROGRESS)
                                .w_full(),
                        )
                    })
                    .when(running, |this| {
                        this.child(
                            AppButton::new("download-cancel", i18n::lang("Common.Action.Cancel"))
                                .color(ButtonColor::Red)
                                .w_full()
                                .on_click(cx.listener(|this, _, _, cx| this.cancel_install(cx))),
                        )
                    }),
            )
            .into_any_element()
    }
}
