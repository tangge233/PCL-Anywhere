//! [`crate::init`] 的参数。

use std::path::PathBuf;

use flexi_logger::Duplicate;
use log::LevelFilter;

/// 日志文件的轮转方式。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rotation {
    /// 当前文件超过 `max_bytes` 时开始新文件。同一次运行的分片共享文件名里的会话标记。
    BySize {
        /// 单个文件的字节数上限。
        max_bytes: u64,
    },
    /// 每次进程启动一个文件，文件名含启动时间。
    BySession,
    /// 每天一个文件。同一天内重新启动会追加到同一份。
    ByDate,
}

/// 除文件外同时写入 stderr 的最低级别。
///
/// 输出的是启动器自身的运行过程，终端通常不需要看，默认取 [`Console::Warn`]。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Console {
    /// 不写入 stderr。
    Off,
    /// error 及以上。
    Error,
    /// warn 及以上。
    Warn,
    /// info 及以上。
    Info,
    /// debug 及以上。
    Debug,
    /// 全部级别。
    Trace,
}

impl Console {
    /// 映射到 flexi_logger 的 stderr 复制级别。
    pub(crate) fn to_duplicate(self) -> Duplicate {
        match self {
            Console::Off => Duplicate::None,
            Console::Error => Duplicate::Error,
            Console::Warn => Duplicate::Warn,
            Console::Info => Duplicate::Info,
            Console::Debug => Duplicate::Debug,
            Console::Trace => Duplicate::All,
        }
    }
}

/// 安装参数，只在 [`crate::init`] 时读取。
///
/// 安装后修改级别用 [`crate::set_level`] 或 [`crate::set_spec`]。
#[derive(Clone, Debug)]
pub struct Options {
    /// 日志目录；`None` 使用 [`crate::log_dir`]。
    pub dir: Option<PathBuf>,
    /// 默认级别，适用于未在 [`Options::targets`] 中列出的分类。
    pub level: LevelFilter,
    /// 按分类覆盖级别。键与记录的 `target` 比较，前缀命中即生效：`("Download",
    /// LevelFilter::Debug)` 让 `target: "Download"` 的记录输出 debug。
    pub targets: Vec<(String, LevelFilter)>,
    /// 轮转方式。
    pub rotation: Rotation,
    /// 目录中最多保留的日志文件数，至少 1。
    pub keep: usize,
    /// 除文件外同时写入 stderr 的最低级别。
    pub console: Console,
    /// 缓冲写入。减少写调用次数，但进程异常退出时最后一批可能丢失。
    pub buffered: bool,
    /// 在日志目录中维护 `latest.log` 软链（仅 Unix）。
    pub latest_link: bool,
    /// 时间戳使用 UTC；默认本地时间。
    pub utc: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            dir: None,
            // 默认不开 DEBUG：gpui / wgpu / cosmic-text 在 DEBUG 级别逐帧记录，实测 20 秒
            // 即写出 8 MB。需要某个分类的细节时用 `Options::targets` 或 `PCL_LOG` 单独放开。
            level: if cfg!(debug_assertions) {
                LevelFilter::Info
            } else {
                LevelFilter::Warn
            },
            targets: Vec::new(),
            rotation: Rotation::BySession,
            keep: 20,
            console: Console::Warn,
            buffered: false,
            latest_link: true,
            utc: false,
        }
    }
}
