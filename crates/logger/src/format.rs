//! 日志行格式。

use std::io::{self, Write};

use flexi_logger::DeferredNow;
use log::Record;

/// 时间戳格式：本地时间、毫秒精度、含日期。
const TIMESTAMP: &str = "%Y-%m-%d %H:%M:%S%.3f";

/// 行格式：`[时间] [级别] [分类] 消息`，例如
/// `[2026-10-04 12:00:00.123] [INFO ] [Setup] 已初始化启动页设置`。
///
/// 分类取自记录的 `target`；级别左对齐补齐到 5 列。
pub(crate) fn line(
    w: &mut dyn Write,
    now: &mut DeferredNow,
    record: &Record<'_>,
) -> io::Result<()> {
    write!(
        w,
        "[{}] [{:<5}] [{}] {}",
        now.format(TIMESTAMP),
        record.level(),
        record.target(),
        record.args()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::Level;

    fn render(level: Level, target: &str, message: &str) -> String {
        let mut buffer = Vec::new();
        let mut now = DeferredNow::new();
        line(
            &mut buffer,
            &mut now,
            &Record::builder()
                .args(format_args!("{message}"))
                .level(level)
                .target(target)
                .build(),
        )
        .unwrap();
        String::from_utf8(buffer).unwrap()
    }

    #[test]
    fn renders_timestamp_level_target_message() {
        let text = render(Level::Info, "Setup", "已初始化启动页设置");
        let (stamp, rest) = text.split_once("] ").expect("时间戳后有分隔");
        assert!(stamp.starts_with('['), "{text}");
        // 24 字节，形如 `2026-10-04 12:00:00.123`。
        assert_eq!(stamp.len(), 24, "{text}");
        assert_eq!(rest, "[INFO ] [Setup] 已初始化启动页设置");
    }

    #[test]
    fn short_levels_are_padded() {
        assert!(render(Level::Warn, "T", "x").contains("[WARN ]"));
        assert!(render(Level::Error, "T", "x").contains("[ERROR]"));
        assert!(render(Level::Trace, "T", "x").contains("[TRACE]"));
    }
}
