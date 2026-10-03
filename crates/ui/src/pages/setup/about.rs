//! 设置 · 软件信息（对应 PCL 的 `PageSetupAbout` 设置页：软件信息 / 特别鸣谢 /
//! 贡献者 / PCL-Anywhere 法律 / 上游法律 / 第三方许可六张卡片）。
//!
//! 贡献者头像与列表需要 GitHub API、许可列表需要依赖元数据，本仓库尚未接入，
//! 因此贡献者列表为空、许可列表为示例数据；所有「打开网页」按钮只保留排版。

use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::avatar::Avatar;
use gpui_kit::component::setting::{SettingItem, SettingPage};
use gpui_kit::component::{Sizable as _, Size};
use gpui_kit::*;

use super::{Fields, group};
use crate::components::{AppButton, ButtonColor, lucide};
use crate::i18n;

/// 示例数据：第三方许可（真实列表取自依赖元数据，尚未接入）。
const SAMPLE_LICENSES: &[(&str, &str)] = &[("gpui-kit", "0.7"), ("lucide", "图标集")];

pub(super) fn page(fields: &Fields) -> SettingPage {
    // 卡片一：软件信息。
    let info = vec![
        person_row(
            fields,
            "images/Heads/LTCat.jpg",
            i18n::lang("Setup.About.OriginalAuthor.Name"),
            i18n::lang("Setup.About.OriginalAuthor.Info"),
            Some((
                "about-sponsor-author",
                "Setup.About.SponsorOriginalAuthor",
                ButtonColor::Normal,
            )),
        ),
        person_row(
            fields,
            "images/Heads/PCL-Community.png",
            i18n::lang("Setup.About.AnywhereEdition.Name"),
            i18n::lang("Setup.About.AnywhereEdition.Info"),
            Some((
                "about-github-home",
                "Setup.About.GitHubHome",
                ButtonColor::Normal,
            )),
        ),
        // 版本行的标题是程序名（不是界面文案），版本信息来自构建元数据，这里只给占位。
        person_row(
            fields,
            "images/Heads/Logo-CE.png",
            SharedString::from("PCL-Anywhere"),
            version_info(),
            Some((
                "about-source-code",
                "Setup.About.SourceCode",
                ButtonColor::Normal,
            )),
        ),
    ];

    // 卡片二：特别鸣谢。
    let thanks = vec![
        person_row(
            fields,
            "images/Heads/bangbang93.png",
            i18n::lang("Setup.About.SpecialThanks.Name.Bangbang93"),
            i18n::lang("Setup.About.SpecialThanks.Info.Bmclapi"),
            Some((
                "about-sponsor-mirror",
                "Setup.About.SpecialThanks.SponsorMirror",
                ButtonColor::Normal,
            )),
        ),
        person_row(
            fields,
            "images/Heads/wiki.png",
            i18n::lang("Setup.About.SpecialThanks.Name.Mcmod"),
            i18n::lang("Setup.About.SpecialThanks.Info.Mcmod"),
            Some((
                "about-mcmod",
                "Setup.About.SpecialThanks.OpenMcmod",
                ButtonColor::Normal,
            )),
        ),
        person_row(
            fields,
            "images/Heads/Pysio.jpg",
            i18n::lang("Setup.About.SpecialThanks.Name.Pysio"),
            i18n::lang("Setup.About.SpecialThanks.Info.Cloud"),
            Some((
                "about-pysio",
                "Setup.About.SpecialThanks.GoBlog",
                ButtonColor::Normal,
            )),
        ),
        person_row(
            fields,
            "images/Heads/Yunmoan.jpg",
            i18n::lang("Setup.About.SpecialThanks.Name.Yunmoan"),
            i18n::lang("Setup.About.SpecialThanks.Info.Cloud"),
            Some((
                "about-yunmoan",
                "Setup.About.SpecialThanks.OpenWebsite",
                ButtonColor::Normal,
            )),
        ),
        person_row(
            fields,
            "images/Heads/EasyTier.png",
            i18n::lang("Setup.About.SpecialThanks.Name.EasyTier"),
            i18n::lang("Setup.About.SpecialThanks.Info.LinkModule"),
            Some((
                "about-easytier",
                "Setup.About.SpecialThanks.OpenWebsite",
                ButtonColor::Normal,
            )),
        ),
        person_row(
            fields,
            "images/Heads/z0z0r4.png",
            i18n::lang("Setup.About.SpecialThanks.Name.Z0z0r4"),
            i18n::lang("Setup.About.SpecialThanks.Info.Mcim"),
            None,
        ),
        person_row(
            fields,
            "images/Heads/Emperormummy.png",
            i18n::lang("Setup.About.SpecialThanks.Name.Emperormummy"),
            i18n::lang("Setup.About.SpecialThanks.Info.IconDesign"),
            None,
        ),
    ];

    // 卡片三：贡献者。列表由 GitHub API 填充（尚未接入），因此只有「查看更多」按钮。
    let contributors = vec![fields.stub_button(
        "about-contributors-more",
        "Setup.About.Contributors.ViewMore",
        None,
        ButtonColor::Normal,
    )];

    // 卡片四：PCL-Anywhere 法律。
    let legal = vec![
        fields.paragraph("Setup.About.Legal.PrivacyNotice", true),
        fields.paragraph("Setup.About.Legal.PrivacyText", false),
        fields.paragraph("Setup.About.Legal.OtherInfo", true),
        fields.paragraph("Setup.About.Legal.CopyrightText", false),
        fields.stub_button(
            "about-legal-source",
            "Setup.About.Legal.CommunitySource",
            None,
            ButtonColor::Highlight,
        ),
        fields.stub_button(
            "about-legal-privacy",
            "Setup.About.Legal.CommunityLobbyPrivacy",
            None,
            ButtonColor::Normal,
        ),
        fields.stub_button(
            "about-legal-natayark",
            "Setup.About.Legal.NatayarkTerms",
            None,
            ButtonColor::Normal,
        ),
    ];

    // 卡片五：上游法律。
    let upstream = vec![
        fields.paragraph("Setup.About.Legal.PrivacyNotice", true),
        fields.paragraph("Setup.About.UpstreamLegal.PrivacyText", false),
        fields.paragraph("Setup.About.Legal.OtherInfo", true),
        fields.paragraph("Setup.About.UpstreamLegal.CopyrightText", false),
        fields.stub_button(
            "about-upstream-terms",
            "Setup.About.UpstreamLegal.TermsAndDisclaimer",
            None,
            ButtonColor::Highlight,
        ),
        fields.stub_button(
            "about-upstream-source",
            "Setup.About.UpstreamLegal.OpenSource",
            None,
            ButtonColor::Normal,
        ),
    ];

    // 卡片六：第三方许可（示例数据）。
    let mut licenses = Vec::new();
    for &(name, information) in SAMPLE_LICENSES {
        let name = SharedString::from(name);
        let information = SharedString::from(information);
        // id 由许可名派生（域派生，不随列表顺序/下标变化）。
        let website_id = SharedString::from(format!("about-license-website-{name}"));
        let file_id = SharedString::from(format!("about-license-file-{name}"));
        licenses.push(
            fields
                .element(&["Setup.About.Licenses.Title"], move |_, cx| {
                    v_flex()
                        .gap_2()
                        .w_full()
                        .child(
                            h_flex()
                                .gap_4()
                                .items_center()
                                .child(div().font_weight(FontWeight::BOLD).child(name.clone()))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(information.clone()),
                                ),
                        )
                        .into_any_element()
                })
                .keywords([i18n::lang("Setup.About.Licenses.Title")]),
        );
        licenses.push(fields.stub_button(
            website_id,
            "Setup.About.Licenses.ViewWebsite",
            None,
            ButtonColor::Normal,
        ));
        licenses.push(fields.stub_button(
            file_id,
            "Setup.About.Licenses.ViewLicense",
            None,
            ButtonColor::Normal,
        ));
    }

    SettingPage::new(i18n::lang("Setup.Left.Item.About"))
        .icon(lucide("info"))
        .group(group("Setup.About.Title", info))
        .group(group("Setup.About.SpecialThanks.Title", thanks))
        .group(group("Setup.About.Contributors.Title", contributors))
        .group(group("Setup.About.Legal.Title", legal))
        .group(group("Setup.About.UpstreamLegal.Title", upstream))
        .group(group("Setup.About.Licenses.Title", licenses))
}

/// 一行「名称 + 说明 + 右侧按钮」（PCL 的 `MyListItem` + `MyButton`）。
fn person_row(
    fields: &Fields,
    avatar: &'static str,
    name: SharedString,
    info: SharedString,
    action: Option<(&'static str, &'static str, ButtonColor)>,
) -> SettingItem {
    let keyword = name.clone();
    let action_label = action.map(|(_, label_key, _)| i18n::lang(label_key));
    // 搜索关键词直接用界面上显示的两行文字（`Fields::element` 的入参是文案键，
    // 这里的信息可能是替换过占位符的成品文案，所以走 `keywords` 而不是键列表）。
    let keywords = info.clone();
    fields
        .element(&[], move |_, cx| {
            let mut row = h_flex()
                .w_full()
                .justify_between()
                .items_center()
                .gap_4()
                // 34px 圆形头像：与 PCL 的 `MyImage Width/Height=34 CornerRadius=17` 一致。
                .child(Avatar::new().src(avatar).with_size(Size::Size(px(34.))))
                .child(
                    v_flex()
                        .gap_1()
                        .child(div().font_weight(FontWeight::BOLD).child(name.clone()))
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(info.clone()),
                        ),
                );
            if let (Some((id, _, color)), Some(label)) = (action, action_label.clone()) {
                row = row.child(AppButton::new(id, label).color(color));
            }
            row.into_any_element()
        })
        .keywords([keyword, keywords])
}

/// 版本信息行：PCL 的 `当前版本: %VERSION% (%BRANCH%, %VERSIONCODE%, %COMMIT_HASH%)` 里
/// 的占位符由构建元数据替换；本仓库用包版本与构建配置填充，分支与提交号等构建元数据
/// 将来由核心 crate 提供（这里显式写成 unknown，避免界面上出现未替换的占位符）。
fn version_info() -> SharedString {
    i18n::lang("Setup.About.Version.Info")
        .replace("%VERSION%", env!("CARGO_PKG_VERSION"))
        .replace("%VERSIONCODE%", env!("CARGO_PKG_VERSION_MAJOR"))
        .replace(
            "%BRANCH%",
            if cfg!(debug_assertions) {
                "debug"
            } else {
                "release"
            },
        )
        .replace("%COMMIT_HASH%", "unknown")
        .into()
}
