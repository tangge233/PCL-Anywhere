//! 对话框请求：按钮、结果与句柄。
//!
//! 请求只描述「显示什么」与「按钮返回什么」；排队、焦点、动效在 [`super::host`]。
//! 结果类型 `R` 由调用方决定——按钮在**点击时**求值，因此结果可以是内容实体里的任意状态
//! （输入框的文本、列表的选中项），不必先塞回请求里。

use std::rc::Rc;

use gpui_kit::base::Root;
use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::*;

use super::buttons;
use super::host::DialogHost;
use crate::components::{ButtonColor, Callback, ClickHandler};
use crate::i18n::Text;

/// 对话框配色。警告态（[`DialogTheme::Warning`] 与 [`DialogTheme::Error`]）把标题、主按钮与
/// 遮罩转成红色系。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DialogTheme {
    #[default]
    Info,
    Warning,
    Error,
}

impl DialogTheme {
    pub(crate) fn uses_red(self) -> bool {
        !matches!(self, Self::Info)
    }

    /// 不给标题时的默认标题（对应 PCL `MsgBoxWrapper._ResolveCaption`）。
    pub(crate) fn default_title(self) -> Text {
        match self {
            Self::Info => buttons::DEFAULT_TITLE,
            Self::Warning => buttons::WARNING_TITLE,
            Self::Error => buttons::ERROR_TITLE,
        }
    }
}

/// 按钮外观：文案与配色。配色为空时由框架按位置与主题定。
#[derive(Clone, Debug)]
pub struct ButtonStyle {
    pub label: Text,
    pub color: Option<ButtonColor>,
}

impl ButtonStyle {
    pub const fn new(label: Text) -> Self {
        Self { label, color: None }
    }

    /// 带配色的外观；`const`，供 [`super::buttons`] 的常量表使用。
    pub const fn with_color(label: Text, color: ButtonColor) -> Self {
        Self {
            label,
            color: Some(color),
        }
    }
}

impl From<Text> for ButtonStyle {
    fn from(label: Text) -> Self {
        Self::new(label)
    }
}

impl From<SharedString> for ButtonStyle {
    fn from(label: SharedString) -> Self {
        Self::new(Text::Literal(label))
    }
}

type ValueFn<R> = Rc<dyn Fn(&mut Window, &mut App) -> Option<R>>;
type DisabledFn = Rc<dyn Fn(&mut Window, &mut App) -> bool>;

/// 对话框按钮。
pub struct DialogButton<R> {
    pub(crate) style: ButtonStyle,
    pub(crate) value: Option<ValueFn<R>>,
    pub(crate) on_click: Option<ClickHandler>,
    pub(crate) disabled: Option<DisabledFn>,
}

impl<R: 'static> DialogButton<R> {
    /// 用常用按钮（[`super::buttons`]）或一段文案建按钮。
    pub fn new(style: impl Into<ButtonStyle>) -> Self {
        Self {
            style: style.into(),
            value: None,
            on_click: None,
            disabled: None,
        }
    }

    /// 点击时固定返回 `value`。
    pub fn value(self, value: R) -> Self
    where
        R: Clone,
    {
        self.value_with(move |_, _| Some(value.clone()))
    }

    /// 点击时求值：`None` 表示这次点击不作答，弹窗保持打开（例如内容还没准备好）。
    pub fn value_with(self, value: impl Fn(&mut Window, &mut App) -> Option<R> + 'static) -> Self {
        Self {
            value: Some(Rc::new(value)),
            ..self
        }
    }

    /// 先执行的副作用，不影响是否作答（「复制详情」这类按钮：做完事继续留着）。
    pub fn on_click(self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        Self {
            on_click: Some(Rc::new(handler)),
            ..self
        }
    }

    /// 覆盖常用按钮的配色。
    pub fn color(mut self, color: ButtonColor) -> Self {
        self.style.color = Some(color);
        self
    }

    /// 始终禁用。
    pub fn disabled(self, disabled: bool) -> Self {
        self.disabled_when(move |_, _| disabled)
    }

    /// 每帧求值的禁用条件：随内容状态变化（例如选择列表还没选中）。
    pub fn disabled_when(self, disabled: impl Fn(&mut Window, &mut App) -> bool + 'static) -> Self {
        Self {
            disabled: Some(Rc::new(disabled)),
            ..self
        }
    }
}

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
        let handle = DialogHandle {
            inner: Rc::new(HandleInner {
                id,
                host: host.downgrade(),
                on_result: self.on_result.take(),
            }),
        };

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
                let on_result = handle.inner.on_result.clone();
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

struct HandleInner<R> {
    id: u64,
    host: WeakEntity<DialogHost>,
    /// 作答回调：按钮的求值闭包与 [`DialogHandle::answer`] 各持一份。
    on_result: Option<Callback<R>>,
}

/// 弹窗句柄：内容可以拿它自己作答（异步流程），调用方也可以拿它关闭弹窗。
///
/// 只有当前正在显示的弹窗会接受作答：重复作答、过期句柄（弹窗已被别的路径关掉）都返回
/// `false` 且不触发回调。
///
/// 作答与关闭都要在渲染之外调用：按钮的副作用、求值与作答回调都跑在宿主之外；但渲染中的
/// 闭包（内容构造、禁用条件）在宿主更新里执行，在里面调用会 panic（重入宿主）。
pub struct DialogHandle<R> {
    inner: Rc<HandleInner<R>>,
}

impl<R> Clone for DialogHandle<R> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<R: 'static> DialogHandle<R> {
    /// 没有图层时拿到的句柄：任何操作都是空操作。
    fn detached() -> Self {
        Self {
            inner: Rc::new(HandleInner {
                id: 0,
                host: WeakEntity::new_invalid(),
                on_result: None,
            }),
        }
    }

    /// 作答并关闭；成功返回 `true`。
    pub fn answer(&self, value: R, window: &mut Window, cx: &mut App) -> bool {
        if !self.close(window, cx) {
            return false;
        }
        if let Some(handler) = self.inner.on_result.clone() {
            handler(&value, window, cx);
        }
        true
    }

    /// 只关闭，不作答。
    pub fn close(&self, window: &mut Window, cx: &mut App) -> bool {
        let Some(host) = self.inner.host.upgrade() else {
            return false;
        };
        host.update(cx, |host, cx| host.begin_close(self.inner.id, window, cx))
    }
}
