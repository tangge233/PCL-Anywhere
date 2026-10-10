//! 原子性：崩在任何一步都只留下完整的旧内容或完整的新内容。

mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use common::{Plain, temp_config};
use lite_config::Config;

const TIMEOUT: Duration = Duration::from_secs(5);

/// 子进程要写的目标文件。设了它说明当前进程是被重新拉起来的"写入方"。
const CHILD_TARGET: &str = "PCL_CONFIG_CRASH_CHILD_TARGET";

/// 写入负荷：文档要足够大，"写到一半"才有可观察的窗口。
const PAYLOAD: u32 = 20_000;

const READY_MARKER: &str = ".child-ready";
const CHILD_READY_BUDGET: Duration = Duration::from_secs(15);

fn uniform(seed: u32) -> Vec<u32> {
    vec![seed; PAYLOAD as usize]
}

/// 文件要么解析不了（半截），要么 `payload` 是同一个值（两次写入的混合）。
fn assert_intact(path: &Path, stage: &str) {
    let text = fs::read_to_string(path).expect("能读到文件");
    let config: Plain = toml_edit::de::from_str(&text)
        .unwrap_or_else(|error| panic!("{stage}：文件被写坏了：{error}\n{text}"));
    assert!(
        config.payload.is_empty() || config.payload.iter().all(|v| *v == config.payload[0]),
        "{stage}：读到了两次写入的混合内容"
    );
}

#[test]
fn a_killed_writer_never_leaves_a_partial_file() {
    if let Some(target) = std::env::var_os(CHILD_TARGET) {
        writer_until_killed(&PathBuf::from(target));
    }

    let (_dir, path) = temp_config();
    let cfg = Config::<Plain>::open(&path).expect("打开");
    cfg.mutate(|c| c.payload = uniform(1));
    cfg.persist(TIMEOUT).expect("铺底色");

    // 反复来几轮：每一轮都可能在临时文件写完、rename 之前被打死。
    let ready = path.with_file_name(READY_MARKER);
    let mut child_wrote = false;
    for round in 0..3 {
        let _ = fs::remove_file(&ready);
        let mut child = spawn_child(&path);
        // 等它真写完一次再杀：固定 sleep 会卡在写线程 300ms 防抖的边界上。
        child_wrote |= wait_until_ready(&ready, CHILD_READY_BUDGET);
        // SIGKILL：不给任何清理机会，正是要验的场景。
        child.kill().expect("杀子进程");
        child.wait().expect("收子进程");
        assert_intact(&path, &format!("第 {round} 轮被杀之后"));
    }
    // 子进程一次都没写成的话，这条测试什么都没验到——不能让它悄悄退化成空跑。
    assert!(child_wrote, "子进程在被杀之前一次都没写完，测试无效");
}

/// 被父进程反复 SIGKILL 的写入方：写完一笔就落下标记，然后继续写，直到被杀。
fn writer_until_killed(path: &Path) -> ! {
    let ready = path.with_file_name(READY_MARKER);
    let cfg = Config::<Plain>::open(path).expect("子进程打开");
    let mut seed = 0;
    let mut announced = false;
    loop {
        seed += 1;
        cfg.mutate(|c| c.payload = uniform(seed));
        // persist 每次都触发一次真正的写入，所以子进程绝大多数时间待在写盘里。
        if cfg.persist(TIMEOUT).is_ok() && !announced {
            let _ = fs::write(&ready, b"");
            announced = true;
        }
    }
}

fn wait_until_ready(ready: &Path, budget: Duration) -> bool {
    let deadline = Instant::now() + budget;
    while Instant::now() < deadline {
        if ready.exists() {
            return true;
        }
        thread::sleep(Duration::from_millis(2));
    }
    false
}

fn spawn_child(path: &Path) -> Child {
    Command::new(std::env::current_exe().expect("测试二进制路径"))
        .args([
            "--exact",
            "a_killed_writer_never_leaves_a_partial_file",
            "--test-threads=1",
        ])
        .env(CHILD_TARGET, path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("拉子进程")
}

#[test]
fn readers_never_see_a_partial_file() {
    let (_dir, path) = temp_config();
    let cfg = Config::<Plain>::open(&path).expect("打开");

    let stop = AtomicBool::new(false);
    let reads = AtomicUsize::new(0);

    thread::scope(|scope| {
        let reader = scope.spawn(|| {
            while !stop.load(Ordering::Relaxed) {
                // 读者不取任何锁，就靠 rename 的原子性。
                assert_intact(&path, "并发读");
                reads.fetch_add(1, Ordering::Relaxed);
            }
        });

        for round in 2..=40u32 {
            cfg.mutate(|c| c.payload = uniform(round));
            cfg.persist(TIMEOUT).expect("落盘");
        }

        stop.store(true, Ordering::Relaxed);
        reader.join().expect("读线程");
    });

    assert!(reads.load(Ordering::Relaxed) > 0, "读线程一次都没跑完");
}

#[test]
fn a_successful_write_leaves_no_temporary_file_behind() {
    let (dir, path) = temp_config();
    let cfg = Config::<Plain>::open(&path).expect("打开");
    cfg.mutate(|c| c.memory_gib = 4);
    cfg.persist(TIMEOUT).expect("落盘");

    let names: Vec<String> = fs::read_dir(dir.path())
        .expect("列目录")
        .map(|entry| {
            entry
                .expect("目录项")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    // 临时文件必须同目录（跨设备 rename 会 EXDEV），并且提交后立刻消失。
    assert_eq!(
        names,
        vec!["config.toml".to_string()],
        "目录里多了残留：{names:?}"
    );
}

#[test]
fn a_path_that_cannot_be_written_reports_an_io_error() {
    let dir = tempfile::tempdir().expect("建临时目录");
    // 父路径是文件，不是目录：这是 IO 错误，不是"内容坏了"。
    let blocker = dir.path().join("blocker");
    fs::write(&blocker, b"").expect("占位文件");
    let path = blocker.join("config.toml");

    let error = Config::<Plain>::open(&path).expect_err("写不进去必须报错");
    assert!(
        matches!(error, lite_config::Error::Read { .. }),
        "实际：{error}"
    );

    // 也不能被当成损坏去"重建"——重建同样写不进去。
    let error = Config::<Plain>::open_or_default_replace(&path).expect_err("同样报错");
    assert!(
        matches!(error, lite_config::Error::Read { .. }),
        "实际：{error}"
    );
}
