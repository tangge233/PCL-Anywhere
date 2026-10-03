//! 构建期生成 `i18n.rs`（`Locale` 枚举与各语言文案表）与 `assets.rs`（内嵌图片表），
//! 并校验文案 JSON 与源码里的键引用；运行时文案查询不在本文件，在 `src/i18n.rs`。

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::exit;

use crate::assets::write_asset_table;
use crate::i18n::write_copy_tables;
use crate::source_keys::check_referenced_keys;

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("缺少 CARGO_MANIFEST_DIR"));
    let out = PathBuf::from(env::var("OUT_DIR").expect("缺少 OUT_DIR"));
    for input in ["i18n", "assets", "src", "tests"] {
        println!("cargo:rerun-if-changed={input}");
    }

    let keys = write_copy_tables(&manifest, &out);
    write_asset_table(&manifest, &out);
    check_referenced_keys(&manifest, &keys);
}

/// 往生成的源码里写一行；写 `String` 不会失败。
macro_rules! code_ln {
    ($source:expr, $($arg:tt)*) => {
        let _ = writeln!($source, $($arg)*);
    };
}

/// 校验失败：写清原因并中断构建。
fn fail(message: impl AsRef<str>) -> ! {
    eprintln!("构建脚本：{}", message.as_ref());
    exit(1);
}

fn write(out: &Path, name: &str, source: String) {
    let path = out.join(name);
    fs::write(&path, source)
        .unwrap_or_else(|error| fail(format!("写不了 {}：{error}", path.display())));
}

/// 文案键的形状：大写字母开头的点分标识符，段内只能用字母、数字、下划线。
///
/// 与源码里 `i18n::lang("…")` 的字面量同形，`check_referenced_keys` 才能逐字匹配；
/// 大写开头同时也是不把 `a1.2.6` 这类版本号字面量误判成键的前提。
fn is_table_key(key: &str) -> bool {
    key.contains('.')
        && key.starts_with(|c: char| c.is_ascii_uppercase())
        && key.split('.').all(|segment| {
            !segment.is_empty()
                && segment
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_')
        })
}

/// 递归收集目录下的文件。
fn collect_files(directory: &Path, files: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) => fail(format!("读不到 {}：{error}", directory.display())),
    };
    for entry in entries.filter_map(|entry| entry.ok()) {
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, files);
        } else {
            files.push(path);
        }
    }
}

// Cargo 只把 `build.rs` 编成构建脚本的 crate 根，`build/` 下的子模块用 `#[path]` 引入；
// `mod` 声明必须排在 `code_ln!` 之后，宏的文本作用域按声明顺序生效。
#[path = "build/assets.rs"]
mod assets;
#[path = "build/i18n.rs"]
mod i18n;
#[path = "build/source_keys.rs"]
mod source_keys;
