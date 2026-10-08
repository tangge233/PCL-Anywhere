//! 界面集成测试的公共辅助。
//!
//! 每个 `tests/*.rs` 都是独立编译的测试目标，所以共享代码放在子目录里（子目录不会被当成测试目标）。

use std::time::Duration;

use gpui_kit::TestAppContext;

/// 等弹窗动画停稳：入场位移按真实时钟推进（不是模拟时钟），点击用的是上一帧的位置，
/// 动画没停就点会偏；出场则靠后台定时器推进关闭流程，模拟时钟也要往前推。
pub async fn settle(cx: &mut TestAppContext) {
    std::thread::sleep(Duration::from_millis(400));
    cx.executor().advance_clock(Duration::from_millis(500));
    cx.executor().timer(Duration::from_millis(50)).await;
    cx.run_until_parked();
}
