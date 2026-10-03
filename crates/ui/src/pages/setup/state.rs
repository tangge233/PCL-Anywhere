//! `SetupGroup` 的受控界面状态：一页一个子结构。
//!
//! 这些值只活在内存里（不落盘、不接业务逻辑）。默认值按以下来源填写：
//! * PCL 启动器的同名配置项；
//! * PCL 各设置页控件的 `MaxValue` / `Value`；
//! * 两者都没有的（例如尚未接线的外观项）在字段上逐条注明「示例值」。

use gpui_kit::SharedString;

/// 全部受控值。
pub(super) struct SetupState {
    pub(super) launch: LaunchState,
    pub(super) java: JavaState,
    pub(super) game_manage: GameManageState,
    pub(super) game_link: GameLinkState,
    pub(super) ui: UiState,
    pub(super) language: LanguageState,
    pub(super) misc: MiscState,
    pub(super) log: LogState,
}

impl SetupState {
    pub(super) fn new() -> Self {
        Self {
            launch: LaunchState::defaults(),
            java: JavaState::defaults(),
            game_manage: GameManageState::defaults(),
            game_link: GameLinkState::defaults(),
            ui: UiState::defaults(),
            language: LanguageState::defaults(),
            misc: MiscState::defaults(),
            log: LogState::defaults(),
        }
    }
}

/// 设置 · 启动（对应 PCL 的 `PageSetupLaunch` 设置页）。
pub(super) struct LaunchState {
    /// `LaunchArgumentIndieV2`，默认 4（隔离所有实例）。
    pub(super) isolation: usize,
    /// `LaunchPreferredIpStack`，默认 1（Java 默认）。
    pub(super) ip_stack: usize,
    /// `LaunchArgumentInfo`，默认 "PCL"。
    pub(super) custom_info: SharedString,
    /// `LaunchRamType`，默认 0（自动配置）。
    pub(super) memory_mode: usize,
    /// `LaunchAdvanceLockMemory`，默认 false。
    pub(super) lock_memory: bool,
    /// `LaunchArgumentPriority`，默认 1（中）。
    pub(super) priority: usize,
}

impl LaunchState {
    /// 出厂默认值（页面「初始化」按钮复用）。
    pub(super) fn defaults() -> Self {
        Self {
            isolation: 4,
            ip_stack: 1,
            custom_info: SharedString::from("PCL"),
            memory_mode: 0,
            lock_memory: false,
            priority: 1,
        }
    }

    /// 手动分配内存（滑块可用）：`LaunchRamType == 1`。
    pub(super) fn memory_is_manual(&self) -> bool {
        self.memory_mode == 1
    }
}

/// 设置 · Java（对应 PCL 的 `PageSetupJava` 设置页）。
pub(super) struct JavaState {
    /// 用作默认 Java 的条目下标：0 = 自动选择，第 i 条示例 Java 对应 i + 1
    /// （`java.rs` 的 `auto_select_item` / `runtime_item` 按此偏移读写）。
    pub(super) default_index: usize,
    /// 每条示例 Java 的启用状态。
    pub(super) enabled: Vec<bool>,
}

impl JavaState {
    /// 出厂默认值（页面「初始化」按钮复用）。
    pub(super) fn defaults() -> Self {
        Self {
            default_index: 0,
            enabled: vec![true; super::java::SAMPLE_RUNTIMES.len()],
        }
    }
}

/// 设置 · 游戏管理（对应 PCL 的 `PageSetupGameManage` 设置页）。
pub(super) struct GameManageState {
    /// `ToolDownloadHttpMode`，默认 0（自动）。
    pub(super) http_mode: usize,
    /// `ToolDownloadSource`，默认 1。
    pub(super) file_source: usize,
    /// `ToolDownloadVersion`，默认 1。
    pub(super) version_source: usize,
    /// `ToolDownloadAutoSelectVersion`，默认 true。
    pub(super) auto_select_instance: bool,
    /// `ToolFixAuthlib`，默认 true。
    pub(super) fix_authlib: bool,
    /// `ToolDownloadTranslateV2`，默认 1。
    pub(super) name_format: usize,
    /// `ToolModLocalNameStyle`，默认 0。
    pub(super) comp_name_style: usize,
    /// `ToolDownloadQuickBehavior`，默认 0（总是询问）。
    pub(super) quick_behavior: usize,
    /// `ToolDownloadIgnoreQuilt`，默认 true。
    pub(super) ignore_quilt: bool,
    /// `ToolDownloadAutoInstallDependencies`，默认 true。
    pub(super) auto_install_deps: bool,
}

impl GameManageState {
    /// 页面「初始化」按钮使用的默认值。
    pub(super) fn defaults() -> Self {
        Self {
            http_mode: 0,
            file_source: 1,
            version_source: 1,
            auto_select_instance: true,
            fix_authlib: true,
            name_format: 1,
            comp_name_style: 0,
            quick_behavior: 0,
            ignore_quilt: true,
            auto_install_deps: true,
        }
    }
}

/// 设置 · 联机（对应 PCL 的 `PageSetupGameLink` 设置页）。
pub(super) struct GameLinkState {
    /// `LinkUsername`，默认空。
    pub(super) username: SharedString,
    /// `LinkProtocolPreference`，默认 0（TCP）。
    pub(super) protocol: usize,
    /// `LinkLatencyFirstMode`，默认 true。
    pub(super) latency_first: bool,
    /// `LinkTryPunchSym`，默认 true。
    pub(super) try_punch_sym: bool,
    /// `LinkEnableIPv6`，默认 true。
    pub(super) enable_ipv6: bool,
    /// `LinkEnableCliOutput`，默认 false。
    pub(super) cli_output: bool,
}

impl GameLinkState {
    /// 出厂默认值（页面「初始化」按钮复用）。
    pub(super) fn defaults() -> Self {
        Self {
            username: SharedString::default(),
            protocol: 0,
            latency_first: true,
            try_punch_sym: true,
            enable_ipv6: true,
            cli_output: false,
        }
    }
}

/// 设置 · 个性化（对应 PCL 的 `PageSetupUI` 设置页）。
///
/// PCL 已接线部分只有配色模式与亮/暗配色主题，其余外观项尚未接线；
/// 这里的默认值取设置页控件的 `Value` 与 PCL 同名配置的常见取值，未接线项标为示例值。
pub(super) struct UiState {
    /// `UiDarkMode`，默认 2（跟随系统）。
    pub(super) theme_mode: usize,
    /// `UiLightColor`，默认 1（CatBlue）。
    pub(super) light_color: usize,
    /// `UiDarkColor`，默认 1（CatBlue）。
    pub(super) dark_color: usize,
    /// `UiHiddenFunctionHidden` 之外的显示开关，默认显示（true）。
    pub(super) show_logo: bool,
    pub(super) lock_window: bool,
    pub(super) trivia_hint: bool,
    /// 高级材质，示例值 false。
    pub(super) advanced_material: bool,
    /// `UiBlurType`，示例值 0（高斯模糊）。
    pub(super) blur_method: usize,
    /// 全局字体，示例值为空（跟随系统）。
    pub(super) font_global: SharedString,
    /// MOTD 字体，示例值为空（跟随系统）。
    pub(super) font_motd: SharedString,
    /// `UiBackgroundSuit`，示例值 0（智能）。
    pub(super) background_suit: usize,
    /// `UiAutoPauseVideo`，默认 true。
    pub(super) pause_video: bool,
    /// `UiBackgroundColorful`，默认 true。
    pub(super) colorful: bool,
    /// `UiMusicRandom`，默认 true。
    pub(super) music_shuffle: bool,
    /// `UiMusicAuto`，默认 true。
    pub(super) music_auto: bool,
    /// `UiMusicStart`，默认 false。
    pub(super) music_start: bool,
    /// `UiMusicStop`，默认 false。
    pub(super) music_stop: bool,
    /// `UiMusicSMTC`，默认 true。
    pub(super) music_smtc: bool,
    /// 自定义主页样式，示例值 1（预设）。
    pub(super) homepage_type: usize,
    /// 自定义主页预设，示例值 0（PCL 小知识）。
    pub(super) homepage_preset: usize,
    /// `UiCustomNet`，示例值为空。
    pub(super) homepage_url: SharedString,
    /// 启动器 Logo 样式，示例值 0（无）。
    pub(super) logo_type: usize,
    /// Logo 左对齐，示例值 false。
    pub(super) logo_left: bool,
    /// 自定义 Logo 文字，示例值为空。
    pub(super) logo_text: SharedString,
    /// 「隐藏功能」开关，顺序与 [`super::ui::HIDDEN_ITEMS`] 一致，默认全部关闭。
    pub(super) hidden: Vec<bool>,
}

impl UiState {
    /// 出厂默认值（页面「初始化」按钮复用）。
    pub(super) fn defaults() -> Self {
        Self {
            theme_mode: 2,
            light_color: 1,
            dark_color: 1,
            show_logo: true,
            lock_window: false,
            trivia_hint: true,
            advanced_material: false,
            blur_method: 0,
            font_global: SharedString::default(),
            font_motd: SharedString::default(),
            background_suit: 0,
            pause_video: true,
            colorful: true,
            music_shuffle: true,
            music_auto: true,
            music_start: false,
            music_stop: false,
            music_smtc: true,
            homepage_type: 1,
            homepage_preset: 0,
            homepage_url: SharedString::default(),
            logo_type: 0,
            logo_left: false,
            logo_text: SharedString::default(),
            hidden: vec![false; super::ui::HIDDEN_ITEMS.len()],
        }
    }
}

/// 设置 · 语言（对应 PCL 的 `PageSetupLauncherLanguage` 设置页）。
pub(super) struct LanguageState {
    /// UI 语言下标（0 = 跟随系统）。
    pub(super) language: usize,
    /// 区域格式下标（0 = 跟随系统）。
    pub(super) format_culture: usize,
}

impl LanguageState {
    /// 出厂默认值（页面「初始化」按钮复用）。
    pub(super) fn defaults() -> Self {
        Self {
            language: 0,
            format_culture: 0,
        }
    }
}

/// 设置 · 杂项（对应 PCL 的 `PageSetupLauncherMisc` 设置页）。
pub(super) struct MiscState {
    /// `SystemTelemetry`，默认 false。
    pub(super) telemetry: bool,
    /// `SystemNetEnableDoH`，默认 true。
    pub(super) doh: bool,
    /// `SystemHttpProxyType`，默认 1（使用系统代理）。
    pub(super) proxy_type: usize,
}

impl MiscState {
    /// 出厂默认值（页面「初始化」按钮复用）。
    pub(super) fn defaults() -> Self {
        Self {
            telemetry: false,
            doh: true,
            proxy_type: 1,
        }
    }

    /// 自定义代理时才显示地址等输入框。
    pub(super) fn is_custom_proxy(&self) -> bool {
        self.proxy_type == 2
    }
}

/// 设置 · 日志（对应 PCL 的 `PageSetupLog` 设置页）：日志列表为示例数据，可被「清理历史日志」
/// 清空。
pub(super) struct LogState {
    pub(super) lines: Vec<SharedString>,
}

impl LogState {
    /// 出厂默认值（页面「初始化」按钮复用）。
    pub(super) fn defaults() -> Self {
        Self {
            lines: super::log::SAMPLE_LINES
                .iter()
                .map(|line| SharedString::from(*line))
                .collect(),
        }
    }
}
