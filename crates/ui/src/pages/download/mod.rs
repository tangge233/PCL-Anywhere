//! 下载页分组（对应 PCL 的下载页）。
//!
//! 选择栏的 17 个条目由路由目录生成；内容区目前只实现版本安装（`DownloadRoute::Minecraft`）：
//! 版本清单（`list`）与安装面板（`install`）两种界面，其余条目尚未实现，这里渲染占位内容。
//!
//! 模块划分：本文件持有页面状态与路由分发，[`list`] 是版本清单页，[`install`] 是安装面板，
//! [`loader_versions`] 是加载器版本选择弹窗的内容，[`state`] 是示例清单与常量；
//! 红 / 黄提示行是公共构件 [`crate::components::hint`]。

mod install;
mod list;
mod loader_versions;
mod state;

use gpui_kit::base::h_flex;
use gpui_kit::component::input::InputState;
use gpui_kit::*;
use std::time::Duration;

use self::state::{CATEGORIES, InstallPanel, InstallState, LOADERS, LoadPhase, VERSION_SAMPLES};
use crate::components::PagePlaceholder;
use crate::i18n;
use crate::shell::route::DownloadRoute;
use crate::shell::{GroupView, Navigate, Route};

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
    /// 版本分类开关（多选按钮组的受控状态）：与 [`CATEGORIES`] 等长，默认全选；
    /// 关掉的分类不进入下方版本列表。
    category_on: [bool; CATEGORIES.len()],
    /// 各加载器选中的版本下标（与 [`LOADERS`] 等长，`None` 表示这个加载器未选版本）。
    /// 多个加载器可以各选一个（Forge + Fabric API 这类组合），离开安装面板时清空。
    loader_choice: Vec<Option<usize>>,
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
            category_on: [true; CATEGORIES.len()],
            loader_choice: vec![None; LOADERS.len()],
            install: InstallState::Idle,
            _search_subscription: subscription,
            _load_task: None,
        };
        // 进入页面即拉一次清单，这里先进入加载态。
        this.reload(cx);
        this
    }

    /// 模拟一次清单请求：真实实现应换成版本清单服务，这里只驱动加载环
    /// （`PanLoad` / `MyLoading` + `Download.Version.LoadingList`）。
    fn reload(&mut self, cx: &mut Context<Self>) {
        self.load = LoadPhase::Loading;
        logger::log::info!(target: "Download", "刷新版本清单");
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
        logger::log::info!(target: "Download", "选中版本：{name}");
        self.instance_name
            .update(cx, |state, cx| state.set_value(name, window, cx));
        cx.notify();
    }

    /// 返回版本清单（对应 `ExitSelectPageCommand`）。
    ///
    /// 顺带清空加载器选择：加载器版本是按 Minecraft 版本取的，换了版本还留着旧选择，
    /// 会装出不相干的组合。
    fn exit_select(&mut self, cx: &mut Context<Self>) {
        self.panel = InstallPanel::List;
        self.selected = None;
        self.install = InstallState::Idle;
        self.loader_choice.fill(None);
        cx.notify();
    }

    /// 「开始安装」：安装流程尚未接入，这里只把界面切到安装中的状态。
    fn start_install(&mut self, cx: &mut Context<Self>) {
        let Some(index) = self.selected else {
            return;
        };
        if self.install == InstallState::Running {
            return;
        }
        self.install = InstallState::Running;
        logger::log::info!(target: "Download", "开始安装：{}", VERSION_SAMPLES[index].id);
        cx.notify();
    }

    /// 「取消」：对应 `CancelInstallCommand`（原版取消 `CancellationTokenSource`）。
    fn cancel_install(&mut self, cx: &mut Context<Self>) {
        self.install = InstallState::Cancelled;
        logger::log::info!(target: "Download", "取消安装");
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

/// 未迁移页面的名称：取选择栏条目的文案。
/// （`Route::label` 对 17 个下载子页都是「下载」，区分不开，所以这里查目录。）
fn sub_page_name(route: Route) -> SharedString {
    route
        .selector_entries()
        .iter()
        .find(|entry| entry.route == route)
        .map(|entry| i18n::lang(entry.title_key))
        .unwrap_or_else(|| route.label())
}
