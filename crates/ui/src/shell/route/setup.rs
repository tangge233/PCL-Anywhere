//! 设置页的子路由与分类目录：顺序即 gpui-kit `Settings` 的页序。
//!
//! `section` 按 PCL 的分组登记，但 gpui-component 的 `Settings` 画不出分类标题，
//! 这些键暂时只是数据（见 `pages::setup` 的已知缺口）。

use crate::pages::setup::SetupGroup;
use crate::pages::setup::{
    about, game_link, game_manage, java, language, launch, log, misc, placeholder, ui,
};
use crate::shell::route::page::setting_pages;

setting_pages! {
    /// 设置页左栏分类（PCL 的 11 个设置分类）。
    pub enum SetupRoute for SetupGroup as Route::Setup {
        Launch: "Setup.Left.Item.Launch", "rocket", section "Setup.Left.Category.Game",
            setting launch::page;
        Java: "Setup.Left.Item.Java", "coffee", setting java::page;
        GameManage: "Setup.Left.Item.GameManage", "book-marked", setting game_manage::page;
        GameLink: "Setup.Left.Item.GameLink", "bubbles", section "Setup.Left.Category.Tools",
            setting game_link::page;
        Ui: "Setup.Left.Item.Ui", "palette", section "Setup.Left.Category.Launcher",
            setting ui::page;
        Language: "Setup.Language.Title", "earth", setting language::page;
        Misc: "Setup.Left.Item.Misc", "monitor-cog", setting misc::page;
        About: "Setup.Left.Item.About", "info", section "Setup.Left.Category.About",
            setting about::page;
        Update: "Setup.Left.Item.Update", "refresh-cw", setting placeholder::update;
        Feedback: "Setup.Left.Item.Feedback", "message-circle", setting placeholder::feedback;
        Log: "Setup.Left.Item.Log", "scroll-text", setting log::page;
    }
}

#[cfg(test)]
mod tests {
    use super::SetupRoute;

    use crate::shell::route::page::{PageRoute as _, snapshot_rows};

    /// 显示顺序的规格：顺序、文案键、图标、分组标题。改表要同步改这里。
    /// 快照放在表旁边，加 / 删一页只动这一个文件。
    const SNAPSHOT: &[(SetupRoute, &str, &str, Option<&str>)] = &[
        (
            SetupRoute::Launch,
            "Setup.Left.Item.Launch",
            "rocket",
            Some("Setup.Left.Category.Game"),
        ),
        (SetupRoute::Java, "Setup.Left.Item.Java", "coffee", None),
        (
            SetupRoute::GameManage,
            "Setup.Left.Item.GameManage",
            "book-marked",
            None,
        ),
        (
            SetupRoute::GameLink,
            "Setup.Left.Item.GameLink",
            "bubbles",
            Some("Setup.Left.Category.Tools"),
        ),
        (
            SetupRoute::Ui,
            "Setup.Left.Item.Ui",
            "palette",
            Some("Setup.Left.Category.Launcher"),
        ),
        (SetupRoute::Language, "Setup.Language.Title", "earth", None),
        (
            SetupRoute::Misc,
            "Setup.Left.Item.Misc",
            "monitor-cog",
            None,
        ),
        (
            SetupRoute::About,
            "Setup.Left.Item.About",
            "info",
            Some("Setup.Left.Category.About"),
        ),
        (
            SetupRoute::Update,
            "Setup.Left.Item.Update",
            "refresh-cw",
            None,
        ),
        (
            SetupRoute::Feedback,
            "Setup.Left.Item.Feedback",
            "message-circle",
            None,
        ),
        (SetupRoute::Log, "Setup.Left.Item.Log", "scroll-text", None),
    ];

    #[test]
    fn matches_the_frozen_catalog() {
        assert_eq!(snapshot_rows(SetupRoute::ALL).as_slice(), SNAPSHOT);
    }
}
