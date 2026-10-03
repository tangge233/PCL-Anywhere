//! 页面路由与导航元数据（对应 PCL 启动器的页面目录）。
//!
//! 目录在子模块 [`catalog`] 中显式登记，不再用字符串反射：主导航顺序、选择栏顺序、分组标题、
//! 图标与条目右侧动作按钮都由该表决定，界面代码只负责渲染。

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

/// 页面路由：变体携带分组内子页；`Instance(_)` 为副页面（`is_sub`，标题栏显示返回栏），
/// 其余变体属于主导航分组，在标题栏显示选中态。
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
    /// 分组标题（键名）与该组顶部间距（px），对应 `SelectorSectionKey` / `SelectorSectionTopMargin`。
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

    /// 页面在选择栏 / 列表中的名称，也用于子页尚未接入独立标题时的占位文案。
    ///
    /// `InstanceSetup`、`SaveManagement` 两个文案键带 `{0}` 参数，`label` 返回未替换的模板；
    /// 当前仅 `sub_page_name`（下载分组）调用本方法，不得拿它直接展示实例副页标题。
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
}
