//! 错误类型。
//!
//! `is_corrupt` 区分「内容坏了」与「读不了 / 版本过新」：只有前者能被默认值替换。

use std::io;
use std::path::PathBuf;

/// 配置的读、写、迁移失败。每个变体都带出错文件的路径。
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// 读盘失败。文件不存在不会走到这里——那种情况会用默认值创建。
    #[error("读取配置失败：{path}")]
    Read {
        /// 配置文件路径。
        path: PathBuf,
        /// 底层 IO 错误。
        #[source]
        source: io::Error,
    },

    /// 语法错误或反序列化失败。内容坏了，允许被 `open_or_default_replace` 替换。
    #[error("配置无法解析：{path}")]
    Parse {
        /// 配置文件路径。
        path: PathBuf,
        /// 底层解析错误。
        #[source]
        source: toml_edit::de::Error,
    },

    /// 配置里有 TOML 表达不了的值，schema 定义有误。
    #[error("配置无法序列化：{path}")]
    Serialize {
        /// 配置文件路径。
        path: PathBuf,
        /// 底层序列化错误。
        #[source]
        source: toml_edit::ser::Error,
    },

    /// 写盘失败：建目录、写临时文件、rename 任一步出错。
    #[error("写入配置失败：{path}")]
    Write {
        /// 配置文件路径。
        path: PathBuf,
        /// 底层 IO 错误。
        #[source]
        source: io::Error,
    },

    /// 损坏的文件没能改名挪走。
    #[error("备份损坏的配置失败：{path}")]
    Backup {
        /// 配置文件路径。
        path: PathBuf,
        /// 底层 IO 错误。
        #[source]
        source: io::Error,
    },

    /// `version` 键存在但不是 ≥ 1 的整数。
    #[error("配置的版本号无效：{path}")]
    BadVersion {
        /// 配置文件路径。
        path: PathBuf,
    },

    /// 文件由更新的版本写入。这不是损坏，是版本闸门：老版本拒绝加载，才不会把新版本写的
    /// 字段整份抹掉。
    #[error("配置由更新的版本写入（{found} > {supported}）：{path}")]
    VersionTooNew {
        /// 配置文件路径。
        path: PathBuf,
        /// 文件里的版本号。
        found: u32,
        /// 本程序支持的版本号。
        supported: u32,
    },

    /// 某一步迁移失败。
    #[error("从 v{version} 迁移失败（{note}）：{path}")]
    Migrate {
        /// 配置文件路径。
        path: PathBuf,
        /// 出错的迁移步负责的起始版本号。
        version: u32,
        /// 该迁移步的说明。
        note: &'static str,
        /// 迁移步自己的错误。
        #[source]
        source: MigrateError,
    },

    /// `Schema::VERSION` 与迁移链长度对不上。加载任何文件都会立刻报错。
    #[error("schema 声明 v{version}，迁移链却只有 {steps} 步（应为 {expected} 步）")]
    MigrationChainBroken {
        /// schema 声明的版本号。
        version: u32,
        /// 实际提供的迁移步数。
        steps: usize,
        /// 该版本号要求的迁移步数。
        expected: u32,
    },

    /// 等待落盘超时。
    #[error("等待落盘超时：{path}")]
    FlushTimeout {
        /// 配置文件路径。
        path: PathBuf,
    },

    /// 写入线程已退出，改动无法落盘。
    #[error("配置写入线程已退出：{path}")]
    WriterGone {
        /// 配置文件路径。
        path: PathBuf,
    },

    /// 定位配置目录失败。
    #[error("无法定位配置目录：{0}")]
    NoConfigDir(&'static str),
}

impl Error {
    /// 内容是否坏到可以用默认值重建。
    ///
    /// 语法错误、无效版本号、迁移失败算坏；`VersionTooNew` 不算——那是刻意的版本闸门，替换
    /// 它会把新版本写的字段永久吃掉。IO 错误也不算：读不动不等于内容坏了。
    pub(crate) fn is_corrupt(&self) -> bool {
        matches!(
            self,
            Self::Parse { .. } | Self::BadVersion { .. } | Self::Migrate { .. }
        )
    }
}

/// 迁移步内部的错误。
#[derive(Debug, thiserror::Error)]
pub enum MigrateError {
    /// 文档形状与该迁移步的预期对不上。
    ///
    /// 硬失败是刻意的：迁移里蒙混过去等于静默重置用户配置。
    #[error("文档形状与迁移步骤预期不符：{0}")]
    Shape(&'static str),

    /// 路径不合法（空路径，或目标落在源内部）。
    #[error("迁移路径非法：{0}")]
    Path(&'static str),
}
