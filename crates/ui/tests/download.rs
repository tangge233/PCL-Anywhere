//! 下载页界面集成测试：版本分类连体按钮组（连体 + 独立多选）与安装面板加载器卡片高度。
//! 都在 headless 窗口里渲染真实页面来验证。

use std::time::Duration;

use gpui_kit::component::Root;
use gpui_kit::test::{TestAppContextExt as _, TestWindowExt as _};
use gpui_kit::{AppContext as _, TestAppContext, WindowHandle, px, size};

use pcl_ui::pages::download::DownloadGroup;

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
