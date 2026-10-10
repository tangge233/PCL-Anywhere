//! 全局 logger 的生命周期凭据与运行时状态。

use std::path::PathBuf;
use std::sync::Mutex;

use flexi_logger::{LogfileSelector, LoggerHandle};
use log::LevelFilter;

use crate::error::Error;
use crate::options::Options;
use crate::spec;

/// 全局 logger 的生命周期凭据，由 [`crate::init`] 返回。
///
/// drop 时关闭写线程，因此必须绑定到变量并活到进程结束：`let _ = ...` 会立即 drop，之后的
/// 日志不再写入。
pub struct LogGuard {
    handle: LoggerHandle,
}

impl LogGuard {
    pub(crate) fn new(handle: LoggerHandle) -> Self {
        Self { handle }
    }

    /// 立即写出已缓冲的日志。默认的非缓冲模式下为空操作。
    pub fn flush(&self) {
        self.handle.flush();
    }

    /// 立即轮转一次，不等待大小或时间条件。
    pub fn rotate_now(&self) -> Result<(), Error> {
        self.handle.trigger_rotation().map_err(Error::from)
    }

    /// 当前存在的日志文件，包含正在写入的那个。
    pub fn log_files(&self) -> Result<Vec<PathBuf>, Error> {
        self.handle
            .existing_log_files(&LogfileSelector::default().with_r_current())
            .map_err(Error::from)
    }
}

impl Drop for LogGuard {
    fn drop(&mut self) {
        // 静态变量中另有一份句柄供运行时调级，且永不 drop，因此关闭写线程必须显式做。
        self.handle.shutdown();
    }
}

/// 已安装 logger 的运行时状态。
pub(crate) struct Runtime {
    handle: LoggerHandle,
    /// [`crate::init`] 收到的参数，供 [`Runtime::set_level`] 重建级别规范。
    options: Mutex<Options>,
}

impl Runtime {
    pub(crate) fn new(handle: LoggerHandle, options: Options) -> Self {
        Self {
            handle,
            options: Mutex::new(options),
        }
    }

    /// 修改默认级别，保留按分类的覆盖。
    pub(crate) fn set_level(&self, level: LevelFilter) {
        let spec = {
            // 持锁范围仅限构造规范；写日志不经过这里。
            let mut options = self.options.lock().expect("日志配置锁不该中毒");
            options.level = level;
            spec::from_options(&options)
        };
        self.handle.set_new_spec(spec);
    }

    /// 整体替换级别规范。
    pub(crate) fn set_spec(&self, raw: &str) -> Result<(), Error> {
        self.handle.set_new_spec(spec::parse(raw)?);
        Ok(())
    }
}
