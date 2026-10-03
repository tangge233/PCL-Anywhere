//! 实例页两栏收起条的界面集成测试。
//!
//! 收起条在 PCL 里是一条整条可点的窄条（`collapseBar` 的 `PointerPressed`），
//! 因此这里在 headless 窗口里渲染真实页面，并在条的最底部派发真实点击：
//! 只要命中区域不是整条，点到底部就不会切换栏位。

use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{AppContext as _, px, size};
use gpui_kit::{TestAppContext, point};

use pcl_ui::pages::instance::{ColumnState, InstanceGroup};
use pcl_ui::shell::route::InstanceRoute;
use pcl_ui::shell::{GroupView, Route};

/// 打开实例页（文件夹态：文件夹 + 实例列表）。
fn open_instance_page(
    cx: &mut TestAppContext,
) -> (
    gpui_kit::Entity<InstanceGroup>,
    gpui_kit::WindowHandle<gpui_kit::component::Root>,
) {
    cx.update(pcl_ui::init);
    let mut group = None;
    let handle = cx.open_window(size(px(850.), px(500.)), |window, cx| {
        let view = cx.new(|cx| InstanceGroup::new(window, cx));
        group = Some(view.clone());
        gpui_kit::component::Root::new(view, window, cx)
    });
    (group.expect("页面创建成功"), handle)
}

/// 点击窄条的最底部：点击偏移必须严格落在元素 bounds 内（`click_at` 会断言
/// `offset < size`），因此取底边内侧 2px；条中央是居中按钮，点底部才能验证整条可点。
fn click_bar_bottom(window: &mut gpui_kit::Window, id: &'static str, cx: &mut gpui_kit::App) {
    let bounds = window.find(id).bounds();
    window.click_at(id, point(px(0.), bounds.size.height - px(2.)), cx);
}

#[gpui_kit::test]
fn collapse_bar_is_clickable_along_its_whole_height(cx: &mut TestAppContext) {
    let (group, handle) = open_instance_page(cx);

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(
            group.read(cx).column_state(),
            ColumnState::Folders,
            "初始应为文件夹态"
        );

        // 窄条最底部（离居中按钮很远）：整条可点时才应切换。
        click_bar_bottom(window, "bar-expand", cx);
        assert_eq!(
            group.read(cx).column_state(),
            ColumnState::Manage,
            "点窄条底部也应切换到管理态"
        );

        // 回切：此时窄条在左侧，同样点最底部。
        click_bar_bottom(window, "bar-collapse", cx);
        assert_eq!(
            group.read(cx).column_state(),
            ColumnState::Folders,
            "点左侧窄条底部应切回文件夹态"
        );
    })
    .unwrap();
}

/// 文件夹态叫「实例选择」、管理态叫「实例详情」；单击只选中，双击才进详情。
#[gpui_kit::test]
fn instance_row_selects_on_single_click_and_opens_detail_on_double_click(cx: &mut TestAppContext) {
    let (group, handle) = open_instance_page(cx);
    let select_title = pcl_ui::i18n::lang("Launch.Home.SelectInstance");
    let detail_title = pcl_ui::i18n::lang("Main.Title.InstanceSelect");

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(
            group.read(cx).page_title(),
            Some(select_title.clone()),
            "初始应为「实例选择」标题"
        );

        // 单击：只选中该实例，栏位不动。
        window.click("instance-0", cx);
        assert_eq!(
            group.read(cx).column_state(),
            ColumnState::Folders,
            "单击不应进入详情"
        );
        assert_eq!(window.find("instance-0").selected(), Some(true));
        assert_eq!(group.read(cx).page_title(), Some(select_title.clone()));

        // 双击：进入实例详情（展开管理栏）。
        window.double_click("instance-0", cx);
        assert_eq!(
            group.read(cx).column_state(),
            ColumnState::Manage,
            "双击应进入实例详情"
        );
        assert_eq!(group.read(cx).page_title(), Some(detail_title));

        // 左侧窄条退回文件夹态，标题也跟着回到「实例选择」。
        click_bar_bottom(window, "bar-collapse", cx);
        assert_eq!(group.read(cx).column_state(), ColumnState::Folders);
        assert_eq!(group.read(cx).page_title(), Some(select_title));
    })
    .unwrap();
}

#[gpui_kit::test]
fn entering_instance_setup_shows_the_manage_column(cx: &mut TestAppContext) {
    let (group, handle) = open_instance_page(cx);

    cx.update_window(handle.into(), |_, window, cx| {
        group.update(cx, |group, cx| {
            group.set_route(Route::Instance(InstanceRoute::Setup), window, cx);
        });
        window.render_frame(cx);
        assert_eq!(
            group.read(cx).column_state(),
            ColumnState::Manage,
            "实例设置直接展开管理栏"
        );
    })
    .unwrap();
}

/// 管理栏弹到独立窗口后，主窗口改显示提示页：提示页的操作栏是状态卡片的 child，
/// 随卡片一起渲染，且按钮真的接到「收回」上（点一下就恢复内嵌管理栏）。
#[gpui_kit::test]
fn popped_manage_page_shows_hint_with_action_child(cx: &mut TestAppContext) {
    let (group, handle) = open_instance_page(cx);

    cx.update_window(handle.into(), |_, window, cx| {
        group.update(cx, |group, cx| {
            group.set_route(Route::Instance(InstanceRoute::Setup), window, cx);
        });
        window.render_frame(cx);
        assert!(
            window.find("manage-pop-out").visible(),
            "内嵌时应有弹出按钮"
        );

        window.click("manage-pop-out", cx);
        window.render_frame(cx);
        assert!(
            window.find("manage-restore").visible(),
            "提示页的操作栏是状态卡片的 child，应随卡片一起渲染"
        );

        window.click("manage-restore", cx);
        window.render_frame(cx);
        assert!(
            window.find("manage-pop-out").visible(),
            "收回后主窗口应恢复内嵌管理栏"
        );
    })
    .unwrap();
}
