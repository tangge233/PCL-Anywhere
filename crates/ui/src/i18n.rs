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

    /// 漏配的键在调试构建下直接断言失败。
    #[test]
    #[cfg_attr(not(debug_assertions), ignore = "断言只在调试构建下生效")]
    #[should_panic(expected = "缺少文案键")]
    fn missing_key_panics_in_debug() {
        lang("No.Such.Key");
    }
}
