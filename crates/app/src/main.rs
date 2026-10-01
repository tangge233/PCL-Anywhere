fn main() {
    gpui_kit::application()
        .with_assets(pcl_ui::assets::Assets)
        .run(|cx| {
            pcl_ui::init(cx);
            pcl_ui::open_main_window(cx).expect("failed to open the main window");
        });
}
