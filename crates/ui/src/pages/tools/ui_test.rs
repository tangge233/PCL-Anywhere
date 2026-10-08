//! UI 测试页：把 [`crate::dialog`] 的每条路径摆成一个按钮，作答写进卡片下方的记录。
//!
//! 这里只做界面：显示、作答与排队都由对话框模块负责，本页把返回值记下来供人工核对
//! （也是对话框用法的一份可点示例）。

use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::*;

use super::ToolsGroup;
use crate::components::{AppButton, AppRadio, Card, PageScroll};
use crate::dialog::{self, Dialog, DialogButton, DialogInput, DialogTheme, buttons};
use crate::i18n::{self, Text};

/// 记录里保留的结果行数。
const RESULT_LINES: usize = 6;
/// 自定义内容里的选项个数。
const CHOICES: usize = 3;

impl ToolsGroup {
    pub(super) fn render_ui_test(&self, cx: &mut Context<Self>) -> AnyElement {
        let buttons = h_flex()
            .flex_wrap()
            .px_5()
            .gap_4()
            .child(test_button(
                cx,
                "ui-test-info",
                "Tools.UiTest.Alert.Info",
                |this, window, cx| this.ui_test_alert(DialogTheme::Info, window, cx),
            ))
            .child(test_button(
                cx,
                "ui-test-warning",
                "Tools.UiTest.Alert.Warning",
                |this, window, cx| this.ui_test_alert(DialogTheme::Warning, window, cx),
            ))
            .child(test_button(
                cx,
                "ui-test-error",
                "Tools.UiTest.Alert.Error",
                |this, window, cx| this.ui_test_alert(DialogTheme::Error, window, cx),
            ))
            .child(test_button(
                cx,
                "ui-test-confirm",
                "Tools.UiTest.Confirm",
                |this, window, cx| this.ui_test_confirm(window, cx),
            ))
            .child(test_button(
                cx,
                "ui-test-input",
                "Tools.UiTest.Input",
                |this, window, cx| this.ui_test_input(window, cx),
            ))
            .child(test_button(
                cx,
                "ui-test-custom",
                "Tools.UiTest.Custom",
                |this, window, cx| this.ui_test_custom(window, cx),
            ))
            .child(test_button(
                cx,
                "ui-test-queue",
                "Tools.UiTest.Queue",
                |this, window, cx| this.ui_test_queue(window, cx),
            ))
            .child(test_button(
                cx,
                "ui-test-side-effect",
                "Tools.UiTest.SideEffect",
                |this, window, cx| this.ui_test_side_effect(window, cx),
            ));

        PageScroll::new("tools-ui-test")
            .child(
                Card::new("tools-ui-test-dialog")
                    .title(i18n::lang("Tools.UiTest.Dialog"))
                    .child(buttons)
                    .child(self.ui_test_results()),
            )
            .into_any_element()
    }

    /// 结果记录：最新的在最上面，方便连点几个按钮后回看。
    fn ui_test_results(&self) -> AnyElement {
        let mut lines = v_flex().gap_1().child(
            div()
                .text_xs()
                .opacity(0.6)
                .child(i18n::lang("Tools.UiTest.Result")),
        );

        if self.ui_results.is_empty() {
            lines = lines.child(
                div()
                    .id("ui-test-result-empty")
                    .text_xs()
                    .opacity(0.6)
                    .child(i18n::lang("Tools.UiTest.Result.Empty"))
                    .test_support(),
            );
        }

        v_flex()
            .px_5()
            .pt_4()
            .pb_5()
            .child(
                lines.children(self.ui_results.iter().enumerate().map(|(index, line)| {
                    div()
                        .id(SharedString::from(format!("ui-test-result-{index}")))
                        .text_xs()
                        .aria_label(line.clone())
                        .test_support()
                        .child(line.clone())
                        .into_any_element()
                })),
            )
            .into_any_element()
    }

    /// 记一行结果；只留最近 [`RESULT_LINES`] 条。
    fn push_result(&mut self, line: SharedString, cx: &mut Context<Self>) {
        logger::log::info!(target: "Tools", "UI 测试作答：{line}");
        self.ui_results.insert(0, line);
        self.ui_results.truncate(RESULT_LINES);
        cx.notify();
    }

    /// 信息 / 警告 / 错误三态提示：单按钮，关闭时记一行。
    fn ui_test_alert(&mut self, theme: DialogTheme, window: &mut Window, cx: &mut Context<Self>) {
        let title_key = match theme {
            DialogTheme::Info => "Tools.UiTest.Alert.Info",
            DialogTheme::Warning => "Tools.UiTest.Alert.Warning",
            DialogTheme::Error => "Tools.UiTest.Alert.Error",
        };
        let label = i18n::lang(title_key);

        Dialog::<()>::new()
            .title(Text::key(title_key))
            .theme(theme)
            .content(test_caption())
            .button(DialogButton::new(buttons::GOT_IT).value(()))
            .on_result(cx.listener(move |this, _, _, cx| {
                let value = i18n::lang("Tools.UiTest.Value.Closed");
                this.push_result(result_line(&label, &value), cx);
            }))
            .open(window, cx);
    }

    /// 确定 / 取消：记录返回的布尔值。
    fn ui_test_confirm(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let label = i18n::lang("Tools.UiTest.Confirm");
        // 先建回调再传 `cx`：同一个表达式里既借 `cx` 又把它可变传给弹窗会冲突。
        let on_result = cx.listener(move |this, confirmed: &bool, _, cx| {
            let key = if *confirmed {
                "Tools.UiTest.Value.Confirmed"
            } else {
                "Tools.UiTest.Value.Cancelled"
            };
            this.push_result(result_line(&label, &i18n::lang(key)), cx);
        });
        dialog::dialog_confirm(
            window,
            cx,
            Text::key("Tools.UiTest.Confirm"),
            Text::key("Tools.UiTest.Caption"),
            on_result,
        );
    }

    /// 输入框：校验不通过时确定按钮禁用，结果记录输入内容。
    fn ui_test_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let label = i18n::lang("Tools.UiTest.Input");
        let on_result = cx.listener(move |this, result: &Option<SharedString>, _, cx| {
            let value = match result {
                Some(text) if text.is_empty() => i18n::lang("Tools.UiTest.Value.Empty"),
                Some(text) => text.clone(),
                None => i18n::lang("Tools.UiTest.Value.Cancelled"),
            };
            this.push_result(result_line(&label, &value), cx);
        });
        dialog::dialog_input(
            window,
            cx,
            DialogInput::new(Text::key("Tools.UiTest.Input"))
                .caption(Text::key("Tools.UiTest.Caption"))
                .hint(Text::key("Tools.UiTest.Input.Hint"))
                .validate(|text| {
                    text.trim()
                        .is_empty()
                        .then(|| Text::key("Tools.UiTest.Input.Invalid"))
                }),
            on_result,
        );
    }

    /// 自定义内容：单选列表，结果取自列表实体。
    fn ui_test_custom(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let label = i18n::lang("Tools.UiTest.Custom");
        let choices = cx.new(|_| ChoiceList { selected: None });

        Dialog::<Option<usize>>::new()
            .title(Text::key("Tools.UiTest.Custom"))
            .content({
                let choices = choices.clone();
                move |_, _, _| choices.clone().into_any_element()
            })
            .button(
                DialogButton::new(buttons::CONFIRM)
                    .value_with({
                        let choices = choices.clone();
                        // 未选中时返回 `None`：这次点击不作答，弹窗保持打开。
                        move |_, cx| choices.read(cx).selected.map(Some)
                    })
                    .disabled_when({
                        let choices = choices.clone();
                        move |_, cx| choices.read(cx).selected.is_none()
                    }),
            )
            .button(DialogButton::new(buttons::CANCEL).value(None))
            .observe(&choices)
            .on_result(cx.listener(move |this, choice: &Option<usize>, _, cx| {
                let value = match choice {
                    Some(index) => option_label(*index),
                    None => i18n::lang("Tools.UiTest.Value.Cancelled"),
                };
                this.push_result(result_line(&label, &value), cx);
            }))
            .open(window, cx);
    }

    /// 排队：一次点两个，前一个答完才显示后一个。
    fn ui_test_queue(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let first = i18n::lang("Tools.UiTest.Queue.First");
        let on_first = cx.listener(move |this, confirmed: &bool, _, cx| {
            let key = if *confirmed {
                "Tools.UiTest.Value.Confirmed"
            } else {
                "Tools.UiTest.Value.Cancelled"
            };
            this.push_result(result_line(&first, &i18n::lang(key)), cx);
        });
        dialog::dialog_confirm(
            window,
            cx,
            Text::key("Tools.UiTest.Queue.First"),
            Text::key("Tools.UiTest.Caption"),
            on_first,
        );

        let second = i18n::lang("Tools.UiTest.Queue.Second");
        Dialog::<()>::new()
            .title(Text::key("Tools.UiTest.Queue.Second"))
            .content(test_caption())
            .button(DialogButton::new(buttons::GOT_IT).value(()))
            .on_result(cx.listener(move |this, _, _, cx| {
                let value = i18n::lang("Tools.UiTest.Value.Closed");
                this.push_result(result_line(&second, &value), cx);
            }))
            .open(window, cx);
    }

    /// 副作用按钮：点「复制」只记一行、弹窗留着；点「取消」才关。
    fn ui_test_side_effect(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let label = i18n::lang("Tools.UiTest.SideEffect");
        Dialog::<()>::new()
            .title(Text::key("Tools.UiTest.SideEffect"))
            .content(test_caption())
            .button(
                DialogButton::new(buttons::COPY)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        let value = i18n::lang("Tools.UiTest.Value.Copied");
                        this.push_result(result_line(&label, &value), cx);
                    }))
                    .value_with(|_, _| None),
            )
            .button(DialogButton::new(buttons::CANCEL).value(()))
            .open(window, cx);
    }
}

/// 测试按钮：点击后执行 `action`（每个按钮都要窗口，所以走 `cx.listener`）。
fn test_button(
    cx: &Context<ToolsGroup>,
    id: &'static str,
    label_key: &'static str,
    action: impl Fn(&mut ToolsGroup, &mut Window, &mut Context<ToolsGroup>) + 'static,
) -> AnyElement {
    AppButton::new(id, i18n::lang(label_key))
        .min_width(px(120.))
        .on_click(cx.listener(move |this, _, window, cx| action(this, window, cx)))
        .into_any_element()
}

/// 自定义内容里的单选列表（测试用）：选中项由它自己保存。
struct ChoiceList {
    selected: Option<usize>,
}

impl Render for ChoiceList {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut column = v_flex().gap_2();
        for index in 0..CHOICES {
            column = column.child(
                AppRadio::new(SharedString::from(format!("ui-test-choice-{index}")))
                    .label(option_label(index))
                    .selected(self.selected == Some(index))
                    .on_change(cx.listener(move |this, _, _, cx| {
                        this.selected = Some(index);
                        cx.notify();
                    })),
            );
        }
        column
    }
}

/// 选项文案（`Tools.UiTest.Value.Option`，序号从 1 起）。
fn option_label(index: usize) -> SharedString {
    let number = (index + 1).to_string();
    i18n::lang_with_args("Tools.UiTest.Value.Option", &[number.as_str()])
}

/// 一行结果：`标题 → 取值`。
fn result_line(label: &SharedString, value: &SharedString) -> SharedString {
    i18n::lang_with_args(
        "Tools.UiTest.Result.Line",
        &[label.as_ref(), value.as_ref()],
    )
}

/// 测试用弹窗的正文。
fn test_caption() -> impl Fn(&dialog::DialogHandle<()>, &mut Window, &mut App) -> AnyElement {
    move |_, _, cx| dialog::caption(Text::key("Tools.UiTest.Caption"), cx)
}
