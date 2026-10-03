//! 单个配置文件的句柄。

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use arc_swap::ArcSwap;

use crate::error::Error;
use crate::io::{self, OnCorrupt};
use crate::schema::Schema;
use crate::writer::Writer;

/// 写线程与所有句柄共享的部分。
///
/// 写线程只拿这个 `Arc`，不持有 [`Config`]：否则两者互相引用，`Drop` 永远不会跑，退出前的
/// 最后一次落盘也就没了。
pub(crate) struct Shared<S: Schema> {
    pub(crate) live: ArcSwap<S>,
    pub(crate) version: AtomicU64,
}

/// 一个配置文件的句柄。
///
/// 以 `Arc<Config<S>>` 共享，全部方法取 `&self`。最后一个句柄析构时尽力把待写的改动落盘，
/// 最多等 1 秒。
pub struct Config<S: Schema> {
    path: PathBuf,
    shared: Arc<Shared<S>>,
    /// 写锁，只护「克隆 → 改闭包 → 发布快照」；`fsync` 在写线程上，不占它。
    write: Mutex<()>,
    /// 首次 `mutate` 时才建写线程；只读打开的配置没有线程。
    writer: OnceLock<Writer>,
}

impl<S: Schema> Config<S> {
    /// 打开配置，内容坏了就报错。
    ///
    /// 文件不存在时用默认值原子创建。
    pub fn open(path: impl Into<PathBuf>) -> Result<Arc<Self>, Error> {
        Self::load(path.into(), OnCorrupt::Fail)
    }

    /// 打开配置，内容坏了就用全新默认值代替它。
    ///
    /// 损坏的文件先改名成 `<名字>.corrupt-<时间戳>` 再重建，不当场销毁。适合当启动路径：
    /// 一次手滑的编辑不会让程序起不来。
    ///
    /// 只有内容坏了才会被替换。版本比当前新、权限不足、IO 失败照常返回 `Err`。
    pub fn open_or_default_replace(path: impl Into<PathBuf>) -> Result<Arc<Self>, Error> {
        Self::load(path.into(), OnCorrupt::Replace)
    }

    fn load(path: PathBuf, on_corrupt: OnCorrupt) -> Result<Arc<Self>, Error> {
        let value = io::load::<S>(&path, on_corrupt)?;
        Ok(Arc::new(Self {
            path,
            shared: Arc::new(Shared {
                live: ArcSwap::from_pointee(value),
                version: AtomicU64::new(0),
            }),
            write: Mutex::new(()),
            writer: OnceLock::new(),
        }))
    }

    /// 配置文件的路径。
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// 当前快照。一次原子读加一次引用计数自增，不取锁、不做 IO。
    pub fn snapshot(&self) -> Arc<S> {
        self.shared.live.load_full()
    }

    /// 在快照上求值，比 [`Config::snapshot`] 少一次引用计数自增。
    pub fn read<R>(&self, f: impl FnOnce(&S) -> R) -> R {
        f(&self.shared.live.load())
    }

    /// 变更计数：内容变了加一，没变不动。界面用它判断要不要重绘。
    pub fn version(&self) -> u64 {
        self.shared.version.load(Ordering::Acquire)
    }

    /// 改配置。返回后 [`Config::snapshot`] 立刻可见新值，落盘由写线程稍后完成。
    ///
    /// 闭包保证只执行一次（写者由 `Mutex` 串行化，不是无锁 CAS 重试），里面放日志、错误处理、
    /// 界面通知都是安全的。
    ///
    /// 改完与当前值完全相同时直接返回：不动版本号，也不惊动写线程。
    pub fn mutate(&self, f: impl FnOnce(&mut S)) {
        // 锁中毒不级联：取回内部值继续用，否则一次 panic 会让此后所有写入都失败。
        let _guard = self
            .write
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let current = self.shared.live.load_full();
        let mut next = (*current).clone();
        f(&mut next);
        if next == *current {
            return;
        }
        self.shared.live.store(Arc::new(next));
        self.shared.version.fetch_add(1, Ordering::Release);
        self.writer().mark_dirty();
    }

    /// 等到本文件的所有改动都落盘。从未 `mutate` 过时直接返回。
    ///
    /// 后台落盘失败不会自动重试，下一次 `mutate` 或 `flush` 会再试。
    pub fn flush(&self, timeout: Duration) -> Result<(), Error> {
        match self.writer.get() {
            Some(writer) => writer.flush(timeout),
            None => Ok(()),
        }
    }

    fn writer(&self) -> &Writer {
        self.writer
            .get_or_init(|| Writer::spawn::<S>(self.path.clone(), Arc::clone(&self.shared)))
    }
}

impl<S: Schema> std::fmt::Debug for Config<S> {
    /// 只输出路径与版本号；快照整份打进日志没有意义。
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("path", &self.path)
            .field("version", &self.version())
            .finish_non_exhaustive()
    }
}
