//! 基本读写：默认值、往返、立即可见、无变化不落盘。

mod common;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::time::Duration;

use common::{Plain, ThemeMode, read, temp_config};
use lite_config::Config;

const TIMEOUT: Duration = Duration::from_secs(5);

#[test]
fn first_open_creates_the_file_with_defaults() {
    let (_dir, path) = temp_config();
    let cfg = Config::<Plain>::open(&path).expect("首次打开应当创建文件");

    let text = read(&path);
    assert!(
        text.starts_with("version = 1\n"),
        "版本号必须在最前：\n{text}"
    );
    assert_eq!(cfg.read(|c| c.memory_gib), 15);
    assert_eq!(cfg.read(|c| c.theme.mode), ThemeMode::System);
    // 用户打开就能看到全部可选项，而不是一个空文件。
    assert!(text.contains("memory_gib = 15"));
    assert!(text.contains("payload = []"));
}

#[test]
fn new_files_are_private_and_the_directory_too() {
    let dir = tempfile::tempdir().expect("建临时目录");
    let path = dir.path().join("PCL").join("config.toml");
    Config::<Plain>::open(&path).expect("创建");

    let file_mode = fs::metadata(&path)
        .expect("文件元数据")
        .permissions()
        .mode()
        & 0o777;
    let dir_mode = fs::metadata(path.parent().expect("父目录"))
        .expect("目录元数据")
        .permissions()
        .mode()
        & 0o777;
    // 配置里会有账号路径、JVM 参数、将来的登录态，默认不给同机其它用户读。
    assert_eq!(file_mode, 0o600, "配置文件应为 0600");
    assert_eq!(dir_mode, 0o700, "配置目录应为 0700");
}

#[test]
fn mutate_is_visible_immediately_and_reaches_the_disk_after_flush() {
    let (_dir, path) = temp_config();
    let cfg = Config::<Plain>::open(&path).expect("打开");
    assert_eq!(cfg.version(), 0);

    cfg.mutate(|c| {
        c.memory_gib = 8;
        c.theme.mode = ThemeMode::Dark;
        c.theme.font = Some("Noto Sans".into());
    });

    // 界面要立刻看到新值，不能等 fsync。
    assert_eq!(cfg.read(|c| c.memory_gib), 8);
    assert_eq!(cfg.read(|c| c.theme.mode), ThemeMode::Dark);
    assert_eq!(cfg.version(), 1);

    cfg.flush(TIMEOUT).expect("落盘");
    let text = read(&path);
    assert!(text.contains("memory_gib = 8"), "{text}");
    assert!(text.contains("mode = \"dark\""), "{text}");
    assert!(text.contains("font = \"Noto Sans\""), "{text}");
}

#[test]
fn snapshots_are_independent_copies() {
    let (_dir, path) = temp_config();
    let cfg = Config::<Plain>::open(&path).expect("打开");

    let before = cfg.snapshot();
    cfg.mutate(|c| c.memory_gib = 7);
    let after = cfg.snapshot();

    // 旧快照不被改写——这是"读到的值一次渲染内一致"的前提。
    assert_eq!(before.memory_gib, 15);
    assert_eq!(after.memory_gib, 7);
}

#[test]
fn mutating_to_the_same_value_does_not_touch_the_file() {
    let (_dir, path) = temp_config();
    let cfg = Config::<Plain>::open(&path).expect("打开");
    std::thread::sleep(Duration::from_millis(20));
    let before = fs::metadata(&path)
        .expect("元数据")
        .modified()
        .expect("mtime");

    cfg.mutate(|c| c.memory_gib = 15);
    assert_eq!(cfg.version(), 0, "值没变就不该动版本号");
    assert!(cfg.flush(TIMEOUT).is_ok());

    // 睡过一个合并窗口：真写了盘就一定看得出来。
    std::thread::sleep(Duration::from_millis(400));
    let after = fs::metadata(&path)
        .expect("元数据")
        .modified()
        .expect("mtime");
    assert_eq!(before, after, "值没变就不该落盘");
}

#[test]
fn flush_before_any_mutation_is_a_no_op() {
    let (_dir, path) = temp_config();
    let cfg = Config::<Plain>::open(&path).expect("打开");
    // 没改过就没有写线程，磁盘就是内存。
    assert!(cfg.flush(TIMEOUT).is_ok());
}

#[test]
fn a_fresh_handle_sees_what_the_previous_one_wrote() {
    let (_dir, path) = temp_config();
    {
        let cfg = Config::<Plain>::open(&path).expect("打开");
        cfg.mutate(|c| {
            c.memory_gib = 3;
            c.payload = vec![1, 2, 3];
        });
        cfg.flush(TIMEOUT).expect("落盘");
    }

    let reopened = Config::<Plain>::open(&path).expect("重新打开");
    assert_eq!(reopened.read(|c| c.memory_gib), 3);
    assert_eq!(reopened.read(|c| c.payload.clone()), vec![1, 2, 3]);
    assert_eq!(reopened.version(), 0, "重新打开是新的句柄，版本号从零起");
}

#[test]
fn missing_keys_fall_back_to_defaults() {
    let dir = tempfile::tempdir().expect("建临时目录");
    let path = dir.path().join("config.toml");
    // 手写的文件常常只有几个键。
    fs::write(&path, "version = 1\nmemory_gib = 4\n").expect("写夹具");

    let cfg = Config::<Plain>::open(&path).expect("打开");
    assert_eq!(cfg.read(|c| c.memory_gib), 4);
    assert_eq!(cfg.read(|c| c.theme.mode), ThemeMode::System);
    assert_eq!(cfg.read(|c| c.payload.clone()), Vec::<u32>::new());
}

#[test]
fn unknown_keys_are_ignored() {
    let dir = tempfile::tempdir().expect("建临时目录");
    let path = dir.path().join("config.toml");
    // 模拟更新版本写过的文件：老版本读到新键不能炸。
    fs::write(
        &path,
        "version = 1\nmemory_gib = 4\n\n[future]\nthing = true\n",
    )
    .expect("写夹具");

    let cfg = Config::<Plain>::open(&path).expect("打开");
    assert_eq!(cfg.read(|c| c.memory_gib), 4);
}
