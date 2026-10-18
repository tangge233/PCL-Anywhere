//! 进程级应用状态。

use std::sync::Arc;

use lite_config::{Config as ConfigFile, Error};

use crate::config::{self, AppConfig};

/// 应用生命周期内共享的状态。
pub(crate) struct AppState {
    config: Arc<ConfigFile<AppConfig>>,
}

impl gpui_kit::Global for AppState {}

impl AppState {
    /// 加载全局配置；失败时由调用方直接退出。
    pub(crate) fn load() -> Result<Self, Error> {
        Ok(Self {
            config: config::load()?,
        })
    }

    /// 当前配置句柄。
    pub(crate) fn config(&self) -> &ConfigFile<AppConfig> {
        &self.config
    }
}
