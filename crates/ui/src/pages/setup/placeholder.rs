//! 尚未实现的设置分类：路由表里登记了分类，但 PCL 没有对应的设置页，这里只给占位。

use gpui_kit::component::setting::{SettingGroup, SettingItem};
use gpui_kit::*;

use super::SetupGroup;
use crate::components::PagePlaceholder;

pub(crate) fn update(
    _this: &mut SetupGroup,
    _window: &mut Window,
    _cx: &mut Context<SetupGroup>,
) -> Vec<SettingGroup> {
    placeholder_group("Setup.Left.Item.Update")
}

pub(crate) fn feedback(
    _this: &mut SetupGroup,
    _window: &mut Window,
    _cx: &mut Context<SetupGroup>,
) -> Vec<SettingGroup> {
    placeholder_group("Setup.Left.Item.Feedback")
}

/// `SettingItem::Element` 的搜索只看 keywords（标题不参与），关键词必须与本页标签一致。
fn placeholder_group(title_key: &str) -> Vec<SettingGroup> {
    let label = crate::i18n::lang(title_key);
    let keyword = label.clone();
    vec![
        SettingGroup::new().item(
            SettingItem::render(move |_, _, _| {
                div()
                    .h_80()
                    .child(PagePlaceholder::for_route(label.as_ref()))
            })
            .keywords([keyword]),
        ),
    ]
}
