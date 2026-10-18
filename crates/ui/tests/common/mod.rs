//! 界面集成测试的公共辅助。
//!
//! 每个 `tests/*.rs` 都是独立编译的测试目标，所以共享代码放在子目录里（子目录不会被当成测试目标）。

use std::time::Duration;

use gpui_kit::TestAppContext;

/// 等弹窗动画停稳：测试不验证动效，直接让动画落在终点帧；出场流程的关闭定时器
/// 由模拟时钟推进，避免真实时钟等待造成的慢测试与偶发抖动。
pub async fn settle(cx: &mut TestAppContext) {
    cx.update(|cx| cx.set_reduce_motion(true));
    cx.executor().advance_clock(Duration::from_millis(500));
    cx.executor().timer(Duration::from_millis(50)).await;
    cx.run_until_parked();
}
