//! 设置 · 游戏管理（对应 PCL 的 `PageSetupGameManage` 设置页：下载行为 / 社区资源行为 /
//! 初始化三张卡片）。

use gpui_kit::component::setting::SettingGroup;
use gpui_kit::*;

use super::{NO_ENABLE, SetupGroup, choices, group, slider_state};
use crate::components::ButtonColor;

/// 滑块上限与默认值取自 PCL 设置页控件（`MaxValue` / `Value`）与 `Config.Download.*`：
/// `ToolDownloadThread` 63、`ToolDownloadFileConnection` 7、`ToolDownloadSpeed` 42。
/// 线程数与连接数的下限 1 为本页设定（PCL 的 `MySlider` 范围是 0..MaxValue）。
const MAX_THREADS: f32 = 256.;
const DEFAULT_THREADS: f32 = 63.;
const MAX_CONNECTIONS: f32 = 64.;
const DEFAULT_CONNECTIONS: f32 = 7.;
const MAX_SPEED: f32 = 1024.;
const DEFAULT_SPEED: f32 = 42.;

pub(crate) fn page(
    this: &mut SetupGroup,
    _window: &mut Window,
    cx: &mut Context<SetupGroup>,
) -> Vec<SettingGroup> {
    let fields = &this.fields;

    let threads = slider_state(cx, 1., MAX_THREADS, 1., DEFAULT_THREADS);
    let connections = slider_state(cx, 1., MAX_CONNECTIONS, 1., DEFAULT_CONNECTIONS);
    let speed = slider_state(cx, 0., MAX_SPEED, 1., DEFAULT_SPEED);

    // 卡片一：下载行为。PCL 设置页的卡片标题与首个条目同为
    // `Setup.GameManage.Download.Threads`，这里改用同页更贴切的
    // `Setup.GameManage.Source.Title`。
    let download = vec![
        fields.slider(
            "Setup.GameManage.Download.Threads",
            Some("Setup.GameManage.Download.Threads.ToolTip"),
            &threads,
            "",
            NO_ENABLE,
        ),
        fields.slider(
            "Setup.GameManage.Download.FileConnections",
            Some("Setup.GameManage.Download.FileConnections.ToolTip"),
            &connections,
            "",
            NO_ENABLE,
        ),
        fields.slider(
            "Setup.GameManage.Download.SpeedLimit",
            Some("Setup.GameManage.Download.SpeedLimit.ToolTip"),
            &speed,
            " MB/s",
            NO_ENABLE,
        ),
        fields.dropdown(
            "Setup.GameManage.Download.HttpProtocol",
            None,
            choices(&[
                "Setup.GameManage.Download.HttpProtocol.Auto",
                "Setup.GameManage.Download.HttpProtocol.Http11",
                "Setup.GameManage.Download.HttpProtocol.Http2",
            ]),
            |state| state.game_manage.http_mode,
            |state, index| state.game_manage.http_mode = index,
        ),
        // PCL 设置页该下拉框的 `SourceItems` 首项误用了本行的标签键
        // `Setup.GameManage.Community.Source`（「下载源」，即本下拉框的标题），
        // 这里改用游戏资源源的三项语义键。
        fields.dropdown(
            "Setup.GameManage.Community.Source",
            None,
            choices(&[
                "Setup.GameManage.Source.UseMirror",
                "Setup.GameManage.Source.PreferOfficialThenMirror",
                "Setup.GameManage.Source.UseOfficial",
            ]),
            |state| state.game_manage.file_source,
            |state, index| state.game_manage.file_source = index,
        ),
        // PCL 设置页此处标签为 `Setup.GameManage.Download.InstallBehavior`（安装行为），
        // 但绑定的是版本列表源，这里按语义改用 `Setup.GameManage.Source.Version`。
        fields.dropdown(
            "Setup.GameManage.Source.Version",
            None,
            choices(&[
                "Setup.GameManage.Source.UseOfficial",
                "Setup.GameManage.Source.Version.Mirror",
            ]),
            |state| state.game_manage.version_source,
            |state, index| state.game_manage.version_source = index,
        ),
        fields.checkbox(
            "gamemanage-auto-select-instance",
            "Setup.GameManage.Download.AutoSelectInstance",
            None,
            |state| state.game_manage.auto_select_instance,
            |state, value| state.game_manage.auto_select_instance = value,
        ),
        fields.checkbox(
            "gamemanage-fix-authlib",
            "Setup.GameManage.Download.FixAuthlib",
            Some("Setup.GameManage.Download.FixAuthlib.ToolTip"),
            |state| state.game_manage.fix_authlib,
            |state, value| state.game_manage.fix_authlib = value,
        ),
    ];

    // 卡片二：社区资源获取行为。
    let community = vec![
        fields.dropdown(
            "Setup.GameManage.Community.FilenameFormat",
            Some("Setup.GameManage.Community.FilenameFormat.ToolTip"),
            choices(&[
                "Setup.GameManage.Community.FilenameFormat.Style0",
                "Setup.GameManage.Community.FilenameFormat.Style1",
                "Setup.GameManage.Community.FilenameFormat.Style2",
                "Setup.GameManage.Community.FilenameFormat.Style3",
                "Setup.GameManage.Community.FilenameFormat.Style4",
            ]),
            |state| state.game_manage.name_format,
            |state, index| state.game_manage.name_format = index,
        ),
        fields.dropdown(
            "Setup.GameManage.Community.ModManageStyle",
            Some("Setup.GameManage.Community.ModManageStyle.ToolTip"),
            choices(&[
                "Setup.GameManage.Community.ModManageStyle.TitleFilename",
                "Setup.GameManage.Community.ModManageStyle.TitleTranslation",
            ]),
            |state| state.game_manage.comp_name_style,
            |state, index| state.game_manage.comp_name_style = index,
        ),
        fields.dropdown(
            "Setup.GameManage.Community.QuickDownload",
            Some("Setup.GameManage.Community.QuickDownload.ToolTip"),
            choices(&[
                "Setup.GameManage.Community.QuickDownload.AlwaysAsk",
                "Setup.GameManage.Community.QuickDownload.CurrentInstance",
                "Setup.GameManage.Community.QuickDownload.AskInstance",
                "Setup.GameManage.Community.QuickDownload.AskPath",
            ]),
            |state| state.game_manage.quick_behavior,
            |state, index| state.game_manage.quick_behavior = index,
        ),
        fields.checkbox(
            "gamemanage-hide-quilt",
            "Setup.GameManage.Community.HideQuilt",
            Some("Setup.GameManage.Community.HideQuilt.ToolTip"),
            |state| state.game_manage.ignore_quilt,
            |state, value| state.game_manage.ignore_quilt = value,
        ),
        fields.checkbox(
            "gamemanage-auto-install-deps",
            "Setup.GameManage.Community.AutoInstallDependencies",
            Some("Setup.GameManage.Community.AutoInstallDependencies.ToolTip"),
            |state| state.game_manage.auto_install_deps,
            |state, value| state.game_manage.auto_install_deps = value,
        ),
    ];

    // 卡片三：初始化（PCL 的 `Config.Download.Reset()`，这里只重置本页的界面状态）。
    let reset_threads = threads.clone();
    let reset_connections = connections.clone();
    let reset_speed = speed.clone();
    let reset = vec![fields.button(
        "gamemanage-reset",
        "Common.Action.Reset",
        ButtonColor::Normal,
        move |state, window, cx| {
            state.game_manage = super::state::GameManageState::defaults();
            reset_threads.update(cx, |slider, cx| {
                slider.set_value(DEFAULT_THREADS, window, cx);
            });
            reset_connections.update(cx, |slider, cx| {
                slider.set_value(DEFAULT_CONNECTIONS, window, cx);
            });
            reset_speed.update(cx, |slider, cx| {
                slider.set_value(DEFAULT_SPEED, window, cx);
            });
        },
    )];

    vec![
        group("Setup.GameManage.Source.Title", download),
        group("Setup.GameManage.Community.Title", community),
        group("Common.Action.Reset", reset),
    ]
}
