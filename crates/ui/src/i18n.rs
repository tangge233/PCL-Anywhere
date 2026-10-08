//! 文案查询：按 `Locale` 从 `build.rs` 生成的静态文案表取值，并支持 `{0}` 占位替换。
//!
//! 本文件不负责文案表本身（由 `crates/ui/i18n/*.json` 生成、`build.rs` 构建期校验键集）；
//! 语言切换与配置系统尚未实现，当前语言固定为 `Locale::DEFAULT`。

use std::collections::HashMap;

use gpui_kit::SharedString;

include!(concat!(env!("OUT_DIR"), "/i18n.rs"));

/// 一个语言的文案表。
type LangTable = HashMap<&'static str, SharedString>;

/// 把生成的静态表装进哈希表，只在首次查询时执行。
fn load(entries: &'static [(&'static str, &'static str)]) -> LangTable {
    entries
        .iter()
        .map(|(key, text)| (*key, SharedString::from(*text)))
        .collect()
}

/// 取当前语言的一条文案。
pub fn lang(key: &str) -> SharedString {
    lang_in(Locale::DEFAULT, key)
}

/// 取指定语言的一条文案；键缺失时返回键名本身，调试构建下同时断言失败。
pub fn lang_in(locale: Locale, key: &str) -> SharedString {
    match table(locale).get(key) {
        Some(text) => text.clone(),
        None => {
            debug_assert!(false, "缺少文案键: {key}（语言 {}）", locale.code());
            SharedString::from(key)
        }
    }
}

/// 取当前语言的一条带占位符的文案。
pub fn lang_with_args(key: &str, args: &[&str]) -> SharedString {
    lang_with_args_in(Locale::DEFAULT, key, args)
}

/// 取指定语言的一条带占位符的文案，把 `{0}`、`{1}`… 依次替换为 `args`。
pub fn lang_with_args_in(locale: Locale, key: &str, args: &[&str]) -> SharedString {
    let template = lang_in(locale, key);
    if args.is_empty() || !template.contains('{') {
        return template;
    }
    let mut text = template.to_string();
    for (index, value) in args.iter().enumerate() {
        text = text.replace(&format!("{{{index}}}"), value);
    }
    SharedString::from(text)
}

/// 一段界面文案的来源：字面量、文案键，或带 `{0}`、`{1}` 占位符的文案键。
///
/// 解析推迟到渲染时（[`Text::resolve`]），因此「先构造、后渲染」的结构可以保存文案来源
/// 而不是解析结果，语言切换后已构造的元素跟着变。
///
/// 不收裸 `&str`：它是字面量还是键无法判断，猜错只会静默显示错误文本。键用 [`Text::key`]，
/// 字面量用 [`Text::literal`]；键是编译期常量，因此这里取 `&'static str`。
#[derive(Clone, Debug)]
pub enum Text {
    Literal(SharedString),
    Key(&'static str),
    KeyArgs(&'static str, &'static [&'static str]),
}

impl Text {
    /// 已经确定的文案（含运行期拼出的文本）。
    pub fn literal(text: impl Into<SharedString>) -> Self {
        Self::Literal(text.into())
    }

    /// 文案表里的键。
    pub const fn key(key: &'static str) -> Self {
        Self::Key(key)
    }

    /// 带占位符的文案键，`args` 依次替换 `{0}`、`{1}`…
    pub const fn key_args(key: &'static str, args: &'static [&'static str]) -> Self {
        Self::KeyArgs(key, args)
    }

    /// 取当前语言的文案。
    pub fn resolve(&self) -> SharedString {
        match self {
            Self::Literal(text) => text.clone(),
            Self::Key(key) => lang(key),
            Self::KeyArgs(key, args) => lang_with_args(key, args),
        }
    }
}

impl From<SharedString> for Text {
    fn from(text: SharedString) -> Self {
        Self::Literal(text)
    }
}

impl From<String> for Text {
    fn from(text: String) -> Self {
        Self::Literal(SharedString::from(text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 每种语言都取得到真实文案，而不是回退成键名。
    #[test]
    fn keys_resolve_in_every_locale() {
        for locale in Locale::ALL {
            for key in ["Setup.Left.Item.Ui", "Main.PageLaunch.Launch"] {
                let text = lang_in(locale, key);
                assert_ne!(text.as_ref(), key, "{} 未解析 {key}", locale.code());
            }
        }
    }

    /// `{0}`、`{1}`… 按 `args` 的顺序替换。
    #[test]
    fn args_replace_placeholders() {
        let text = lang_with_args_in(Locale::ZhCn, "Main.Title.InstanceSetup", &["1.20.1"]);
        assert!(text.contains("1.20.1"), "占位符未被替换：{text}");
        assert!(!text.contains("{0}"), "占位符残留：{text}");
    }

    /// 键与字面量各自解析，带参键替换占位符。
    #[test]
    fn text_resolves_keys_and_literals() {
        assert_eq!(
            Text::key("Common.Action.Confirm").resolve().as_ref(),
            lang("Common.Action.Confirm").as_ref()
        );
        assert_eq!(Text::literal("自定义文案").resolve().as_ref(), "自定义文案");
        assert!(
            Text::key_args("Main.Title.InstanceSetup", &["1.20.1"])
                .resolve()
                .contains("1.20.1")
        );
    }

    /// 漏配的键在调试构建下直接断言失败。
    #[test]
    #[cfg_attr(not(debug_assertions), ignore = "断言只在调试构建下生效")]
    #[should_panic(expected = "缺少文案键")]
    fn missing_key_panics_in_debug() {
        lang("No.Such.Key");
    }
}
