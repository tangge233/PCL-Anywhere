//! 设置 · 个性化的「隐藏功能」卡片（对应 PCL `PageSetupUI` 设置页的同名卡片）。
//!
//! 条目顺序与 PCL 界面的复选框顺序一致，也与 `Config.Preference.Ui.Hidden` 下的配置项一一对应。

use gpui_kit::component::setting::SettingItem;

use super::super::Fields;
use super::HIDDEN_ITEMS;

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
