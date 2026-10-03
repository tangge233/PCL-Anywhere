//! 安装面板（对应 PCL 的安装面板：返回栏、兼容性提示、加载器选择与安装信息）。
use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::input::Input;
use gpui_kit::component::progress::Progress;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::state::{
    HINTS, InstallState, LOADERS, SAMPLE_INSTALL_PROGRESS, VERSION_SAMPLES, VersionSample,
};
use super::*;
use crate::components::{
    AppButton, ButtonColor, Card, IconButton, IconButtonTheme, PageScroll, hint_row, lucide,
};
use crate::i18n;
use crate::theme;

impl DownloadGroup {
    pub(super) fn render_install_panel(&self, cx: &mut Context<Self>) -> AnyElement {
        let selected = self.selected.map(|index| &VERSION_SAMPLES[index]);

        let mut children: Vec<AnyElement> = Vec::new();
        children.push(self.select_header_card(selected, cx));
        for (key, level) in HINTS {
            children.push(hint_row(key, *level, cx));
        }
        children.push(self.loader_cards(cx));
        children.push(self.install_info_card(selected, cx));

        PageScroll::new("download-select")
            .gap(px(12.)) // 与清单区一致；提示行自带间距。
            .children(children)
            .into_any_element()
    }

    /// 安装面板顶部的返回栏：返回按钮 + 版本图标 + 实例名输入框 + 「开始安装」按钮。
    /// 实例名输入框占满剩余宽度，按钮固定在行末。
    fn select_header_card(
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
                    )
                    .child(
                        AppButton::new(
                            "download-start",
                            i18n::lang("Download.Install.StartInstall"),
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

    /// 加载器选择：11 张横向小卡片按容器宽度自适应换行，每张卡显示自身是否已选中。
    fn loader_cards(&self, cx: &Context<Self>) -> AnyElement {
        let mut group = h_flex().items_start().gap_3().flex_wrap();
        for (index, (title_key, block_image)) in LOADERS.iter().enumerate() {
            group = group.child(self.loader_card(index, title_key, block_image, cx));
        }
        group.into_any_element()
    }

    /// 单个加载器小卡片：横向排列的 [18px 方块图 + 竖向排列的（加载器名称、选择情况）]。
    ///
    /// 选择情况是卡片自身状态：本卡被选中时显示「已选择」，否则显示「未选择」。
    /// 点击应在 Dialog 中选择加载器，Dialog 待实现，因此卡片不挂点击回调。
    fn loader_card(
        &self,
        index: usize,
        title_key: &'static str,
        block_image: &str,
        cx: &Context<Self>,
    ) -> AnyElement {
        let palette = theme::palette(cx);
        let status = if self.selected_loader == Some(index) {
            i18n::lang("Download.Install.Loader.Selected")
        } else {
            i18n::lang("Download.Install.Loader.None")
        };

        Card::new(SharedString::from(format!("download-loader-{index}")))
            .child(
                h_flex()
                    .items_center()
                    .gap_2()
                    .px_3()
                    .py_2()
                    .child(img(block_image).w(px(18.)).h(px(18.)).flex_shrink_0())
                    .child(
                        v_flex()
                            // 不能再加 `min_w_0`：它与 Card 内容层的 `min_w_0` 叠加后，外层
                            // 换行行会把卡片高度按「按最小宽度换行」算，高度暴涨。
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(palette.gray_level(1))
                                    .child(i18n::lang(title_key)),
                            )
                            .child(div().text_xs().opacity(0.7).child(status)),
                    ),
            )
            .into_any_element()
    }

    /// 安装信息卡片（`Main.PageDownload.InstallCard`）：版本号、详情、进度与取消按钮。
    fn install_info_card(
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
