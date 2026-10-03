//! 实例设置的独立窗口：把整块管理栏（概览 / 设置 / 存档 / 资源）弹到系统窗口里。
//!
//! 窗口有自己的「看哪个实例、停在哪个 Tab」（[`ManageView::Popped`]），与主窗口互不影响；
//! 实例数据按实例存放（见 [`super::data::InstanceData`]），两个窗口共享同一张实例数据表。
//! 装饰与标题栏交给系统，窗口标题用「实例设置 - <实例名>」。

use gpui_kit::base::v_flex;
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::*;

use super::data::InstanceId;
use super::{InstanceGroup, ManageView};
use crate::shell::windows;

/// 独立窗口的初始尺寸与最小尺寸。
const WINDOW_SIZE: (Pixels, Pixels) = (px(700.), px(600.));
const WINDOW_MIN_SIZE: (Pixels, Pixels) = (px(480.), px(400.));

/// 独立窗口的根视图：固定显示某一个实例的管理栏。
struct ManageWindow {
    group: Entity<InstanceGroup>,
    id: InstanceId,
}

/// 打开某个实例的管理栏窗口，返回它的窗口句柄（关闭时用得上）。
///
/// 窗口挂在 `parent`（主窗口）名下：主窗口关闭时它一并关闭。
pub(super) fn open(
    parent: AnyWindowHandle,
    group: Entity<InstanceGroup>,
    id: InstanceId,
    title: SharedString,
    cx: &mut App,
) -> Result<AnyWindowHandle> {
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds {
            origin: point(px(0.), px(0.)),
            size: size(WINDOW_SIZE.0, WINDOW_SIZE.1),
        })),
        window_min_size: Some(size(WINDOW_MIN_SIZE.0, WINDOW_MIN_SIZE.1)),
        // 独立窗口用系统装饰与标题栏，只有主窗口是自绘外壳。
        window_decorations: Some(WindowDecorations::Server),
        titlebar: Some(TitlebarOptions {
            title: Some(title),
            ..Default::default()
        }),
        icon: Some(crate::assets::window_icon()),
        app_id: Some("PCL-Anywhere".to_owned()),
        ..Default::default()
    };

    windows::open_child(parent, options, cx, move |_, cx| {
        cx.new(move |cx| {
            // 页面状态变化时窗口要跟着重绘：这里显式订阅（视图不是窗口根视图，GPUI 不会自动跟）。
            cx.observe(&group, |_, _, cx| cx.notify()).detach();
            ManageWindow { group, id }
        })
    })
}

/// 这个窗口是否还开着。
///
/// 不能按视图类型 `read::<ManageWindow>` 判断：窗口的根视图是 gpui-kit 包的一层 `Root`，
/// downcast 永远失败。直接看它还在不在窗口列表里。
pub(super) fn is_open(handle: AnyWindowHandle, cx: &App) -> bool {
    cx.windows().contains(&handle)
}

impl Render for ManageWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let instance_id = self.id.clone();
        v_flex()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(self.group.update(cx, |group, cx| {
                group.render_manage_column(ManageView::Popped(instance_id), window, cx)
            }))
    }
}
