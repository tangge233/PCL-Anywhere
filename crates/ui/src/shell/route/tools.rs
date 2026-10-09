//! 工具页的子路由与左栏目录。

use crate::pages::tools::ToolsGroup;
use crate::shell::route::page::nav_pages;

nav_pages! {
    /// 工具页左栏（联机 / 百宝箱 / UI 测试）。
    pub enum ToolsRoute for ToolsGroup as Route::Tools {
        GameLink: "Tools.Left.Lobby", "bubbles", section "Tools.Left.Multiplayer",
            render ToolsGroup::render_game_link;
        Test: "Tools.Test.Title", "flask-conical", section "Tools.Left.Utilities",
            render ToolsGroup::render_test;
        UiTest: "Tools.UiTest.Title", "panels-top-left", render ToolsGroup::render_ui_test;
    }
}

#[cfg(test)]
mod tests {
    use super::ToolsRoute;

    use crate::shell::route::page::{PageRoute as _, snapshot_rows};

    /// 显示顺序的规格：顺序、文案键、图标、分组标题。改表要同步改这里。
    /// 快照放在表旁边，加 / 删一页只动这一个文件。
    const SNAPSHOT: &[(ToolsRoute, &str, &str, Option<&str>)] = &[
        (
            ToolsRoute::GameLink,
            "Tools.Left.Lobby",
            "bubbles",
            Some("Tools.Left.Multiplayer"),
        ),
        (
            ToolsRoute::Test,
            "Tools.Test.Title",
            "flask-conical",
            Some("Tools.Left.Utilities"),
        ),
        (
            ToolsRoute::UiTest,
            "Tools.UiTest.Title",
            "panels-top-left",
            None,
        ),
    ];

    #[test]
    fn matches_the_frozen_catalog() {
        assert_eq!(snapshot_rows(ToolsRoute::ALL).as_slice(), SNAPSHOT);
    }
}
