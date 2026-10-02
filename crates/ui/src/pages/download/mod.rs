//! 下载页分组（对应 `PCL/Views/Download/*`）。
//!
//! 选择栏的 17 个条目由路由目录生成；内容区目前只实现版本安装（`DownloadRoute::Minecraft`）：
//! 版本清单（`list`）与安装面板（`install`）两种界面。其余条目在 .NET 版本里同样尚未实现，
//! 这里渲染占位内容。
//!
//! 模块划分：本文件持有页面状态与路由分发，[`state`] 是示例清单与常量，
//! [`list`] 是版本清单页，[`install`] 是安装面板。

mod install;
mod list;
mod state;

use gpui_kit::base::h_flex;
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::input::InputState;
use gpui_kit::*;
use std::time::Duration;

use self::state::{CATEGORIES, InstallPanel, InstallState, LOADERS, LoadPhase, VERSION_SAMPLES};
use crate::components::PagePlaceholder;
use crate::i18n;
use crate::shell::route::DownloadRoute;
use crate::shell::{GroupView, Navigate, Route};

use self::list::sub_page_name;
use super::route_selector;

pub struct DownloadGroup {
    route: Route,
    /// 版本清单 / 安装面板。
    panel: InstallPanel,
    /// 清单加载状态。
    load: LoadPhase,
    /// 清单筛选框（顶部的搜索）。
    search: Entity<InputState>,
    /// 安装面板里的实例名输入框（`TextSelectName`）。
    instance_name: Entity<InputState>,
    /// 选中版本在 [`VERSION_SAMPLES`] 中的下标。
    selected: Option<usize>,
    /// 三个版本分类卡片的展开状态（最新版卡片始终展开）。
    expanded: [bool; CATEGORIES.len()],
    /// 11 张加载器卡片的展开状态。
    loader_expanded: [bool; LOADERS.len()],
    /// 安装状态。
    install: InstallState,
    /// 搜索框变化时重绘（`InputState` 是独立实体，页面需要观察它）。
    _search_subscription: Subscription,
    /// 模拟清单请求的任务；`Task` 在 drop 时取消，所以挂起期间必须持有它。
    _load_task: Option<Task<()>>,
}

impl DownloadGroup {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search = cx
            .new(|cx| InputState::new(window, cx).placeholder(i18n::lang("Common.Action.Search")));
        let instance_name = cx.new(|cx| InputState::new(window, cx));
        let subscription = cx.observe(&search, |_, _, cx| cx.notify());

        let mut this = Self {
            route: Route::Download(DownloadRoute::Minecraft),
            panel: InstallPanel::List,
            load: LoadPhase::Loading,
            search,
            instance_name,
            selected: None,
            expanded: [false; CATEGORIES.len()],
            loader_expanded: [false; LOADERS.len()],
            install: InstallState::Idle,
            _search_subscription: subscription,
            _load_task: None,
        };
        // .NET 版在 `OnLoaded` 里拉一次清单（`RefreshCommand`），这里同样先进入加载态。
        this.reload(cx);
        this
    }

    /// 模拟一次清单请求：真实实现应换成版本清单服务，这里只驱动加载环
    /// （`PanLoad` / `MyLoading` + `Download.Version.LoadingList`）。
    fn reload(&mut self, cx: &mut Context<Self>) {
        self.load = LoadPhase::Loading;
        cx.notify();
        self._load_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(700))
                .await;
            this.update(cx, |this, cx| {
                this.load = LoadPhase::Ready;
                cx.notify();
            })
            .ok();
        }));
    }

    /// 选中版本并进入安装面板（对应 `SelectVersionCommand` + `EnterSelectPage`）。
    fn select_version(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.selected = Some(index);
        self.install = InstallState::Idle;
        self.panel = InstallPanel::Select;
        // 实例名默认取版本号（对应 ViewModel 的 `SelectName = version.Id`）。
        let name = VERSION_SAMPLES[index].id;
        self.instance_name
            .update(cx, |state, cx| state.set_value(name, window, cx));
        cx.notify();
    }

    /// 返回版本清单（对应 `ExitSelectPageCommand`）。
    fn exit_select(&mut self, cx: &mut Context<Self>) {
        self.panel = InstallPanel::List;
        self.selected = None;
        self.install = InstallState::Idle;
        cx.notify();
    }

    /// 「开始下载」：.NET 版在这里调用 `GameInstaller` 并订阅进度；
    /// 未接安装流程，只把界面切到安装中的状态。
    fn start_install(&mut self, cx: &mut Context<Self>) {
        if self.selected.is_none() || self.install == InstallState::Running {
            return;
        }
        self.install = InstallState::Running;
        cx.notify();
    }

    /// 「取消」：对应 `CancelInstallCommand`（原版取消 `CancellationTokenSource`）。
    fn cancel_install(&mut self, cx: &mut Context<Self>) {
        self.install = InstallState::Cancelled;
        cx.notify();
    }

    /// 选择栏条目右侧的动作按钮：版本安装页的刷新按钮重新拉清单，其余页面暂无动作。
    fn on_selector_action(&mut self, route: Route, cx: &mut Context<Self>) {
        if route == Route::Download(DownloadRoute::Minecraft) {
            self.reload(cx);
        }
    }
}

impl GroupView for DownloadGroup {
    fn set_route(&mut self, route: Route, _: &mut Window, cx: &mut Context<Self>) {
        self.route = route;
        cx.notify();
    }
}

impl EventEmitter<Navigate> for DownloadGroup {}

impl Render for DownloadGroup {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let route = self.route;
        let navigate = cx.listener(|_, route: &Route, _, cx| cx.emit(Navigate(*route)));
        let action = cx.listener(|this, route: &Route, _, cx| this.on_selector_action(*route, cx));

        let content = match route {
            Route::Download(DownloadRoute::Minecraft) => self.render_minecraft(cx),
            _ => PagePlaceholder::for_route(&sub_page_name(route)).into_any_element(),
        };

        h_flex()
            .items_stretch()
            .size_full()
            .child(route_selector(
                "download",
                route,
                px(255.),
                navigate,
                action,
            ))
            .child(div().flex_1().min_w_0().child(content))
    }
}
