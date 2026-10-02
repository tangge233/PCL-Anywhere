//! 主窗口外壳：标题栏 + 当前分组（选择栏 + 内容区）。
//!
//! 外壳只做三件事：持有路由与返回栈、把导航请求转成路由变化、渲染窗口框架。
//! 每个分组自己拥有选择栏与内容，因此下载页与启动页可以有完全不同的左栏。

pub mod route;
pub mod title_bar;

pub use route::{NavItem, PageGroup, Route, SelectorAction, SelectorEntry};

use gpui_kit::base::v_flex;
use gpui_kit::component::{ActiveTheme as _, ThemeMode};
use gpui_kit::*;

use crate::pages;
use crate::theme;
use title_bar::TitleBar;

/// 窗口初始尺寸与最小尺寸，取自 `FormMain.axaml`（850×500 / 810×470）。
const WINDOW_SIZE: (Pixels, Pixels) = (px(850.), px(500.));
const WINDOW_MIN_SIZE: (Pixels, Pixels) = (px(810.), px(470.));

/// 页面请求切换路由。分组视图发出，外壳统一处理，页面之间不互相依赖。
pub struct Navigate(pub Route);

/// 分组视图：路由变化时外壳把当前页面同步给它。
pub trait GroupView: Render + Sized {
    fn set_route(&mut self, route: Route, window: &mut Window, cx: &mut Context<Self>);

    /// 副页面标题。
    ///
    /// 分组自己知道当前栏位/子状态（例如实例页的第一态是「实例选择」、第二态才是「实例详情」），
    /// 所以标题优先由分组给出；返回 `None` 时用路由目录里的标题。
    fn page_title(&self) -> Option<SharedString> {
        None
    }
}

/// 安装主题与界面基础设施；必须在打开窗口前调用。
pub fn init(cx: &mut App) {
    gpui_kit::init(cx);
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

struct MainWindow {
    route: Route,
    history: Vec<Route>,
    launch: Entity<pages::launch::LaunchGroup>,
    download: Entity<pages::download::DownloadGroup>,
    setup: Entity<pages::setup::SetupGroup>,
    tools: Entity<pages::tools::ToolsGroup>,
    instance: Entity<pages::instance::InstanceGroup>,
    _subscriptions: Vec<Subscription>,
}

impl MainWindow {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut subscriptions = Vec::new();
        let this = Self {
            route: Route::Launch,
            history: Vec::new(),
            launch: cx.new(|cx| pages::launch::LaunchGroup::new(window, cx)),
            download: cx.new(|cx| pages::download::DownloadGroup::new(window, cx)),
            setup: cx.new(|cx| pages::setup::SetupGroup::new(window, cx)),
            tools: cx.new(|cx| pages::tools::ToolsGroup::new(window, cx)),
            instance: cx.new(|cx| pages::instance::InstanceGroup::new(window, cx)),
            _subscriptions: Vec::new(),
        };

        let mut this = this;
        // 初始分组也要对齐首屏路由：分组各自持有自己那一页的状态。
        this.sync_group(this.route, window, cx);
        Self::subscribe_group(&this.launch, &mut subscriptions, window, cx);
        Self::subscribe_group(&this.download, &mut subscriptions, window, cx);
        Self::subscribe_group(&this.setup, &mut subscriptions, window, cx);
        Self::subscribe_group(&this.tools, &mut subscriptions, window, cx);
        Self::subscribe_group(&this.instance, &mut subscriptions, window, cx);
        this._subscriptions = subscriptions;
        this
    }

    /// 订阅一个分组视图的导航请求。
    fn subscribe_group<T>(
        entity: &Entity<T>,
        subscriptions: &mut Vec<Subscription>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) where
        T: GroupView + EventEmitter<Navigate> + 'static,
    {
        subscriptions.push(cx.subscribe_in(
            entity,
            window,
            |this, _: &Entity<T>, event: &Navigate, window, cx| {
                this.navigate(event.0, window, cx);
            },
        ));
    }

    /// 切换路由；进入副页面时记入返回栈。
    fn navigate(&mut self, route: Route, window: &mut Window, cx: &mut Context<Self>) {
        if route == self.route {
            return;
        }
        if route.is_sub() && !self.route.is_sub() {
            self.history.push(self.route);
        }
        self.route = route;
        self.sync_group(route, window, cx);
        cx.notify();
    }

    /// 返回上一级页面。
    fn go_back(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(previous) = self.history.pop() else {
            return;
        };
        self.route = previous;
        self.sync_group(previous, window, cx);
        cx.notify();
    }

    /// 当前副页面标题：由分组给出（同一分组的不同栏位标题不同）。
    fn page_title(&self, cx: &App) -> Option<SharedString> {
        match self.route.group() {
            PageGroup::Launch => self.launch.read(cx).page_title(),
            PageGroup::Download => self.download.read(cx).page_title(),
            PageGroup::Setup => self.setup.read(cx).page_title(),
            PageGroup::Tools => self.tools.read(cx).page_title(),
            PageGroup::Instance => self.instance.read(cx).page_title(),
        }
    }

    fn sync_group(&mut self, route: Route, window: &mut Window, cx: &mut Context<Self>) {
        match route.group() {
            PageGroup::Launch => self
                .launch
                .update(cx, |group, cx| group.set_route(route, window, cx)),
            PageGroup::Download => self
                .download
                .update(cx, |group, cx| group.set_route(route, window, cx)),
            PageGroup::Setup => self
                .setup
                .update(cx, |group, cx| group.set_route(route, window, cx)),
            PageGroup::Tools => self
                .tools
                .update(cx, |group, cx| group.set_route(route, window, cx)),
            PageGroup::Instance => self
                .instance
                .update(cx, |group, cx| group.set_route(route, window, cx)),
        }
    }
}

impl Render for MainWindow {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let body = match self.route.group() {
            PageGroup::Launch => self.launch.clone().into_any_element(),
            PageGroup::Download => self.download.clone().into_any_element(),
            PageGroup::Setup => self.setup.clone().into_any_element(),
            PageGroup::Tools => self.tools.clone().into_any_element(),
            PageGroup::Instance => self.instance.clone().into_any_element(),
        };

        v_flex()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child({
                // 标题栏只持有弱引用，窗口关闭后不会被它拖住。
                let window_handle = cx.entity().downgrade();
                let navigate = {
                    let window_handle = window_handle.clone();
                    move |route: &Route, window: &mut Window, cx: &mut App| {
                        window_handle
                            .update(cx, |this, cx| this.navigate(*route, window, cx))
                            .ok();
                    }
                };
                let back = move |window: &mut Window, cx: &mut App| {
                    window_handle
                        .update(cx, |this, cx| this.go_back(window, cx))
                        .ok();
                };
                let title = self.page_title(cx);
                TitleBar::new(self.route, title, navigate, back)
            })
            .child(div().flex_1().min_h_0().child(body))
    }
}
