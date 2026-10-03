//! 实例设置页的界面集成测试：标题栏带的是选中的实例名，而不是「未知」占位。

use gpui_kit::component::Root;
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{AppContext as _, Entity, TestAppContext, WindowHandle, px, size};

use pcl_ui::i18n;
use pcl_ui::pages::instance::InstanceGroup;
use pcl_ui::shell::route::InstanceRoute;
use pcl_ui::shell::{GroupView as _, Route};

/// 打开实例页并切到「实例设置」。
fn open_setup(cx: &mut TestAppContext) -> (Entity<InstanceGroup>, WindowHandle<Root>) {
    cx.update(pcl_ui::init);

    let mut group = None;
    let handle = cx.open_window(size(px(850.), px(500.)), |window, cx| {
        let view = cx.new(|cx| InstanceGroup::new(window, cx));
        group = Some(view.clone());
        Root::new(view, window, cx)
    });
    let group = group.expect("页面创建成功");

    cx.update_window(handle.into(), |_, window, cx| {
        group.update(cx, |group, cx| {
            group.set_route(Route::Instance(InstanceRoute::Setup), window, cx);
        });
        window.render_frame(cx);
    })
    .unwrap();
    (group, handle)
}

#[gpui_kit::test]
fn setup_title_names_the_instance(cx: &mut TestAppContext) {
    let (group, _) = open_setup(cx);

    let title = cx
        .update(|cx| group.read(cx).page_title())
        .expect("实例设置应有标题");
    // 模板形如「实例设置 - {0}」（zh-CN.json）：用空名字取出前缀，标题必须是「前缀 + 实例名」。
    let prefix = i18n::lang_with_args("Main.Title.InstanceSetup", &[""]);
    assert!(
        title.as_ref().starts_with(prefix.as_ref()),
        "标题应套用实例设置模板：{title}"
    );
    let name = title
        .as_ref()
        .strip_prefix(prefix.as_ref())
        .expect("已断言前缀");
    let unknown = i18n::lang("Common.State.Unknown");
    assert!(
        !name.is_empty() && name != unknown.as_ref(),
        "标题未填入实例名：{title}"
    );
}

/// 每个实例的设置数据互相独立：改过实例 0，切到实例 1 仍是默认值，切回来修改还在。
#[gpui_kit::test]
fn instances_keep_their_own_settings(cx: &mut TestAppContext) {
    let (_, handle) = open_setup(cx);

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(
            window.find("memory-default").checked(),
            Some(true),
            "实例 0 初始应为默认内存设置"
        );
        window.click("memory-custom", cx);
        assert_eq!(
            window.find("memory-custom").checked(),
            Some(true),
            "实例 0 已改成独立内存设置"
        );
    })
    .unwrap();

    cx.update_window(handle.into(), |_, window, cx| {
        window.click("instance-1", cx);
        window.render_frame(cx);
        assert_eq!(
            window.find("memory-default").checked(),
            Some(true),
            "实例 1 不应被实例 0 的改动影响"
        );
    })
    .unwrap();

    cx.update_window(handle.into(), |_, window, cx| {
        window.click("instance-0", cx);
        window.render_frame(cx);
        assert_eq!(
            window.find("memory-custom").checked(),
            Some(true),
            "切回实例 0 后修改应保留"
        );
    })
    .unwrap();
}
