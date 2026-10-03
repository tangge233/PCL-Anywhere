//! `DocumentMut` 的路径寻址读写。
//!
//! 路径是**键名切片**（`&["ui", "theme", "mode"]`），不是 `"ui.theme.mode"` 字符串：TOML 的
//! 键本身允许含点（`"a.b" = 1`），字符串路径必须引入转义规则，切片没有歧义。

use toml_edit::{DocumentMut, Item, Table, TableLike, Value};

use crate::error::MigrateError;

/// 用键路径读写 TOML 文档的节点。
pub trait DocAccess {
    /// 按键路径取节点。路径中间不存在或不是表时返回 `None`。
    fn get_path(&self, path: &[&str]) -> Option<&Item>;

    /// 按键路径取可变节点。
    fn get_path_mut(&mut self, path: &[&str]) -> Option<&mut Item>;

    /// 按键路径写入。沿途缺失的表自动创建；中间层已存在但不是表时报错。
    fn set_path(&mut self, path: &[&str], item: impl Into<Item>) -> Result<(), MigrateError>;

    /// 按键路径删除并返回原节点；键不存在返回 `None`。
    fn remove_path(&mut self, path: &[&str]) -> Option<Item>;

    /// 整棵子树改名或搬家，目标已存在则被覆盖。
    ///
    /// 源不存在视为无事可做——老文件常常只写了部分键。
    fn move_path(&mut self, from: &[&str], to: &[&str]) -> Result<(), MigrateError>;

    /// 仅当当前值等于 `expected` 时替换，返回是否真的改了。
    ///
    /// 用于「只修正老版本发过的那个错默认值，别动用户调过的值」。
    fn replace_if(
        &mut self,
        path: &[&str],
        expected: impl Into<Item>,
        value: impl Into<Item>,
    ) -> Result<bool, MigrateError>;
}

impl DocAccess for DocumentMut {
    fn get_path(&self, path: &[&str]) -> Option<&Item> {
        get_in(self.as_table(), path)
    }

    fn get_path_mut(&mut self, path: &[&str]) -> Option<&mut Item> {
        get_in_mut(self.as_table_mut(), path)
    }

    fn set_path(&mut self, path: &[&str], item: impl Into<Item>) -> Result<(), MigrateError> {
        set_in(self.as_table_mut(), path, item.into())
    }

    fn remove_path(&mut self, path: &[&str]) -> Option<Item> {
        let (last, parents) = path.split_last()?;
        remove_in(self.as_table_mut(), parents, last)
    }

    fn move_path(&mut self, from: &[&str], to: &[&str]) -> Result<(), MigrateError> {
        if to.starts_with(from) {
            return Err(MigrateError::Path("迁移目标位于源内部"));
        }
        match self.remove_path(from) {
            Some(item) => self.set_path(to, item),
            None => Ok(()),
        }
    }

    fn replace_if(
        &mut self,
        path: &[&str],
        expected: impl Into<Item>,
        value: impl Into<Item>,
    ) -> Result<bool, MigrateError> {
        let expected = expected.into();
        match self.get_path(path) {
            Some(current) if same_value(current, &expected) => {}
            _ => return Ok(false),
        }
        self.set_path(path, value)?;
        Ok(true)
    }
}

/// 两个节点的值是同一个值。
///
/// `Item` 没有 `PartialEq`，而 `Display` 会把 `=` 之后的空格当装饰一并输出（解析来的
/// `" 4096"` vs 新建的 `"4096"`），所以先抹平装饰再比文本。指向表时一律不等。
fn same_value(a: &Item, b: &Item) -> bool {
    match (a.as_value(), b.as_value()) {
        (Some(a), Some(b)) => canonical(a) == canonical(b),
        _ => false,
    }
}

/// 去掉排版装饰后的文本形态。
fn canonical(value: &Value) -> String {
    let mut value = value.clone();
    clear_decor(&mut value);
    value.to_string()
}

fn clear_decor(value: &mut Value) {
    value.decor_mut().clear();
    match value {
        Value::Array(array) => array.iter_mut().for_each(clear_decor),
        Value::InlineTable(table) => table.iter_mut().for_each(|(_, item)| clear_decor(item)),
        _ => {}
    }
}

// 四个递归辅助函数：用 `TableLike` 下钻，`[a.b]` 段与 `a = { b = 1 }` 行内表都能走通。

fn get_in<'a>(table: &'a dyn TableLike, path: &[&str]) -> Option<&'a Item> {
    let (key, rest) = path.split_first()?;
    let item = table.get(key)?;
    match rest.split_first() {
        None => Some(item),
        Some(_) => get_in(item.as_table_like()?, rest),
    }
}

fn get_in_mut<'a>(table: &'a mut dyn TableLike, path: &[&str]) -> Option<&'a mut Item> {
    let (key, rest) = path.split_first()?;
    if rest.is_empty() {
        return table.get_mut(key);
    }
    let next = table.get_mut(key)?.as_table_like_mut()?;
    get_in_mut(next, rest)
}

fn set_in(table: &mut dyn TableLike, path: &[&str], item: Item) -> Result<(), MigrateError> {
    let (key, rest) = path
        .split_first()
        .ok_or(MigrateError::Path("迁移路径为空"))?;
    if rest.is_empty() {
        table.insert(key, item);
        return Ok(());
    }
    if table.get(key).is_none() {
        table.insert(key, Item::Table(Table::new()));
    }
    let next = table
        .get_mut(key)
        .and_then(|item| item.as_table_like_mut())
        .ok_or(MigrateError::Shape("迁移路径的中间层不是表"))?;
    set_in(next, rest, item)
}

fn remove_in(table: &mut dyn TableLike, parents: &[&str], last: &str) -> Option<Item> {
    let (key, rest) = parents.split_first()?;
    let next = table.get_mut(key)?.as_table_like_mut()?;
    if rest.is_empty() {
        next.remove(last)
    } else {
        remove_in(next, rest, last)
    }
}
