//! 无锁计数器随机数：splitmix64。
//!
//! P2C 只需要均匀选两个下标，不需要密码学随机；这里用原子计数器避免给选择路径加锁。

use std::sync::atomic::{AtomicU64, Ordering};

/// splitmix64 的黄金比例增量。
const GOLDEN_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

/// 线程安全、可播种的随机数生成器。
pub(crate) struct CounterRng {
    state: AtomicU64,
}

impl CounterRng {
    pub(crate) fn new(seed: u64) -> Self {
        Self {
            state: AtomicU64::new(seed),
        }
    }

    pub(crate) fn next_u64(&self) -> u64 {
        let mut z = self.state.fetch_add(GOLDEN_GAMMA, Ordering::Relaxed);

        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);

        z ^ (z >> 31)
    }

    /// 返回两个不同的下标，以及一个用于平局的随机位。
    #[allow(clippy::cast_possible_truncation)] // 取模结果小于 len，usize 一定装得下。
    pub(crate) fn pick_pair(&self, len: usize) -> (usize, usize, bool) {
        debug_assert!(len >= 2);

        let first = (self.next_u64() % len as u64) as usize;

        let mut second = (self.next_u64() % (len as u64 - 1)) as usize;
        if second >= first {
            second += 1;
        }

        (first, second, (self.next_u64() & 1) == 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pair_is_distinct() {
        let rng = CounterRng::new(42);

        for len in 2..100 {
            for _ in 0..100 {
                let (first, second, _) = rng.pick_pair(len);
                assert!(first < len);
                assert!(second < len);
                assert_ne!(first, second);
            }
        }
    }

    #[test]
    fn same_seed_same_sequence() {
        let a = CounterRng::new(7);
        let b = CounterRng::new(7);

        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }
}
