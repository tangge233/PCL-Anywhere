//! 单个源的分数状态。

use std::sync::atomic::{AtomicU32, Ordering};

/// 还没有上报过分数。`f32::from_bits(u32::MAX)` 是 NaN，不会和合法分数冲突。
const UNINIT: u32 = u32::MAX;

/// 一个源的分数槽。
pub(crate) struct Entry {
    id: String,
    score_bits: AtomicU32,
    swrr_current_bits: AtomicU32,
}

impl Entry {
    pub(crate) fn new(id: String) -> Self {
        Self {
            id,
            score_bits: AtomicU32::new(UNINIT),
            swrr_current_bits: AtomicU32::new(0.0f32.to_bits()),
        }
    }

    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    /// 选择路径读取分数；未上报过的源按 0.0 处理。
    pub(crate) fn score(&self) -> f32 {
        let bits = self.score_bits.load(Ordering::Relaxed);

        if bits == UNINIT {
            0.0
        } else {
            f32::from_bits(bits)
        }
    }

    /// 用本次 0..=255 的分数更新 EWMA。
    pub(crate) fn update(&self, value: u8, alpha: f32) {
        let value = f32::from(value);
        let mut old = self.score_bits.load(Ordering::Relaxed);

        loop {
            let next = if old == UNINIT {
                value
            } else {
                let next = alpha * value + (1.0 - alpha) * f32::from_bits(old);

                // 极端大值相加可能舍入到 infinity，回退到本次值。
                if next.is_finite() { next } else { value }
            };

            match self.score_bits.compare_exchange_weak(
                old,
                next.to_bits(),
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => return,
                Err(current) => old = current,
            }
        }
    }

    /// SWRR 的 current weight。调用方持有选择器的 SWRR 锁。
    pub(crate) fn swrr_current(&self) -> f32 {
        f32::from_bits(self.swrr_current_bits.load(Ordering::Relaxed))
    }

    pub(crate) fn set_swrr_current(&self, value: f32) {
        self.swrr_current_bits
            .store(value.to_bits(), Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uninitialized_score_is_zero() {
        let entry = Entry::new("a".to_owned());
        assert_eq!(entry.score(), 0.0);
    }

    #[test]
    fn first_update_sets_value() {
        let entry = Entry::new("a".to_owned());
        entry.update(10, 0.5);
        assert_eq!(entry.score(), 10.0);
    }

    #[test]
    fn update_applies_ewma() {
        let entry = Entry::new("a".to_owned());
        entry.update(10, 0.5);
        entry.update(20, 0.5);
        assert_eq!(entry.score(), 15.0);
    }

    #[test]
    fn max_score_is_stored() {
        let entry = Entry::new("a".to_owned());
        entry.update(255, 1.0);
        assert_eq!(entry.score(), 255.0);
    }
}
