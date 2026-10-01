//! 页面目录表：主导航与各分组选择栏的条目。
//!
//! 顺序即显示顺序；分组标题、图标与条目右侧的动作按钮都由这张表决定，界面代码只负责渲染。

use super::{DownloadRoute, SetupRoute, ToolsRoute};
use super::{NavItem, Route, SelectorAction, SelectorEntry};

const fn entry(
    route: Route,
    title_key: &'static str,
    icon: &'static str,
    action: Option<SelectorAction>,
) -> SelectorEntry {
    SelectorEntry {
        route,
        title_key,
        icon,
        section: None,
        action,
    }
}

const fn grouped(
    mut entry: SelectorEntry,
    section_key: &'static str,
    top_margin: f32,
) -> SelectorEntry {
    entry.section = Some((section_key, top_margin));
    entry
}

/// 主导航（顺序即显示顺序）。
pub const NAV_ITEMS: &[NavItem] = &[
    NavItem {
        route: Route::Launch,
        title_key: "Main.Tab.Launch",
        icon: "play",
    },
    NavItem {
        route: Route::Download(DownloadRoute::Minecraft),
        title_key: "Main.Tab.Download",
        icon: "download",
    },
    NavItem {
        route: Route::Setup(SetupRoute::Launch),
        title_key: "Main.Tab.Settings",
        icon: "settings",
    },
    NavItem {
        route: Route::Tools(ToolsRoute::GameLink),
        title_key: "Main.Tab.Tools",
        icon: "wrench",
    },
];

pub(super) const DOWNLOAD_ENTRIES: &[SelectorEntry] = &[
    entry(
        Route::Download(DownloadRoute::Minecraft),
        "Download.Left.Minecraft",
        "boxes",
        Some(SelectorAction::Refresh),
    ),
    grouped(
        entry(
            Route::Download(DownloadRoute::Mod),
            "Download.Left.Mod",
            "puzzle",
            Some(SelectorAction::Refresh),
        ),
        "Download.Left.CommunityResources",
        10.,
    ),
    entry(
        Route::Download(DownloadRoute::Modpack),
        "Download.Left.Modpack",
        "package",
        Some(SelectorAction::Refresh),
    ),
    entry(
        Route::Download(DownloadRoute::DataPack),
        "Download.Left.DataPack",
        "file-archive",
        Some(SelectorAction::Refresh),
    ),
    entry(
        Route::Download(DownloadRoute::ResourcePack),
        "Download.Left.ResourcePack",
        "layers",
        Some(SelectorAction::Refresh),
    ),
    entry(
        Route::Download(DownloadRoute::Shader),
        "Download.Left.Shader",
        "sparkles",
        Some(SelectorAction::Refresh),
    ),
    entry(
        Route::Download(DownloadRoute::World),
        "Download.Left.World",
        "globe",
        Some(SelectorAction::Refresh),
    ),
    entry(
        Route::Download(DownloadRoute::Favorites),
        "Download.Left.Favorites",
        "heart",
        Some(SelectorAction::Refresh),
    ),
    grouped(
        entry(
            Route::Download(DownloadRoute::Client),
            "Download.Left.Minecraft",
            "package",
            Some(SelectorAction::Refresh),
        ),
        "Download.Left.Installation",
        10.,
    ),
    entry(
        Route::Download(DownloadRoute::OptiFine),
        "Download.Left.Optifine",
        "gauge",
        Some(SelectorAction::Refresh),
    ),
    entry(
        Route::Download(DownloadRoute::Forge),
        "Download.Left.Forge",
        "anvil",
        Some(SelectorAction::Refresh),
    ),
    entry(
        Route::Download(DownloadRoute::NeoForge),
        "Download.Left.NeoForge",
        "cat",
        Some(SelectorAction::Refresh),
    ),
    entry(
        Route::Download(DownloadRoute::Cleanroom),
        "Download.Left.Cleanroom",
        "flask-conical",
        Some(SelectorAction::Refresh),
    ),
    entry(
        Route::Download(DownloadRoute::Fabric),
        "Download.Left.Fabric",
        "scroll",
        Some(SelectorAction::Refresh),
    ),
    entry(
        Route::Download(DownloadRoute::LegacyFabric),
        "Download.Left.LegacyFabric",
        "scroll",
        Some(SelectorAction::Refresh),
    ),
    entry(
        Route::Download(DownloadRoute::LabyMod),
        "Download.Left.LabyMod",
        "box",
        Some(SelectorAction::Refresh),
    ),
    entry(
        Route::Download(DownloadRoute::LiteLoader),
        "Download.Left.LiteLoader",
        "egg",
        Some(SelectorAction::Refresh),
    ),
];

pub(super) const SETUP_ENTRIES: &[SelectorEntry] = &[
    grouped(
        entry(
            Route::Setup(SetupRoute::Launch),
            "Setup.Left.Item.Launch",
            "rocket",
            Some(SelectorAction::Reset),
        ),
        "Setup.Left.Category.Game",
        0.,
    ),
    entry(
        Route::Setup(SetupRoute::Java),
        "Setup.Left.Item.Java",
        "coffee",
        Some(SelectorAction::Refresh),
    ),
    entry(
        Route::Setup(SetupRoute::GameManage),
        "Setup.Left.Item.GameManage",
        "book-marked",
        Some(SelectorAction::Reset),
    ),
    grouped(
        entry(
            Route::Setup(SetupRoute::GameLink),
            "Setup.Left.Item.GameLink",
            "bubbles",
            Some(SelectorAction::Reset),
        ),
        "Setup.Left.Category.Tools",
        10.,
    ),
    grouped(
        entry(
            Route::Setup(SetupRoute::Ui),
            "Setup.Left.Item.Ui",
            "palette",
            Some(SelectorAction::Reset),
        ),
        "Setup.Left.Category.Launcher",
        10.,
    ),
    entry(
        Route::Setup(SetupRoute::Language),
        "Setup.Language.Title",
        "earth",
        Some(SelectorAction::Reset),
    ),
    entry(
        Route::Setup(SetupRoute::Misc),
        "Setup.Left.Item.Misc",
        "monitor-cog",
        Some(SelectorAction::Reset),
    ),
    grouped(
        entry(
            Route::Setup(SetupRoute::About),
            "Setup.Left.Item.About",
            "info",
            None,
        ),
        "Setup.Left.Category.About",
        10.,
    ),
    entry(
        Route::Setup(SetupRoute::Update),
        "Setup.Left.Item.Update",
        "refresh-cw",
        None,
    ),
    entry(
        Route::Setup(SetupRoute::Feedback),
        "Setup.Left.Item.Feedback",
        "message-circle",
        Some(SelectorAction::Refresh),
    ),
    entry(
        Route::Setup(SetupRoute::Log),
        "Setup.Left.Item.Log",
        "scroll-text",
        None,
    ),
];

pub(super) const TOOLS_ENTRIES: &[SelectorEntry] = &[
    grouped(
        entry(
            Route::Tools(ToolsRoute::GameLink),
            "Tools.Left.Lobby",
            "bubbles",
            None,
        ),
        "Tools.Left.Multiplayer",
        10.,
    ),
    grouped(
        entry(
            Route::Tools(ToolsRoute::Test),
            "Tools.Test.Title",
            "flask-conical",
            None,
        ),
        "Tools.Left.Utilities",
        10.,
    ),
];
