//! 构造级别过滤规范。
//!
//! 规范字符串的语法同 env_logger，例如 `warn, Download=debug`。每个键与记录的 `target`
//! 比较，前缀命中即生效。
//!
//! flexi_logger 按 [`log::Record::target`] 过滤，不按模块路径：`FlexiLogger::enabled` 调用
//! `primary_enabled(level, metadata.target())`。只有调用点未指定 `target:` 时，模块路径才会
//! 成为 `target`。

use std::env;

use flexi_logger::LogSpecification;

use crate::ENV_SPEC;
use crate::error::Error;
use crate::options::Options;

/// 环境变量 [`ENV_SPEC`] 优先，否则由 [`Options`] 构造。
pub(crate) fn resolve(options: &Options) -> Result<LogSpecification, Error> {
    match env::var_os(ENV_SPEC) {
        Some(raw) => parse(&raw.to_string_lossy()),
        None => Ok(from_options(options)),
    }
}

/// `Options.level` 作默认级别，`Options.targets` 覆盖到具体分类。
pub(crate) fn from_options(options: &Options) -> LogSpecification {
    let mut builder = LogSpecification::builder();
    builder.default(options.level);
    for (target, level) in &options.targets {
        // flexi_logger 的方法名是 `module`，实际匹配的是记录的 `target`。
        builder.module(target.as_str(), *level);
    }
    builder.build()
}

/// 解析规范字符串。
pub(crate) fn parse(raw: &str) -> Result<LogSpecification, Error> {
    LogSpecification::parse(raw).map_err(|error| Error::Spec(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::{Level, LevelFilter};

    fn options(level: LevelFilter, targets: &[(&str, LevelFilter)]) -> Options {
        Options {
            level,
            targets: targets
                .iter()
                .map(|(target, level)| ((*target).to_owned(), *level))
                .collect(),
            ..Options::default()
        }
    }

    #[test]
    fn default_level_applies_to_unlisted_targets() {
        let spec = from_options(&options(LevelFilter::Warn, &[]));
        assert!(!spec.enabled(Level::Info, "Setup"));
        assert!(spec.enabled(Level::Warn, "Setup"));
    }

    #[test]
    fn target_override_wins() {
        let spec = from_options(&options(
            LevelFilter::Warn,
            &[("Download", LevelFilter::Debug)],
        ));
        assert!(spec.enabled(Level::Debug, "Download"));
        assert!(!spec.enabled(Level::Debug, "Setup"));
    }

    #[test]
    fn keys_match_by_prefix() {
        let spec = from_options(&options(LevelFilter::Warn, &[("Down", LevelFilter::Debug)]));
        assert!(spec.enabled(Level::Debug, "Download"));
    }

    #[test]
    fn keys_are_case_sensitive() {
        let spec = from_options(&options(
            LevelFilter::Warn,
            &[("download", LevelFilter::Debug)],
        ));
        assert!(!spec.enabled(Level::Debug, "Download"));
    }

    #[test]
    fn rejects_garbage() {
        assert!(matches!(parse("nonsense=verbose"), Err(Error::Spec(_))));
        assert!(parse("info, pcl=debug").is_ok());
    }
}
