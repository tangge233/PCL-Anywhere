//! 下载页界面集成测试：版本分类连体按钮组（连体 + 独立多选）、安装面板的加载器卡片高度，
//! 以及加载器卡片选版本（各加载器各选各的）。都在 headless 窗口里渲染真实页面来验证。

use std::time::Duration;

use gpui_kit::component::Root;
use gpui_kit::test::{TestAppContextExt as _, TestWindowExt as _};
use gpui_kit::{AppContext as _, TestAppContext, WindowHandle, px, size};

use pcl_ui::i18n;
use pcl_ui::pages::download::DownloadGroup;

mod common;

/// 打开下载页（默认进入版本安装 / 版本清单）。
fn open_download_page(cx: &mut TestAppContext) -> WindowHandle<Root> {
    cx.update(pcl_ui::init);
    let mut group = None;
    let handle = cx.open_window(size(px(920.), px(620.)), |window, cx| {
        let view = cx.new(|cx| DownloadGroup::new(window, cx));
        group = Some(view.clone());
        Root::new(view, window, cx)
    });
    group.expect("页面创建成功");
    handle
}

/// 分段标识。
fn segment(index: usize) -> String {
    format!("download-kind-{index}")
}

/// 等待清单加载完成（首个版本行出现）。
async fn wait_for_list(cx: &mut TestAppContext, handle: WindowHandle<Root>) {
    cx.wait_for(handle.into(), Duration::from_secs(10), |window, cx| {
        window.render_frame(cx);
        window.try_find("download-version-1.21.4").is_some()
    })
    .await;
}

#[gpui_kit::test]
async fn category_segments_are_joined_and_toggle_independently(cx: &mut TestAppContext) {
    let handle = open_download_page(cx);
    wait_for_list(cx, handle).await;

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);

        // 连体：四段首尾相接、同一行、同高。
        let bounds: Vec<_> = (0..4).map(|i| window.find(segment(i)).bounds()).collect();
        for pair in bounds.windows(2) {
            let (left, right) = (pair[0], pair[1]);
            assert_eq!(left.origin.y, right.origin.y, "分段必须同处一行");
            assert_eq!(left.size.height, right.size.height, "分段必须同高");
            assert_eq!(
                left.origin.x + left.size.width,
                right.origin.x,
                "相邻分段必须首尾相接（无间隙）"
            );
        }

        for i in 0..4 {
            assert_eq!(
                window.find(segment(i)).checked(),
                Some(true),
                "分段 {i} 初始应为选中"
            );
        }

        // 点第二段只翻转它自己，相邻分段不受影响。
        window.click(segment(1), cx);
        window.render_frame(cx);
        assert_eq!(
            window.find(segment(1)).checked(),
            Some(false),
            "第二段应翻转为未选中"
        );
        for i in [0, 2, 3] {
            assert_eq!(
                window.find(segment(i)).checked(),
                Some(true),
                "分段 {i} 不应被连带取消"
            );
        }

        window.click(segment(1), cx);
        window.render_frame(cx);
        assert_eq!(
            window.find(segment(1)).checked(),
            Some(true),
            "第二段应恢复选中"
        );
    })
    .unwrap();
}

/// 进入安装面板后，加载器小卡片必须是紧凑、等高的：曾经卡片里两层 `min_w_0` 叠加
/// 换行行把高度算成「按最小宽度换行」的结果，卡片被拉成几百像素高且行内参差。
#[gpui_kit::test]
async fn loader_tiles_keep_a_compact_uniform_height(cx: &mut TestAppContext) {
    let handle = open_download_page(cx);
    wait_for_list(cx, handle).await;

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        window.click("download-version-1.21.4", cx);
        window.render_frame(cx);

        let tiles: Vec<_> = (0..11)
            .map(|i| window.find(format!("download-loader-{i}")).bounds())
            .collect();

        // 所有卡片同高，且是单个内容行的高度（远小于被拉高的异常值）。
        let height = tiles[0].size.height;
        assert!(height < px(80.), "加载器卡片高度异常：{height:?}");
        for (i, tile) in tiles.iter().enumerate() {
            assert_eq!(tile.size.height, height, "卡片 {i} 的高度与其余不一致");
        }
    })
    .unwrap();
}

/// 加载器卡片选的是「这个加载器的版本」：点卡片弹出版本列表，选中后写到卡片上；
/// 各加载器各选各的互不影响，✗ 取消本加载器的选择，离开安装面板时清空。
#[gpui_kit::test]
async fn loader_card_picks_that_loaders_version(cx: &mut TestAppContext) {
    let handle = open_download_page(cx);
    wait_for_list(cx, handle).await;
    let none = || i18n::lang("Download.Install.Loader.None");

    // 进安装面板：卡片先显示「未选择」。
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        window.click("download-version-1.21.4", cx);
        window.render_frame(cx);
        assert_eq!(
            window.find("download-loader-status-0").label(),
            Some(none().as_ref()),
            "未选版本时卡片显示「未选择」"
        );
        window.click("download-loader-click-0", cx);
    })
    .expect("窗口仍开着");
    common::settle(cx).await;

    // 弹窗标题点出是哪个加载器，列出的是它的版本。
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        let title = window
            .find("dialog-title")
            .label()
            .unwrap_or_default()
            .to_owned();
        assert!(
            title.contains(i18n::lang("Common.Installation.Forge").as_ref()),
            "标题应点出加载器名：{title}"
        );

        // 未选中：确定点了也不关。
        window.click("dialog-button-0", cx);
        window.render_frame(cx);
        assert!(
            window.try_find("dialog-panel").is_some(),
            "未选中版本时确定按钮不得关闭弹窗"
        );
        window.click("loader-version-1", cx);
        window.render_frame(cx);
        assert_eq!(
            window.find("loader-version-1").checked(),
            Some(true),
            "点单选项后应处于选中态"
        );
        window.click("dialog-button-0", cx);
    })
    .expect("窗口仍开着");
    common::settle(cx).await;

    // 写回卡片：只改这一个加载器；再打开时停在当前版本上。
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        assert!(
            window.try_find("dialog-panel").is_none(),
            "作答后弹窗应关闭"
        );
        assert_eq!(
            window.find("download-loader-status-0").label(),
            Some("47.2.0"),
            "卡片应显示选中的版本"
        );
        assert_eq!(
            window.find("download-loader-status-1").label(),
            Some(none().as_ref()),
            "别的加载器不受影响"
        );

        window.click("download-loader-click-0", cx);
        window.render_frame(cx);
        assert_eq!(
            window.find("loader-version-1").checked(),
            Some(true),
            "弹窗应停在当前已选的版本上"
        );
        window.click("dialog-button-1", cx); // 取消
    })
    .expect("窗口仍开着");
    common::settle(cx).await;

    // 第二个加载器各选各的。
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        window.click("download-loader-click-1", cx);
    })
    .expect("窗口仍开着");
    common::settle(cx).await;

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        window.click("loader-version-0", cx);
        window.render_frame(cx);
        window.click("dialog-button-0", cx);
    })
    .expect("窗口仍开着");
    common::settle(cx).await;

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(
            window.find("download-loader-status-1").label(),
            Some("0.2.4"),
            "第二个加载器记自己的版本"
        );
        assert_eq!(
            window.find("download-loader-status-0").label(),
            Some("47.2.0"),
            "第一个加载器的选择不受影响"
        );

        // ✗ 只清本加载器。
        window.click("download-loader-clear-0", cx);
        window.render_frame(cx);
        assert_eq!(
            window.find("download-loader-status-0").label(),
            Some(none().as_ref()),
            "✗ 应清掉本加载器的选择"
        );
        assert_eq!(
            window.find("download-loader-status-1").label(),
            Some("0.2.4"),
            "✗ 不得影响别的加载器"
        );

        // 离开安装面板再进来：清空（原版 ExitSelectPage 调 ClearSelected）。
        window.click("download-back", cx);
        window.render_frame(cx);
        window.click("download-version-1.21.4", cx);
        window.render_frame(cx);
        assert_eq!(
            window.find("download-loader-status-1").label(),
            Some(none().as_ref()),
            "离开面板应清空加载器选择"
        );
        assert!(
            window.try_find("download-loader-clear-1").is_none(),
            "没有选择时不应有清除按钮"
        );
    })
    .expect("窗口仍开着");
}
