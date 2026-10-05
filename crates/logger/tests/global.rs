//! 安装全局 logger 后，验证 `log` 宏、按分类过滤、运行时调级与手动轮转。
//!
//! 只放一个测试：一个进程只能安装一个全局 logger。

use logger::log;

#[test]
fn init_wires_the_global_logger() {
    let dir = tempfile::tempdir().unwrap();
    let guard = logger::init(logger::Options {
        dir: Some(dir.path().to_path_buf()),
        level: log::LevelFilter::Info,
        // 过滤键与宏的 `target` 比较，不是模块路径。
        targets: vec![("Quiet".into(), log::LevelFilter::Error)],
        rotation: logger::Rotation::BySize { max_bytes: 4096 },
        keep: 5,
        console: logger::Console::Off,
        ..Default::default()
    })
    .expect("初始化日志器");

    log::info!(target: "Loud", "第一条");
    log::info!(target: "Quiet", "第二条应当被 Quiet 过滤掉");
    log::error!(target: "Quiet", "第三条");
    guard.flush();

    // 运行时调级：只改默认级别，`targets` 里的覆盖保持不变。
    logger::set_level(log::LevelFilter::Warn).unwrap();
    log::info!(target: "Loud", "第四条应当被全局级别丢掉");
    log::warn!(target: "Loud", "第五条");
    guard.flush();

    // 手动轮转一次，并枚举当前存在的日志文件。
    guard.rotate_now().unwrap();
    let files = guard.log_files().unwrap();

    let text: String = files
        .iter()
        .map(|path| std::fs::read_to_string(path).unwrap())
        .collect();
    assert!(text.contains("[INFO ] [Loud] 第一条"), "{text}");
    assert!(!text.contains("第二条应当被 Quiet 过滤掉"), "{text}");
    assert!(text.contains("[ERROR] [Quiet] 第三条"), "{text}");
    assert!(!text.contains("第四条应当被全局级别丢掉"), "{text}");
    assert!(text.contains("[WARN ] [Loud] 第五条"), "{text}");

    let names: Vec<String> = files
        .iter()
        .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert!(
        names.iter().any(|name| name.ends_with("_rCURRENT.log")),
        "{names:?}"
    );
    assert!(
        names.iter().any(|name| name.contains("_r00000.log")),
        "{names:?}"
    );
}
