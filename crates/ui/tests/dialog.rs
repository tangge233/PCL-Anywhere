//! 对话框集成测试：面板几何、按钮作答、Enter、自定义内容取值与排队。
//!
//! 都在 headless 窗口里渲染真实界面、派发真实事件；弹窗宿主挂在窗口根 `Root` 上，
//! 所以断言的是应用实际会渲染出来的那套图层。

use gpui_kit::base::v_flex;
use gpui_kit::component::Root;
use gpui_kit::test::TestWindowExt as _;
use std::cell::RefCell;
use std::rc::Rc;
// 不 glob 导入 gpui_kit：它会带进 GPUI 的 `test` 宏，遮蔽内置的 `#[test]`。
use gpui_kit::base::TestSupportExt as _;
use gpui_kit::{
    AppContext as _, Context, Entity, FocusHandle, InteractiveElement as _, IntoElement,
    ParentElement as _, Pixels, Render, SharedString, Styled as _, TestAppContext, Window,
    WindowHandle, div, px, size,
};

use pcl_ui::components::{AppButton, AppRadio};
use pcl_ui::dialog::{self, Dialog, DialogButton, DialogInput, DialogTheme, buttons};
use pcl_ui::i18n::{self, Text};

mod common;

/// 只用来承载弹窗图层的空视图；`focus` 是页面自己的焦点，用来验证弹窗关闭后焦点归还。
struct Blank {
    focus: FocusHandle,
}

impl Blank {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus: cx.focus_handle(),
        }
    }
}

impl Render for Blank {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .id("page-focus")
            .test_support()
            .track_focus(&self.focus)
            .child(AppButton::new("page-button", "页面按钮"))
    }
}

/// 打开一个装了弹窗图层的窗口（尺寸与主窗口一致），并给出承载图层的视图。
fn open_window_with_page(cx: &mut TestAppContext) -> (WindowHandle<Root>, Entity<Blank>) {
    cx.update(pcl_ui::init);
    let mut page = None;
    let handle = cx.open_window(size(px(850.), px(500.)), |window, cx| {
        let view = cx.new(Blank::new);
        page = Some(view.clone());
        Root::new(view, window, cx)
    });
    (handle, page.expect("视图创建成功"))
}

/// 打开一个装了弹窗图层的窗口（尺寸与主窗口一致）。
fn open_window(cx: &mut TestAppContext) -> WindowHandle<Root> {
    open_window_with_page(cx).0
}

/// 面板相对内容区（窗口去掉边框内缩）的四边留白。
fn panel_margins(window: &Window) -> (Pixels, Pixels, Pixels, Pixels) {
    let panel = window.find("dialog-panel").bounds();
    let paddings = gpui_kit::component::window_paddings(window);
    let viewport = window.viewport_size();
    (
        panel.origin.x - paddings.left,
        panel.origin.y - paddings.top,
        viewport.width - paddings.right - (panel.origin.x + panel.size.width),
        viewport.height - paddings.bottom - (panel.origin.y + panel.size.height),
    )
}

#[gpui_kit::test]
async fn confirm_dialog_is_centered_and_answers(cx: &mut TestAppContext) {
    let handle = open_window(cx);
    let answer = Rc::new(RefCell::new(None));
    let recorded = answer.clone();

    cx.update_window(handle.into(), |_, window, cx| {
        dialog::confirm(
            window,
            cx,
            buttons::DEFAULT_TITLE,
            Text::literal("确定要删除这个实例吗？"),
            move |confirmed: &bool, _, _| *recorded.borrow_mut() = Some(*confirmed),
        );
    })
    .expect("窗口仍开着");
    common::settle(cx).await;

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        let (left, top, right, bottom) = panel_margins(window);
        assert!(
            (left - right).abs() <= px(1.),
            "面板必须水平居中：左 {left:?} 右 {right:?}"
        );
        assert!(
            (top - bottom).abs() <= px(1.),
            "面板必须垂直居中：上 {top:?} 下 {bottom:?}"
        );
        assert!(
            window.find("dialog-panel").bounds().size.width >= px(400.),
            "面板宽度不得小于 400"
        );

        window.click("dialog-button-0", cx);
    })
    .expect("窗口仍开着");

    common::settle(cx).await;
    assert_eq!(*answer.borrow(), Some(true), "确定按钮应回传 true");

    // 作答后弹窗关闭（出场动画走完）。
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        assert!(window.try_find("dialog-panel").is_none(), "弹窗应已关闭");
    })
    .expect("窗口仍开着");
}

#[gpui_kit::test]
async fn enter_activates_the_first_button(cx: &mut TestAppContext) {
    let handle = open_window(cx);
    let answer = Rc::new(RefCell::new(None));
    let recorded = answer.clone();

    cx.update_window(handle.into(), |_, window, cx| {
        dialog::confirm(
            window,
            cx,
            buttons::DEFAULT_TITLE,
            Text::literal("继续吗？"),
            move |confirmed: &bool, _, _| *recorded.borrow_mut() = Some(*confirmed),
        );
        window.render_frame(cx);
        window.press("enter", cx);
    })
    .expect("窗口仍开着");

    common::settle(cx).await;
    assert_eq!(*answer.borrow(), Some(true), "Enter 应命中第一个按钮");
}

#[gpui_kit::test]
async fn input_dialog_submits_on_enter(cx: &mut TestAppContext) {
    let handle = open_window(cx);
    let answer = Rc::new(RefCell::new(None));
    let recorded = answer.clone();

    cx.update_window(handle.into(), |_, window, cx| {
        dialog::input(
            window,
            cx,
            DialogInput::new(Text::key("Common.Dialog.Title")).default_value("我的实例"),
            move |result: &Option<SharedString>, _, _| {
                *recorded.borrow_mut() = Some(result.clone())
            },
        );
        window.render_frame(cx);
        window.press("enter", cx);
    })
    .expect("窗口仍开着");

    common::settle(cx).await;
    assert_eq!(
        answer
            .borrow()
            .as_ref()
            .map(|result| result.as_deref().map(str::to_owned)),
        Some(Some("我的实例".to_owned())),
        "输入框里按 Enter 应等于按下确定，并回传输入内容"
    );
}

/// 自定义内容：结果取自内容实体，未选中时确定按钮不可用。
struct Picker {
    selected: Option<usize>,
}

const CHOICES: [&str; 3] = ["Forge", "Fabric", "NeoForge"];

impl Render for Picker {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut column = v_flex().gap_2();
        for (index, label) in CHOICES.iter().enumerate() {
            column = column.child(
                AppRadio::new(SharedString::from(format!("picker-{index}")))
                    .label(*label)
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

#[gpui_kit::test]
async fn custom_content_result_comes_from_its_entity(cx: &mut TestAppContext) {
    let handle = open_window(cx);
    let answer = Rc::new(RefCell::new(None));
    let recorded = answer.clone();
    let mut picker = None;

    cx.update_window(handle.into(), |_, window, cx| {
        let view = cx.new(|_| Picker { selected: None });
        picker = Some(view.clone());
        Dialog::<Option<usize>>::new()
            .title(buttons::DEFAULT_TITLE)
            .content({
                let view = view.clone();
                move |_, _, _| view.clone().into_any_element()
            })
            .button(
                DialogButton::new(buttons::CONFIRM)
                    .value_with({
                        let view = view.clone();
                        // 未选中时返回 None：这次点击不作答，弹窗留着。
                        move |_, cx| view.read(cx).selected.map(Some)
                    })
                    .disabled_when({
                        let view = view.clone();
                        move |_, cx| view.read(cx).selected.is_none()
                    }),
            )
            .button(DialogButton::new(buttons::CANCEL).value(None))
            .observe(&view)
            .on_result(move |choice: &Option<usize>, _, _| *recorded.borrow_mut() = Some(*choice))
            .open(window, cx);
        window.render_frame(cx);
    })
    .expect("窗口仍开着");
    common::settle(cx).await;

    // 未选中：确定按钮禁用，点击不作答。
    cx.update_window(handle.into(), |_, window, cx| {
        window.click("dialog-button-0", cx);
    })
    .expect("窗口仍开着");
    common::settle(cx).await;
    assert_eq!(*answer.borrow(), None, "未选中时不应作答");
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        assert!(window.try_find("dialog-panel").is_some(), "弹窗应仍开着");
        window.click("picker-2", cx);
    })
    .expect("窗口仍开着");

    // 选中之后：确定按钮可用，作答取自内容实体。
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        window.click("dialog-button-0", cx);
    })
    .expect("窗口仍开着");
    common::settle(cx).await;
    assert_eq!(
        *answer.borrow(),
        Some(Some(2)),
        "作答应取自内容实体的选中项"
    );
    assert!(
        picker
            .expect("内容实体")
            .read_with(cx, |picker, _| picker.selected)
            == Some(2),
        "内容实体自己保存选中项"
    );
}

#[gpui_kit::test]
async fn dialogs_are_shown_one_at_a_time(cx: &mut TestAppContext) {
    let handle = open_window(cx);
    let answers = Rc::new(RefCell::new(Vec::new()));

    cx.update_window(handle.into(), |_, window, cx| {
        // 第一个请求有两个按钮，第二个只有一个：用它区分当前显示的是哪一个。
        let recorded = answers.clone();
        Dialog::<&'static str>::new()
            .title(buttons::DEFAULT_TITLE)
            .content(|_, _, _| div().into_any_element())
            .button(DialogButton::new(buttons::CONFIRM).value_with(move |_, _| Some("第一个")))
            .button(DialogButton::new(buttons::CANCEL).value("第一个-取消"))
            .on_result(move |result: &&'static str, _, _| recorded.borrow_mut().push(*result))
            .open(window, cx);

        let recorded = answers.clone();
        Dialog::<&'static str>::new()
            .title(buttons::DEFAULT_TITLE)
            .content(|_, _, _| div().into_any_element())
            .button(DialogButton::new(buttons::CONFIRM).value("第二个"))
            .on_result(move |result: &&'static str, _, _| recorded.borrow_mut().push(*result))
            .open(window, cx);

        window.render_frame(cx);
        assert!(
            window.try_find("dialog-button-1").is_some(),
            "先到先显示：当前应是第一个弹窗"
        );
    })
    .expect("窗口仍开着");
    common::settle(cx).await;

    cx.update_window(handle.into(), |_, window, cx| {
        window.click("dialog-button-0", cx);
    })
    .expect("窗口仍开着");
    common::settle(cx).await;

    // 出场动画结束后才轮到第二个请求。
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        assert!(
            window.try_find("dialog-button-1").is_none(),
            "第二个弹窗只有一个按钮"
        );
        window.click("dialog-button-0", cx);
    })
    .expect("窗口仍开着");
    common::settle(cx).await;

    assert_eq!(*answers.borrow(), vec!["第一个", "第二个"]);
}

#[gpui_kit::test]
async fn long_caption_stays_inside_the_window(cx: &mut TestAppContext) {
    let handle = open_window(cx);

    cx.update_window(handle.into(), |_, window, cx| {
        dialog::alert(
            window,
            cx,
            buttons::ERROR_TITLE,
            DialogTheme::Error,
            Text::literal("很长的正文。".repeat(400)),
        );
    })
    .expect("窗口仍开着");

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        let panel = window.find("dialog-panel").bounds();
        let paddings = gpui_kit::component::window_paddings(window);
        let viewport = window.viewport_size();
        let available = viewport.height - paddings.top - paddings.bottom;
        assert!(
            panel.size.height <= available,
            "面板不得超出内容区：面板 {:?}，内容区 {:?}",
            panel.size.height,
            available
        );
        // 正文远长于窗口：面板必须顶到上限，而不是把窗口撑破。
        assert!(
            panel.size.height > available - px(60.),
            "面板应被夹到上限附近：面板 {:?}，上限 {:?}",
            panel.size.height,
            available
        );
        // 按钮仍在面板内（正文滚动，按钮不被挤出去）。
        let button = window.find("dialog-button-0").bounds();
        assert!(
            button.origin.y + button.size.height <= panel.origin.y + panel.size.height,
            "按钮必须留在面板内"
        );
    })
    .expect("窗口仍开着");
}

#[gpui_kit::test]
async fn side_effect_button_keeps_the_dialog_open(cx: &mut TestAppContext) {
    let handle = open_window(cx);
    let copied = Rc::new(RefCell::new(0));
    let counter = copied.clone();

    cx.update_window(handle.into(), |_, window, cx| {
        Dialog::<bool>::new()
            .title(buttons::DEFAULT_TITLE)
            .content(|_, _, _| div().into_any_element())
            .button(
                DialogButton::new(buttons::COPY)
                    .on_click(move |_, _, _| *counter.borrow_mut() += 1)
                    // 副作用按钮不作答：做完事继续留着。
                    .value_with(|_, _| None),
            )
            .button(DialogButton::new(buttons::CANCEL).value(true))
            .open(window, cx);
    })
    .expect("窗口仍开着");
    common::settle(cx).await;

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        window.click("dialog-button-0", cx);
    })
    .expect("窗口仍开着");
    common::settle(cx).await;

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(*copied.borrow(), 1, "副作用按钮应执行自己的动作");
        assert!(
            window.try_find("dialog-panel").is_some(),
            "不作答的按钮不得关闭弹窗"
        );
        window.click("dialog-button-1", cx);
    })
    .expect("窗口仍开着");
    common::settle(cx).await;

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        assert!(window.try_find("dialog-panel").is_none(), "取消应关闭弹窗");
    })
    .expect("窗口仍开着");
}

/// 自定义内容可以自己作答：内容拿到句柄，异步流程结束后由它关闭弹窗并给出结果。
#[gpui_kit::test]
async fn content_can_answer_through_the_handle(cx: &mut TestAppContext) {
    let handle = open_window(cx);
    let answer = Rc::new(RefCell::new(None));
    let recorded = answer.clone();
    let slot: Rc<RefCell<Option<dialog::DialogHandle<u8>>>> = Rc::new(RefCell::new(None));

    cx.update_window(handle.into(), |_, window, cx| {
        let slot = slot.clone();
        Dialog::<u8>::new()
            .title(buttons::DEFAULT_TITLE)
            .content(move |handle, _, _| {
                // 内容把句柄留给自己（真实场景是交给轮询任务）。
                *slot.borrow_mut() = Some(handle.clone());
                div().into_any_element()
            })
            .button(DialogButton::new(buttons::CANCEL).value_with(|_, _| None))
            .on_result(move |result: &u8, _, _| *recorded.borrow_mut() = Some(*result))
            .open(window, cx);
    })
    .expect("窗口仍开着");
    common::settle(cx).await;

    let dialog = slot.borrow().clone().expect("内容应拿到句柄");
    cx.update_window(handle.into(), |_, window, cx| {
        assert!(dialog.answer(7, window, cx), "当前弹窗应接受作答");
    })
    .expect("窗口仍开着");
    common::settle(cx).await;

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        assert!(
            window.try_find("dialog-panel").is_none(),
            "作答后弹窗应关闭"
        );
    })
    .expect("窗口仍开着");
    assert_eq!(*answer.borrow(), Some(7));
}

/// 作答回调里再开弹窗（确认后接着弹下一个）：回调不能跑在宿主的更新里，否则重入。
#[gpui_kit::test]
async fn result_handler_can_open_another_dialog(cx: &mut TestAppContext) {
    let handle = open_window(cx);

    cx.update_window(handle.into(), |_, window, cx| {
        dialog::confirm(
            window,
            cx,
            buttons::DEFAULT_TITLE,
            Text::literal("第一个"),
            |_, window, cx| {
                dialog::alert(
                    window,
                    cx,
                    buttons::DEFAULT_TITLE,
                    DialogTheme::Info,
                    Text::literal("第二个"),
                );
            },
        );
        window.render_frame(cx);
    })
    .expect("窗口仍开着");
    common::settle(cx).await;

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        window.click("dialog-button-0", cx);
    })
    .expect("窗口仍开着");
    common::settle(cx).await;

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        assert!(
            window.try_find("dialog-panel").is_some(),
            "作答回调里开的弹窗应显示出来"
        );
    })
    .expect("窗口仍开着");
}

/// 排队中的输入框弹窗不能抢走当前弹窗的焦点（否则可见弹窗的 Enter 与空格失效）。
#[gpui_kit::test]
async fn queued_input_dialog_does_not_steal_focus(cx: &mut TestAppContext) {
    let handle = open_window(cx);
    let answered = Rc::new(RefCell::new(None));
    let recorded = answered.clone();

    cx.update_window(handle.into(), |_, window, cx| {
        // 第一个：可见的普通弹窗。
        dialog::confirm(
            window,
            cx,
            buttons::DEFAULT_TITLE,
            Text::literal("第一个"),
            move |confirmed: &bool, _, _| *recorded.borrow_mut() = Some(*confirmed),
        );
        // 第二个：输入框弹窗，此刻只能排队。
        dialog::input(
            window,
            cx,
            DialogInput::new(Text::literal("第二个")),
            |_, _, _| {},
        );
        window.render_frame(cx);
    })
    .expect("窗口仍开着");
    common::settle(cx).await;

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        window.press("enter", cx);
    })
    .expect("窗口仍开着");
    common::settle(cx).await;

    assert_eq!(
        *answered.borrow(),
        Some(true),
        "可见弹窗应仍能用 Enter 作答"
    );
}

/// 焦点：打开时从页面移走（面板接管，Enter 才有效），关闭后还给打开前的控件。
#[gpui_kit::test]
async fn focus_returns_to_the_page_after_closing(cx: &mut TestAppContext) {
    let (handle, page) = open_window_with_page(cx);
    let page_focus = page.read_with(cx, |page, _| page.focus.clone());

    cx.update_window(handle.into(), |_, window, cx| {
        page_focus.focus(window, cx);
        window.render_frame(cx);
        assert_eq!(
            window.find("page-focus").focused(),
            Some(true),
            "页面应先拿到焦点"
        );
    })
    .expect("窗口仍开着");

    cx.update_window(handle.into(), |_, window, cx| {
        dialog::confirm(
            window,
            cx,
            buttons::DEFAULT_TITLE,
            Text::literal("焦点"),
            |_, _, _| {},
        );
        window.render_frame(cx);
        assert_eq!(
            window.find("page-focus").focused(),
            Some(false),
            "弹窗打开时焦点应离开页面"
        );
    })
    .expect("窗口仍开着");
    common::settle(cx).await;

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        window.click("dialog-button-1", cx);
    })
    .expect("窗口仍开着");
    common::settle(cx).await;

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        assert!(window.try_find("dialog-panel").is_none(), "弹窗应已关闭");
        assert_eq!(
            window.find("page-focus").focused(),
            Some(true),
            "关闭后焦点应还给页面"
        );
    })
    .expect("窗口仍开着");
}

/// 不给标题时按主题取（PCL 的 `_ResolveCaption`）：警告态取警告标题，而不是默认标题。
#[gpui_kit::test]
fn theme_supplies_the_default_title(cx: &mut TestAppContext) {
    let handle = open_window(cx);

    cx.update_window(handle.into(), |_, window, cx| {
        Dialog::<()>::new()
            .theme(DialogTheme::Warning)
            .content(|_, _, _| div().into_any_element())
            .button(DialogButton::new(buttons::CANCEL).value(()))
            .open(window, cx);
        window.render_frame(cx);

        let title = window.find("dialog-title").label().map(str::to_owned);
        assert_eq!(
            title.as_deref(),
            Some(i18n::lang("Common.Dialog.Warning").as_ref()),
            "警告态应取警告标题"
        );
    })
    .expect("窗口仍开着");
}

/// 内容槽是裁剪容器：控件的焦点环画在自身外侧 3px（gpui-kit 的 `FOCUS_RING_WIDTH`），
/// 内容贴着裁剪边界时环会被切掉，所以槽的内边距要留够。
#[gpui_kit::test]
async fn content_slot_leaves_room_for_the_focus_ring(cx: &mut TestAppContext) {
    let handle = open_window(cx);

    cx.update_window(handle.into(), |_, window, cx| {
        Dialog::<()>::new()
            .title(buttons::DEFAULT_TITLE)
            .content(|_, _, _| {
                v_flex()
                    .child(AppButton::new("ring-probe", "内容").w_full())
                    .into_any_element()
            })
            .button(DialogButton::new(buttons::CANCEL).value(()))
            .open(window, cx);
    })
    .expect("窗口仍开着");
    common::settle(cx).await;

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        let slot = window.find("dialog-content").bounds();
        let probe = window.find("ring-probe").bounds();
        let ring = px(3.);
        assert!(
            probe.origin.x - slot.origin.x >= ring,
            "左侧余量不足：内容槽 {slot:?}，控件 {probe:?}"
        );
        assert!(
            slot.origin.x + slot.size.width - (probe.origin.x + probe.size.width) >= ring,
            "右侧余量不足：内容槽 {slot:?}，控件 {probe:?}"
        );
        assert!(
            probe.origin.y - slot.origin.y >= ring,
            "上侧余量不足：内容槽 {slot:?}，控件 {probe:?}"
        );
        assert!(
            slot.origin.y + slot.size.height - (probe.origin.y + probe.size.height) >= ring,
            "下侧余量不足：内容槽 {slot:?}，控件 {probe:?}"
        );
    })
    .expect("窗口仍开着");
}

/// 入场从下方滑入：动画没跑起来时位移会是 0，且面板必须始终水平居中（不做水平位移）。
#[gpui_kit::test]
async fn panel_slides_in_from_below(cx: &mut TestAppContext) {
    let handle = open_window(cx);
    let mut early = None;

    cx.update_window(handle.into(), |_, window, cx| {
        dialog::confirm(
            window,
            cx,
            buttons::DEFAULT_TITLE,
            Text::literal("动画"),
            |_, _, _| {},
        );
        window.render_frame(cx);
        early = Some(window.find("dialog-panel").bounds());
    })
    .expect("窗口仍开着");

    common::settle(cx).await;
    let early = early.expect("第一帧就该有面板");
    let settled = cx
        .update_window(handle.into(), |_, window, cx| {
            window.render_frame(cx);
            window.find("dialog-panel").bounds()
        })
        .expect("窗口仍开着");

    assert!(
        early.origin.y > settled.origin.y + px(20.),
        "入场应从下方滑入：起始 {early:?}，停稳 {settled:?}"
    );
    assert_eq!(
        early.origin.x, settled.origin.x,
        "位移只走纵向，横向不得偏移：起始 {early:?}，停稳 {settled:?}"
    );
}
