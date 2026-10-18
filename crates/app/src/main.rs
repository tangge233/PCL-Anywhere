#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::panic::{AssertUnwindSafe, catch_unwind};

#[cfg(any(target_os = "linux", target_os = "windows"))]
mod alloc;
mod config;
mod state;

fn main() {
    let _log = logger::init(logger::Options::default()).expect("初始化日志失败");

    gpui_kit::application()
        .with_assets(pcl_ui::assets::Assets)
        .run(|cx| {
            let startup = catch_unwind(AssertUnwindSafe(|| {
                pcl_ui::init(cx);
                let state = state::AppState::load().map_err(|error| error.to_string())?;
                pcl_ui::open_main_window(cx).map_err(|error| error.to_string())?;
                Ok::<_, String>(state)
            }));

            match startup {
                Ok(Ok(state)) => {
                    let _config = state.config();
                    cx.set_global(state);
                }
                Ok(Err(message)) => show_startup_error(cx, message),
                Err(payload) => show_startup_error(cx, panic_message(payload)),
            }
        });
}

fn show_startup_error(cx: &mut gpui_kit::App, message: String) {
    logger::log::error!(target: "App", "启动失败：{message}");

    if let Err(error) = pcl_ui::open_startup_error(cx, message) {
        logger::log::error!(target: "App", "打开启动错误窗口失败：{error}");
        std::process::exit(1);
    }
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_owned()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "unknown panic".to_owned()
    }
}
