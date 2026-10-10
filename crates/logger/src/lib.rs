//! 启动器进程的运行日志。
//!
//! 进程内一个全局 logger：启动时调用 [`init`] 一次，之后任意线程用 `log` 宏记录。gpui 与
//! gpui-kit 也通过 `log` 记录，它们的日志同样会被写入。
//!
//! # 保证
//!
//! * 进程内只有一个 logger；再次调用 [`init`] 返回 [`Error::AlreadyInitialized`]。
//! * 三种轮转方式使用同一个目录与同一个文件名前缀；`init` 会按 `keep` 删除多余文件。
//! * 默认不缓冲：宏返回时内容已写入内核，进程被 kill 不丢已记录的行。
//!
//! # 不保证
//!
//! * 两个进程共享同一目录时各自独立写入，清理可能删掉另一个进程正在写的文件。Linux 允许
//!   删除已打开的文件，被删一侧继续写入已解链的 inode。
//! * 磁盘写满、目录不可写等写失败由 flexi_logger 输出到 stderr，调用方拿不到。
//! * 不写崩溃报告，不收集子进程输出。
//!
//! # 用法
//!
//! ```
//! use logger::log;
//!
//! let dir = std::env::temp_dir().join("logger-doc-example");
//! let _ = std::fs::remove_dir_all(&dir);
//!
//! let _guard = logger::init(logger::Options {
//!     dir: Some(dir.clone()),
//!     console: logger::Console::Off,
//!     ..Default::default()
//! })
//! .expect("初始化日志");
//!
//! log::info!(target: "Setup", "已初始化启动页设置");
//! # drop(_guard);
//! # let _ = std::fs::remove_dir_all(&dir);
//! ```

#![warn(missing_docs)]

mod error;
mod format;
mod guard;
mod location;
mod options;
mod plan;
mod prune;
mod spec;

#[cfg(test)]
mod tests;

use std::fs;
use std::path::Path;
use std::sync::OnceLock;

use flexi_logger::{Logger, LoggerHandle, WriteMode};
use log::LevelFilter;

pub use error::Error;
pub use guard::LogGuard;
pub use location::{log_dir, log_dir_in};
pub use log;
pub use options::{Console, Options, Rotation};

use guard::Runtime;

/// 日志文件名的公共前缀；[`prune`] 据此识别本 crate 生成的文件。
const BASENAME: &str = "pcl-anywhere";
/// 指向当前日志文件的软链名（仅 Unix）。
const LATEST_LINK: &str = "latest.log";
/// 会话标记的时间格式。按大小轮转时写进文件名，用于区分不同次运行的分片。
const SESSION_FORMAT: &str = "%Y-%m-%d_%H-%M-%S";
/// 级别规范的环境变量名。设置后覆盖 [`Options`]，语法同 `RUST_LOG`。
const ENV_SPEC: &str = "PCL_LOG";

static INSTALLED: OnceLock<Runtime> = OnceLock::new();

/// 安装全局 logger。
///
/// `options.dir` 为 `None` 时使用 [`log_dir`]；目录不存在则创建。随后删除该目录中多余的
/// 日志文件，只保留最新的 [`Options::keep`] 个。
pub fn init(options: Options) -> Result<LogGuard, Error> {
    if INSTALLED.get().is_some() {
        return Err(Error::AlreadyInitialized);
    }

    let mut options = options;
    options.keep = options.keep.max(1);
    let dir = match &options.dir {
        Some(dir) => dir.clone(),
        None => location::log_dir()?,
    };
    fs::create_dir_all(&dir)?;
    prune::prune(&dir, options.keep)?;

    let (logger, handle) = build(&options, &dir)?;
    log::set_boxed_logger(logger).map_err(|_| Error::AlreadyInitialized)?;
    let guard = LogGuard::new(handle.clone());
    INSTALLED
        .set(Runtime::new(handle, options))
        .map_err(|_| Error::AlreadyInitialized)?;
    Ok(guard)
}

/// 修改默认级别；[`Options::targets`] 中按分类的覆盖保持不变。
pub fn set_level(level: LevelFilter) -> Result<(), Error> {
    INSTALLED
        .get()
        .ok_or(Error::NotInitialized)?
        .set_level(level);
    Ok(())
}

/// 用 env_logger 语法的规范整体替换级别配置。
///
/// 替换后 [`Options::targets`] 中的按分类设置不再生效；随后调用 [`set_level`] 会依据
/// [`Options`] 重建规范，使其恢复。
pub fn set_spec(spec: &str) -> Result<(), Error> {
    INSTALLED
        .get()
        .ok_or(Error::NotInitialized)?
        .set_spec(spec)?;
    Ok(())
}

/// 组装 logger，但不安装到全局，便于测试并排创建互不影响的实例。
fn build(options: &Options, dir: &Path) -> Result<(Box<dyn log::Log>, LoggerHandle), Error> {
    let session = chrono::Local::now().format(SESSION_FORMAT).to_string();
    let mut builder = Logger::with(spec::resolve(options)?)
        .log_to_file(plan::file_spec(options, dir, &session))
        .format(format::line)
        .duplicate_to_stderr(options.console.to_duplicate())
        .write_mode(if options.buffered {
            WriteMode::BufferAndFlush
        } else {
            WriteMode::Direct
        })
        // flexi_logger 写不出自身错误时会 panic。GUI 进程的 stderr 可能已关闭，不能因此
        // 终止进程。
        .panic_if_error_channel_is_broken(false);

    if let Some((criterion, naming, cleanup)) = plan::rotation(options) {
        builder = builder.rotate(criterion, naming, cleanup);
    }
    if plan::appends(options) {
        builder = builder.append();
    }
    if options.latest_link {
        builder = builder.create_symlink(dir.join(LATEST_LINK));
    }
    if options.utc {
        builder = builder.use_utc();
    }

    Ok(builder.build()?)
}
