//! 源码引用键校验：核对 `src/`、`tests/` 里出现的文案键字面量，缺失即中断构建；
//! 纯检查，不产出生成文件。

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use crate::{collect_files, fail, is_table_key};

/// `check_referenced_keys` 要跳过的字面量：`src/i18n.rs` 的缺键测试
/// （`missing_key_panics_in_debug`）故意引用不存在的键。
const SKIP_LITERALS: &[&str] = &["No.Such.Key"];

/// 源码里出现的文案键必须存在：键名写错时界面上只会显示键名，这里提前拦住。
pub(super) fn check_referenced_keys(manifest: &Path, keys: &BTreeSet<String>) {
    let mut sources = Vec::new();
    collect_files(&manifest.join("src"), &mut sources);
    collect_files(&manifest.join("tests"), &mut sources);
    sources.sort();

    let mut missing: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for path in sources {
        if path.extension().is_none_or(|extension| extension != "rs") {
            continue;
        }
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|error| fail(format!("读不到 {}：{error}", path.display())));
        let file = path
            .strip_prefix(manifest)
            .unwrap_or(&path)
            .display()
            .to_string();
        for (index, line) in text.lines().enumerate() {
            for literal in string_literals(line) {
                if !is_table_key(&literal) || SKIP_LITERALS.contains(&literal.as_str()) {
                    continue;
                }
                if !keys.contains(&literal) {
                    missing
                        .entry(literal)
                        .or_default()
                        .push(format!("{file}:{}", index + 1));
                }
            }
        }
    }

    if !missing.is_empty() {
        let report: Vec<String> = missing
            .iter()
            .map(|(key, places)| format!("  {key}（{}）", places.join("、")))
            .collect();
        fail(format!("文案键不存在：\n{}", report.join("\n")));
    }
}

/// 一行源码里的字符串字面量内容；文案键不会跨行，因此不处理跨行字面量。
fn string_literals(line: &str) -> Vec<String> {
    let mut literals = Vec::new();
    let mut current: Option<String> = None;
    let mut escaped = false;
    for character in line.chars() {
        match current.as_mut() {
            Some(text) => {
                if escaped {
                    text.push(character);
                    escaped = false;
                } else if character == '\\' {
                    escaped = true;
                } else if character == '"' {
                    literals.push(std::mem::take(text));
                    current = None;
                } else {
                    text.push(character);
                }
            }
            None => {
                if character == '"' {
                    current = Some(String::new());
                }
            }
        }
    }
    literals
}
