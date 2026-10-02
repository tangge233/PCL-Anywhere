//! 设置 · 个性化（对应 `PCL/Views/Setup/PageSetupUI.axaml`：外观 / 字体 / 背景 / 音乐 / 图标
//! / 主页 / 隐藏功能七张卡片）。
//!
//! PCL 在本页只接线了配色模式与亮/暗配色主题，其余外观项尚未接线；未接线项的默认值取自
//! XAML 上控件的 `Value`，无法确认的标为示例值。字体选择器（`FontSelector`）与
//! 打开文件夹 / 刷新 / 清理等文件操作未移植，相关控件只保留排版。

use gpui_kit::component::setting::SettingPage;
use gpui_kit::component::slider::SliderState;
use gpui_kit::*;

mod cards;
mod hidden;

use super::{Fields, SetupGroup, group};
use crate::components::lucide;
use crate::i18n;

/// 「隐藏功能」条目：(配置项 id, 文案键, 分组标题键, 提示键)。
///
/// 顺序与 `PageSetupUI.axaml` 的复选框顺序一致，也与 `Config.Preference.Ui.Hidden` 下的
/// 配置项一一对应。
pub(super) const HIDDEN_ITEMS: &[(&str, &str, &str, Option<&str>)] = &[
    // 主页
    (
        "UiHiddenPageDownload",
        "Setup.Ui.FeatureHide.Item.Download",
        "Setup.Ui.FeatureHide.MainPage",
        None,
    ),
    (
        "UiHiddenPageSetup",
        "Setup.Ui.FeatureHide.Item.Settings",
        "Setup.Ui.FeatureHide.MainPage",
        None,
    ),
    (
        "UiHiddenPageTools",
        "Setup.Ui.FeatureHide.Item.Tools",
        "Setup.Ui.FeatureHide.MainPage",
        None,
    ),
    // 设置
    (
        "UiHiddenSetupLaunch",
        "Setup.Ui.FeatureHide.Item.Launch",
        "Setup.Ui.FeatureHide.SubSetup",
        None,
    ),
    // XAML 此处字面写 "Java"；改用同名文案键（取值同为 "Java"），避免绕过文案表。
    (
        "UiHiddenSetupJava",
        "Setup.Ui.FeatureHide.Item.Java",
        "Setup.Ui.FeatureHide.SubSetup",
        None,
    ),
    (
        "UiHiddenSetupGameManage",
        "Setup.Ui.FeatureHide.Item.GameManage",
        "Setup.Ui.FeatureHide.SubSetup",
        None,
    ),
    (
        "UiHiddenSetupGameLink",
        "Setup.Ui.FeatureHide.Item.GameLink",
        "Setup.Ui.FeatureHide.SubSetup",
        None,
    ),
    (
        "UiHiddenSetupUi",
        "Setup.Ui.FeatureHide.Item.Ui",
        "Setup.Ui.FeatureHide.SubSetup",
        None,
    ),
    (
        "UiHiddenSetupLauncherLanguage",
        "Setup.Ui.FeatureHide.Item.Language",
        "Setup.Ui.FeatureHide.SubSetup",
        None,
    ),
    (
        "UiHiddenSetupLauncherMisc",
        "Setup.Ui.FeatureHide.Item.Misc",
        "Setup.Ui.FeatureHide.SubSetup",
        None,
    ),
    (
        "UiHiddenSetupUpdate",
        "Setup.Ui.FeatureHide.Item.Update",
        "Setup.Ui.FeatureHide.SubSetup",
        None,
    ),
    (
        "UiHiddenSetupAbout",
        "Setup.Ui.FeatureHide.Item.About",
        "Setup.Ui.FeatureHide.SubSetup",
        None,
    ),
    (
        "UiHiddenSetupFeedback",
        "Setup.Ui.FeatureHide.Item.Feedback",
        "Setup.Ui.FeatureHide.SubSetup",
        None,
    ),
    (
        "UiHiddenSetupLog",
        "Setup.Ui.FeatureHide.Item.Log",
        "Setup.Ui.FeatureHide.SubSetup",
        None,
    ),
    // 工具
    (
        "UiHiddenToolsGameLink",
        "Setup.Ui.FeatureHide.Item.Lobby",
        "Setup.Ui.FeatureHide.SubTools",
        None,
    ),
    (
        "UiHiddenToolsTest",
        "Setup.Ui.FeatureHide.Item.Toolbox",
        "Setup.Ui.FeatureHide.SubTools",
        None,
    ),
    // 实例
    (
        "UiHiddenVersionEdit",
        "Setup.Ui.FeatureHide.Item.Edit",
        "Setup.Ui.FeatureHide.SubInstance",
        None,
    ),
    (
        "UiHiddenVersionExport",
        "Setup.Ui.FeatureHide.Item.Export",
        "Setup.Ui.FeatureHide.SubInstance",
        None,
    ),
    (
        "UiHiddenVersionSave",
        "Setup.Ui.FeatureHide.Item.Save",
        "Setup.Ui.FeatureHide.SubInstance",
        None,
    ),
    (
        "UiHiddenVersionScreenshot",
        "Setup.Ui.FeatureHide.Item.Screenshot",
        "Setup.Ui.FeatureHide.SubInstance",
        None,
    ),
    // XAML 此处字面写 "Mod"，但文案表已把它译作「模组」，与同组其他条目一致，故改用文案键。
    (
        "UiHiddenVersionMod",
        "Setup.Ui.FeatureHide.Item.Mod",
        "Setup.Ui.FeatureHide.SubInstance",
        None,
    ),
    (
        "UiHiddenVersionResourcePack",
        "Setup.Ui.FeatureHide.Item.ResourcePack",
        "Setup.Ui.FeatureHide.SubInstance",
        None,
    ),
    (
        "UiHiddenVersionShader",
        "Setup.Ui.FeatureHide.Item.Shader",
        "Setup.Ui.FeatureHide.SubInstance",
        None,
    ),
    (
        "UiHiddenVersionSchematic",
        "Setup.Ui.FeatureHide.Item.Schematic",
        "Setup.Ui.FeatureHide.SubInstance",
        None,
    ),
    (
        "UiHiddenVersionServer",
        "Setup.Ui.FeatureHide.Item.Server",
        "Setup.Ui.FeatureHide.SubInstance",
        None,
    ),
    // 具体功能
    (
        "UiHiddenFunctionSelect",
        "Setup.Ui.FeatureHide.Item.InstanceSelect",
        "Setup.Ui.FeatureHide.SpecificFeature",
        Some("Setup.Ui.FeatureHide.Item.InstanceSelect.ToolTip"),
    ),
    (
        "UiHiddenFunctionModUpdate",
        "Setup.Ui.FeatureHide.Item.ModUpdate",
        "Setup.Ui.FeatureHide.SpecificFeature",
        Some("Setup.Ui.FeatureHide.Item.ModUpdate.ToolTip"),
    ),
    (
        "UiHiddenFunctionHidden",
        "Setup.Ui.FeatureHide.Item.FeatureHide",
        "Setup.Ui.FeatureHide.SpecificFeature",
        Some("Setup.Ui.FeatureHide.Item.FeatureHide.ToolTip"),
    ),
];

pub(super) fn page(fields: &Fields, cx: &mut Context<SetupGroup>) -> SettingPage {
    // 滑块状态与默认值：上限取自 XAML 的 `MaxValue`，默认值取自同名配置项。
    let opacity = cx.new(|_| {
        SliderState::new()
            .min(100.)
            .max(600.)
            .step(10.)
            .default_value(600.)
    });
    let blur_radius = cx.new(|_| {
        SliderState::new()
            .min(0.)
            .max(40.)
            .step(1.)
            .default_value(0.)
    });
    let blur_sampling = cx.new(|_| {
        SliderState::new()
            .min(10.)
            .max(100.)
            .step(5.)
            .default_value(10.)
    });
    let background_opacity = cx.new(|_| {
        SliderState::new()
            .min(0.)
            .max(1000.)
            .step(10.)
            .default_value(1000.)
    });
    let background_blur = cx.new(|_| {
        SliderState::new()
            .min(0.)
            .max(40.)
            .step(1.)
            .default_value(0.)
    });
    let music_volume = cx.new(|_| {
        SliderState::new()
            .min(0.)
            .max(1000.)
            .step(10.)
            .default_value(500.)
    });

    // 卡片一：外观。
    let basic = cards::basic(fields, &opacity, &blur_radius, &blur_sampling, cx);

    // 卡片二：字体。PCL 用字体选择器，这里用文本框承载字体名（未接线）。
    let font = cards::font(fields, cx);

    // 卡片三：背景。
    let background = cards::background(fields, &background_opacity, &background_blur, cx);

    // 卡片四：音乐。
    let music = cards::music(fields, &music_volume, cx);

    // 卡片五：启动器图标（4 个单选项 + 左对齐 + 文字内容 + 两个图片按钮）。
    let logo = cards::logo(fields, cx);

    // 卡片六：自定义主页。
    let homepage = cards::homepage(fields, cx);

    // 卡片七：隐藏功能。
    let hidden = hidden::hidden(fields);

    SettingPage::new(i18n::lang("Setup.Left.Item.Ui"))
        .icon(lucide("palette"))
        .group(group("Setup.Ui.Basic.Title", basic))
        .group(group("Setup.Ui.Font.Title", font))
        .group(group("Setup.Ui.Background.TitleDefault", background))
        .group(group("Setup.Ui.Music.Title", music))
        .group(group("Setup.Ui.Logo.Title", logo))
        .group(group("Setup.Ui.Homepage.Title", homepage))
        .group(group("Setup.Ui.FeatureHide.Title", hidden))
}
