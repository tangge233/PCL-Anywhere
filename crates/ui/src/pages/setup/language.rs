//! 设置 · 语言（对应 `PCL/Views/Setup/PageSetupLauncherLanguage.axaml`：语言卡片 + 国际化横幅）。
//!
//! UI 语言列表在 .NET 版本里由本地化服务扫描语言文件生成；本仓库只内置简体中文，
//! 因此下拉项里除「跟随系统」外的语言名是**示例数据**（见 [`SAMPLE_LANGUAGES`]）。
//! 横幅上的两个按钮只保留排版：打开外部链接尚未接入。

use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::Sizable as _;
use gpui_kit::component::setting::SettingPage;
use gpui_kit::*;

use super::{Fields, SetupGroup, group};
use crate::components::{AppButton, lucide};
use crate::i18n;

/// 示例数据：本地化服务支持的语言（原生名）。
const SAMPLE_LANGUAGES: &[&str] = &["简体中文", "繁體中文", "English"];

pub(super) fn page(fields: &Fields, _cx: &mut Context<SetupGroup>) -> SettingPage {
    let mut languages = vec![(
        SharedString::from("0"),
        i18n::text_args("Setup.Language.UiLanguage.Auto", &[SAMPLE_LANGUAGES[0]]),
    )];
    for (index, name) in SAMPLE_LANGUAGES.iter().enumerate() {
        languages.push((
            SharedString::from((index + 1).to_string()),
            SharedString::from(*name),
        ));
    }

    let banner = fields.element(&["Setup.Language.CardTitle"], move |_, cx| {
        let palette = crate::theme::palette(cx);
        let button = |id: &'static str, key: &str, icon: &'static str| {
            AppButton::new(id, i18n::text(key)).icon(icon)
        };
        v_flex()
            .w_full()
            .gap_3()
            .p_5()
            .rounded(px(5.))
            .bg(palette.color_level(7))
            .child(
                h_flex()
                    .justify_between()
                    .items_start()
                    .child(
                        v_flex()
                            .gap_1()
                            .child(
                                div()
                                    .text_xl()
                                    .font_weight(FontWeight::BOLD)
                                    .child(i18n::text("Setup.Language.Banner.TitlePrefix")),
                            )
                            // 产品名不是界面文案。
                            .child(
                                div()
                                    .text_2xl()
                                    .font_weight(FontWeight::BOLD)
                                    .child("PCL-Anywhere"),
                            )
                            .child(
                                div()
                                    .text_xl()
                                    .font_weight(FontWeight::BOLD)
                                    .child(i18n::text("Setup.Language.Banner.TitleSuffix")),
                            ),
                    )
                    .child(div().opacity(0.6).child(lucide("earth").large())),
            )
            .child(
                v_flex()
                    .gap_1()
                    .child(
                        div()
                            .text_sm()
                            .child(i18n::text("Setup.Language.Banner.DescriptionLine1")),
                    )
                    .child(
                        div()
                            .text_sm()
                            .child(i18n::text("Setup.Language.Banner.DescriptionLine2")),
                    )
                    .child(
                        div()
                            .text_sm()
                            .child(i18n::text("Setup.Language.Banner.DescriptionLine3")),
                    ),
            )
            .child(
                h_flex()
                    .justify_end()
                    .gap_2()
                    .child(button(
                        "language-translate",
                        "Setup.Language.Banner.TranslateWebsite",
                        "globe",
                    ))
                    .child(button(
                        "language-pull-request",
                        "Setup.Language.Banner.SubmitPullRequest",
                        "git-pull-request-arrow",
                    )),
            )
            .into_any_element()
    });

    let language = vec![
        fields.dropdown(
            "Setup.Language.UiLanguage",
            None,
            languages,
            |state| state.language.language,
            |state, index| state.language.language = index,
        ),
        fields.dropdown(
            "Setup.Language.FormatCulture",
            None,
            super::choices(&[
                "Setup.Language.FormatCulture.Auto",
                "Setup.Language.FormatCulture.FollowLanguage",
            ]),
            |state| state.language.format_culture,
            |state, index| state.language.format_culture = index,
        ),
        banner,
    ];

    SettingPage::new(i18n::text("Setup.Language.Title"))
        .icon(lucide("earth"))
        .group(group("Setup.Language.CardTitle", language))
}
