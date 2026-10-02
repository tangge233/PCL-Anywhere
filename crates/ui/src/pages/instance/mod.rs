//! 实例页分组（对应 `PCL/Views/Instance/*`）。
//!
//! 实例详情（`InstanceRoute::Select`）与实例设置（`InstanceRoute::Setup`）共用同一个三栏合并页
//! （文件夹 / 实例列表 / 实例管理），仅进入时的初始栏位与默认 Tab 不同；存档管理
//! （`InstanceRoute::Saves`）是「300px 存档选择栏 + 存档内容」两栏。
//!
//! 这里只做界面与界面状态：文件夹、实例、存档都由示例数据填充（见 [`state`]），
//! 不连接任何实例扫描 / 存档解析逻辑。切换栏位、选择实例、切换管理 Tab、切换存档都只改本地状态。
//!
//! 每个实例各自决定管理栏在哪里操作：内嵌在主窗口，或弹到独立窗口。弹出去的那个实例，
//! 主窗口切到它时该位置显示提示页；其它实例照常内嵌操作，也各自可以再弹一个窗口。
//! 实例数据按实例目录的绝对路径索引（见 [`data`]）。
//!
//! 模块划分：本文件持有页面状态、路由与栏位切换；[`data`] 是实例数据表与唯一键，
//! [`merged`] 是三栏合并页的布局与动画，[`manage`] 是管理栏的 Tab 与四块内容，
//! [`saves`] 是存档管理页，[`state`] 是示例数据，[`window`] 是把管理栏整块弹出去的独立窗口。

use gpui_kit::component::WindowExt as _;
use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::component::notification::Notification;
use gpui_kit::*;

use self::data::{InstanceId, InstanceStore, sample_instance_ids};
use self::popped::PoppedPages;
use self::state::{SAMPLE_INSTANCES, SAMPLE_SAVES, SampleInstance, SampleSave};
use self::view::{ManageView, ManageViewState};
use crate::i18n;
use crate::shell::route::InstanceRoute;
use crate::shell::{GroupView, Navigate, Route};
use std::time::Duration;

mod data;
mod manage;
mod merged;
mod popped;
mod saves;
mod state;
mod view;
mod window;

/// 物理尺寸：与 PCL 的 `PageInstanceMerged.axaml.cs` 中的栏宽常量一致。
/// 栏宽是布局约束而非间距，不宜换算成 rem，故直接写 px 并在此集中登记。
const FOLDER_COLUMN_W: f32 = 200.;
const INSTANCE_COLUMN_W: f32 = 260.;
const SELECTOR_W: f32 = 300.;
const MANAGE_SAVES_SELECTOR_W: f32 = 150.;
const COLLAPSE_BAR_W: f32 = 18.;
/// 分隔线宽 1px（XAML 的 `columnDivider`）。
const DIVIDER_W: f32 = 1.;
/// 栏位切换的横滑时长（XAML 代码里的 `SlideDuration`）。
const SLIDE_DURATION: Duration = Duration::from_millis(250);

/// 管理栏 Tab 下标（与 XAML 的 TabItem 顺序一致）。
const TAB_OVERVIEW: usize = 0;
const TAB_SETTINGS: usize = 1;
const TAB_SAVES: usize = 2;
const TAB_MOD: usize = 3;
const TAB_RESOURCE_PACKS: usize = 4;
const TAB_SHADERS: usize = 5;
const TAB_SCHEMATICS: usize = 6;

/// 实例页分组主控：持有路由与全部界面状态。
pub struct InstanceGroup {
    route: Route,
    /// 横向栏位：1 = 文件夹 + 实例列表；2 = 实例列表 + 实例管理。
    column_state: u8,
    /// 栏位切换是否播放滑动动画：进入页面时直接呈现目标状态，点窄条时才滑动。
    animate_slide: bool,
    /// 主窗口的管理栏视图状态（看哪个实例、停在哪个 Tab）。
    main_view: ManageViewState,
    /// 弹出窗口登记表：哪些实例在独立窗口里、各自停在哪个 Tab。
    popped: PoppedPages,
    selected_folder: usize,
    /// 实例数据表，按实例目录的绝对路径索引。
    data: InstanceStore,
    /// 实例列表搜索框与当前过滤词（仅用于示例数据的过滤）。
    search: Entity<InputState>,
    search_query: SharedString,
    _subscriptions: Vec<Subscription>,
}

impl InstanceGroup {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search = cx
            .new(|cx| InputState::new(window, cx).placeholder(i18n::lang("Common.Action.Search")));
        // 搜索框只驱动示例数据的过滤；真实实现应交给实例列表视图模型。
        let search_sub = cx.subscribe_in(
            &search,
            window,
            |this, state: &Entity<InputState>, event: &InputEvent, _, cx| {
                if matches!(event, InputEvent::Change) {
                    this.search_query = state.read(cx).value();
                    cx.notify();
                }
            },
        );

        Self {
            route: Route::Instance(InstanceRoute::Select),
            column_state: 1,
            animate_slide: false,
            main_view: ManageViewState::default(),
            popped: PoppedPages::default(),
            selected_folder: 0,
            data: InstanceStore::new(),
            search,
            search_query: SharedString::default(),
            _subscriptions: vec![search_sub],
        }
    }

    fn instance(&self, index: usize) -> Option<&'static SampleInstance> {
        SAMPLE_INSTANCES.get(index)
    }

    /// 按路径找示例实例。
    fn instance_by_id(&self, id: &InstanceId) -> Option<&'static SampleInstance> {
        sample_instance_ids()
            .iter()
            .position(|candidate| candidate == id)
            .and_then(|index| self.instance(index))
    }

    /// 主窗口管理栏显示的实例。
    fn main_instance(&self) -> Option<&'static SampleInstance> {
        self.main_view
            .instance
            .as_ref()
            .and_then(|id| self.instance_by_id(id))
    }

    /// 主窗口存档页 / 存档 Tab 里选中的存档。
    fn selected_save(&self) -> Option<&'static SampleSave> {
        let id = self.main_view.instance.as_ref()?;
        let index = self.data.get(id)?.selected_save?;
        SAMPLE_SAVES.get(index)
    }

    /// 视图显示的实例。
    fn view_instance(&self, view: &ManageView) -> Option<InstanceId> {
        match view {
            ManageView::Main => self.main_view.instance.clone(),
            ManageView::Popped(id) => Some(id.clone()),
        }
    }

    /// 视图停在哪个 Tab。
    fn view_tab(&self, view: &ManageView) -> usize {
        match view {
            ManageView::Main => self.main_view.tab,
            ManageView::Popped(id) => self.popped.tab(id).unwrap_or(TAB_OVERVIEW),
        }
    }

    /// 切换视图的管理栏 Tab。
    fn set_view_tab(&mut self, view: &ManageView, tab: usize, cx: &mut Context<Self>) {
        match view {
            ManageView::Main => self.main_view.tab = tab,
            ManageView::Popped(id) => self.popped.set_tab(id, tab),
        }
        cx.notify();
    }

    /// 某个实例的管理栏是否已弹到独立窗口（窗口被关掉的记录顺手清掉）。
    fn is_popped(&mut self, id: &InstanceId, cx: &App) -> bool {
        self.popped.is_popped(id, cx)
    }

    // ---- 三种路由的顶层布局 -------------------------------------------------

    /// 选中实例：只改选中项，不动栏位（双击才进详情）。
    fn select_instance(&mut self, index: usize) {
        self.main_view.instance = sample_instance_ids().get(index).cloned();
    }

    /// 当前栏位状态：1 = 文件夹 + 实例列表，2 = 实例列表 + 管理栏。
    pub fn column_state(&self) -> u8 {
        self.column_state
    }

    /// 进入副页面时直接呈现目标栏位（不播滑动动画）。
    fn begin_column_state(&mut self, state: u8) {
        self.column_state = state;
        self.animate_slide = false;
    }

    /// 点窄条切换栏位，带滑动动画。
    fn toggle_column_state(&mut self, to_manage: bool, cx: &mut Context<Self>) {
        self.column_state = if to_manage { 2 } else { 1 };
        self.animate_slide = true;
        cx.notify();
    }

    /// 把主窗口当前这个实例的管理栏弹到独立窗口；该实例之后只在独立窗口里操作。
    fn pop_out_manage_page(&mut self, window: &Window, cx: &mut Context<Self>) {
        let Some(id) = self.main_view.instance.clone() else {
            return;
        };
        if self.is_popped(&id, cx) {
            return;
        }
        self.popped.begin(id.clone(), self.main_view.tab);

        let name = self
            .instance_by_id(&id)
            .map(|instance| instance.name)
            .unwrap_or_default();
        let title = i18n::lang_with_args("Main.Title.InstanceSetup", &[name]);

        let group = cx.entity();
        let parent = window.window_handle();
        // 开窗会立刻渲染新窗口，而它要读同一份页面状态：等本轮更新结束再做。
        cx.spawn_in(window, async move |this, cx| {
            let opened = cx
                .update(
                    |window, cx| match window::open(parent, group, id.clone(), title, cx) {
                        Ok(handle) => Some(handle),
                        Err(error) => {
                            // 开不出来就继续内嵌显示，并把原因告诉用户。
                            window.push_notification(
                                Notification::error(i18n::lang("Instance.Manage.OpenFailed")),
                                cx,
                            );
                            eprintln!("打开实例设置窗口失败：{error}");
                            None
                        }
                    },
                )
                .ok()
                .flatten();
            this.update(cx, |this, cx| {
                match opened {
                    Some(handle) => this.popped.finish(&id, handle),
                    // 开窗失败：撤销登记，该实例继续在主窗口内嵌操作。
                    None => {
                        this.popped.remove(&id);
                    }
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// 收回某个实例的管理栏：关掉它的独立窗口，主窗口恢复内嵌内容。
    fn retract_manage_page(&mut self, id: &InstanceId, cx: &mut Context<Self>) {
        if let Some(handle) = self.popped.remove(id) {
            handle
                .update(cx, |_, window, _| window.remove_window())
                .ok();
        }
        cx.notify();
    }

    /// 把某个实例的独立窗口提到最前。
    fn focus_manage_window(&self, id: &InstanceId, cx: &mut App) {
        if let Some(handle) = self.popped.window(id) {
            handle
                .update(cx, |_, window, _| window.activate_window())
                .ok();
        }
    }
}

impl GroupView for InstanceGroup {
    /// 标题栏文案：实例详情跟着栏位走（第一态「实例选择」、第二态「实例详情」），
    /// 实例设置与存档管理带上当前选中的实例名 / 存档名。
    fn page_title(&self) -> Option<SharedString> {
        let unknown = i18n::lang("Common.State.Unknown");
        let name = |value: Option<&'static str>| value.unwrap_or_else(|| unknown.as_ref());

        match self.route {
            Route::Instance(InstanceRoute::Select) => Some(if self.column_state == 2 {
                i18n::lang("Main.Title.InstanceSelect")
            } else {
                i18n::lang("Launch.Home.SelectInstance")
            }),
            Route::Instance(InstanceRoute::Setup) => Some(i18n::lang_with_args(
                "Main.Title.InstanceSetup",
                &[name(self.main_instance().map(|instance| instance.name))],
            )),
            Route::Instance(InstanceRoute::Saves) => Some(i18n::lang_with_args(
                "Main.Title.SaveManagement",
                &[name(self.selected_save().map(|save| save.name))],
            )),
            _ => None,
        }
    }

    fn set_route(&mut self, route: Route, window: &mut Window, cx: &mut Context<Self>) {
        self.route = route;
        // 界面演示：没选实例时先选首个，否则这些页面没有内容可展示。
        let first = || sample_instance_ids().first().cloned();

        match route {
            // 实例详情：文件夹 + 实例列表。
            Route::Instance(InstanceRoute::Select) => {
                self.begin_column_state(1);
                self.main_view.tab = TAB_OVERVIEW;
            }
            // 实例设置：进入时展开管理栏并停在「设置」Tab。
            Route::Instance(InstanceRoute::Setup) => {
                self.begin_column_state(2);
                self.main_view.tab = TAB_SETTINGS;
                if self.main_view.instance.is_none() {
                    self.main_view.instance = first();
                }
            }
            // 存档管理：选中首个实例与首个存档，方便直接看到存档内容。
            Route::Instance(InstanceRoute::Saves) => {
                if self.main_view.instance.is_none() {
                    self.main_view.instance = first();
                }
                if let Some(id) = self.main_view.instance.clone() {
                    self.data
                        .get_or_create(&id, window, cx)
                        .selected_save
                        .get_or_insert(0);
                }
            }
            _ => {}
        }
        cx.notify();
    }
}

impl EventEmitter<Navigate> for InstanceGroup {}

impl Render for InstanceGroup {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        match self.route {
            Route::Instance(InstanceRoute::Saves) => self.render_saves(window, cx),
            _ => self.render_merged(window, cx),
        }
    }
}
