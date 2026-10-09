//! 下载页的子路由与左栏目录：一行一页，顺序即显示顺序。

use crate::pages::download::DownloadGroup;
use crate::shell::route::page::nav_pages;

nav_pages! {
    /// 下载页左栏（PCL 的下载页 17 个条目）。
    pub enum DownloadRoute for DownloadGroup as Route::Download {
        Minecraft: "Download.Left.Minecraft", "boxes", render DownloadGroup::render_minecraft;
        Mod: "Download.Left.Mod", "puzzle", section "Download.Left.CommunityResources";
        Modpack: "Download.Left.Modpack", "package";
        DataPack: "Download.Left.DataPack", "file-archive";
        ResourcePack: "Download.Left.ResourcePack", "layers";
        Shader: "Download.Left.Shader", "sparkles";
        World: "Download.Left.World", "globe";
        Favorites: "Download.Left.Favorites", "heart";
        Client: "Download.Left.Minecraft", "package", section "Download.Left.Installation";
        OptiFine: "Download.Left.Optifine", "gauge";
        Forge: "Download.Left.Forge", "anvil";
        NeoForge: "Download.Left.NeoForge", "cat";
        Cleanroom: "Download.Left.Cleanroom", "flask-conical";
        Fabric: "Download.Left.Fabric", "scroll";
        LegacyFabric: "Download.Left.LegacyFabric", "scroll";
        LabyMod: "Download.Left.LabyMod", "box";
        LiteLoader: "Download.Left.LiteLoader", "egg";
    }
}

#[cfg(test)]
mod tests {
    use super::DownloadRoute;

    use crate::shell::route::page::{PageRoute as _, snapshot_rows};

    /// 显示顺序的规格：顺序、文案键、图标、分组标题。改表要同步改这里。
    /// 快照放在表旁边，加 / 删一页只动这一个文件。
    const SNAPSHOT: &[(DownloadRoute, &str, &str, Option<&str>)] = &[
        (
            DownloadRoute::Minecraft,
            "Download.Left.Minecraft",
            "boxes",
            None,
        ),
        (
            DownloadRoute::Mod,
            "Download.Left.Mod",
            "puzzle",
            Some("Download.Left.CommunityResources"),
        ),
        (
            DownloadRoute::Modpack,
            "Download.Left.Modpack",
            "package",
            None,
        ),
        (
            DownloadRoute::DataPack,
            "Download.Left.DataPack",
            "file-archive",
            None,
        ),
        (
            DownloadRoute::ResourcePack,
            "Download.Left.ResourcePack",
            "layers",
            None,
        ),
        (
            DownloadRoute::Shader,
            "Download.Left.Shader",
            "sparkles",
            None,
        ),
        (DownloadRoute::World, "Download.Left.World", "globe", None),
        (
            DownloadRoute::Favorites,
            "Download.Left.Favorites",
            "heart",
            None,
        ),
        (
            DownloadRoute::Client,
            "Download.Left.Minecraft",
            "package",
            Some("Download.Left.Installation"),
        ),
        (
            DownloadRoute::OptiFine,
            "Download.Left.Optifine",
            "gauge",
            None,
        ),
        (DownloadRoute::Forge, "Download.Left.Forge", "anvil", None),
        (
            DownloadRoute::NeoForge,
            "Download.Left.NeoForge",
            "cat",
            None,
        ),
        (
            DownloadRoute::Cleanroom,
            "Download.Left.Cleanroom",
            "flask-conical",
            None,
        ),
        (
            DownloadRoute::Fabric,
            "Download.Left.Fabric",
            "scroll",
            None,
        ),
        (
            DownloadRoute::LegacyFabric,
            "Download.Left.LegacyFabric",
            "scroll",
            None,
        ),
        (DownloadRoute::LabyMod, "Download.Left.LabyMod", "box", None),
        (
            DownloadRoute::LiteLoader,
            "Download.Left.LiteLoader",
            "egg",
            None,
        ),
    ];

    #[test]
    fn matches_the_frozen_catalog() {
        assert_eq!(snapshot_rows(DownloadRoute::ALL).as_slice(), SNAPSHOT);
    }
}
