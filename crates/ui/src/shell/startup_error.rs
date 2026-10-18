//! 启动失败窗口：主窗口尚未建立时显示错误，并提供反馈与日志入口。

use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::*;

use crate::components::{AppButton, ButtonColor};
use crate::i18n;

const FEEDBACK_URL: &str = "https://github.com/tangge233/PCL-Anywhere/issues";
const WINDOW_SIZE: (Pixels, Pixels) = (px(620.), px(360.));
const WINDOW_MIN_SIZE: (Pixels, Pixels) = (px(480.), px(280.));

/// 打开启动失败窗口。
pub fn open(cx: &mut App, message: impl Into<SharedString>) -> Result<AnyWindowHandle> {
    let message = message.into();
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds {
            origin: point(px(0.), px(0.)),
            size: size(WINDOW_SIZE.0, WINDOW_SIZE.1),
        })),
        window_min_size: Some(size(WINDOW_MIN_SIZE.0, WINDOW_MIN_SIZE.1)),
        window_decorations: Some(WindowDecorations::Server),
        titlebar: Some(TitlebarOptions {
            title: Some("PCL-Anywhere".into()),
            ..Default::default()
        }),
        icon: crate::assets::window_icon(),
        app_id: Some("PCL-Anywhere".to_owned()),
        ..Default::default()
    };

    let (handle, _) = gpui_kit::open_window(options, cx, move |_, cx| {
        cx.new(|_| StartupErrorWindow { message })
    })?;
    Ok(handle)
}

struct StartupErrorWindow {
    message: SharedString,
}

impl Render for StartupErrorWindow {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .p_6()
            .gap_4()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::BOLD)
                    .child(i18n::lang("Startup.Error.Title")),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .text_sm()
                    .child(self.message.clone()),
            )
            .child(
                div()
                    .text_sm()
                    .opacity(0.7)
                    .child(i18n::lang("Startup.Error.Hint")),
            )
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        AppButton::new(
                            "startup-error-feedback",
                            i18n::lang("Startup.Error.Feedback"),
                        )
                        .icon("message-circle")
                        .color(ButtonColor::Highlight)
                        .on_click(|_, _, cx| cx.open_url(FEEDBACK_URL)),
                    )
                    .child(
                        AppButton::new("startup-error-logs", i18n::lang("Startup.Error.OpenLogs"))
                            .icon("folder-open")
                            .on_click(|_, _, cx| match logger::log_dir() {
                                Ok(dir) => cx.reveal_path(&dir),
                                Err(error) => {
                                    logger::log::error!(
                                        target: "StartupError",
                                        "定位日志目录失败：{error}"
                                    );
                                }
                            }),
                    ),
            )
    }
}
