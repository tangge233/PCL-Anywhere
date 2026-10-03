//! 主窗口外壳：标题栏 + 当前分组（选择栏 + 内容区）。
//!
//! 本模块持有外壳的公共契约与启动入口：`Navigate` 导航事件、`GroupView` 分组视图、
//! 窗口尺寸常量、`init` / `open_main_window`；主窗口自身的路由状态机与渲染在子模块
//! `main_window`。每个分组自己拥有选择栏与内容，因此下载页与启动页可以有完全不同的左栏。

mod main_window;
pub mod route;
pub mod title_bar;
pub(crate) mod windows;

pub use route::{NavItem, PageGroup, Route, SelectorAction, SelectorEntry};

use gpui_kit::component::ThemeMode;
use gpui_kit::*;

use crate::theme;
use main_window::MainWindow;

/// 窗口初始尺寸与最小尺寸，与 PCL 启动器主窗口一致（850×500 / 810×470）。
const WINDOW_SIZE: (Pixels, Pixels) = (px(850.), px(500.));
const WINDOW_MIN_SIZE: (Pixels, Pixels) = (px(810.), px(470.));

/// 页面请求切换路由。分组视图发出，外壳统一处理，页面之间不互相依赖。
pub struct Navigate(pub Route);

/// 分组视图：路由变化时外壳把当前页面同步给它。
pub trait GroupView: Render + Sized {
    fn set_route(&mut self, route: Route, window: &mut Window, cx: &mut Context<Self>);

    /// 副页面标题。
    ///
    /// 分组自己知道当前栏位/子状态（例如实例页文件夹态显示「实例选择」，管理态才是「实例详情」），
    /// 因此标题由分组给出；副页面（`Route::is_sub`）当前都返回 `Some`，
    /// 标题栏对 `None` 显示空标题。不要改为用 `Route::label()` 兜底：
    /// 它对实例副页返回带 `{0}` 的模板（见 `route::Route::label`）。
    fn page_title(&self) -> Option<SharedString> {
        None
    }
}

/// 安装主题与界面基础设施；必须在打开窗口前调用。
pub fn init(cx: &mut App) {
    gpui_kit::init(cx);
    windows::init(cx);
    // gpui-kit 自带的中文文案（设置侧栏搜索框、对话框按钮等）取简体中文；
    // 应用自身的文案由 crate::i18n 提供，与这里无关。
    gpui_kit::component::set_locale("zh-CN");
    theme::install(cx, ThemeMode::Light);
}

/// 打开主窗口。
pub fn open_main_window(cx: &mut App) -> Result<()> {
    let bounds = Bounds {
        origin: point(px(0.), px(0.)),
        size: size(WINDOW_SIZE.0, WINDOW_SIZE.1),
    };
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        window_min_size: Some(size(WINDOW_MIN_SIZE.0, WINDOW_MIN_SIZE.1)),
        window_decorations: Some(WindowDecorations::Client),
        app_id: Some("PCL-Anywhere".to_owned()),
        // X11 用窗口图标；Wayland 下图标来自桌面文件，传了也不会报错。
        icon: Some(crate::assets::window_icon()),
        // 窗口标题供窗口管理器与任务栏使用（自绘标题栏不显示它）。
        titlebar: Some(TitlebarOptions {
            title: Some("PCL-Anywhere".into()),
            ..gpui_kit::component::TitleBar::title_bar_options()
        }),
        ..gpui_kit::component::TitleBar::window_options()
    };

    gpui_kit::open_window(options, cx, |window, cx| {
        cx.new(|cx| MainWindow::new(window, cx))
    })?;
    Ok(())
}
