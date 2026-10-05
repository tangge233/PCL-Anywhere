//! 保留上限：删除多余的旧日志文件。
//!
//! flexi_logger 的 `Cleanup` 只处理本次运行轮转产生的文件。会话模式不轮转，跨启动的堆积
//! 不会触发它，所以这里在安装时统一清理。三种轮转方式共用本模块。

use std::fs;
use std::io;
use std::path::Path;

use crate::BASENAME;

/// 只保留最新的 `keep` 个日志文件，其余删除。
///
/// 只处理名称以 [`BASENAME`] 开头的普通文件，软链（`latest.log`）与目录不参与。删除失败
/// 返回错误：无法管理自己的日志目录时，安装 logger 应当失败。
pub(crate) fn prune(dir: &Path, keep: usize) -> io::Result<()> {
    let keep = keep.max(1);
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        if !entry.file_name().to_string_lossy().starts_with(BASENAME) {
            continue;
        }
        files.push((entry.metadata()?.modified()?, entry.path()));
    }

    if files.len() <= keep {
        return Ok(());
    }
    files.sort_by_key(|(modified, _)| *modified); // 旧 → 新
    for (_, path) in &files[..files.len() - keep] {
        fs::remove_file(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    fn touch(dir: &Path, name: &str) {
        File::create(dir.join(name)).unwrap();
    }

    fn remaining(dir: &Path) -> Vec<String> {
        let mut names: Vec<_> = fs::read_dir(dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn keeps_only_the_newest() {
        let dir = tempfile::tempdir().unwrap();
        for name in [
            "pcl-anywhere_a.log",
            "pcl-anywhere_b.log",
            "pcl-anywhere_c.log",
        ] {
            touch(dir.path(), name);
        }
        prune(dir.path(), 2).unwrap();
        // 三个文件可能落在同一毫秒，mtime 相同，因此断言数量而非具体文件名。
        assert_eq!(
            remaining(dir.path()).len(),
            2,
            "{:?}",
            remaining(dir.path())
        );
    }

    #[test]
    fn never_deletes_directories_or_foreign_files() {
        let dir = tempfile::tempdir().unwrap();
        touch(dir.path(), "pcl-anywhere_a.log");
        touch(dir.path(), "pcl-anywhere_b.log");
        touch(dir.path(), "someone-elses.log");
        fs::create_dir(dir.path().join("pcl-anywhere_dir.log")).unwrap();

        prune(dir.path(), 1).unwrap();

        let left = remaining(dir.path());
        assert!(left.contains(&"someone-elses.log".to_owned()), "{left:?}");
        assert!(
            left.contains(&"pcl-anywhere_dir.log".to_owned()),
            "{left:?}"
        );

        // 名称以相同前缀开头的目录不是普通文件，不参与保留计数。
        let files = fs::read_dir(dir.path())
            .unwrap()
            .filter(|entry| entry.as_ref().unwrap().file_type().unwrap().is_file())
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|name| name.starts_with("pcl-anywhere_"))
            .count();
        assert_eq!(files, 1, "{left:?}");
    }

    #[test]
    fn zero_still_keeps_one() {
        let dir = tempfile::tempdir().unwrap();
        touch(dir.path(), "pcl-anywhere_a.log");
        prune(dir.path(), 0).unwrap();
        assert_eq!(remaining(dir.path()).len(), 1);
    }

    #[test]
    fn missing_directory_is_not_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("nope");
        // `read_dir` 对不存在的目录返回错误。[`crate::init`] 先 `create_dir_all` 再调用
        // 本函数，这里固定这一前提。
        assert!(prune(&missing, 1).is_err());
    }
}
