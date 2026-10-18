//! 注册错误。

/// `MultiSelector::register` 失败的原因。
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RegisterError {
    /// 相同 ID 已经注册。
    #[error("源已注册：{id}")]
    AlreadyRegistered {
        /// 冲突的源 ID。
        id: String,
    },
}
