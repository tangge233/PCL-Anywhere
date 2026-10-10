//! 面板骨架、视觉常量与动效。
//!
//! 尺寸、配色与动效取自 PCL .NET 版对话框的 XAML（`Controls/MyMsg/MyMsgText.xaml`、
//! `MyMsgInput.xaml` 等），逐项对应；两处已知差异：
//!
//! - 入场 / 出场的旋转（-4°→0°、0°→+6°）无法表达：GPUI 的元素样式没有旋转变换，
//!   面板因此只做上下位移与淡入淡出，不做倾斜（见 [`channels`]）。
//! - 原版遮罩会对背后的窗口内容做模糊采样（`BlurBorder`），跨平台拿不到「已渲染像素」，
//!   这里只有半透明色。

use std::time::Duration;

use gpui_kit::StatefulInteractiveElement as _;
use gpui_kit::base::TestSupportExt as _;
use gpui_kit::base::{box_shadow, v_flex};
use gpui_kit::*;

use super::request::DialogTheme;
use crate::theme;

/// 面板最小宽度。
pub(crate) const MIN_WIDTH: Pixels = px(400.);
/// 面板与窗口边缘的最小间距。
pub(crate) const MARGIN: Pixels = px(25.);
/// 面板圆角。
const RADIUS: Pixels = px(7.);
/// 面板内边距（原版 `PanMain Margin="22,22,22,23"`）。
const PADDING_TOP: Pixels = px(22.);
const PADDING_BOTTOM: Pixels = px(23.);
const PADDING_LEFT: Pixels = px(22.);
const PADDING_RIGHT: Pixels = px(22.);
/// 标题：23px，左距 7、上移 1、右侧留 70、下距 9。
const TITLE_SIZE: Rems = rems(1.4375);
const TITLE_LEFT: Pixels = px(7.);
const TITLE_RIGHT: Pixels = px(70.);
const TITLE_BOTTOM: Pixels = px(9.);
/// 正文：15px / 行高 18px。
const CAPTION_SIZE: Rems = rems(0.9375);
const CAPTION_LINE_HEIGHT: Rems = rems(1.125);
/// 标题下的分割线与它到内容的间距。
const DIVIDER_HEIGHT: Pixels = px(2.);
const DIVIDER_GAP: Pixels = px(13.);
/// 内容到按钮行的间距。
const CONTENT_GAP: Pixels = px(17.);
/// 内容槽的内边距：左右取原版正文的 `Padding="7,0,15,0"`，上下留出焦点环的余量。
///
/// 内容槽是裁剪容器（内容超出面板高度时滚动），而 gpui-kit 的焦点环画在控件外侧 3px
/// （`FOCUS_RING_WIDTH`），内容贴着裁剪边界时环会被切掉，所以上下各留 4px。
const CONTENT_LEFT: Pixels = px(7.);
const CONTENT_RIGHT: Pixels = px(15.);
const CONTENT_TOP: Pixels = px(4.);
const CONTENT_BOTTOM: Pixels = px(4.);
/// 按钮行：右对齐，按钮间距 12、右侧留 8、左侧至少 150。
const BUTTON_GAP: Pixels = px(12.);
const BUTTON_RIGHT: Pixels = px(8.);
const BUTTON_LEFT: Pixels = px(150.);
/// 按钮高度：原版 `Padding="5,0"` + `TextPadding=7` + 13px 行高 + 上下描边 ≈ 32。
pub(crate) const BUTTON_HEIGHT: Pixels = px(32.);
/// 面板阴影：`#3C3C3C`、模糊 20、下移 2、不透明度 0.8。
const SHADOW_COLOR: Rgba = Rgba {
    r: 0x3c as f32 / 255.,
    g: 0x3c as f32 / 255.,
    b: 0x3c as f32 / 255.,
    a: 0.8,
};

/// 按钮的元素标识：打开时的焦点与 Enter 都按位置找按钮。
pub(crate) fn button_id(index: usize) -> ElementId {
    ElementId::Name(SharedString::from(format!("dialog-button-{index}")))
}

/// 遮罩色：普通 `ARGB(90,0,0,0)`、警告 `ARGB(140,80,0,0)`；`progress` 是淡入进度。
pub(crate) fn overlay_color(theme: DialogTheme, progress: f32) -> Hsla {
    let (red, green, blue, alpha) = if theme.uses_red() {
        (80. / 255., 0., 0., 140. / 255.)
    } else {
        (0., 0., 0., 90. / 255.)
    };
    Rgba {
        r: red,
        g: green,
        b: blue,
        a: alpha * progress.clamp(0., 1.),
    }
    .into()
}

/// 面板动效方向。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Phase {
    Enter,
    Leave,
}

/// 把「延迟 + 时长」压成一个动画：总时长 = 延迟 + 时长，延迟段停在起点。
fn animation(delay_ms: u64, duration_ms: u64, ease: impl Fn(f32) -> f32 + 'static) -> Animation {
    let total = delay_ms + duration_ms;
    let delay = delay_ms as f32 / total as f32;
    Animation::new(Duration::from_millis(total)).with_easing(move |t: f32| {
        if t <= delay {
            0.
        } else {
            ease((t - delay) / (1. - delay))
        }
    })
}

/// 遮罩淡入 / 淡出：200ms，平滑结束（原版 `AniEaseOutFluent(Middle)`）。
pub(crate) fn overlay_animation() -> Animation {
    animation(0, 200, cubic_bezier_ease(EASE_OUT_CUBIC))
}

/// 入场位移：从下方滑入（原版 `TransformPos.Y = +40`）。
const ENTER_OFFSET: Pixels = px(40.);
/// 出场位移：向下掉出（原版 `+20`）。
const LEAVE_OFFSET: Pixels = px(20.);

/// 进场位移缓动：拟合原版 `AniEaseOutBack(Weak)`（`1-(1-t)²·cos(1.5πt)`，末端回弹）。
/// 拟合残差：rms 0.015、最大 0.036。
const EASE_BACK: (f32, f32, f32, f32) = (0.553, 1.715, 0.323, 1.080);
/// 出场位移缓动：拟合原版 `AniEaseOutFluent(Middle)`（`1-(1-t)³`）。残差 rms 0.003、最大 0.005。
const EASE_OUT_CUBIC: (f32, f32, f32, f32) = (0.186, 0.527, 0.396, 1.054);

/// 单位三次贝塞尔缓动（`p0=(0,0)`、`p3=(1,1)`，控制点即 CSS 的 `cubic-bezier(x1,y1,x2,y2)`）。
///
/// 不用 `gpui_kit::base::animation::cubic_bezier`：它把输出夹到 `[0, 1]`（注释里写着
/// 「GPUI asserts easing deltas stay within [0, 1]」），回弹需要的过冲会被它抹平。
/// 这里解 `x(s) = t` 后直接取 `y(s)`，不夹取，因此可以过冲（`y > 1`）与回退。
fn cubic_bezier_ease((x1, y1, x2, y2): (f32, f32, f32, f32)) -> impl Fn(f32) -> f32 + 'static {
    // 多项式形式：x(s) = ((ax·s + bx)·s + cx)·s，y(s) 同理。
    let (cx, cy) = (3. * x1, 3. * y1);
    let (bx, by) = (3. * (x2 - x1) - cx, 3. * (y2 - y1) - cy);
    let (ax, ay) = (1. - cx - bx, 1. - cy - by);
    let sample_x = move |s: f32| ((ax * s + bx) * s + cx) * s;
    let sample_y = move |s: f32| ((ay * s + by) * s + cy) * s;
    let slope_x = move |s: f32| (3. * ax * s + 2. * bx) * s + cx;

    move |t: f32| {
        // `t` 是时间轴上的进度，先解出曲线参数 `s` 再取 `y`，否则曲线会比 CSS 慢得多。
        let t = t.clamp(0., 1.);
        let mut s = t;
        for _ in 0..8 {
            let error = sample_x(s) - t;
            if error.abs() < 1e-6 {
                break;
            }
            let slope = slope_x(s);
            if slope.abs() < 1e-6 {
                break;
            }
            s = (s - error / slope).clamp(0., 1.);
        }
        // 牛顿法不收敛时用二分兜底。
        let (mut low, mut high) = (0., 1.);
        for _ in 0..24 {
            let x = sample_x(s);
            if (x - t).abs() < 1e-6 {
                break;
            }
            if x < t {
                low = s;
            } else {
                high = s;
            }
            s = (low + high) / 2.;
        }
        sample_y(s)
    }
}

/// 面板动效的一个通道：延迟、时长与缓动。
///
/// 缓动是单位三次贝塞尔（见 [`cubic_bezier_ease`]），`None` 表示线性。
/// 原版的 `AniEase*` 是 cos 多项式而非贝塞尔，上面的控制点由它拟合而来
/// （解 `x(s)=t` 再取 `y(s)`，因此拟合在参数域上做）。
#[derive(Clone, Copy)]
struct Channel {
    delay_ms: u64,
    duration_ms: u64,
    bezier: Option<(f32, f32, f32, f32)>,
}

impl Channel {
    /// 本通道在动画总进度下的进度（含各自的延迟与缓动）。
    fn progress(self, total: Duration, progress: f32) -> f32 {
        let total = total.as_millis() as f32;
        let start = self.delay_ms as f32 / total;
        let span = self.duration_ms as f32 / total;
        let linear = ((progress - start) / span).clamp(0., 1.);
        match self.bezier {
            Some(points) => cubic_bezier_ease(points)(linear),
            None => linear,
        }
    }
}

/// 面板动效的两个并行通道：上下位移与淡入淡出。
///
/// 原版还有第三条——绕左侧水平黄金分割点 `(0.382, 0.5)`（原版是左中点 `(0, 0.5)`）的旋转，
/// 进场 `-4° → 0°`、出场 `0° → +6°`，斜上斜下的观感大半来自它。GPUI 的元素样式没有旋转变换
/// （`Style` / `StyleRefinement` 里没有 transform 字段，`paint_quad` / `paint_glyph` 都不收
/// 变换矩阵，全仓只有 SVG 与 `Path` 能转），整棵子树转不了，因此这里不做旋转，也不做水平
/// 位移的替代——面板只是上下移动并淡入淡出。
fn channels(phase: Phase) -> [Channel; 2] {
    match phase {
        Phase::Enter => [
            // 淡入。
            Channel {
                delay_ms: 60,
                duration_ms: 120,
                bezier: None,
            },
            // 上移，末端回弹。
            Channel {
                delay_ms: 60,
                duration_ms: 300,
                bezier: Some(EASE_BACK),
            },
        ],
        Phase::Leave => [
            // 淡出。
            Channel {
                delay_ms: 20,
                duration_ms: 80,
                bezier: None,
            },
            // 下移，平滑结束。
            Channel {
                delay_ms: 0,
                duration_ms: 150,
                bezier: Some(EASE_OUT_CUBIC),
            },
        ],
    }
}

/// 面板动效：两个通道由一个动画元素并行驱动，总时长取两者最长。
///
/// 不能用 `with_animations`：那是**串行**链（前一个跑完才把索引加一、再启动下一个），
/// 串起来会变成「先原地淡入、再跳回起点滑入」。所以动画本身线性，进度在
/// [`apply_panel_animation`] 里按通道各自的延迟、时长与缓动拆分。
pub(crate) fn panel_animation(phase: Phase) -> Animation {
    Animation::new(Duration::from_millis(total_ms(phase)))
}

/// 两个通道里最晚的结束时间。
fn total_ms(phase: Phase) -> u64 {
    channels(phase)
        .iter()
        .map(|channel| channel.delay_ms + channel.duration_ms)
        .max()
        .unwrap_or(0)
}

/// 两个通道在给定线性进度下的取值：`(纵向位移, 不透明度)`。
///
/// 抽成纯函数是为了单独验证曲线形状（起点、终点、回弹幅度），不必依赖真实时钟。
fn panel_motion(phase: Phase, progress: f32) -> (Pixels, f32) {
    let total = Duration::from_millis(total_ms(phase));
    let [fade, offset] = channels(phase).map(|channel| channel.progress(total, progress));
    match phase {
        Phase::Enter => (ENTER_OFFSET * (1. - offset), fade),
        Phase::Leave => (LEAVE_OFFSET * offset, 1. - fade),
    }
}

/// 按线性进度套用两个通道：进场从下方淡入上移（带回弹），出场向下淡出。
pub(crate) fn apply_panel_animation<E: Styled>(element: E, phase: Phase, progress: f32) -> E {
    let (y, opacity) = panel_motion(phase, progress);
    element.opacity(opacity).top(y)
}

/// 面板骨架：标题、分割线、内容槽与按钮行。
///
/// `available` 是面板可占的最大尺寸（视口扣掉窗口边框与两侧外边距）；内容超出时由内容槽滚动。
pub(crate) fn panel(
    cx: &App,
    title: SharedString,
    theme: DialogTheme,
    content: AnyElement,
    buttons: Vec<AnyElement>,
    available: Size<Pixels>,
) -> impl IntoElement {
    let palette = theme::palette(cx);
    let title_color = if theme.uses_red() {
        palette.red_light
    } else {
        palette.color_level(2)
    };

    v_flex()
        .id("dialog-panel")
        .relative()
        .min_w(MIN_WIDTH)
        .max_w(available.width)
        .max_h(available.height)
        .min_h_0()
        .bg(palette.background)
        .rounded(RADIUS)
        .shadow(vec![box_shadow(
            px(0.),
            px(2.),
            px(20.),
            px(0.),
            SHADOW_COLOR.into(),
        )])
        .pt(PADDING_TOP)
        .pb(PADDING_BOTTOM)
        .pl(PADDING_LEFT)
        .pr(PADDING_RIGHT)
        .child(
            div()
                .id("dialog-title")
                .flex_shrink_0()
                .ml(TITLE_LEFT)
                .mt(px(-1.))
                .mr(TITLE_RIGHT)
                .mb(TITLE_BOTTOM)
                .text_size(TITLE_SIZE)
                .text_color(title_color)
                .aria_label(title.clone())
                .child(title)
                .test_support(),
        )
        .child(
            div()
                .flex_shrink_0()
                .h(DIVIDER_HEIGHT)
                .w_full()
                .bg(title_color),
        )
        // 内容槽：高度自适应内容，被面板的最大高度夹住时才滚动（gpui 原生 overflow，
        // 组件版 Scrollable 用 `size_full()`，在自适应高度的面板里会塌成 0 高）。
        .child(
            div()
                .id("dialog-content")
                .mt(DIVIDER_GAP)
                .min_h_0()
                .overflow_y_scroll()
                .pt(CONTENT_TOP)
                .pb(CONTENT_BOTTOM)
                .pl(CONTENT_LEFT)
                .pr(CONTENT_RIGHT)
                .child(content)
                .test_support(),
        )
        .child(
            h_flex_row()
                .flex_shrink_0()
                .mt(CONTENT_GAP)
                .justify_end()
                .gap(BUTTON_GAP)
                .pl(BUTTON_LEFT)
                .pr(BUTTON_RIGHT)
                .children(buttons),
        )
        .test_support()
}

/// 内容槽里的正文段落（15px / 行高 18px，正文色）。
pub(crate) fn caption(cx: &App, text: SharedString) -> Div {
    div()
        .text_size(CAPTION_SIZE)
        .line_height(CAPTION_LINE_HEIGHT)
        .text_color(theme::palette(cx).gray_level(1))
        .child(text)
}

/// 按钮行里的一行：不撑高、纵向居中。
fn h_flex_row() -> Div {
    div().flex().flex_row().items_center()
}

#[cfg(test)]
mod tests {
    // 逐个导入：`use super::*` 会把模块里的 `gpui_kit::*` 一起带进来，
    // 那会遮蔽内置的 `#[test]`（GPUI 也导出了一个同名宏）。
    use super::{ENTER_OFFSET, LEAVE_OFFSET, Phase, panel_motion};

    /// 采样位移通道，返回纵向最小值（回弹时会小于终点值 0）。
    fn min_offset(phase: Phase, steps: usize) -> f32 {
        (0..=steps)
            .map(|step| f32::from(panel_motion(phase, step as f32 / steps as f32).0))
            .fold(f32::INFINITY, f32::min)
    }

    /// 进场：起点在下方且透明，终点回到正中且不透明。
    #[test]
    fn enter_starts_offset_and_ends_settled() {
        let (y, opacity) = panel_motion(Phase::Enter, 0.);
        assert_eq!(y, ENTER_OFFSET);
        assert_eq!(opacity, 0.);

        let (y, opacity) = panel_motion(Phase::Enter, 1.);
        assert!(f32::from(y).abs() < 0.01, "终点应回到 0：{y:?}");
        assert_eq!(opacity, 1.);
    }

    /// 出场：起点在正中且不透明，终点掉到下方且透明。
    #[test]
    fn leave_ends_offset_and_transparent() {
        let (y, opacity) = panel_motion(Phase::Leave, 0.);
        assert!(f32::from(y).abs() < 0.01, "起点应在 0：{y:?}");
        assert_eq!(opacity, 1.);

        let (y, opacity) = panel_motion(Phase::Leave, 1.);
        // 贝塞尔解在终点是数值解，留一点浮点余量。
        assert!((f32::from(y) - f32::from(LEAVE_OFFSET)).abs() < 0.01);
        assert!(opacity < 0.001);
    }

    /// 回弹：位移要冲过终点再回正（`1-(1-t)²·cos(1.5πt)` 的观感）。
    #[test]
    fn enter_overshoots_the_translation() {
        let min = min_offset(Phase::Enter, 200);
        // 原版公式 `40·(1-1.177)` 给的是 -7.1px，拟合后的贝塞尔实测 -7.2px。
        assert!(min < -6.5, "位移应过冲到终点之上（实测约 7.2px）：{min}");
    }

    /// 出场没有回弹：位移单调靠近终点。
    #[test]
    fn leave_is_monotonic() {
        let mut previous = 0.;
        for step in 0..=200 {
            let y = f32::from(panel_motion(Phase::Leave, step as f32 / 200.).0);
            assert!(y >= previous - 0.01, "出场位移不得回退：{y}");
            previous = y;
        }
        assert!(min_offset(Phase::Leave, 200) >= -0.01);
    }

    /// 淡入淡出是线性的，且按各自的延迟开始。
    #[test]
    fn opacity_is_linear_and_delayed() {
        let (_, enter) = panel_motion(Phase::Enter, 0.);
        assert_eq!(enter, 0.);
        // 进场总时长 360ms、淡入延迟 60ms：总进度 1/6 之前保持全透明。
        let (_, before) = panel_motion(Phase::Enter, 1. / 6. - 0.01);
        assert_eq!(before, 0.);
        let (_, after) = panel_motion(Phase::Enter, 1. / 6. + 0.01);
        assert!(after > 0. && after < 0.2, "延迟后应开始淡入：{after}");
    }
}
