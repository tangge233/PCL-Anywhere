//! schema 定义与版本迁移。

use serde::Serialize;
use serde::de::DeserializeOwned;
use toml_edit::DocumentMut;

use crate::error::MigrateError;

/// 一步迁移：把版本 `n` 的文档就地改成版本 `n + 1`。
///
/// 步与版本的对应关系由切片下标决定：第 i 项负责 `i+1 → i+2`。不设 `from` 字段——冗余声明
/// 会和顺序不一致。
pub struct Migration {
    /// 一句话说明，出现在错误信息和测试名里。
    pub note: &'static str,
    /// 纯文档变换：不得依赖网络、时间、其它文件或进程状态，否则不可重放。
    pub apply: fn(&mut DocumentMut) -> Result<(), MigrateError>,
}

/// 一个配置文件的内容类型：普通的 serde 结构即可。
///
/// 每个结构（含嵌套）都要加 `#[serde(default)]`，且**不要**加 `deny_unknown_fields`：前者
/// 让老文件缺的字段落到默认值，后者让老版本读到新写的字段时不报错。两条合起来，增删字段
/// 永远不需要升版本。
///
/// `Option` 字段为 `None` 时键会消失，但外层表头仍会写出——空的 `[java]` 依然出现。要去掉
/// 空段，用 `#[serde(skip_serializing_if = ...)]`，或把字段本身改成 `Option<子结构>`。
pub trait Schema:
    Serialize + DeserializeOwned + Default + Clone + PartialEq + Send + Sync + 'static
{
    /// 写入文件顶部的 `version` 键。
    const VERSION: u32;

    /// 迁移链，从版本 1 逐级到 [`Self::VERSION`]，长度必须**恰好**是 `VERSION - 1`。
    ///
    /// 每次加载都会校验长度；不符会返回 [`Error`](crate::Error)，而不是把老配置悄悄读成
    /// 默认值。
    fn migrations() -> &'static [Migration] {
        &[]
    }
}
