//! 用 [`crate::build`] 构造 logger 并写入记录，断言生成的文件名与内容。
//!
//! 不使用 `init`：一个测试二进制内只能安装一个全局 logger，而三种轮转方式需要各自独立
//! 验证。全局路径由 `tests/global.rs` 覆盖。

use std::fs;
use std::path::{Path, PathBuf};

use log::{Level, LevelFilter, Record};

use crate::options::{Console, Options, Rotation};

fn write(logger: &dyn log::Log, level: Level, target: &str, message: &str) {
    logger.log(
        &Record::builder()
            .args(format_args!("{message}"))
            .level(level)
            .target(target)
            .module_path(Some("logger::tests"))
            .file(Some("tests.rs"))
            .line(Some(1))
            .build(),
    );
}

/// 目录里以 `pcl-anywhere` 开头的日志文件，按名字排序；`latest.log` 软链不算。
fn log_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with("pcl-anywhere"))
        })
        .collect();
    files.sort();
    files
}

fn options(dir: &Path, rotation: Rotation) -> Options {
    Options {
        dir: Some(dir.to_path_buf()),
        rotation,
        console: Console::Off, // 避免测试输出被 stderr 污染
        ..Options::default()
    }
}

#[test]
fn by_session_writes_one_file_per_run() {
    let dir = tempfile::tempdir().unwrap();
    let options = Options {
        level: LevelFilter::Info,
        ..options(dir.path(), Rotation::BySession)
    };
    let (logger, handle) = crate::build(&options, dir.path()).unwrap();

    write(&*logger, Level::Info, "Setup", "已初始化启动页设置");
    write(
        &*logger,
        Level::Warn,
        "Instance",
        "打开实例设置窗口失败：boom",
    );
    handle.flush();
    drop(handle);

    let files = log_files(dir.path());
    assert_eq!(files.len(), 1, "{files:?}");
    let name = files[0].file_name().unwrap().to_string_lossy().into_owned();
    assert!(
        name.starts_with("pcl-anywhere_") && name.ends_with(".log"),
        "{name}"
    );

    let text = fs::read_to_string(&files[0]).unwrap();
    assert!(
        text.contains("[INFO ] [Setup] 已初始化启动页设置"),
        "{text}"
    );
    assert!(
        text.contains("[WARN ] [Instance] 打开实例设置窗口失败：boom"),
        "{text}"
    );
}

#[test]
fn by_size_rotates_within_the_session() {
    let dir = tempfile::tempdir().unwrap();
    let options = Options {
        level: LevelFilter::Info,
        // 把保留上限调大，避免清理线程删掉待断言的分片；保留策略由 `prune` 的测试覆盖。
        keep: 100,
        ..options(dir.path(), Rotation::BySize { max_bytes: 256 })
    };
    let (logger, handle) = crate::build(&options, dir.path()).unwrap();

    for index in 0..40 {
        write(
            &*logger,
            Level::Info,
            "Download",
            &format!("分片第 {index} 行，凑点字节"),
        );
    }
    handle.flush();
    drop(handle);

    let names: Vec<String> = log_files(dir.path())
        .iter()
        .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert!(names.len() > 1, "应当滚动出多个分片：{names:?}");
    assert!(
        names.iter().any(|name| name.ends_with("_rCURRENT.log")),
        "{names:?}"
    );
    assert!(
        names.iter().any(|name| name.contains("_r00000.log")),
        "{names:?}"
    );
}

#[test]
fn by_date_writes_a_current_file() {
    let dir = tempfile::tempdir().unwrap();
    let options = Options {
        level: LevelFilter::Info,
        ..options(dir.path(), Rotation::ByDate)
    };
    let (logger, handle) = crate::build(&options, dir.path()).unwrap();

    write(&*logger, Level::Error, "App", "第三条");
    handle.flush();
    drop(handle);

    let files = log_files(dir.path());
    assert_eq!(files.len(), 1, "{files:?}");
    assert_eq!(
        files[0].file_name().unwrap().to_string_lossy(),
        "pcl-anywhere_rCURRENT.log"
    );
    assert!(
        fs::read_to_string(&files[0])
            .unwrap()
            .contains("[ERROR] [App] 第三条")
    );
}

#[test]
fn latest_link_points_at_the_current_file() {
    let dir = tempfile::tempdir().unwrap();
    let options = options(dir.path(), Rotation::BySession);
    let (logger, handle) = crate::build(&options, dir.path()).unwrap();

    write(&*logger, Level::Info, "App", "x");
    handle.flush();
    drop(handle);

    let target = fs::read_link(dir.path().join("latest.log")).expect("latest.log 应当是软链");
    let name = target.file_name().unwrap().to_string_lossy().into_owned();
    assert!(
        name.starts_with("pcl-anywhere_") && name.ends_with(".log"),
        "{target:?}"
    );
}
