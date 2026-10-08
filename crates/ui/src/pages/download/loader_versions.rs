//! 加载器版本选择弹窗的内容：一列单选框，选中项由本视图自己保存。
//!
//! 弹窗只负责显示与取值：结果在按下确定时从本实体读取（见 [`super::install`] 的
//! `pick_loader_version`）。

use gpui_kit::base::v_flex;
use gpui_kit::*;

use crate::components::AppRadio;

/// 某个加载器可选的版本列表。
pub(super) struct LoaderVersions {
    /// 可选版本的文案，次序与调用方给的清单一致。
    versions: &'static [&'static str],
    /// 选中项在 `versions` 中的下标。
    pub(super) selected: Option<usize>,
}

impl LoaderVersions {
    /// `initial` 是该加载器当前已选的版本下标，弹窗因此每次打开都停在现状上。
    pub(super) fn new(versions: &'static [&'static str], initial: Option<usize>) -> Self {
        Self {
            versions,
            selected: initial,
        }
    }
}

impl Render for LoaderVersions {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut column = v_flex().gap_2();
        for (index, version) in self.versions.iter().enumerate() {
            column = column.child(
                AppRadio::new(SharedString::from(format!("loader-version-{index}")))
                    .label(SharedString::from(*version))
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
