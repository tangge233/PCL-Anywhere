//! 设置 · 杂项（对应 `PCL/Views/Setup/PageSetupLauncherMisc.axaml`：系统卡片 + 网络卡片）。
//!
//! 上游同页的「系统公告」「最大日志行数」「动画帧率」「禁用硬件加速」「设置导出/导入」等卡片
//! 在本仓库没有对应配置项或消费方，未移植。

use gpui_kit::base::v_flex;
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::component::setting::{SettingField, SettingItem, SettingPage};
use gpui_kit::*;

use super::{Fields, SetupGroup, group, item};
use crate::components::{AppButton, lucide};
use crate::i18n;

/// 代理地址输入框的示例提示（XAML 的 `HintText="http://127.0.0.1:1080/"`）。
const SAMPLE_PROXY_ADDRESS: &str = "http://127.0.0.1:1080/";

pub(super) fn page(
    fields: &Fields,
    window: &mut Window,
    cx: &mut Context<SetupGroup>,
) -> SettingPage {
    // 代理信息只在界面上填写，不写回状态（应用代理属于业务逻辑）。
    let address = cx.new(|cx| {
        InputState::new(window, cx).placeholder(SharedString::from(SAMPLE_PROXY_ADDRESS))
    });
    let username_placeholder = i18n::lang("Setup.Misc.Network.Proxy.Optional");
    let username =
        cx.new(|cx| InputState::new(window, cx).placeholder(username_placeholder.clone()));
    let password = cx.new(|cx| InputState::new(window, cx).placeholder(username_placeholder));

    let system = vec![fields.checkbox(
        "misc-telemetry",
        "Setup.Misc.System.Telemetry",
        Some("Setup.Misc.System.Telemetry.ToolTip"),
        |state| state.misc.telemetry,
        |state, value| state.misc.telemetry = value,
    )];

    let network = vec![
        fields.checkbox(
            "misc-doh",
            "Setup.Misc.Network.DoH",
            Some("Setup.Misc.Network.DoH.ToolTip"),
            |state| state.misc.doh,
            |state, value| state.misc.doh = value,
        ),
        fields.radios(
            "misc-proxy-type",
            "Setup.Misc.Network.HttpProxy",
            None,
            &[
                "Setup.Misc.Network.Proxy.None",
                "Setup.Misc.Network.Proxy.System",
                "Setup.Misc.Network.Proxy.Custom",
            ],
            |state| state.misc.proxy_type,
            |state, index| state.misc.proxy_type = index,
        ),
        fields.hint("Setup.Misc.Network.Proxy.UntrustedWarning"),
        proxy_input(
            fields,
            "Setup.Misc.Network.Proxy.Custom",
            Some("Setup.Misc.Network.Proxy.InvalidWarning"),
            &address,
        ),
        proxy_input(fields, "Setup.Misc.Network.Proxy.Username", None, &username),
        proxy_input(fields, "Setup.Misc.Network.Proxy.Password", None, &password),
        // 「应用代理信息」：应用代理是业务逻辑，这里只保留排版（上游由 ViewModel 命令完成）。
        fields.element(&["Setup.Misc.Network.Proxy.Apply"], move |_, _| {
            v_flex()
                .w_full()
                .child(AppButton::new(
                    "misc-proxy-apply",
                    i18n::lang("Setup.Misc.Network.Proxy.Apply"),
                ))
                .into_any_element()
        }),
    ];

    SettingPage::new(i18n::lang("Setup.Left.Item.Misc"))
        .icon(lucide("monitor-cog"))
        .group(group("Setup.Misc.System.Title", system))
        .group(group("Setup.Misc.Network.Title", network))
}

/// 代理信息输入框：只有选中「自定义代理」时才可编辑（PCL 隐藏其余两种模式下的子表单）。
fn proxy_input(
    fields: &Fields,
    title_key: &str,
    tip_key: Option<&str>,
    input: &Entity<InputState>,
) -> SettingItem {
    let fields = fields.clone();
    let input = input.clone();
    let field = SettingField::<SharedString>::render(move |_, _, _| {
        let disabled = !fields.read(|state| state.misc.is_custom_proxy());
        Input::new(&input).disabled(disabled).into_any_element()
    });
    super::tip(item(title_key, field), tip_key)
}
