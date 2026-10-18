//! 实例数据：以游戏文件夹 + 实例名为唯一键（[`InstanceId`]）。
//!
//! 身份类型直接使用 `lite-config` 的 `InstanceKey`：路径规范化只有那一份实现，
//! 将来按实例读写配置时键可以直接复用。
//!
//! 表用 [`DashMap`]：实例扫描、配置读写将来会在后台线程上跑，与界面线程并发访问同一张表，
//! 分片锁让两边不必互相等整张表。
//!
//! 注意：`DashMap` 的 `Ref` / `RefMut` 会占住分片锁，**不要跨 GPUI 调用持有**——实体更新可能
//! 反过来再取同一个分片而自锁。取值、写值都在一条语句里做完。

use std::path::Path;
use std::sync::LazyLock;

use dashmap::DashMap;
use dashmap::mapref::one::{Ref, RefMut};
use gpui_kit::component::IndexPath;
use gpui_kit::component::input::InputState;
use gpui_kit::component::select::SelectState;
use gpui_kit::component::slider::SliderState;
use gpui_kit::*;

use super::InstanceGroup;
use super::state::{SAMPLE_INSTANCES, SAMPLE_JVM_ARGS, sample_difficulties};

/// 实例的唯一标识：游戏文件夹 + 实例名。
///
/// 列表排序、过滤、增删实例都会让下标漂移，因此实例数据一律按身份索引。
/// 底层是 `lite-config` 的 `InstanceKey`，这里保留页面惯用的
/// `InstanceId` 名字；路径规范化与配置定位都由那个 crate 负责。
pub(crate) use lite_config::InstanceKey as InstanceId;

/// 单个实例的界面数据。
///
/// 控件状态即值的存放处，不再单独保存：需要值时从控件读取（例如文本框的 `value()`），
/// 将来接数据层时在这张表上读写即可。
///
/// 可以克隆：渲染时先克隆再使用，避免把 `DashMap` 的分片锁带进 GPUI 调用。
#[derive(Clone)]
pub(super) struct InstanceData {
    /// 内存分配模式：0 = 继承全局，1 = 独立设置。
    pub(super) memory_mode: usize,
    pub(super) max_memory: Entity<SliderState>,
    pub(super) initial_memory: Entity<SliderState>,
    /// 高级设置：JVM 参数 / 游戏参数 / 自定义信息 / 前置 Classpath。
    pub(super) jvm_args: Entity<InputState>,
    pub(super) game_args: Entity<InputState>,
    pub(super) custom_info: Entity<InputState>,
    pub(super) classpath: Entity<InputState>,
    /// 存档设置：允许作弊、锁定难度、难度档位，以及存档列表里选中的那一项。
    pub(super) allow_commands: bool,
    pub(super) lock_difficulty: bool,
    pub(super) difficulty: Entity<SelectState<Vec<SharedString>>>,
    pub(super) selected_save: Option<usize>,
}

impl InstanceData {
    fn new(window: &mut Window, cx: &mut Context<InstanceGroup>) -> Self {
        Self {
            memory_mode: 0,
            max_memory: cx.new(|_| {
                SliderState::new()
                    .min(0.)
                    .max(64.)
                    .step(1.)
                    .default_value(15_f32)
            }),
            initial_memory: cx.new(|_| {
                SliderState::new()
                    .min(0.)
                    .max(64.)
                    .step(1.)
                    .default_value(0_f32)
            }),
            jvm_args: text_input(window, cx, SAMPLE_JVM_ARGS),
            game_args: text_input(window, cx, ""),
            custom_info: text_input(window, cx, ""),
            classpath: text_input(window, cx, ""),
            allow_commands: false,
            lock_difficulty: true,
            // 难度下拉：4 个难度档位，初始选中「普通」（示例数据）。
            difficulty: cx.new(|cx| {
                SelectState::new(sample_difficulties(), Some(IndexPath::new(2)), window, cx)
            }),
            selected_save: None,
        }
    }
}

/// 新建一个带初始值的文本框状态。
fn text_input(
    window: &mut Window,
    cx: &mut Context<InstanceGroup>,
    value: &str,
) -> Entity<InputState> {
    cx.new(|cx| InputState::new(window, cx).default_value(value.to_owned()))
}

/// 实例数据表。
pub(super) struct InstanceStore(DashMap<InstanceId, InstanceData>);

impl InstanceStore {
    pub(super) fn new() -> Self {
        Self(DashMap::new())
    }

    /// 读取实例数据；键不存在时返回 `None`。
    pub(super) fn get(&self, id: &InstanceId) -> Option<Ref<'_, InstanceId, InstanceData>> {
        self.0.get(id)
    }

    /// 取实例数据；不存在时按默认值创建。
    pub(super) fn get_or_create(
        &self,
        id: &InstanceId,
        window: &mut Window,
        cx: &mut Context<InstanceGroup>,
    ) -> RefMut<'_, InstanceId, InstanceData> {
        self.0
            .entry(id.clone())
            .or_insert_with(|| InstanceData::new(window, cx))
    }
}

/// 示例实例的 id，下标与 [`SAMPLE_INSTANCES`] 一致。
///
/// 路径规范化要分配，用 `LazyLock` 计算一次，不放进每帧的渲染。
pub(super) fn sample_instance_ids() -> &'static [InstanceId] {
    static IDS: LazyLock<Vec<InstanceId>> = LazyLock::new(|| {
        SAMPLE_INSTANCES
            .iter()
            .map(|instance| {
                InstanceId::from_version_dir(Path::new(instance.path))
                    .expect("示例实例路径必须是 .minecraft/versions/<name> 布局")
            })
            .collect()
    });
    &IDS
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    // 只导入用到的类型：`use super::*` 会把 gpui-kit 的 `test` 宏带进来，盖掉 `#[test]`。
    use super::InstanceId;
    use super::SAMPLE_INSTANCES;

    /// 示例数据必须符合标准实例目录布局，否则 `sample_instance_ids` 会在启动时 panic。
    #[test]
    fn sample_paths_use_the_standard_layout() {
        for (index, instance) in SAMPLE_INSTANCES.iter().enumerate() {
            assert!(
                InstanceId::from_version_dir(Path::new(instance.path)).is_some(),
                "示例实例 #{index} 的路径不是 .minecraft/versions/<name> 布局：{}",
                instance.path
            );
        }
    }
}
