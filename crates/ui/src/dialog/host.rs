//! 弹窗图层：窗口级宿主，负责排队、焦点、Enter 与动效。
//!
//! 图层挂在窗口根 [`Root`] 的插件位上，因此覆盖整个应用内容（含自绘标题栏）并压住通知层；
//! 每个窗口一份实例，子窗口（实例设置窗等）自动可用。

use std::collections::VecDeque;
use std::time::Duration;

use gpui_kit::base::{FocusTrapElement as _, Root, RootPlugin};
use gpui_kit::*;

use super::frame::{self, Phase};
use super::request::{ButtonSpec, PendingDialog};
use crate::components::{AppButton, ButtonColor};

/// 出场动画时长；动画结束后清掉请求，再显示队列里的下一个。
const CLOSE_DURATION: Duration = Duration::from_millis(170);

/// 安装弹窗图层。
///
/// 必须在 `gpui_kit::init` 之后调用：注册顺序就是图层顺序，弹窗要盖在通知层之上。
pub fn init(cx: &mut App) {
    Root::register_plugin::<DialogHost>(cx, DialogHost::new);
}

/// 当前窗口的弹窗宿主。
pub struct DialogHost {
    queue: VecDeque<PendingDialog>,
    active: Option<Active>,
    /// 面板容器的焦点句柄：请求没指定焦点目标时交给它，Enter 也由它接。
    focus_handle: FocusHandle,
    next_id: u64,
    /// 动效按相位预建：`with_animation` 每帧都要一个 `Animation`，而构造会分配缓动闭包。
    panel_enter: Animation,
    panel_leave: Animation,
    overlay: Animation,
}

struct Active {
    dialog: PendingDialog,
    /// 按钮行的元素标识：按钮行每帧重建，标识按位置建一次，免得每帧拼字符串。
    button_ids: Vec<ElementId>,
    /// 出场动画中：不再接受作答。
    closing: bool,
    /// 打开前的焦点，关闭后还回去。
    previous_focus: Option<FocusHandle>,
    /// 出场动画结束的任务；随弹窗一起丢弃。
    _close_task: Option<Task<()>>,
}

impl DialogHost {
    fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            queue: VecDeque::new(),
            active: None,
            focus_handle: cx.focus_handle(),
            next_id: 0,
            panel_enter: frame::panel_animation(Phase::Enter),
            panel_leave: frame::panel_animation(Phase::Leave),
            overlay: frame::overlay_animation(),
        }
    }

    pub(crate) fn allocate_id(&mut self) -> u64 {
        self.next_id += 1;
        self.next_id
    }

    /// 入队；空闲时立即显示。
    pub(crate) fn push(
        &mut self,
        dialog: PendingDialog,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.queue.push_back(dialog);
        if self.active.is_none() {
            self.show_next(window, cx);
        }
        cx.notify();
    }

    fn show_next(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(dialog) = self.queue.pop_front() else {
            return;
        };
        logger::log::info!(target: "Dialog", "显示弹窗：{}", dialog.title.resolve());
        let previous_focus = window.focused(cx);
        let button_ids = (0..dialog.buttons.len()).map(frame::button_id).collect();
        self.active = Some(Active {
            dialog,
            button_ids,
            closing: false,
            previous_focus,
            _close_task: None,
        });
        // 焦点交给请求指定的控件，没指定就是面板：Enter 由面板处理，Tab 在面板内循环
        // （面板是焦点陷阱）。焦点在这里给，而不是在构造请求时给——那时弹窗可能还在排队。
        let focus = self
            .active
            .as_ref()
            .and_then(|active| active.dialog.focus_on_show.clone())
            .unwrap_or_else(|| self.focus_handle.clone());
        focus.focus(window, cx);
    }

    /// 关闭当前弹窗并开始出场动画；成功返回 `true`。
    pub(crate) fn begin_close(
        &mut self,
        id: u64,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(active) = self.active.as_mut() else {
            return false;
        };
        if active.dialog.id != id || active.closing {
            return false;
        }
        self.start_closing(window, cx);
        true
    }

    /// 开始出场动画：动画走完才清掉请求（见 [`Self::finish_close`]）。
    fn start_closing(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let Some(active) = self.active.as_mut() else {
            return false;
        };
        active.closing = true;
        active._close_task = Some(cx.spawn_in(window, async move |this, cx| {
            cx.background_executor().timer(CLOSE_DURATION).await;
            if let Err(error) = this.update_in(cx, |host, window, cx| host.finish_close(window, cx))
            {
                // 窗口已经关掉时拿不到视图，属于正常收尾路径。
                logger::log::debug!(target: "Dialog", "关闭弹窗时窗口已不可用：{error}");
            }
        }));
        cx.notify();
        true
    }

    fn finish_close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(previous) = self.active.take().and_then(|active| active.previous_focus) {
            previous.focus(window, cx);
        }
        self.show_next(window, cx);
        cx.notify();
    }

    /// 按下第 `index` 个按钮：副作用 → 求值 → 作答并关闭。
    ///
    /// 三步都推迟到本次更新之外执行，而且必须走窗口级 [`Window::defer`]：回调里再开或关弹窗
    /// 是常见流程（确认后再弹一个），而 `Context::defer_in` 的回调仍在 `DialogHost::update`
    /// 里跑，回调再 `host.update` 会 panic（cannot update while it is already being updated）。
    fn press(
        &mut self,
        index: usize,
        event: ClickEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(active) = self.active.as_ref() else {
            return;
        };
        if active.closing {
            return;
        }
        let Some(button) = active.dialog.buttons.get(index) else {
            return;
        };
        if is_disabled(button, window, cx) {
            return;
        }
        let id = active.dialog.id;
        let on_click = button.on_click.clone();
        let value = button.value.clone();

        window.defer(cx, move |window, cx| {
            if let Some(on_click) = on_click {
                on_click(&event, window, cx);
            }
            // `None` = 这次点击不作答，弹窗保持打开。
            let Some(answer) = value(window, cx) else {
                return;
            };
            let Some(host) = Root::read(window, cx).plugin::<DialogHost>() else {
                return;
            };
            // 只关闭这次按下时那个弹窗：延后执行期间它可能已被别的路径关掉、换成了别的弹窗。
            if !host.update(cx, |host, cx| host.begin_close(id, window, cx)) {
                return;
            }
            answer(window, cx);
        });
    }

    /// 激活第一个可用按钮（内容里的事件要等同于按下主按钮时用，例如输入框回车）。
    pub(crate) fn press_first(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(active) = self.active.as_ref() else {
            return;
        };
        if active.closing {
            return;
        }
        let Some(index) = active
            .dialog
            .buttons
            .iter()
            .position(|button| !is_disabled(button, window, cx))
        else {
            return;
        };
        self.press(index, ClickEvent::default(), window, cx);
    }

    /// Enter：焦点在面板上时激活第一个可用按钮。
    ///
    /// 焦点在按钮或内容控件上时由它们自己处理 Enter（输入框回车走 [`Self::press_first`]）。
    fn activate_first(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.focus_handle.is_focused(window) {
            return;
        }
        self.press_first(window, cx);
    }

    fn button_row(&self, window: &mut Window, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let Some(active) = self.active.as_ref() else {
            return Vec::new();
        };
        let warn = active.dialog.theme.is_warning();
        let count = active.dialog.buttons.len();
        let host = cx.entity();
        active
            .dialog
            .buttons
            .iter()
            .zip(&active.button_ids)
            .enumerate()
            .map(|(index, (spec, id))| {
                let disabled = is_disabled(spec, window, cx);
                let color = spec
                    .style
                    .color
                    .unwrap_or_else(|| default_button_color(index, count, warn));
                let host = host.clone();
                AppButton::new(id.clone(), spec.style.label.resolve())
                    .color(color)
                    .height(frame::BUTTON_HEIGHT)
                    .disabled(disabled)
                    .on_click(move |event, window, cx| {
                        let event = event.clone();
                        host.update(cx, |host, cx| host.press(index, event, window, cx));
                    })
                    .into_any_element()
            })
            .collect()
    }
}

impl RootPlugin for DialogHost {}

impl Render for DialogHost {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(active) = self.active.as_ref() else {
            return div();
        };
        let theme = active.dialog.theme;
        let phase = if active.closing {
            Phase::Leave
        } else {
            Phase::Enter
        };

        let (panel_animation, overlay_animation) = match phase {
            Phase::Enter => (self.panel_enter.clone(), self.overlay.clone()),
            Phase::Leave => (self.panel_leave.clone(), self.overlay.clone()),
        };

        let backdrop = div()
            .id("dialog-backdrop")
            .test_support()
            .absolute()
            .inset_0()
            // 点遮罩拖动窗口（原版 `FormDragMove`）：遮罩不关闭弹窗。
            .window_control_area(WindowControlArea::Drag)
            .with_animation(
                backdrop_animation_id(phase),
                overlay_animation,
                move |element, t| {
                    let progress = if phase == Phase::Leave { 1. - t } else { t };
                    element.bg(frame::overlay_color(theme, progress))
                },
            );

        let title = active.dialog.title.resolve();
        let content = (active.dialog.content)(window, cx);
        let buttons = self.button_row(window, cx);
        let panel = frame::panel(cx, title, theme, content, buttons, available_size(window));

        let focus_handle = self.focus_handle.clone();
        let host = cx.entity();
        let panel = div()
            .relative()
            .focus_trap("dialog", &focus_handle)
            .on_key_down(move |event, window, cx| {
                if event.keystroke.key == "enter" {
                    host.update(cx, |host, cx| host.activate_first(window, cx));
                }
            })
            .child(panel)
            .with_animation(
                panel_animation_id(phase),
                panel_animation,
                move |element, progress| frame::apply_panel_animation(element, phase, progress),
            );

        div().absolute().inset_0().occlude().child(backdrop).child(
            div()
                .absolute()
                .inset_0()
                .flex()
                .items_center()
                .justify_center()
                .p(frame::MARGIN)
                .child(panel),
        )
    }
}

/// 面板可占的最大尺寸：视口扣掉窗口边框与两侧外边距。
fn available_size(window: &Window) -> Size<Pixels> {
    let paddings = gpui_kit::component::window_paddings(window);
    let viewport = window.viewport_size();
    size(
        (viewport.width - paddings.left - paddings.right - frame::MARGIN * 2.).max(px(0.)),
        (viewport.height - paddings.top - paddings.bottom - frame::MARGIN * 2.).max(px(0.)),
    )
}

/// 入场与出场用不同的动画标识：动画进度按元素标识缓存，复用标识出场不会重新开始。
fn panel_animation_id(phase: Phase) -> &'static str {
    match phase {
        Phase::Enter => "dialog-panel-enter",
        Phase::Leave => "dialog-panel-leave",
    }
}

fn backdrop_animation_id(phase: Phase) -> &'static str {
    match phase {
        Phase::Enter => "dialog-backdrop-enter",
        Phase::Leave => "dialog-backdrop-leave",
    }
}

/// 按钮配色：第一个按钮是主按钮（有两个以上按钮时高亮），警告态转红。
fn default_button_color(index: usize, count: usize, warn: bool) -> ButtonColor {
    match (index, warn, count) {
        (0, true, _) => ButtonColor::Red,
        (0, false, count) if count > 1 => ButtonColor::Highlight,
        _ => ButtonColor::Normal,
    }
}

fn is_disabled(spec: &ButtonSpec, window: &mut Window, cx: &mut App) -> bool {
    spec.disabled
        .as_ref()
        .is_some_and(|disabled| disabled(window, cx))
}
