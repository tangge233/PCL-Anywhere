//! 设置 · 语言（对应 PCL 的 `PageSetupLauncherLanguage` 设置页：语言卡片 + 国际化横幅）。
//!
//! 下拉项来自 [`i18n::Locale::ALL`]；选择结果只存在界面状态里——语言切换与配置系统尚未实现。
//! 横幅上的两个按钮只保留排版：打开外部链接尚未接入。

use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::Sizable as _;
use gpui_kit::component::setting::SettingPage;
use gpui_kit::*;

use super::{Fields, group};
use crate::components::{AppButton, lucide};
use crate::i18n;

pub(super) fn page(fields: &Fields) -> SettingPage {
    let mut languages = vec![(
        SharedString::from("0"),
        i18n::lang_with_args(
            "Setup.Language.UiLanguage.Auto",
            &[i18n::Locale::DEFAULT.native_name()],
        ),
    )];
    for (index, locale) in i18n::Locale::ALL.iter().enumerate() {
        languages.push((
            SharedString::from((index + 1).to_string()),
            SharedString::from(locale.native_name()),
        ));
    }

    let banner = fields.element(&["Setup.Language.CardTitle"], move |_, cx| {
        let palette = crate::theme::palette(cx);
        let button = |id: &'static str, key: &str, icon: &'static str| {
            AppButton::new(id, i18n::lang(key)).icon(icon)
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
                                    .child(i18n::lang("Setup.Language.Banner.TitlePrefix")),
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
                                    .child(i18n::lang("Setup.Language.Banner.TitleSuffix")),
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
                            .child(i18n::lang("Setup.Language.Banner.DescriptionLine1")),
                    )
                    .child(
                        div()
                            .text_sm()
                            .child(i18n::lang("Setup.Language.Banner.DescriptionLine2")),
                    )
                    .child(
                        div()
                            .text_sm()
                            .child(i18n::lang("Setup.Language.Banner.DescriptionLine3")),
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

    SettingPage::new(i18n::lang("Setup.Language.Title"))
        .icon(lucide("earth"))
        .group(group("Setup.Language.CardTitle", language))
}
