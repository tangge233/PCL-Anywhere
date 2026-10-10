//! 文案表生成：读入并校验 `i18n/*.json`，写出 `i18n.rs`（`Locale` 枚举与各语言文案表）；
//! 源码键引用与图片表不在本模块。

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde::de::{Error as _, MapAccess, Visitor};

use crate::{fail, is_table_key, write};

const LANGUAGE_METADATA: &str = "$language";
const LANGUAGE_TABLE_NAME: &str = "LangTable";

struct Language {
    code: String,
    name: String,
    is_default: bool,
    /// 键 → 文案，按键排序。
    entries: Vec<(String, String)>,
}

/// JSON 对象的键值对，保留重复键以便报错（`serde_json` 默认会静默覆盖）。
struct Pairs(Vec<(String, serde_json::Value)>);

impl<'de> Deserialize<'de> for Pairs {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct PairsVisitor;

        impl<'de> Visitor<'de> for PairsVisitor {
            type Value = Pairs;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("文案表的 JSON 对象")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Pairs, A::Error> {
                let mut pairs: Vec<(String, serde_json::Value)> = Vec::new();
                while let Some((key, value)) = map.next_entry::<String, serde_json::Value>()? {
                    if pairs.iter().any(|(existing, _)| existing == &key) {
                        return Err(A::Error::custom(format!("重复的键 {key}")));
                    }
                    pairs.push((key, value));
                }
                Ok(Pairs(pairs))
            }
        }

        deserializer.deserialize_map(PairsVisitor)
    }
}

/// 生成 `Locale` 枚举与各语言的文案表，返回全部文案键。
pub(super) fn write_locale(manifest: &Path, out: &Path) -> BTreeSet<String> {
    let languages = read_languages(manifest);
    let default = languages
        .iter()
        .find(|language| language.is_default)
        .expect("已校验：恰好一个缺省语言");
    let mut rest: Vec<&Language> = languages
        .iter()
        .filter(|language| !language.is_default)
        .collect();
    rest.sort_by(|a, b| a.code.cmp(&b.code));
    let ordered: Vec<&Language> = std::iter::once(default).chain(rest).collect();

    let mut source = String::from("// 由 build.rs 依据 i18n/*.json 生成，请勿手工编辑。\n\n");
    source.push_str("#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]\npub enum Locale {\n");
    for language in &ordered {
        code_ln!(source, "    /// {}（{}）", language.name, language.code);
        code_ln!(source, "    {},", variant(&language.code));
    }
    source.push_str("}\n\nimpl Locale {\n");

    let variants: Vec<String> = ordered
        .iter()
        .map(|language| format!("Locale::{}", variant(&language.code)))
        .collect();
    code_ln!(
        source,
        "    /// 全部受支持的语言，数组顺序即语言列表的展示顺序。"
    );
    code_ln!(
        source,
        "    pub const ALL: [Locale; {}] = [{}];\n",
        variants.len(),
        variants.join(", ")
    );
    source.push_str("    /// 缺省语言，也是切换逻辑接入前的固定语言。\n");
    code_ln!(
        source,
        "    pub const DEFAULT: Locale = Locale::{};\n",
        variant(&default.code)
    );

    for (index, (doc, method, values)) in [
        (
            "语言代码",
            "code",
            ordered.iter().map(|l| l.code.clone()).collect::<Vec<_>>(),
        ),
        (
            "原生名称",
            "native_name",
            ordered.iter().map(|l| l.name.clone()).collect::<Vec<_>>(),
        ),
    ]
    .into_iter()
    .enumerate()
    {
        if index > 0 {
            source.push('\n');
        }
        code_ln!(source, "    /// 语言的{doc}。");
        code_ln!(
            source,
            "    pub fn {method}(self) -> &'static str {{\n        match self {{"
        );
        for (language, value) in ordered.iter().zip(values) {
            code_ln!(
                source,
                "            Locale::{} => {},",
                variant(&language.code),
                rust_string(&value)
            );
        }
        source.push_str("        }\n    }\n");
    }
    source.push_str("}\n\n");

    for language in &ordered {
        code_ln!(
            source,
            "#[rustfmt::skip]\nstatic {}_ENTRIES: &[(&str, &str)] = &[",
            upper(&language.code)
        );
        for (key, text) in &language.entries {
            code_ln!(source, "    ({}, {}),", rust_string(key), rust_string(text));
        }
        source.push_str("];\n\n");
    }
    for language in &ordered {
        code_ln!(
            source,
            "static {}: std::sync::LazyLock<{}> = std::sync::LazyLock::new(|| load({}_ENTRIES));",
            upper(&language.code),
            LANGUAGE_TABLE_NAME,
            upper(&language.code)
        );
    }
    code_ln!(source, "/// 取一个语言的文案表，首次查询时构建。");
    code_ln!(
        source,
        "fn table(locale: Locale) -> &'static {LANGUAGE_TABLE_NAME}"
    );
    source.push_str(" {\n    match locale {");
    for language in &ordered {
        code_ln!(
            source,
            "        Locale::{} => &{},",
            variant(&language.code),
            upper(&language.code)
        );
    }
    source.push_str("    }\n}\n");

    write(out, "i18n.rs", source);
    languages
        .iter()
        .flat_map(|language| language.entries.iter().map(|(key, _)| key.clone()))
        .collect()
}

/// 读入并校验 `i18n/` 下的语言文件。
fn read_languages(manifest: &Path) -> Vec<Language> {
    let directory = manifest.join("i18n");
    let mut files: Vec<PathBuf> = fs::read_dir(&directory)
        .unwrap_or_else(|error| fail(format!("读不到 {}：{error}", directory.display())))
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect();
    files.sort();
    if files.is_empty() {
        fail(format!("{} 下没有语言文件", directory.display()));
    }

    let mut languages = Vec::new();
    for path in files {
        let file = path
            .file_name()
            .and_then(|name| name.to_str())
            .expect("语言文件名是 UTF-8")
            .to_owned();
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|error| fail(format!("读不到 {}：{error}", path.display())));
        let Pairs(pairs) = serde_json::from_str(&text)
            .unwrap_or_else(|error| fail(format!("{file} 不是合法的键值 JSON：{error}")));

        let mut metadata = None;
        let mut entries: BTreeMap<String, String> = BTreeMap::new();
        for (key, value) in pairs {
            if key == LANGUAGE_METADATA {
                metadata = Some(value);
                continue;
            }
            if key.starts_with('$') {
                fail(format!(
                    "{file}: 只有 {LANGUAGE_METADATA} 可以以 $ 开头，收到 {key}"
                ));
            }
            if !is_table_key(&key) {
                fail(format!(
                    "{file}: 键 {key} 必须是大写字母开头的点分标识符（段内只能用字母、数字、下划线），以便在源码里逐字校验引用"
                ));
            }
            let Some(text) = value.as_str() else {
                fail(format!("{file}: 键 {key} 的值必须是字符串"));
            };
            if text.trim().is_empty() {
                fail(format!("{file}: 键 {key} 的文案为空"));
            }
            entries.insert(key, text.to_owned());
        }

        let Some(metadata) = metadata else {
            fail(format!("{file}: 缺少 {LANGUAGE_METADATA} 元数据"));
        };
        let object = metadata
            .as_object()
            .unwrap_or_else(|| fail(format!("{file}: {LANGUAGE_METADATA} 必须是对象")));
        let field = |name: &str| {
            object
                .get(name)
                .and_then(|value| value.as_str())
                .unwrap_or_else(|| {
                    fail(format!("{file}: {LANGUAGE_METADATA} 缺少字符串字段 {name}"))
                })
                .to_owned()
        };
        let code = field("code");
        let name = field("name");
        let is_default = object
            .get("default")
            .and_then(|value| value.as_bool())
            .unwrap_or(false);
        if file != format!("{code}.json") {
            fail(format!("{file}: 文件名必须与语言代码一致（{code}.json）"));
        }

        languages.push(Language {
            code,
            name,
            is_default,
            entries: entries.into_iter().collect(),
        });
    }

    let defaults: Vec<&Language> = languages
        .iter()
        .filter(|language| language.is_default)
        .collect();
    match defaults.len() {
        1 => {}
        0 => fail("没有任何语言标记 default: true"),
        count => fail(format!("有 {count} 个语言标记了 default: true，只能有一个")),
    }

    // 以缺省语言为基准，一次报出其余语言的漏翻与多写。
    let reference = languages
        .iter()
        .find(|language| language.is_default)
        .expect("已校验：恰好一个缺省语言");
    for language in languages.iter().filter(|language| !language.is_default) {
        let missing: Vec<&str> = reference
            .entries
            .iter()
            .map(|(key, _)| key.as_str())
            .filter(|key| !language.entries.iter().any(|(other, _)| other == key))
            .collect();
        let extra: Vec<&str> = language
            .entries
            .iter()
            .map(|(key, _)| key.as_str())
            .filter(|key| !reference.entries.iter().any(|(other, _)| other == key))
            .collect();
        if !missing.is_empty() || !extra.is_empty() {
            let mut report = format!("{} 与 {} 的键集不一致：", language.code, reference.code);
            if !missing.is_empty() {
                write!(
                    report,
                    "\n  {} 缺 {} 个：{}",
                    language.code,
                    missing.len(),
                    missing.join("、")
                )
                .expect("写入字符串");
            }
            if !extra.is_empty() {
                write!(
                    report,
                    "\n  {} 多 {} 个：{}",
                    language.code,
                    extra.len(),
                    extra.join("、")
                )
                .expect("写入字符串");
            }
            fail(report);
        }
    }

    languages
}

// ---------------------------------------------------------------- 生成用的小工具

/// 语言代码 → 枚举变体名：`zh-CN` → `ZhCn`。
fn variant(code: &str) -> String {
    code.split(['-', '_'])
        .map(|part| {
            let mut characters = part.chars();
            match characters.next() {
                Some(first) => {
                    first.to_uppercase().collect::<String>() + &characters.as_str().to_lowercase()
                }
                None => String::new(),
            }
        })
        .collect()
}

/// 语言代码 → 静态表名：`zh-CN` → `ZH_CN`。
fn upper(code: &str) -> String {
    code.replace(['-', '_'], "_").to_uppercase()
}

/// 文案 → Rust 字符串字面量。
fn rust_string(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for character in text.chars() {
        match character {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            control if control < ' ' || control == '\u{7f}' => {
                write!(out, "\\u{{{:x}}}", control as u32).expect("写入字符串");
            }
            other => out.push(other),
        }
    }
    out.push('"');
    out
}
