//! 实例页两栏收起条的界面集成测试。
//!
//! 收起条在 PCL 里是一条整条可点的窄条（`collapseBar` 的 `PointerPressed`），
//! 因此这里在 headless 窗口里渲染真实页面，并在条的最底部派发真实点击：
//! 只要命中区域不是整条，点到底部就不会切换栏位。

use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{AppContext as _, px, size};
use gpui_kit::{TestAppContext, point};

use pcl_ui::pages::instance::InstanceGroup;
use pcl_ui::shell::route::InstanceRoute;
use pcl_ui::shell::{GroupView, Route};

/// 打开实例详情页（第一态：文件夹 + 实例列表）。
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

#[gpui_kit::test]
fn collapse_bar_is_clickable_along_its_whole_height(cx: &mut TestAppContext) {
    let (group, handle) = open_instance_page(cx);

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(group.read(cx).column_state(), 1, "初始应为第一态");

        // 窄条最底部（离居中按钮很远）：整条可点时才应切换。
        let bounds = window.find("bar-expand").bounds();
        window.click_at("bar-expand", point(px(0.), bounds.size.height - px(2.)), cx);
        assert_eq!(
            group.read(cx).column_state(),
            2,
            "点窄条底部也应切换到第二态"
        );

        // 回切：此时窄条在左侧，同样点最底部。
        let bounds = window.find("bar-collapse").bounds();
        window.click_at(
            "bar-collapse",
            point(px(0.), bounds.size.height - px(2.)),
            cx,
        );
        assert_eq!(
            group.read(cx).column_state(),
            1,
            "点左侧窄条底部应切回第一态"
        );
    })
    .unwrap();
}

/// 第一态叫「实例选择」、第二态叫「实例详情」；单击只选中，双击才进详情。
#[gpui_kit::test]
fn instance_row_selects_on_single_click_and_opens_detail_on_double_click(cx: &mut TestAppContext) {
    let (group, handle) = open_instance_page(cx);
    let select_title = pcl_ui::i18n::text("Launch.Home.SelectInstance");
    let detail_title = pcl_ui::i18n::text("Main.Title.InstanceSelect");

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(group.read(cx).page_title(), Some(select_title.clone()));

        // 单击：只选中该实例，栏位不动。
        window.click("instance-0", cx);
        assert_eq!(group.read(cx).column_state(), 1, "单击不应进入详情");
        assert_eq!(window.find("instance-0").selected(), Some(true));
        assert_eq!(group.read(cx).page_title(), Some(select_title.clone()));

        // 双击：进入实例详情（展开管理栏）。
        window.double_click("instance-0", cx);
        assert_eq!(group.read(cx).column_state(), 2, "双击应进入实例详情");
        assert_eq!(group.read(cx).page_title(), Some(detail_title.clone()));

        // 左侧窄条退回第一态，标题也跟着回到「实例选择」。
        let bounds = window.find("bar-collapse").bounds();
        window.click_at(
            "bar-collapse",
            point(px(0.), bounds.size.height - px(2.)),
            cx,
        );
        assert_eq!(group.read(cx).column_state(), 1);
        assert_eq!(group.read(cx).page_title(), Some(select_title.clone()));
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
        assert_eq!(group.read(cx).column_state(), 2, "实例设置直接展开管理栏");
    })
    .unwrap();
}
