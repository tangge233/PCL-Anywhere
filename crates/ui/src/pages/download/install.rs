//! 安装面板（对应 PCL 的安装面板：返回栏、兼容性提示、加载器选择与安装信息）。
use gpui_kit::base::{TestSupportExt as _, h_flex, v_flex};
use gpui_kit::component::input::Input;
use gpui_kit::component::progress::Progress;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::loader_versions::LoaderVersions;
use super::state::{
    HINTS, InstallState, LOADER_VERSIONS, LOADERS, SAMPLE_INSTALL_PROGRESS, VERSION_SAMPLES,
    VersionSample,
};
use super::*;
use crate::components::{
    AppButton, ButtonColor, Card, IconButton, IconButtonTheme, PageScroll, hint_row_key, lucide,
};
use crate::dialog::{self, DialogButton};
use crate::i18n::{self, Text};
use crate::theme;

impl DownloadGroup {
    pub(super) fn render_install_panel(&self, cx: &mut Context<Self>) -> AnyElement {
        let selected = self.selected.map(|index| &VERSION_SAMPLES[index]);

        let mut children: Vec<AnyElement> = Vec::new();
        children.push(self.select_header_card(selected, cx));
        for (key, level) in HINTS {
            children.push(hint_row_key(key, *level, cx));
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
                        .theme(IconButtonTheme::Foreground)
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

    /// 加载器选择：11 张横向小卡片按容器宽度自适应换行，每张卡显示自己选中的版本。
    fn loader_cards(&self, cx: &Context<Self>) -> AnyElement {
        let mut group = h_flex().items_start().gap_3().flex_wrap();
        for (index, (title_key, block_image)) in LOADERS.iter().enumerate() {
            group = group.child(self.loader_card(index, title_key, block_image, cx));
        }
        group.into_any_element()
    }

    /// 单个加载器小卡片：[18px 方块图 + 竖向排列的（加载器名称、选中的版本）+ 清除]。
    ///
    /// 第二行就是本加载器选中的版本，未选时显示「未选择」；点卡片在弹窗里挑版本，
    /// ✗ 取消本加载器的选择（加载器是可叠加的，所以每个加载器各记各的）。
    fn loader_card(
        &self,
        index: usize,
        title_key: &'static str,
        block_image: &str,
        cx: &Context<Self>,
    ) -> AnyElement {
        let palette = theme::palette(cx);
        let chosen = self.loader_choice[index];
        let status = chosen
            .map(|version| SharedString::from(LOADER_VERSIONS[index][version]))
            .unwrap_or_else(|| i18n::lang("Download.Install.Loader.None"));

        let card = Card::new(SharedString::from(format!("download-loader-{index}"))).child(
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
                        .child(
                            div()
                                .id(SharedString::from(format!(
                                    "download-loader-status-{index}"
                                )))
                                .text_xs()
                                .opacity(0.7)
                                .aria_label(status.clone())
                                .test_support()
                                .child(status),
                        ),
                )
                // 清除按钮只在选过版本后出现。
                .when_some(chosen, |this, _| {
                    this.child(
                        IconButton::new(
                            SharedString::from(format!("download-loader-clear-{index}")),
                            "x",
                            i18n::lang("Download.Install.Loader.Clear"),
                        )
                        .theme(IconButtonTheme::Foreground)
                        .size(px(18.))
                        // 卡片整块可点：✗ 必须吃掉按下事件，否则清完选择又弹出挑选弹窗。
                        .consume_mouse_down()
                        .on_click(cx.listener(move |this, _, _, cx| this.clear_loader(index, cx))),
                    )
                }),
        );

        // 卡片自身不可点击，包一层命中区承接点击（清除按钮吃掉按下事件，不会连带触发）。
        div()
            .id(SharedString::from(format!("download-loader-click-{index}")))
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, window, cx| {
                this.pick_loader_version(index, window, cx);
            }))
            .child(card)
            .test_support()
            .into_any_element()
    }

    /// 清除某个加载器已选的版本。
    fn clear_loader(&mut self, loader: usize, cx: &mut Context<Self>) {
        if self.loader_choice[loader].take().is_some() {
            logger::log::info!(
                target: "Download",
                "清除加载器选择：{}",
                i18n::lang(LOADERS[loader].0)
            );
            cx.notify();
        }
    }

    /// 在弹窗里挑**这个加载器的版本**；未选中时确定按钮禁用，取消不改动当前选择。
    fn pick_loader_version(&mut self, loader: usize, window: &mut Window, cx: &mut Context<Self>) {
        let versions = LOADER_VERSIONS[loader];
        let picker = cx.new(|_| LoaderVersions::new(versions, self.loader_choice[loader]));
        let loader_name = i18n::lang(LOADERS[loader].0);
        let title = i18n::lang_with_args(
            "Download.Install.SelectVersion.Title",
            &[loader_name.as_ref()],
        );
        logger::log::info!(target: "Download", "打开加载器版本弹窗：{loader_name}");

        dialog::Dialog::<Option<usize>>::new()
            .title(Text::literal(title))
            .content({
                let picker = picker.clone();
                move |_, _, _| picker.clone().into_any_element()
            })
            .button(
                DialogButton::new(dialog::buttons::CONFIRM)
                    .value_with({
                        let picker = picker.clone();
                        // 未选中时返回 `None`：这次点击不作答，弹窗保持打开。
                        move |_, cx| picker.read(cx).selected.map(Some)
                    })
                    .disabled_when({
                        let picker = picker.clone();
                        move |_, cx| picker.read(cx).selected.is_none()
                    }),
            )
            .button(DialogButton::new(dialog::buttons::CANCEL).value(None))
            .observe(&picker)
            .on_result(cx.listener(move |this, choice: &Option<usize>, _, cx| {
                let Some(version) = choice else {
                    return;
                };
                this.loader_choice[loader] = Some(*version);
                logger::log::info!(
                    target: "Download",
                    "选择加载器版本：{loader_name} {}",
                    LOADER_VERSIONS[loader][*version]
                );
                cx.notify();
            }))
            .open(window, cx);
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
