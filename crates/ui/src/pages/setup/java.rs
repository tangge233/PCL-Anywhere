//! 设置 · Java（对应 `PCL/Views/Setup/PageSetupJava.axaml`：添加按钮 + Java 列表）。
//!
//! Java 列表在 .NET 版本里由文件系统扫描（`SetupJavaViewModel`）填充，本仓库尚未接入扫描，
//! 因此列表项是**示例数据**（见 [`SAMPLE_RUNTIMES`]）；「添加」按钮与上游一致，只保留排版
//! 而不声明点击行为（文件选择对话框尚未实现）。

use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::setting::{SettingItem, SettingPage};
use gpui_kit::*;

use super::{Fields, group};
use crate::components::{AppButton, AppRadio, ButtonColor, lucide};
use crate::i18n;

/// 示例数据：Java 运行时（名称、类型、版本、架构、位数、发行商、路径）。
pub(super) const SAMPLE_RUNTIMES: &[(&str, &str, &str, &str, &str, &str, &str)] = &[
    (
        "Java 21",
        "Java 21",
        "21.0.4+7-LTS",
        "x64",
        "64 位",
        "Microsoft",
        r"C:\Program Files\Microsoft\jdk-21.0.4\bin\javaw.exe",
    ),
    (
        "Java 17",
        "Java 17",
        "17.0.13+11",
        "x64",
        "64 位",
        "Eclipse Adoptium",
        r"C:\Program Files\Eclipse Adoptium\jdk-17.0.13\bin\javaw.exe",
    ),
];

pub(super) fn page(fields: &Fields) -> SettingPage {
    let mut items = vec![auto_select_item(fields)];
    for index in 0..SAMPLE_RUNTIMES.len() {
        items.push(runtime_item(fields, index));
    }
    items.push(add_item());

    SettingPage::new(i18n::lang("Setup.Left.Item.Java"))
        .icon(lucide("coffee"))
        .group(group("Setup.Java.Info.Title", items))
}

/// 「自动选择」条目：PCL 把自动选择当作列表里的第一项。
fn auto_select_item(fields: &Fields) -> SettingItem {
    let element = fields.clone();
    fields.element(&["Setup.Java.AutoSelect.Title"], move |_, cx| {
        let selected = element.read(|state| state.java.default_index == 0);
        let write = element.clone();
        v_flex()
            .gap_1()
            .child(
                AppRadio::new("java-auto")
                    .label(i18n::lang("Setup.Java.AutoSelect.Title"))
                    .selected(selected)
                    .on_change(move |_, _, cx| {
                        write.write(cx, |state| state.java.default_index = 0)
                    }),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(i18n::lang("Setup.Java.AutoSelect.Info")),
            )
            .into_any_element()
    })
}

/// 一条 Java：单选（设为默认）、信息行、启用/禁用按钮。
fn runtime_item(fields: &Fields, index: usize) -> SettingItem {
    let element = fields.clone();
    fields.element(&["Setup.Left.Item.Java"], move |_, cx| {
        let (name, kind, version, arch, bits, publisher, path) = SAMPLE_RUNTIMES[index];
        let (selected, enabled) = element.read(|state| {
            (
                state.java.default_index == index + 1,
                state.java.enabled[index],
            )
        });
        let info = i18n::lang_with_args(
            "Setup.Java.Info.Format",
            &[kind, version, arch, bits, publisher, path],
        );

        let select = element.clone();
        let toggle = element.clone();
        let toggle_key = if enabled {
            "Setup.Java.Disable"
        } else {
            "Setup.Java.Enable"
        };
        h_flex()
            .w_full()
            .justify_between()
            .items_center()
            .gap_3()
            .child(
                v_flex()
                    .gap_1()
                    .child(
                        AppRadio::new(SharedString::from(format!("java-runtime-{index}")))
                            .label(name)
                            .selected(selected)
                            .disabled(!enabled)
                            .on_change(move |_, _, cx| {
                                select.write(cx, |state| state.java.default_index = index + 1)
                            }),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(info),
                    ),
            )
            .child(
                AppButton::new(
                    SharedString::from(format!("java-toggle-{index}")),
                    i18n::lang(toggle_key),
                )
                .color(ButtonColor::Normal)
                .on_click(move |_, _, cx| {
                    toggle.write(cx, |state| {
                        state.java.enabled[index] = !state.java.enabled[index];
                        // 与 PCL 的提示一致：禁用当前默认 Java 后回落到「自动选择」。
                        if !state.java.enabled[index] && state.java.default_index == index + 1 {
                            state.java.default_index = 0;
                        }
                    });
                }),
            )
            .into_any_element()
    })
}

/// 「添加」按钮；上游同样只保留排版、不声明点击（文件选择对话框未实现）。
fn add_item() -> SettingItem {
    SettingItem::render(move |_, _, _| {
        AppButton::new("java-add", i18n::lang("Setup.Java.Add"))
            .icon("circle-plus")
            .into_any_element()
    })
    .keywords([i18n::lang("Setup.Java.Add")])
}
