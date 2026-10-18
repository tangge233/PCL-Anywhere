//! `MultiSelector` 实现。

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, PoisonError, RwLock};

use arc_swap::ArcSwap;

use crate::config::{Config, Strategy, normalized_alpha, normalized_temperature};
use crate::entry::Entry;
use crate::error::RegisterError;
use crate::rng::CounterRng;

/// 通用多源选择器。
///
/// 同时支持 P2C 与 Softmax SWRR 两种策略；`select` 使用配置里的默认策略，
/// `select_with` 可以显式指定。
///
/// `T` 是每次上报的数据结构；构造时传入的 `scorer` 负责把它换算成 `0..=255`
/// 的分数。一个选择器固定一种 `T`。
pub struct MultiSelector<T> {
    alpha: f32,
    default: Strategy,
    scorer: Box<dyn Fn(&T) -> u8 + Send + Sync>,
    rng: CounterRng,
    swrr_lock: Mutex<()>,
    swrr_temperature: f32,
    sources: ArcSwap<Vec<Arc<Entry>>>,
    by_id: RwLock<BTreeMap<String, Arc<Entry>>>,
}

impl<T> MultiSelector<T> {
    /// 创建选择器。
    ///
    /// `scorer` 返回 `0..=255` 的分数，越小表示源越好；连接失败由调用方打高分
    /// （通常直接用 `255`）。
    pub fn new(config: Config, scorer: impl Fn(&T) -> u8 + Send + Sync + 'static) -> Self {
        Self {
            alpha: normalized_alpha(config.ewma_alpha),
            default: config.default,
            scorer: Box::new(scorer),
            rng: CounterRng::new(config.p2c.seed),
            swrr_lock: Mutex::new(()),
            swrr_temperature: normalized_temperature(config.swrr.temperature),
            sources: ArcSwap::from_pointee(Vec::new()),
            by_id: RwLock::new(BTreeMap::new()),
        }
    }

    /// 注册一个源。
    ///
    /// 删除后重新注册同名 ID 会创建全新的分数状态；旧的在途 `report` 可能落到
    /// 新状态上。
    ///
    /// # Errors
    ///
    /// 相同 ID 已注册时返回 [`RegisterError::AlreadyRegistered`]。
    pub fn register(&self, id: impl Into<String>) -> Result<(), RegisterError> {
        let id = id.into();

        let mut by_id = self.by_id.write().unwrap_or_else(PoisonError::into_inner);

        if by_id.contains_key(&id) {
            return Err(RegisterError::AlreadyRegistered { id });
        }

        by_id.insert(id.clone(), Arc::new(Entry::new(id)));
        self.publish(&by_id);

        Ok(())
    }

    /// 移除一个源；存在并移除成功返回 `true`。
    pub fn remove(&self, id: &str) -> bool {
        let mut by_id = self.by_id.write().unwrap_or_else(PoisonError::into_inner);
        let removed = by_id.remove(id).is_some();

        if removed {
            self.publish(&by_id);
        }

        removed
    }

    /// 当前已注册的源数量。
    pub fn len(&self) -> usize {
        self.sources.load().len()
    }

    /// 是否没有任何源。
    pub fn is_empty(&self) -> bool {
        self.sources.load().is_empty()
    }

    /// 使用配置里的默认策略选择一个源。
    #[must_use]
    pub fn select(&self) -> Option<String> {
        self.select_with(self.default)
    }

    /// 使用指定策略选择一个源。
    #[must_use]
    pub fn select_with(&self, strategy: Strategy) -> Option<String> {
        match strategy {
            Strategy::P2c => self.p2_select(),
            Strategy::SoftmaxSwrr => self.swrr_select(),
        }
    }

    /// 用 P2C 选择一个源，返回其 ID；没有源时返回 `None`。
    ///
    /// 选择基于当前快照：返回后该 ID 可能已被并发 `remove`。
    #[must_use]
    pub fn p2_select(&self) -> Option<String> {
        let sources = self.sources.load();
        let len = sources.len();

        match len {
            0 => None,
            1 => Some(sources[0].id().to_owned()),
            _ => {
                let (first_index, second_index, tie_break) = self.rng.pick_pair(len);
                let first = &sources[first_index];
                let second = &sources[second_index];

                let first_score = first.score();
                let second_score = second.score();

                let chosen = if first_score < second_score {
                    first
                } else if second_score < first_score {
                    second
                } else if tie_break {
                    first
                } else {
                    second
                };

                Some(chosen.id().to_owned())
            }
        }
    }

    /// 用 Softmax 权重 + Smooth Weighted Round-Robin 选择一个源。
    ///
    /// 权重为 `exp(-(score - min_score) / temperature)`，最低分源的权重为 1。
    /// SWRR 的 current weight 更新在内部串行化；没有源时返回 `None`。
    ///
    /// 选择基于当前快照：返回后该 ID 可能已被并发 `remove`。
    #[must_use]
    pub fn swrr_select(&self) -> Option<String> {
        let _guard = self
            .swrr_lock
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let sources = self.sources.load();
        let len = sources.len();

        if len == 0 {
            return None;
        }
        if len == 1 {
            return Some(sources[0].id().to_owned());
        }

        let min_score = sources
            .iter()
            .map(|entry| entry.score())
            .fold(f32::INFINITY, f32::min);

        let mut total = 0.0f32;
        let mut best_index = 0usize;
        let mut best_current = f32::NEG_INFINITY;

        for (index, entry) in sources.iter().enumerate() {
            let weight = ((min_score - entry.score()) / self.swrr_temperature).exp();
            let current = entry.swrr_current() + weight;
            entry.set_swrr_current(current);
            total += weight;

            if current > best_current {
                best_current = current;
                best_index = index;
            }
        }

        let chosen = &sources[best_index];
        chosen.set_swrr_current(chosen.swrr_current() - total);

        Some(chosen.id().to_owned())
    }

    /// 用 `scorer` 计算 `data` 的分数并更新 `id` 对应源。
    ///
    /// `id` 不存在时返回 `false`。
    ///
    /// # Panics
    ///
    /// `scorer` 是调用方提供的闭包；如果它 panic，本方法会 panic。
    #[allow(clippy::needless_pass_by_value)] // 对外 API 按值接收 T；scorer 只借用。
    pub fn report(&self, id: &str, data: T) -> bool {
        let entry = {
            let by_id = self.by_id.read().unwrap_or_else(PoisonError::into_inner);

            match by_id.get(id) {
                Some(entry) => Arc::clone(entry),
                None => return false,
            }
        };

        let score = (self.scorer)(&data);
        entry.update(score, self.alpha);
        true
    }

    fn publish(&self, by_id: &BTreeMap<String, Arc<Entry>>) {
        let snapshot = by_id.values().cloned().collect();
        self.sources.store(Arc::new(snapshot));
    }
}
