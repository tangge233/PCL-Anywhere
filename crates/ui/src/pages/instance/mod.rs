//! 实例页分组（对应 `PCL/Views/Instance/*`）。
//!
//! 实例详情（`InstanceRoute::Select`）与实例设置（`InstanceRoute::Setup`）共用同一个三栏合并页
//! （文件夹 / 实例列表 / 实例管理），仅进入时的初始栏位与默认 Tab 不同；存档管理
//! （`InstanceRoute::Saves`）是「300px 存档选择栏 + 存档内容」两栏。
//!
//! 这里只做界面与界面状态：文件夹、实例、存档都由示例数据填充（见 [`state`]），
//! 不连接任何实例扫描 / 存档解析逻辑。切换栏位、选择实例、切换管理 Tab、切换存档都只改本地状态。
//!
//! 模块划分：本文件持有页面状态、路由与栏位切换；[`merged`] 是三栏合并页的布局与动画，
//! [`manage`] 是管理栏的 Tab 与四块内容，[`saves`] 是存档管理页，[`state`] 是示例数据。

use gpui_kit::component::IndexPath;
use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::component::select::SelectState;
use gpui_kit::component::slider::SliderState;
use gpui_kit::*;

use self::state::{SAMPLE_INSTANCES, SAMPLE_JVM_ARGS, SampleInstance, sample_difficulties};
use crate::i18n;
use crate::shell::route::InstanceRoute;
use crate::shell::{GroupView, Navigate, Route};
use std::time::Duration;

mod manage;
mod merged;
mod saves;
mod state;

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
    manage_tab: usize,
    selected_folder: usize,
    selected_instance: Option<usize>,
    selected_save: Option<usize>,
    /// 实例列表搜索框与当前过滤词（仅用于示例数据的过滤）。
    search: Entity<InputState>,
    search_query: SharedString,
    /// 设置页：内存分配模式（0 = 继承全局 / 1 = 独立设置）与两个内存滑杆。
    memory_mode: usize,
    max_memory: Entity<SliderState>,
    initial_memory: Entity<SliderState>,
    /// 设置页：JVM 参数 / 游戏参数 / 自定义信息 / 前置 Classpath 文本框。
    jvm_args: Entity<InputState>,
    game_args: Entity<InputState>,
    custom_info: Entity<InputState>,
    classpath: Entity<InputState>,
    /// 存档设置：允许作弊 / 锁定难度 / 难度下拉。
    allow_commands: bool,
    lock_difficulty: bool,
    difficulty: Entity<SelectState<Vec<SharedString>>>,
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

        let text_input = |window: &mut Window, cx: &mut Context<Self>, value: &str| {
            cx.new(|cx| InputState::new(window, cx).default_value(value.to_owned()))
        };

        // 难度下拉：4 个难度档位，初始选中「普通」（示例数据）。
        let difficulty = cx
            .new(|cx| SelectState::new(sample_difficulties(), Some(IndexPath::new(2)), window, cx));

        let max_memory = cx.new(|_| {
            SliderState::new()
                .min(0.)
                .max(64.)
                .step(1.)
                .default_value(15_f32)
        });
        let initial_memory = cx.new(|_| {
            SliderState::new()
                .min(0.)
                .max(64.)
                .step(1.)
                .default_value(0_f32)
        });

        Self {
            route: Route::Instance(InstanceRoute::Select),
            column_state: 1,
            animate_slide: false,
            manage_tab: TAB_OVERVIEW,
            selected_folder: 0,
            selected_instance: None,
            selected_save: None,
            search,
            search_query: SharedString::default(),
            memory_mode: 0,
            max_memory,
            initial_memory,
            jvm_args: text_input(window, cx, SAMPLE_JVM_ARGS),
            game_args: text_input(window, cx, ""),
            custom_info: text_input(window, cx, ""),
            classpath: text_input(window, cx, ""),
            allow_commands: false,
            lock_difficulty: true,
            difficulty,
            _subscriptions: vec![search_sub],
        }
    }

    fn selected_instance(&self) -> Option<&'static SampleInstance> {
        self.selected_instance.and_then(|i| SAMPLE_INSTANCES.get(i))
    }

    // ---- 三种路由的顶层布局 -------------------------------------------------

    /// 选中实例：只改选中项，不动栏位（双击才进详情）。
    fn select_instance(&mut self, index: usize) {
        self.selected_instance = Some(index);
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
}

impl GroupView for InstanceGroup {
    /// 标题栏文案跟着栏位走：第一态是「实例选择」，第二态才是「实例详情」。
    /// 进入「实例设置」时沿用路由目录的标题（`实例设置 - <实例名>`）。
    fn page_title(&self) -> Option<SharedString> {
        match self.route {
            Route::Instance(InstanceRoute::Select) => Some(if self.column_state == 2 {
                i18n::lang("Main.Title.InstanceSelect")
            } else {
                i18n::lang("Launch.Home.SelectInstance")
            }),
            _ => None,
        }
    }

    fn set_route(&mut self, route: Route, _: &mut Window, cx: &mut Context<Self>) {
        self.route = route;
        match route {
            // 实例详情：文件夹 + 实例列表。
            Route::Instance(InstanceRoute::Select) => {
                self.begin_column_state(1);
                self.manage_tab = TAB_OVERVIEW;
            }
            // 实例设置：进入时展开管理栏并停在「设置」Tab。
            Route::Instance(InstanceRoute::Setup) => {
                self.begin_column_state(2);
                self.manage_tab = TAB_SETTINGS;
                // 界面演示：无选中实例时先选中首个，否则设置页无内容可展示。
                self.selected_instance.get_or_insert(0);
            }
            // 存档管理：选中首个实例与首个存档，方便直接看到存档内容。
            Route::Instance(InstanceRoute::Saves) => {
                self.selected_instance.get_or_insert(0);
                self.selected_save.get_or_insert(0);
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
            Route::Instance(InstanceRoute::Saves) => self.render_saves(cx),
            _ => self.render_merged(window, cx),
        }
    }
}
