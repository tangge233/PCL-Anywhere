//! 页面路由与导航元数据。
//!
//! 顶层路由 [`Route`] 是外壳状态机的类型；各分组的子路由枚举与目录表由 [`page`] 的宏在
//! `download` / `tools` / `setup` 里生成——加 / 删子页只改那张表。
//!
//! 表文件放在这一层（与 [`Route`] 同处一层），代价是要引用 `crate::pages::*` 的渲染入口；
//! 方向与外壳一致（`shell::main_window` 也持有各分组实体），没有反向引用。

pub mod page;

mod download;
mod setup;
mod tools;

pub use download::DownloadRoute;
pub use setup::SetupRoute;
pub use tools::ToolsRoute;

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

/// 主导航项。
pub struct NavItem {
    pub route: Route,
    pub title_key: &'static str,
    pub icon: &'static str,
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
