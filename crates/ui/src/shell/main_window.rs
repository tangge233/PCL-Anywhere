//! 主窗口运行时：持有路由与返回栈，把导航请求转成路由变化并同步分组，渲染窗口框架。
//!
//! 窗口启动入口与 `Navigate` / `GroupView` 契约在父模块 `shell`。

use gpui_kit::base::v_flex;
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::*;

use super::title_bar::TitleBar;
use super::{GroupView, Navigate, PageGroup, Route};
use crate::pages;

pub(super) struct MainWindow {
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
    pub(super) fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut subscriptions = Vec::new();
        let mut this = Self {
            route: Route::Launch,
            history: Vec::new(),
            launch: cx.new(|cx| pages::launch::LaunchGroup::new(window, cx)),
            download: cx.new(|cx| pages::download::DownloadGroup::new(window, cx)),
            setup: cx.new(|cx| pages::setup::SetupGroup::new(window, cx)),
            tools: cx.new(|cx| pages::tools::ToolsGroup::new(window, cx)),
            instance: cx.new(|cx| pages::instance::InstanceGroup::new(window, cx)),
            _subscriptions: Vec::new(),
        };

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
