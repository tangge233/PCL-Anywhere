//! 弹窗句柄：内容与调用方都可以拿它作答或关闭。

use std::rc::Rc;

use gpui_kit::*;

use super::host::DialogHost;
use crate::components::Callback;

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
    /// 由请求模块在 `open` 时构造。
    pub(super) fn new(
        id: u64,
        host: WeakEntity<DialogHost>,
        on_result: Option<Callback<R>>,
    ) -> Self {
        Self {
            inner: Rc::new(HandleInner {
                id,
                host,
                on_result,
            }),
        }
    }

    /// 给每个按钮克隆一份作答回调。
    pub(super) fn result_handler(&self) -> Option<Callback<R>> {
        self.inner.on_result.clone()
    }

    /// 没有图层时拿到的句柄：任何操作都是空操作。
    pub(super) fn detached() -> Self {
        Self::new(0, WeakEntity::new_invalid(), None)
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
