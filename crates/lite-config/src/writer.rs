//! 单文件的写线程：合并窗口 + 原子替换。
//!
//! 每个被改动过的文件一条线程，首次 `mutate` 时创建；只读打开的配置不起线程。

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender, SyncSender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::config::Shared;
use crate::error::Error;
use crate::io;
use crate::schema::Schema;

/// 改动合并窗口：窗口内的连续改动只落一次盘。拖动滑块会产生一串 `mutate`，而每次落盘都要
/// 付一次 `fsync`。
const DEBOUNCE: Duration = Duration::from_millis(300);

/// `Drop` 时等待落盘的上限。
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(1);

/// 写线程收到的消息。`Flush` 与 `Shutdown` 带回执，发送方等它给出写入结果。
enum Msg {
    /// 内容已变，等合并窗口结束后落盘。
    Dirty,
    Flush(SyncSender<Result<(), Error>>),
    Shutdown(SyncSender<Result<(), Error>>),
}

/// 写线程句柄。析构时尽力把待写内容落盘，再等线程退出。
pub(crate) struct Writer {
    path: PathBuf,
    tx: Sender<Msg>,
    handle: Option<JoinHandle<()>>,
}

impl Writer {
    pub(crate) fn spawn<S: Schema>(path: PathBuf, shared: Arc<Shared<S>>) -> Self {
        let (tx, rx) = mpsc::channel();
        let worker_path = path.clone();
        let handle = thread::Builder::new()
            .name(format!("lite-config {}", path.display()))
            .spawn(move || run::<S>(rx, worker_path, shared))
            .expect("创建配置写入线程失败");
        Self {
            path,
            tx,
            handle: Some(handle),
        }
    }

    /// 标脏。写线程会把窗口内的连续改动合并成一次落盘。
    pub(crate) fn mark_dirty(&self) {
        let _ = self.tx.send(Msg::Dirty);
    }

    /// 等到本文件的改动都已落盘。
    pub(crate) fn flush(&self, timeout: Duration) -> Result<(), Error> {
        let (ack_tx, ack_rx) = mpsc::sync_channel(1);
        self.send(Msg::Flush(ack_tx))?;
        match ack_rx.recv_timeout(timeout) {
            Ok(result) => result,
            Err(RecvTimeoutError::Timeout) => Err(self.timeout_error()),
            Err(RecvTimeoutError::Disconnected) => Err(self.gone_error()),
        }
    }

    fn send(&self, msg: Msg) -> Result<(), Error> {
        self.tx.send(msg).map_err(|_| self.gone_error())
    }

    fn timeout_error(&self) -> Error {
        Error::FlushTimeout {
            path: self.path.clone(),
        }
    }

    fn gone_error(&self) -> Error {
        Error::WriterGone {
            path: self.path.clone(),
        }
    }
}

impl Drop for Writer {
    fn drop(&mut self) {
        // 最后一条改动有机会落盘就落，落不下去也不阻塞退出。
        let (ack_tx, ack_rx) = mpsc::sync_channel(1);
        if self.tx.send(Msg::Shutdown(ack_tx)).is_ok() {
            let _ = ack_rx.recv_timeout(SHUTDOWN_TIMEOUT);
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn run<S: Schema>(rx: Receiver<Msg>, path: PathBuf, shared: Arc<Shared<S>>) {
    let mut dirty = false;
    loop {
        match rx.recv() {
            Err(_) => {
                let _ = write_if_dirty::<S>(&mut dirty, &path, &shared);
                return;
            }
            Ok(Msg::Dirty) => {
                dirty = true;
                // 合并窗口：把这期间到达的消息一并处理，窗口一结束再落盘。
                loop {
                    match rx.recv_timeout(DEBOUNCE) {
                        Ok(Msg::Dirty) => dirty = true,
                        Ok(Msg::Flush(ack)) => {
                            let _ = ack.send(write_if_dirty::<S>(&mut dirty, &path, &shared));
                        }
                        Ok(Msg::Shutdown(ack)) => {
                            let _ = ack.send(write_if_dirty::<S>(&mut dirty, &path, &shared));
                            return;
                        }
                        Err(RecvTimeoutError::Timeout) => break,
                        Err(RecvTimeoutError::Disconnected) => {
                            let _ = write_if_dirty::<S>(&mut dirty, &path, &shared);
                            return;
                        }
                    }
                }
                // 落盘失败时保留 dirty：下一次改动或 flush 会再试一次。
                let _ = write_if_dirty::<S>(&mut dirty, &path, &shared);
            }
            Ok(Msg::Flush(ack)) => {
                let _ = ack.send(write_if_dirty::<S>(&mut dirty, &path, &shared));
            }
            Ok(Msg::Shutdown(ack)) => {
                let _ = ack.send(write_if_dirty::<S>(&mut dirty, &path, &shared));
                return;
            }
        }
    }
}

/// 把当前快照落盘。没脏就是空操作，所以重复调用不会白白 `fsync`。
fn write_if_dirty<S: Schema>(
    dirty: &mut bool,
    path: &Path,
    shared: &Shared<S>,
) -> Result<(), Error> {
    if !*dirty {
        return Ok(());
    }
    let snapshot = shared.live.load_full();
    // 显式解引用：泛型位置不会自动 deref 到 `&S`，会先把 `S` 推成 `Arc<S>`。
    io::write_atomically(path, &*snapshot)?;
    *dirty = false;
    Ok(())
}
