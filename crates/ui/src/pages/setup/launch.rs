//! 设置 · 启动（对应 `PCL/Views/Setup/PageSetupLaunch.axaml` 的三张卡片：启动选项 / 游戏内存 /
//! 高级启动选项）。

use gpui_kit::base::h_flex;
use gpui_kit::component::input::{Input, InputState, Textarea, TextareaState};
use gpui_kit::component::setting::{SettingField, SettingPage};
use gpui_kit::component::slider::SliderState;
use gpui_kit::*;

use super::{Fields, SetupGroup, choices, group, item, tip};
use crate::components::{ButtonColor, lucide};
use crate::i18n;

/// JVM 参数的出厂默认值（`Config.Launch.JvmArgs` 的初始快照）。
const DEFAULT_JVM_ARGS: &str = "-XX:+UseG1GC -XX:-UseAdaptiveSizePolicy -XX:-OmitStackTraceInFastThrow -Djdk.lang.Process.allowAmbiguousCommands=true -Dfml.ignoreInvalidMinecraftCertificates=True -Dfml.ignorePatchDiscrepancies=True -Dlog4j2.formatMsgNoLookups=true";

/// 游戏窗口宽/高（`LaunchArgumentWindowWidth` / `LaunchArgumentWindowHeight`），只作示例展示。
const SAMPLE_WINDOW_WIDTH: &str = "854";
const SAMPLE_WINDOW_HEIGHT: &str = "480";

/// 最大堆与初始堆的滑块上限（XAML：`MaxValue="64"`）。
const MEMORY_MAX: f32 = 64.;
/// 最大堆与初始堆的默认值（`LaunchRamCustom` / `LaunchRamCustomInitial`）。
const DEFAULT_MAX_MEMORY: f32 = 15.;
const DEFAULT_INITIAL_MEMORY: f32 = 0.;

pub(super) fn page(
    fields: &Fields,
    window: &mut Window,
    cx: &mut Context<SetupGroup>,
) -> SettingPage {
    // 滑块状态由实体承载：拖动的值就是受控值。
    let max_memory = cx.new(|_| {
        SliderState::new()
            .min(0.)
            .max(MEMORY_MAX)
            .step(1.)
            .default_value(DEFAULT_MAX_MEMORY)
    });
    let initial_memory = cx.new(|_| {
        SliderState::new()
            .min(0.)
            .max(MEMORY_MAX)
            .step(1.)
            .default_value(DEFAULT_INITIAL_MEMORY)
    });
    // 多行文本框（PCL 的 `AcceptsReturn` 文本框）同样由实体承载。
    let jvm_args = cx.new(|cx| {
        TextareaState::new(window, cx)
            .default_value(DEFAULT_JVM_ARGS)
            .rows(4)
    });
    let game_args = cx.new(|cx| TextareaState::new(window, cx).rows(3));
    let (width, height) = window_size_inputs(window, cx);

    let memory_manual = |state: &super::state::SetupState| state.launch.memory_is_manual();

    // 卡片一：启动选项。
    let options = vec![
        fields.dropdown(
            "Setup.Launch.Options.InstanceIsolation.Label",
            Some("Setup.Launch.Options.InstanceIsolation.ToolTip"),
            choices(&[
                "Common.Action.Close",
                "Setup.Launch.Options.InstanceIsolation.ModdedOnly",
                "Setup.Launch.Options.InstanceIsolation.NonRelease",
                "Setup.Launch.Options.InstanceIsolation.ModdedAndNonRelease",
                "Setup.Launch.Options.InstanceIsolation.All",
            ]),
            |state| state.launch.isolation,
            |state, index| state.launch.isolation = index,
        ),
        fields.dropdown(
            "Setup.Launch.Options.IpStack.Label",
            Some("Setup.Launch.Options.IpStack.ToolTip"),
            choices(&[
                "Setup.Launch.Options.IpStack.Ipv4",
                "Setup.Launch.Options.IpStack.JavaDefault",
                "Setup.Launch.Options.IpStack.Ipv6",
            ]),
            |state| state.launch.ip_stack,
            |state, index| state.launch.ip_stack = index,
        ),
        fields.input(
            "Setup.Launch.Options.CustomInfo.Label",
            Some("Setup.Launch.Options.CustomInfo.ToolTip"),
            |state| state.launch.custom_info.clone(),
            |state, value| state.launch.custom_info = value,
        ),
    ];

    // 卡片二：游戏内存。
    let memory = vec![
        fields.radios(
            "launch-memory-mode",
            "Setup.Launch.Memory.GameAllocation",
            Some("Setup.Launch.Memory.Auto.ToolTip"),
            &["Setup.Launch.Memory.Auto", "Common.Option.Customize"],
            |state| state.launch.memory_mode,
            |state, index| state.launch.memory_mode = index,
        ),
        fields.slider(
            "Setup.Launch.Memory.Max",
            Some("Setup.Launch.Memory.Max.ToolTip"),
            &max_memory,
            " GiB",
            Some(memory_manual),
        ),
        fields.slider(
            "Setup.Launch.Memory.Initial",
            Some("Setup.Launch.Memory.Initial.ToolTip"),
            &initial_memory,
            " GiB",
            Some(memory_manual),
        ),
        fields.checkbox(
            "launch-lock-memory",
            "Setup.Launch.Advanced.LockMemory",
            Some("Setup.Launch.Advanced.LockMemory.ToolTip"),
            |state| state.launch.lock_memory,
            |state, value| state.launch.lock_memory = value,
        ),
    ];

    // 卡片三：高级启动选项。
    let restore_args = jvm_args.clone();
    let advanced = vec![
        fields.dropdown(
            "Setup.Launch.Options.Priority.Label",
            None,
            choices(&[
                "Setup.Launch.Options.Priority.AboveNormal",
                "Setup.Launch.Options.Priority.Normal",
                "Setup.Launch.Options.Priority.BelowNormal",
                "Setup.Launch.Options.Priority.High",
                "Setup.Launch.Options.Priority.RealTime",
            ]),
            |state| state.launch.priority,
            |state, index| state.launch.priority = index,
        ),
        tip(
            item(
                "Setup.Launch.Options.WindowSize.Label",
                SettingField::<SharedString>::render(move |_, _, _| {
                    window_size_row(&width, &height)
                }),
            ),
            Some("Setup.Launch.Options.WindowSize.CustomHeightBug.ToolTip"),
        ),
        tip(
            item(
                "Setup.Launch.Advanced.JvmHead",
                SettingField::<SharedString>::render(move |_, _, _| {
                    Textarea::new(&jvm_args).h_20().into_any_element()
                }),
            ),
            Some("Setup.Launch.Advanced.JvmHead.ToolTip"),
        ),
        fields.button(
            "launch-restore-jvm",
            "Setup.Launch.Advanced.JvmHead.Restore",
            ButtonColor::Normal,
            move |_, window, cx| {
                restore_args.update(cx, |state, cx| {
                    state.set_value(DEFAULT_JVM_ARGS, window, cx);
                });
            },
        ),
        tip(
            item(
                "Setup.Launch.Advanced.GameTail",
                SettingField::<SharedString>::render(move |_, _, _| {
                    Textarea::new(&game_args).h_16().into_any_element()
                }),
            ),
            Some("Setup.Launch.Advanced.GameTail.ToolTip"),
        ),
        fields.paragraph("Setup.Launch.Advanced.MoreInInstanceSetup", false),
    ];

    SettingPage::new(i18n::lang("Setup.Left.Item.Launch"))
        .icon(lucide("rocket"))
        .group(group("Setup.Launch.Options.Title", options))
        .group(group("Setup.Launch.Memory.Title", memory))
        .group(group("Setup.Launch.Advanced.Title", advanced))
}

/// 窗口宽高文本框（上面两个 `InputState`）。
fn window_size_inputs(
    window: &mut Window,
    cx: &mut Context<SetupGroup>,
) -> (Entity<InputState>, Entity<InputState>) {
    let width = cx.new(|cx| InputState::new(window, cx).default_value(SAMPLE_WINDOW_WIDTH));
    let height = cx.new(|cx| InputState::new(window, cx).default_value(SAMPLE_WINDOW_HEIGHT));
    (width, height)
}

/// 「854 × 480」一行；「×」是符号而非文案，宽度取 PCL 的 90px（约 6 rem）。
fn window_size_row(width: &Entity<InputState>, height: &Entity<InputState>) -> AnyElement {
    h_flex()
        .gap_2()
        .items_center()
        .child(div().w_24().child(Input::new(width)))
        .child(div().child("×"))
        .child(div().w_24().child(Input::new(height)))
        .into_any_element()
}
