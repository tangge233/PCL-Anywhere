//! 对话框请求：内容、按钮与结果。
//!
//! 请求只描述「显示什么」与「按钮返回什么」；按钮外观在 [`super::button`]，句柄在
//! [`super::handle`]，排队、焦点、动效在 [`super::host`]。
//! 结果类型 `R` 由调用方决定——按钮在**点击时**求值，因此结果可以是内容实体里的任意状态
//! （输入框的文本、列表的选中项），不必先塞回请求里。

use std::rc::Rc;

use gpui_kit::base::Root;
use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::*;

use super::button::{ButtonStyle, DialogButton, DisabledFn};
use super::handle::DialogHandle;
use super::host::DialogHost;
use super::theme::DialogTheme;
use crate::components::{Callback, ClickHandler};
use crate::i18n::Text;

type ContentFn<R> = Rc<dyn Fn(&DialogHandle<R>, &mut Window, &mut App) -> AnyElement>;

/// 观察目标：`cx.observe` 只能在宿主上下文里建，所以先存成闭包，`open` 时在宿主里建订阅。
type Observer = Rc<dyn Fn(&mut Context<DialogHost>) -> Subscription>;

/// 内容订阅：持有方是宿主，随弹窗关闭释放；回调在宿主上下文里执行。
type Subscriber =
    Rc<dyn Fn(&mut DialogHost, &mut Window, &mut Context<DialogHost>) -> Subscription>;

/// 一个对话框请求。
pub struct Dialog<R> {
    title: Option<Text>,
    theme: DialogTheme,
    focus_on_show: Option<FocusHandle>,
    content: Option<ContentFn<R>>,
    buttons: Vec<DialogButton<R>>,
    on_result: Option<Callback<R>>,
    observers: Vec<Observer>,
    subscribers: Vec<Subscriber>,
}

impl<R: 'static> Dialog<R> {
    /// 标题不给就按主题取（[`DialogTheme::default_title`]），内容不给是空弹窗；
    /// 按钮必须显式给：结果类型由调用方定，核心造不出默认按钮的返回值。
    pub fn new() -> Self {
        Self {
            title: None,
            theme: DialogTheme::default(),
            focus_on_show: None,
            content: None,
            buttons: Vec::new(),
            on_result: None,
            observers: Vec::new(),
            subscribers: Vec::new(),
        }
    }

    pub fn title(mut self, title: impl Into<Text>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn theme(mut self, theme: DialogTheme) -> Self {
        self.theme = theme;
        self
    }

    /// 显示时把键盘焦点交给这个控件（默认给面板）。
    ///
    /// 焦点由宿主在弹窗真正显示时给：请求可能先排队，那时控件还不在元素树里，
    /// 构造请求时就抢焦点会让当前显示的弹窗失去键盘响应。
    pub fn focus_on_show(mut self, focus: &FocusHandle) -> Self {
        self.focus_on_show = Some(focus.clone());
        self
    }

    /// 内容槽：每帧重建，`handle` 让内容自己作答（异步流程用）。
    ///
    /// 闭包在渲染中执行，所以不要在闭包里作答或关闭弹窗（见 [`DialogHandle`]）。
    /// 不给内容时是个只有按钮的空弹窗。
    pub fn content(
        mut self,
        content: impl Fn(&DialogHandle<R>, &mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        self.content = Some(Rc::new(content));
        self
    }

    pub fn button(mut self, button: DialogButton<R>) -> Self {
        self.buttons.push(button);
        self
    }

    /// 得到作答后调用（[`DialogHandle::close`] 关闭不触发）。
    pub fn on_result(mut self, handler: impl Fn(&R, &mut Window, &mut App) + 'static) -> Self {
        self.on_result = Some(Rc::new(handler));
        self
    }

    /// 让宿主跟着这个实体的变化重绘。
    ///
    /// 按钮行由宿主渲染，内容实体自己 `notify` 只重绘它自己那棵子树；禁用条件与取值都读
    /// 内容状态，所以内容状态变了要显式挂上。订阅随弹窗关闭释放。
    pub fn observe<T: Render>(mut self, entity: &Entity<T>) -> Self {
        let entity = entity.clone();
        self.observers
            .push(Rc::new(move |cx: &mut Context<DialogHost>| {
                cx.observe(&entity, |_, _, cx| cx.notify())
            }));
        self
    }

    /// 内容里的输入框回车等同于按下主按钮。
    ///
    /// 订阅由弹窗宿主持有，回调就在宿主上下文里执行，所以这里能直接按下按钮；
    /// 订阅随弹窗关闭释放。
    pub fn submit_on_enter(mut self, input: &Entity<InputState>) -> Self {
        let input = input.clone();
        self.subscribers
            .push(Rc::new(move |_host: &mut DialogHost, window, cx| {
                let input = input.clone();
                cx.subscribe_in(
                    &input,
                    window,
                    move |host: &mut DialogHost, _, event: &InputEvent, window, cx| {
                        if matches!(event, InputEvent::PressEnter { .. }) {
                            host.press_first(window, cx);
                        }
                    },
                )
            }));
        self
    }

    /// 提交请求并返回句柄。宿主按队列一次显示一个。
    pub fn open(mut self, window: &mut Window, cx: &mut App) -> DialogHandle<R> {
        let Some(host) = Root::read(window, cx).plugin::<DialogHost>() else {
            // 窗口根不是 gpui-kit 的 Root（例如没走 `gpui_kit::open_window`）：无处可显示。
            logger::log::error!(target: "Dialog", "窗口没有弹窗图层，请求被丢弃");
            debug_assert!(false, "窗口缺少 DialogHost 图层：请确认先调用 pcl_ui::init");
            return DialogHandle::detached();
        };

        let id = host.update(cx, |host, _| host.allocate_id());
        let handle = DialogHandle::new(id, host.downgrade(), self.on_result.take());

        let content: ErasedContent = {
            let handle = handle.clone();
            let content = self.content.take();
            Rc::new(move |window, cx| match &content {
                Some(content) => content(&handle, window, cx),
                None => div().into_any_element(),
            })
        };

        let buttons = self
            .buttons
            .into_iter()
            .map(|button| {
                let DialogButton {
                    style,
                    value,
                    on_click,
                    disabled,
                } = button;
                let on_result = handle.result_handler();
                ButtonSpec {
                    style,
                    disabled,
                    on_click,
                    // 求值在这里完成，作答（触发结果回调）留成动作：宿主只负责何时执行。
                    value: Rc::new(move |window, cx| {
                        let value = value.as_ref()?(window, cx)?;
                        let on_result = on_result.clone();
                        Some(Box::new(move |window: &mut Window, cx: &mut App| {
                            if let Some(handler) = &on_result {
                                handler(&value, window, cx);
                            }
                        }) as AnswerAction)
                    }),
                }
            })
            .collect();

        let subscriptions = host.update(cx, |host, cx| {
            let mut subscriptions: Vec<Subscription> = self
                .observers
                .into_iter()
                .map(|observe| observe(cx))
                .collect();
            subscriptions.extend(
                self.subscribers
                    .into_iter()
                    .map(|subscribe| subscribe(host, window, cx)),
            );
            subscriptions
        });

        let theme = self.theme;
        let dialog = PendingDialog {
            id,
            title: self.title.unwrap_or_else(|| theme.default_title()),
            theme,
            content,
            buttons,
            focus_on_show: self.focus_on_show,
            _subscriptions: subscriptions,
        };
        host.update(cx, |host, cx| host.push(dialog, window, cx));
        handle
    }
}

impl<R: 'static> Default for Dialog<R> {
    fn default() -> Self {
        Self::new()
    }
}

/// 抹掉结果类型后的按钮：宿主不关心 `R`，只关心「这次点击是否作答」。
pub(crate) struct ButtonSpec {
    pub(crate) style: ButtonStyle,
    pub(crate) disabled: Option<DisabledFn>,
    pub(crate) on_click: Option<ClickHandler>,
    /// 点击时求值：`Some` = 这次的作答，执行它即触发结果回调（不碰宿主）。
    pub(crate) value: ValueSpec,
}

/// 一次已经定下结果的作答。
pub(crate) type AnswerAction = Box<dyn FnOnce(&mut Window, &mut App)>;

/// 抹掉结果类型后的内容构造。
pub(crate) type ErasedContent = Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>;

/// 抹掉结果类型后的按钮求值。
pub(crate) type ValueSpec = Rc<dyn Fn(&mut Window, &mut App) -> Option<AnswerAction>>;

/// 待显示的请求（内容与按钮都已抹掉结果类型）。
pub(crate) struct PendingDialog {
    pub(crate) id: u64,
    pub(crate) title: Text,
    pub(crate) theme: DialogTheme,
    pub(crate) content: ErasedContent,
    pub(crate) buttons: Vec<ButtonSpec>,
    /// 显示时接管键盘焦点的控件；空则由宿主交给面板。
    pub(crate) focus_on_show: Option<FocusHandle>,
    /// 随弹窗存活：关闭时释放观察与事件订阅。
    pub(crate) _subscriptions: Vec<Subscription>,
}
