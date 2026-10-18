//! 选择器配置。

use std::time::{SystemTime, UNIX_EPOCH};

/// EWMA 默认平滑系数。
pub(crate) const DEFAULT_ALPHA: f32 = 0.2;

/// Softmax 默认温度。
pub(crate) const DEFAULT_TEMPERATURE: f32 = 1.0;

/// 选择策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Strategy {
    /// Power of Two Choices。
    #[default]
    P2c,

    /// Softmax 权重 + Smooth Weighted Round-Robin。
    SoftmaxSwrr,
}

/// `MultiSelector` 配置。
///
/// 同时保存两套策略的参数，`default` 决定 `select()` 使用哪一套。
#[derive(Debug, Clone, Copy)]
pub struct Config {
    /// `select()` 默认使用的策略。
    pub default: Strategy,

    /// 分数 EWMA 平滑系数，范围 `0.0..=1.0`。
    ///
    /// - `1.0`：直接使用最新一次上报的分数；
    /// - `0.0`：第一次上报后不再变化；
    /// - 默认 `0.2`。
    ///
    /// 非有限值会回落到默认值；超出范围会夹取到 `0.0` 或 `1.0`。
    pub ewma_alpha: f32,

    /// P2C 策略配置。
    pub p2c: P2cConfig,

    /// Softmax SWRR 策略配置。
    pub swrr: SwrrConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default: Strategy::P2c,
            ewma_alpha: DEFAULT_ALPHA,
            p2c: P2cConfig::default(),
            swrr: SwrrConfig::default(),
        }
    }
}

/// P2C 策略配置。
#[derive(Debug, Clone, Copy)]
pub struct P2cConfig {
    /// 随机种子；固定它可以复现选择序列。
    pub seed: u64,
}

impl Default for P2cConfig {
    fn default() -> Self {
        Self {
            seed: default_seed(),
        }
    }
}

/// Softmax SWRR 策略配置。
#[derive(Debug, Clone, Copy)]
pub struct SwrrConfig {
    /// Softmax 温度。
    ///
    /// 越大权重越接近均匀；越小越偏向低分源。必须为正有限值；非正值或非有限值
    /// 会回落到 `1.0`，超出范围会夹取到 `1e-6..=1e6`。
    pub temperature: f32,
}

impl Default for SwrrConfig {
    fn default() -> Self {
        Self {
            temperature: DEFAULT_TEMPERATURE,
        }
    }
}

pub(crate) fn normalized_alpha(alpha: f32) -> f32 {
    if alpha.is_finite() {
        alpha.clamp(0.0, 1.0)
    } else {
        DEFAULT_ALPHA
    }
}

pub(crate) fn normalized_temperature(temperature: f32) -> f32 {
    if temperature.is_finite() && temperature > 0.0 {
        temperature.clamp(1e-6, 1e6)
    } else {
        DEFAULT_TEMPERATURE
    }
}

fn default_seed() -> u64 {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_nanos()).unwrap_or(u64::MAX));

    time ^ u64::from(std::process::id()).rotate_left(32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alpha_is_normalized() {
        assert_eq!(normalized_alpha(f32::NAN), DEFAULT_ALPHA);
        assert_eq!(normalized_alpha(f32::INFINITY), DEFAULT_ALPHA);
        assert_eq!(normalized_alpha(-1.0), 0.0);
        assert_eq!(normalized_alpha(2.0), 1.0);
        assert_eq!(normalized_alpha(0.5), 0.5);
    }

    #[test]
    fn temperature_is_normalized() {
        assert_eq!(normalized_temperature(f32::NAN), DEFAULT_TEMPERATURE);
        assert_eq!(normalized_temperature(0.0), DEFAULT_TEMPERATURE);
        assert_eq!(normalized_temperature(-1.0), DEFAULT_TEMPERATURE);
        assert_eq!(normalized_temperature(8.0), 8.0);
    }

    #[test]
    fn default_strategy_is_p2c() {
        assert_eq!(Config::default().default, Strategy::P2c);
    }
}
