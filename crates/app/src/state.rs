//! 进程级应用状态。

use std::error::Error as StdError;
use std::sync::Arc;

use aw_job_worker::JobWorker;
use lite_config::Config as ConfigFile;
use tokio::runtime::Runtime;

use crate::config::{self, AppConfig};

/// 应用生命周期内共享的状态。
pub(crate) struct AppState {
    config: Arc<ConfigFile<AppConfig>>,
    runtime: Arc<Runtime>,
    jobs: Arc<JobWorker>,
}

impl gpui_kit::Global for AppState {}

impl AppState {
    /// 加载全局配置并建立 runtime；失败时由调用方直接退出。
    pub(crate) fn load() -> Result<Self, Box<dyn StdError>> {
        let runtime = Arc::new(Runtime::new()?);
        let jobs = Arc::new(JobWorker::new(Arc::clone(&runtime)));

        Ok(Self {
            config: config::load()?,
            runtime,
            jobs,
        })
    }

    /// 当前配置句柄。
    pub(crate) fn config(&self) -> &ConfigFile<AppConfig> {
        &self.config
    }

    /// 全局 runtime；不属于任何 Job 的任务也从这里 spawn。
    #[expect(dead_code, reason = "随 AppState 提供给后续业务模块使用")]
    pub(crate) fn runtime(&self) -> &Arc<Runtime> {
        &self.runtime
    }

    /// 工作流调度器：提交、查询、取消 Job 都走它。
    #[expect(dead_code, reason = "随 AppState 提供给后续业务模块使用")]
    pub(crate) fn jobs(&self) -> &Arc<JobWorker> {
        &self.jobs
    }
}
