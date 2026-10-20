//! 分层执行器：每个 Job 一个 driver task。

use std::sync::Arc;

use tokio::task::JoinSet;

use crate::job::Job;
use crate::state::{JobStatus, WorkStatus};
use crate::work::WorkContext;

/// 推进整个 Job，直到成功、失败或被取消。
pub(crate) async fn drive(job: Arc<Job>) {
    job.set_status(JobStatus::Running);

    let mut failure = None;

    for level in job.levels() {
        if job.is_cancelled() {
            break;
        }

        let mut set = JoinSet::new();
        for &id in level {
            // 同层中途取消：本层剩下的 work 不再启动，交给末尾的收尾逻辑标记。
            if job.is_cancelled() {
                break;
            }

            job.state_of(id).set_status(WorkStatus::Running);
            let work = job.work_of(id);
            let ctx = WorkContext::new(Arc::clone(&job), id);
            let running = Arc::clone(&job);

            // 任务返回失败文本，成功或被取消时返回 None。
            set.spawn(async move {
                match work.run(ctx).await {
                    Ok(()) => {
                        running.state_of(id).set_status(WorkStatus::Succeeded);
                        None
                    }
                    Err(_) if running.is_cancelled() => {
                        running.state_of(id).set_status(WorkStatus::Cancelled);
                        None
                    }
                    Err(error) => {
                        let message = error.to_string();
                        running
                            .state_of(id)
                            .set_status(WorkStatus::Failed(message.clone()));
                        // fail-fast：同层的兄弟 work 立刻收到取消信号。
                        running.cancel();
                        Some(message)
                    }
                }
            });
        }

        while let Some(joined) = set.join_next().await {
            match joined {
                Ok(None) => {}
                Ok(Some(message)) => {
                    failure.get_or_insert(message);
                }
                // work panic 时拿不到它的编号，用 JoinError 文本兜底。
                Err(error) => {
                    failure.get_or_insert(error.to_string());
                }
            }
        }

        if let Some(message) = &failure {
            // panic 的 work 会停在 Running，在本层收尾时一并置为失败。
            for &id in level {
                let state = job.state_of(id);
                if state.status() == WorkStatus::Running {
                    state.set_status(WorkStatus::Failed(message.clone()));
                }
            }
            break;
        }
    }

    // 失败优先于取消：fail-fast 也是通过取消令牌传播的，靠 failure 区分两者。
    match failure {
        Some(message) => {
            job.settle_pending(WorkStatus::Skipped);
            job.set_status(JobStatus::Failed(message));
        }
        None if job.is_cancelled() => {
            job.settle_pending(WorkStatus::Cancelled);
            job.set_status(JobStatus::Cancelled);
        }
        None => job.set_status(JobStatus::Succeeded),
    }
}
