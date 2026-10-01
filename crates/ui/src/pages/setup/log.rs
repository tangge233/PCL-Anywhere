//! 设置 · 日志（对应 `PCL/Views/Setup/PageSetupLog.axaml`：日志操作卡片 + 全部日志卡片）。
//!
//! 日志列表尚未接入实时日志，这里给出**示例数据**；导出 / 导出全部 / 打开日志目录都是文件操作，
//! 只保留按钮排版。「清理历史日志」在本页只清空示例列表。

use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::setting::SettingPage;
use gpui_kit::*;

use super::{Fields, SetupGroup, group};
use crate::components::{AppButton, ButtonColor, lucide};
use crate::i18n;

/// 示例数据：日志行（时间戳与内容都是示例值）。
pub(super) const SAMPLE_LINES: &[&str] = &[
    "[12:00:00] [Init] PCL-Anywhere 已就绪",
    "[12:00:01] [Setup] 已初始化启动设置！",
    "[12:00:01] [Setup] 已初始化其他页设置！",
    "[12:00:02] [Setup] 已初始化杂项页设置！",
];

pub(super) fn page(
    fields: &Fields,
    _window: &mut Window,
    _cx: &mut Context<SetupGroup>,
) -> SettingPage {
    // 日志操作：四个按钮一行（XAML 的 WrapPanel）。
    let clear = fields.clone();
    let operations = vec![fields.element(&["Setup.Log.Operations"], move |_, _| {
        let label = |key: &str| i18n::text(key);
        h_flex()
            .gap_3()
            .flex_wrap()
            .child(
                AppButton::new("log-export", label("Setup.Log.Export"))
                    .color(ButtonColor::Highlight),
            )
            .child(AppButton::new(
                "log-export-all",
                label("Setup.Log.ExportAll"),
            ))
            .child(AppButton::new(
                "log-open-directory",
                label("Setup.Log.OpenDirectory"),
            ))
            .child(
                AppButton::new("log-clear", label("Setup.Log.ClearHistory"))
                    .color(ButtonColor::Red)
                    .on_click({
                        let clear = clear.clone();
                        move |_, _, cx| clear.write(cx, |state| state.log.lines.clear())
                    }),
            )
            .into_any_element()
    })];

    // 全部日志：按行渲染示例列表。
    let element = fields.clone();
    let logs = vec![fields.element(&["Setup.Log.AllLogs"], move |_, _| {
        let lines = element.read(|state| state.log.lines.clone());
        let mut list = v_flex().gap_1().w_full();
        for line in lines {
            list = list.child(div().text_sm().child(line));
        }
        list.into_any_element()
    })];

    SettingPage::new(i18n::text("Setup.Left.Item.Log"))
        .icon(lucide("scroll-text"))
        .group(group("Setup.Log.Operations", operations))
        .group(group("Setup.Log.AllLogs", logs))
}
