//! 通用对话框：固定模板与自定义内容共用同一套面板骨架、按钮与结果机制。
//!
//! 视觉照 PCL .NET 版对话框的 XAML（面板 400/圆角 7/标题 23px/分割线/右对齐按钮行/遮罩淡入
//! 与面板回弹，见 [`frame`]），逻辑自研：
//!
//! - **内容是主干**。请求只带一个内容构造闭包，[`alert`]、[`confirm`] 这些
//!   常用实现都是同一入口的薄封装，没有模板枚举分叉。
//! - **结果类型由调用方定**（[`Dialog`] 的 `R`）。按钮在点击时求值，因此结果可以是内容实体
//!   里的任意状态（输入框文本、列表选中项）；求值返回 `None` 表示这次点击不作答、弹窗保持打开。
//! - **默认值只到标题**。标题不给就按主题取（Info / Warning / Error 各一套，与 PCL 的
//!   `_ResolveCaption` 一致），内容不给是空弹窗；按钮必须显式给——结果类型由调用方定，
//!   核心造不出默认按钮的返回值。常用文案与按钮在 [`buttons`] 里以常量提供。
//! - **一次显示一个**。后来的请求排队，视觉上始终是单层遮罩 + 居中面板。
//!
//! 键盘只有 Enter：焦点在面板上时激活第一个可用按钮，焦点在按钮上时由按钮自己处理
//! （打开时焦点在面板上）。Esc 不绑任何行为。
//!
//! 尚未实现：Markdown 模板（项目里还没有 Markdown 渲染器）。选择列表用自定义内容实现，
//! 见 `pages/download` 的加载器版本选择。
//!
//! # 用法
//!
//! ```ignore
//! // 常用实现：确定 / 取消，回调拿到 bool。
//! dialog::confirm(
//!     window,
//!     cx,
//!     buttons::DEFAULT_TITLE,
//!     Text::key_args("Instance.Overall.Delete.ConfirmMessage", &["1.20.1"]),
//!     cx.listener(|this, confirmed, _, cx| {
//!         if confirmed {
//!             this.delete(cx);
//!         }
//!     }),
//! );
//!
//! // 自定义内容：结果取自内容实体（列表的选中项），未选中时确定按钮禁用。
//! let picker = cx.new(|_| LoaderVersions::new(LOADER_VERSIONS[loader], self.loader_choice[loader]));
//! Dialog::<Option<usize>>::new()
//!     .title(Text::key("Download.Install.SelectVersion.Title"))
//!     .content({
//!         let picker = picker.clone();
//!         move |_, _, _| picker.clone().into_any_element()
//!     })
//!     .button(
//!         DialogButton::new(buttons::CONFIRM)
//!             .value_with({
//!                 let picker = picker.clone();
//!                 // 未选中时返回 None：这次点击不作答，弹窗保持打开。
//!                 move |_, cx| picker.read(cx).selected.map(Some)
//!             })
//!             .disabled_when({
//!                 let picker = picker.clone();
//!                 move |_, cx| picker.read(cx).selected.is_none()
//!             }),
//!     )
//!     .button(DialogButton::new(buttons::CANCEL).value(None))
//!     .observe(&picker)
//!     .on_result(cx.listener(|this, choice: &Option<usize>, _, cx| {
//!         if let Some(index) = choice {
//!             this.select_loader(*index, cx);
//!         }
//!     }))
//!     .open(window, cx);
//! ```

pub mod buttons;
mod frame;
mod host;
mod request;

pub use host::{DialogHost, init};
pub use request::{ButtonStyle, Dialog, DialogButton, DialogHandle, DialogTheme};

use std::rc::Rc;

use gpui_kit::base::v_flex;
use gpui_kit::*;

use crate::components::{HintLevel, hint_row_text};
use crate::i18n::Text;

/// 输入校验：返回 `Some(文案)` 表示不通过，文案显示在输入框下方并禁用确定按钮。
pub type Validator = Rc<dyn Fn(&str) -> Option<Text>>;

/// 输入框对话框的配置。
pub struct DialogInput {
    pub title: Text,
    /// 输入框上方的说明；不给则不显示。
    pub caption: Option<Text>,
    /// 输入框的占位提示。
    pub hint: Text,
    /// 初始内容。
    pub default: SharedString,
    pub validate: Option<Validator>,
}

impl DialogInput {
    pub fn new(title: Text) -> Self {
        Self {
            title,
            caption: None,
            hint: Text::literal(""),
            default: SharedString::default(),
            validate: None,
        }
    }

    pub fn caption(mut self, caption: Text) -> Self {
        self.caption = Some(caption);
        self
    }

    pub fn hint(mut self, hint: Text) -> Self {
        self.hint = hint;
        self
    }

    pub fn default_value(mut self, value: impl Into<SharedString>) -> Self {
        self.default = value.into();
        self
    }

    pub fn validate(mut self, validate: impl Fn(&str) -> Option<Text> + 'static) -> Self {
        self.validate = Some(Rc::new(validate));
        self
    }
}

/// 单按钮提示框：只报个信，按钮返回 `()`。
pub fn alert(
    window: &mut Window,
    cx: &mut App,
    title: Text,
    theme: DialogTheme,
    caption: Text,
) -> DialogHandle<()> {
    Dialog::new()
        .title(title)
        .theme(theme)
        .content(caption_content(caption))
        .button(DialogButton::new(buttons::GOT_IT).value(()))
        .open(window, cx)
}

/// 确定 / 取消：确定返回 `true`，取消返回 `false`。
pub fn confirm(
    window: &mut Window,
    cx: &mut App,
    title: Text,
    caption: Text,
    on_result: impl Fn(&bool, &mut Window, &mut App) + 'static,
) -> DialogHandle<bool> {
    Dialog::new()
        .title(title)
        .content(caption_content(caption))
        .button(DialogButton::new(buttons::CONFIRM).value(true))
        .button(DialogButton::new(buttons::CANCEL).value(false))
        .on_result(on_result)
        .open(window, cx)
}

/// 输入框：确定返回 `Some(输入的文本)`，取消返回 `None`；校验不通过时确定按钮禁用。
///
/// 焦点在输入框，Enter 等同于按下确定（走 [`DialogHost::press_first`]，因此同样受校验约束）。
pub fn input(
    window: &mut Window,
    cx: &mut App,
    spec: DialogInput,
    on_result: impl Fn(&Option<SharedString>, &mut Window, &mut App) + 'static,
) -> DialogHandle<Option<SharedString>> {
    use gpui_kit::component::input::{Input, InputState};

    let DialogInput {
        title,
        caption,
        hint,
        default,
        validate,
    } = spec;

    let input = cx.new(|cx| {
        InputState::new(window, cx)
            .placeholder(hint.resolve())
            .default_value(default)
    });

    let invalid = |input: &Entity<InputState>, validate: &Option<Validator>, cx: &App| {
        validate
            .as_ref()
            .is_some_and(|validate| validate(input.read(cx).value().as_ref()).is_some())
    };

    let confirm = DialogButton::new(buttons::CONFIRM)
        .value_with({
            let input = input.clone();
            let validate = validate.clone();
            move |_, cx| match invalid(&input, &validate, cx) {
                true => None,
                false => Some(Some(input.read(cx).value())),
            }
        })
        .disabled_when({
            let input = input.clone();
            let validate = validate.clone();
            move |_, cx| invalid(&input, &validate, cx)
        });

    let content_input = input.clone();
    let content_caption = caption.map(|caption| caption.resolve());
    let content_validate = validate.clone();
    let content = move |_: &DialogHandle<Option<SharedString>>, _: &mut Window, cx: &mut App| {
        let mut column = v_flex().gap_2();
        if let Some(caption) = content_caption.clone() {
            column = column.child(frame::caption(cx, caption));
        }
        column = column.child(Input::new(&content_input).w_full());
        // 校验不过时在输入框下方给红字：gpui-kit 的输入框没有错误态，输入框本身配色不变。
        let message = content_validate
            .as_ref()
            .and_then(|validate| validate(content_input.read(cx).value().as_ref()));
        if let Some(message) = message {
            column = column.child(hint_row_text(message.resolve(), HintLevel::Error, cx));
        }
        column.into_any_element()
    };

    Dialog::new()
        .title(title)
        .content(content)
        .button(confirm)
        .button(DialogButton::new(buttons::CANCEL).value(None))
        .observe(&input)
        // 输入框吃掉了 Enter，所以让弹窗把它转成「按下主按钮」。
        .submit_on_enter(&input)
        // 焦点给输入框：由宿主在显示时给（请求可能先排队）。
        .focus_on_show(&input.focus_handle(cx))
        .on_result(on_result)
        .open(window, cx)
}

/// 内容槽里的正文段落（15px / 行高 18px）：自定义内容想跟固定模板一样排正文时用。
pub fn caption_element(text: Text, cx: &App) -> AnyElement {
    v_flex()
        .child(frame::caption(cx, text.resolve()))
        .into_any_element()
}

/// 正文段落的内容构造：文字在构造请求时就定下（`Text` 解析一次），渲染只搬元素。
fn caption_content<R: 'static>(
    caption: Text,
) -> impl Fn(&DialogHandle<R>, &mut Window, &mut App) -> AnyElement {
    let caption = caption.resolve();
    move |_, _, cx| {
        v_flex()
            .child(frame::caption(cx, caption.clone()))
            .into_any_element()
    }
}
