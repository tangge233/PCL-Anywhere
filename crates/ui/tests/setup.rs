//! 设置页界面集成测试：侧栏是否恰好列出目录表里的分类。
//!
//! 侧栏由 gpui-component 的 `Settings` 渲染，页面内容没有可观察元素；侧栏条目只能查到
//! 「元素在不在」（`aria_selected` 挂在组件的内层行上，标识取不到）。分类顺序由
//! `shell::route::setup` 的表快照测试钉住，「路由 → 显示哪一页」只能真机核对。

use gpui_kit::component::Root;
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{AppContext as _, Entity, TestAppContext, WindowHandle, px, size};

use pcl_ui::pages::setup::SetupGroup;
use pcl_ui::shell::route::SetupRoute;
use pcl_ui::shell::route::page::PageRoute as _;

/// 打开设置页（默认进入「启动」分类）。
fn open_setup_page(cx: &mut TestAppContext) -> (WindowHandle<Root>, Entity<SetupGroup>) {
    cx.update(pcl_ui::init);
    let mut group = None;
    let handle = cx.open_window(size(px(920.), px(620.)), |window, cx| {
        let view = cx.new(|cx| SetupGroup::new(window, cx));
        group = Some(view.clone());
        Root::new(view, window, cx)
    });
    (handle, group.expect("页面创建成功"))
}

/// 侧栏第 `ix` 个分类的元素标识（组件按下标生成）。
fn nav_item(ix: usize) -> String {
    format!("0-{ix}")
}

#[gpui_kit::test]
async fn sidebar_lists_exactly_the_catalog(cx: &mut TestAppContext) {
    let (handle, _group) = open_setup_page(cx);

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);

        // 目录表里的每个分类都在侧栏里，且不多不少。
        for ix in 0..SetupRoute::ALL.len() {
            assert!(
                window.try_find(nav_item(ix)).is_some(),
                "侧栏缺少第 {ix} 个分类"
            );
        }
        assert!(
            window.try_find(nav_item(SetupRoute::ALL.len())).is_none(),
            "侧栏出现了目录之外的分类"
        );
    })
    .expect("窗口仍开着");
}
