//! logger 安装、调级与日志目录操作的失败。

/// 安装 logger、修改级别或管理日志文件时的失败。
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// [`crate::init`] 已被调用过。进程内只允许安装一个 logger。
    #[error("日志器已初始化，不能重复初始化")]
    AlreadyInitialized,

    /// 调级时 [`crate::init`] 尚未成功调用。
    #[error("日志器尚未初始化")]
    NotInitialized,

    /// 无法确定日志目录，通常是 `XDG_CONFIG_HOME` 与 `HOME` 都不可用。
    #[error("无法确定日志目录：{0}")]
    NoLogDir(&'static str),

    /// 创建目录、删除旧日志等文件系统操作失败。
    #[error("日志目录操作失败：{0}")]
    Io(#[from] std::io::Error),

    /// flexi_logger 自身的失败，例如打开文件、写入或轮转。
    #[error("日志器失败：{0}")]
    Logger(#[from] flexi_logger::FlexiLoggerError),

    /// 规范字符串无法解析。保存错误文本而非 `FlexiLoggerError`，因为该类型已用于
    /// [`Error::Logger`]。
    #[error("日志级别规范无效：{0}")]
    Spec(String),
}
