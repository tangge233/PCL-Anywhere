//! Job 作用域的共享数据。

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

type Entries = HashMap<TypeId, Arc<dyn Any + Send + Sync>>;

/// Job 内跨 work 共享的类型化数据表。
///
/// 类型即键：上游 `insert::<T>` 之后，下游 `get::<T>` 拿到同一个值。同一个 `T`
/// 后写覆盖前写——需要并发累积时，让值本身是 `Mutex`/原子，而不是指望 `insert`
/// 合并。
///
/// 锁只在一次哈希查找内持有，取出的又是 `Arc`，可以放心跨 `await` 持有。
#[derive(Default)]
pub struct SharedData {
    entries: Mutex<Entries>,
}

impl SharedData {
    /// 写入或覆盖，通常作为上游 work 对下游的产出。
    pub fn insert<T: Send + Sync + 'static>(&self, value: T) {
        self.lock().insert(TypeId::of::<T>(), Arc::new(value));
    }

    /// 取回共享值，没写入过时返回 `None`。
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        let value = Arc::clone(self.lock().get(&TypeId::of::<T>())?);
        value.downcast::<T>().ok()
    }

    /// 取走共享值，语义上表示消费掉这份产出。
    pub fn remove<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        let value = self.lock().remove(&TypeId::of::<T>())?;
        value.downcast::<T>().ok()
    }

    fn lock(&self) -> MutexGuard<'_, Entries> {
        // 临界区只有一次表操作，不会 panic；即使中毒，继续用也是安全的。
        self.entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
