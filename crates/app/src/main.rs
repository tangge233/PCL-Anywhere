#[cfg(any(target_os = "linux", target_os = "windows"))]
mod alloc;

fn main() {
    let _log = logger::init(logger::Options::default()).expect("初始化日志失败");

    gpui_kit::application()
        .with_assets(pcl_ui::assets::Assets)
        .run(|cx| {
            pcl_ui::init(cx);
            if let Err(error) = pcl_ui::open_main_window(cx) {
                logger::log::error!(target: "App", "打开主窗口失败：{error}");
            }
        });
}
