//! 设置 · 联机（对应 PCL 的 `PageSetupGameLink` 设置页：协议偏好 / 延迟优先 / 初始化
//! 三张卡片）。
//!
//! PCL 同页还有「NAT 类型网络测试」卡片，需要额外的测试后端，本仓库尚未实现。

use gpui_kit::component::setting::SettingPage;

use super::{Fields, choices, group};
use crate::components::{ButtonColor, lucide};
use crate::i18n;

pub(super) fn page(fields: &Fields) -> SettingPage {
    let preference = vec![
        fields.input(
            "Setup.GameLink.Username",
            Some("Setup.GameLink.Username.ToolTip"),
            |state| state.game_link.username.clone(),
            |state, value| state.game_link.username = value,
        ),
        fields.dropdown(
            "Setup.GameLink.Protocol.Preference",
            None,
            choices(&["Setup.GameLink.Protocol.Tcp", "Setup.GameLink.Protocol.Udp"]),
            |state| state.game_link.protocol,
            |state, index| state.game_link.protocol = index,
        ),
    ];

    let latency = vec![
        fields.checkbox(
            "gamelink-latency-first",
            "Setup.GameLink.LatencyFirstMode",
            Some("Setup.GameLink.LatencyFirstMode.ToolTip"),
            |state| state.game_link.latency_first,
            |state, value| state.game_link.latency_first = value,
        ),
        fields.checkbox(
            "gamelink-port-guess",
            "Setup.GameLink.PortGuessNat",
            Some("Setup.GameLink.PortGuessNat.ToolTip"),
            |state| state.game_link.try_punch_sym,
            |state, value| state.game_link.try_punch_sym = value,
        ),
        fields.checkbox(
            "gamelink-ipv6",
            "Setup.GameLink.EnableIPv6",
            None,
            |state| state.game_link.enable_ipv6,
            |state, value| state.game_link.enable_ipv6 = value,
        ),
        fields.checkbox(
            "gamelink-cli-output",
            "Setup.GameLink.CliDebugOutput",
            Some("Setup.GameLink.CliDebugOutput.ToolTip"),
            |state| state.game_link.cli_output,
            |state, value| state.game_link.cli_output = value,
        ),
        fields.paragraph("Setup.GameLink.RestartRequired", false),
    ];

    let reset = vec![fields.button(
        "gamelink-reset",
        "Common.Action.Reset",
        ButtonColor::Normal,
        |state, _, _| state.game_link = super::state::GameLinkState::defaults(),
    )];

    SettingPage::new(i18n::lang("Setup.Left.Item.GameLink"))
        .icon(lucide("bubbles"))
        .group(group("Setup.GameLink.Protocol.Preference", preference))
        .group(group("Setup.GameLink.LatencyFirstMode", latency))
        .group(group("Common.Action.Reset", reset))
}
