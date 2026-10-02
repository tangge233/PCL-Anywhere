//! 页面路由与导航元数据（对应 .NET 版本的 `PCL/Navigation/*Pages.cs` 页面目录）。
//!
//! 目录在这里显式登记，不再用字符串反射：主导航顺序、选择栏顺序、分组标题、图标与
//! 条目右侧动作按钮都由这张表决定，界面代码只负责渲染。

use gpui_kit::SharedString;

mod catalog;

pub use catalog::NAV_ITEMS;
use catalog::{DOWNLOAD_ENTRIES, SETUP_ENTRIES, TOOLS_ENTRIES};

use crate::i18n;

/// 主导航分组。`Instance` 不参与主导航，只通过副页面进入。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PageGroup {
    Launch,
    Download,
    Setup,
    Tools,
    Instance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Route {
    Launch,
    Instance(InstanceRoute),
    Download(DownloadRoute),
    Setup(SetupRoute),
    Tools(ToolsRoute),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InstanceRoute {
    /// 实例详情：文件夹 / 实例 / 管理三栏（`PageInstanceMerged`）。
    Select,
    /// 实例设置：同一合并页，进入时展开管理栏的「设置」页。
    Setup,
    /// 存档管理。
    Saves,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DownloadRoute {
    Minecraft,
    Mod,
    Modpack,
    DataPack,
    ResourcePack,
    Shader,
    World,
    Favorites,
    Client,
    OptiFine,
    Forge,
    NeoForge,
    Cleanroom,
    Fabric,
    LegacyFabric,
    LabyMod,
    LiteLoader,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SetupRoute {
    Launch,
    Java,
    GameManage,
    GameLink,
    Ui,
    Language,
    Misc,
    About,
    Update,
    Feedback,
    Log,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ToolsRoute {
    GameLink,
    Test,
}

/// 主导航项。
pub struct NavItem {
    pub route: Route,
    pub title_key: &'static str,
    pub icon: &'static str,
}

/// 选择栏条目右侧的图标动作。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectorAction {
    /// `lucide/refresh-cw` + `Common.Action.Refresh`
    Refresh,
    /// `lucide/rotate-ccw` + `Common.Action.Initialize`
    Reset,
}

impl SelectorAction {
    pub fn icon(self) -> &'static str {
        match self {
            Self::Refresh => "refresh-cw",
            Self::Reset => "rotate-ccw",
        }
    }

    pub fn tooltip(self) -> SharedString {
        match self {
            Self::Refresh => i18n::lang("Common.Action.Refresh"),
            Self::Reset => i18n::lang("Common.Action.Initialize"),
        }
    }
}

/// 选择栏条目。
pub struct SelectorEntry {
    pub route: Route,
    pub title_key: &'static str,
    pub icon: &'static str,
    /// 分组标题（键名）与与上一组的间距，对应 `SelectorSectionKey` / `SelectorSectionTopMargin`。
    pub section: Option<(&'static str, f32)>,
    pub action: Option<SelectorAction>,
}

impl Route {
    pub fn group(self) -> PageGroup {
        match self {
            Self::Launch => PageGroup::Launch,
            Self::Instance(_) => PageGroup::Instance,
            Self::Download(_) => PageGroup::Download,
            Self::Setup(_) => PageGroup::Setup,
            Self::Tools(_) => PageGroup::Tools,
        }
    }

    /// 副页面：标题栏显示返回栏与页面标题，主导航隐藏。
    pub fn is_sub(self) -> bool {
        matches!(self, Self::Instance(_))
    }

    /// 副页面的标题栏文案；顶级页面没有标题（返回栏不出现）。
    pub fn title(self) -> Option<SharedString> {
        match self {
            Self::Instance(route) => Some(match route {
                InstanceRoute::Select => i18n::lang("Main.Title.InstanceSelect"),
                InstanceRoute::Setup => i18n::lang_with_args(
                    "Main.Title.InstanceSetup",
                    &[&i18n::lang("Common.State.Unknown")],
                ),
                InstanceRoute::Saves => i18n::lang_with_args(
                    "Main.Title.SaveManagement",
                    &[&i18n::lang("Common.State.Unknown")],
                ),
            }),
            _ => None,
        }
    }

    /// 页面在选择栏 / 列表中的名称，也用于未迁移页面的占位文案。
    pub fn label(self) -> SharedString {
        match self {
            Self::Launch => i18n::lang("Main.Tab.Launch"),
            Self::Instance(route) => match route {
                InstanceRoute::Select => i18n::lang("Main.Title.InstanceSelect"),
                InstanceRoute::Setup => i18n::lang("Main.Title.InstanceSetup"),
                InstanceRoute::Saves => i18n::lang("Main.Title.SaveManagement"),
            },
            Self::Download(_) => i18n::lang("Main.Tab.Download"),
            Self::Setup(_) => i18n::lang("Main.Tab.Settings"),
            Self::Tools(_) => i18n::lang("Main.Tab.Tools"),
        }
    }

    pub fn selector_entries(self) -> &'static [SelectorEntry] {
        match self.group() {
            PageGroup::Launch | PageGroup::Instance => &[],
            PageGroup::Download => DOWNLOAD_ENTRIES,
            PageGroup::Setup => SETUP_ENTRIES,
            PageGroup::Tools => TOOLS_ENTRIES,
        }
    }

    /// 某分组在选择栏中的默认页面。
    pub fn default_route(group: PageGroup) -> Self {
        match group {
            PageGroup::Launch => Self::Launch,
            PageGroup::Download => Self::Download(DownloadRoute::Minecraft),
            PageGroup::Setup => Self::Setup(SetupRoute::Launch),
            PageGroup::Tools => Self::Tools(ToolsRoute::GameLink),
            PageGroup::Instance => Self::Instance(InstanceRoute::Select),
        }
    }

    /// 主导航中对应的条目下标。
    pub fn nav_index(self) -> Option<usize> {
        let group = self.group();
        NAV_ITEMS
            .iter()
            .position(|item| item.route.group() == group)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_match_their_routes() {
        assert_eq!(Route::Launch.group(), PageGroup::Launch);
        assert_eq!(
            Route::Instance(InstanceRoute::Saves).group(),
            PageGroup::Instance
        );
        assert!(Route::Instance(InstanceRoute::Setup).is_sub());
        assert!(!Route::Setup(SetupRoute::Java).is_sub());
    }

    #[test]
    fn every_group_has_a_default_route_in_its_selector() {
        for group in [
            PageGroup::Launch,
            PageGroup::Download,
            PageGroup::Setup,
            PageGroup::Tools,
        ] {
            let default = Route::default_route(group);
            let entries = default.selector_entries();
            if !entries.is_empty() {
                assert!(
                    entries.iter().any(|entry| entry.route == default),
                    "{group:?} 的默认页面不在选择栏中"
                );
            }
        }
    }
}
