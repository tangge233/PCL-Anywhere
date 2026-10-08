//! UI 测试页的界面集成测试：页面按钮 → 对话框 → 作答写回页面记录。
//!
//! 这条链路跨了三层（页面状态 / 对话框图层 / 内容实体），所以断言的是**记录里的那行文字**，
//! 而不是中间任何一层的内部状态。

use gpui_kit::component::Root;
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{AppContext as _, TestAppContext, Window, WindowHandle, px, size};

use pcl_ui::pages::tools::ToolsGroup;
use pcl_ui::shell::route::ToolsRoute;
use pcl_ui::shell::{GroupView as _, Route};

mod common;

/// 打开工具页并切到 UI 测试页。
fn open_ui_test_page(cx: &mut TestAppContext) -> WindowHandle<Root> {
    cx.update(pcl_ui::init);

    let mut group = None;
    let handle = cx.open_window(size(px(850.), px(500.)), |window, cx| {
        let view = cx.new(|cx| ToolsGroup::new(window, cx));
        group = Some(view.clone());
        Root::new(view, window, cx)
    });
    let group = group.expect("页面创建成功");

    cx.update_window(handle.into(), |_, window, cx| {
        group.update(cx, |group, cx| {
            group.set_route(Route::Tools(ToolsRoute::UiTest), window, cx);
        });
        window.render_frame(cx);
    })
    .expect("窗口仍开着");
    handle
}

/// 记录里最新的一行（还没作答时为 `None`）。
fn latest_result(window: &Window) -> Option<String> {
    window
        .try_find("ui-test-result-0")?
        .label()
        .map(str::to_owned)
}

#[gpui_kit::test]
fn page_opens_with_an_empty_result(cx: &mut TestAppContext) {
    let handle = open_ui_test_page(cx);

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        assert!(
            window.try_find("tools-Tools.UiTest.Title").is_some(),
            "选择栏里应有 UI 测试条目"
        );
        assert!(
            window.try_find("ui-test-confirm").is_some(),
            "页面应有对话框测试按钮"
        );
        assert!(
            window.try_find("ui-test-result-empty").is_some(),
            "尚未作答时应显示占位"
        );
        assert!(latest_result(window).is_none());
    })
    .expect("窗口仍开着");
}

#[gpui_kit::test]
async fn confirm_button_logs_the_answer(cx: &mut TestAppContext) {
    let handle = open_ui_test_page(cx);

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        window.click("ui-test-confirm", cx);
    })
    .expect("窗口仍开着");
    common::settle(cx).await;

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        assert!(window.try_find("dialog-panel").is_some(), "应弹出确认框");
        window.click("dialog-button-0", cx);
    })
    .expect("窗口仍开着");
    common::settle(cx).await;

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        assert!(
            window.try_find("dialog-panel").is_none(),
            "作答后弹窗应关闭"
        );
        assert_eq!(
            latest_result(window).as_deref(),
            Some("确定与取消 → 已确定"),
            "页面应记下确定"
        );
    })
    .expect("窗口仍开着");
}

#[gpui_kit::test]
async fn info_alert_closes_with_enter(cx: &mut TestAppContext) {
    let handle = open_ui_test_page(cx);

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        window.click("ui-test-info", cx);
    })
    .expect("窗口仍开着");
    common::settle(cx).await;

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        assert!(window.try_find("dialog-panel").is_some(), "应弹出提示框");
        window.press("enter", cx);
    })
    .expect("窗口仍开着");
    common::settle(cx).await;

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(
            latest_result(window).as_deref(),
            Some("信息提示 → 已关闭"),
            "Enter 应按下唯一按钮并记下结果"
        );
    })
    .expect("窗口仍开着");
}

#[gpui_kit::test]
async fn custom_content_waits_for_a_choice(cx: &mut TestAppContext) {
    let handle = open_ui_test_page(cx);

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        window.click("ui-test-custom", cx);
    })
    .expect("窗口仍开着");
    common::settle(cx).await;

    // 未选中：确定按钮点了也不作答。
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        window.click("dialog-button-0", cx);
    })
    .expect("窗口仍开着");
    common::settle(cx).await;

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        assert!(
            window.try_find("dialog-panel").is_some(),
            "未选中时弹窗不得关闭"
        );
        assert!(latest_result(window).is_none(), "未选中不应记录结果");
        window.click("ui-test-choice-1", cx);
        window.render_frame(cx);
        window.click("dialog-button-0", cx);
    })
    .expect("窗口仍开着");
    common::settle(cx).await;

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(
            latest_result(window).as_deref(),
            Some("自定义内容 → 选项 2"),
            "结果应取自内容实体的选中项"
        );
    })
    .expect("窗口仍开着");
}
