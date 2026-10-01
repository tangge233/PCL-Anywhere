//! 设置 · 个性化的「隐藏功能」卡片（对应 `PageSetupUI.axaml` 的同名卡片）。
//!
//! 条目顺序与 XAML 的复选框顺序一致，也与 `Config.Preference.Ui.Hidden` 下的配置项一一对应。

use gpui_kit::component::setting::SettingItem;

use super::super::Fields;

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

pub(super) fn hidden(fields: &Fields) -> Vec<SettingItem> {
    let mut hidden = vec![fields.paragraph("Setup.Ui.FeatureHide.Description", false)];
    let mut section = "";
    for (index, (id, label_key, section_key, tip)) in HIDDEN_ITEMS.iter().enumerate() {
        if *section_key != section {
            section = section_key;
            hidden.push(fields.paragraph(section_key, true));
        }
        hidden.push(fields.checkbox(
            id,
            label_key,
            *tip,
            move |state| state.ui.hidden[index],
            move |state, value| state.ui.hidden[index] = value,
        ));
    }
    hidden
}
