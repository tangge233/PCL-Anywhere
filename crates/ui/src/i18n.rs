//! 界面文案。
//!
//! 文案源是 `crates/ui/i18n/<语言代码>.json`：扁平的「键 → 文案」JSON 对象，键名沿用 PCL 启动器的
//! 命名（`Launch.*` / `Setup.*` / `Instance.*` 等），文件名就是语言代码（`zh-CN` / `en-US`）。
//! 文件由 `include_str!` 在编译期打进二进制，首次取用时解析成只读表，之后每次查询只做一次哈希查找。
//!
//! 查询表用 `HashMap`：界面渲染期间会有成百次读取，运行期从不遍历，用不到 `BTreeMap` 的有序性；
//! 文案键是带长公共前缀的标识符，哈希查找比有序表二分更少比较。表在首次使用后不再变化，
//! 将来语言数量或键数大幅增长时，可以换成编译期完美哈希（`phf`）或更快的哈希器（`rustc-hash`），
//! 本模块的接口不变。
//!
//! 语言切换逻辑与配置系统尚未实现：[`lang`] 固定读取 [`Locale::DEFAULT`]；需要按指定语言取文案时
//! 用 [`lang_in`]，切换逻辑接入后只需改 [`lang`] 的实现。
//!
//! 新增或修改文案直接编辑 JSON：`tools/check-i18n.py` 校验各语言文件的键格式与界面代码引用的键，
//! 单元测试保证各语言的键集完全一致。

use std::collections::HashMap;
use std::sync::LazyLock;

use gpui_kit::SharedString;

/// 受支持的界面语言。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Locale {
    /// 简体中文（中国大陆）。
    ZhCn,
    /// 英语（美国）。
    EnUs,
}

impl Locale {
    /// 全部受支持的语言，顺序即语言列表的展示顺序。
    pub const ALL: [Locale; 2] = [Locale::ZhCn, Locale::EnUs];

    /// 缺省语言，也是切换逻辑接入前的固定语言。
    pub const DEFAULT: Locale = Locale::ZhCn;

    /// 语言代码，与文案文件名一致。
    pub fn code(self) -> &'static str {
        match self {
            Locale::ZhCn => "zh-CN",
            Locale::EnUs => "en-US",
        }
    }

    /// 语言的原生名称，语言列表展示的就是它。
    pub fn native_name(self) -> &'static str {
        match self {
            Locale::ZhCn => "简体中文",
            Locale::EnUs => "English (US)",
        }
    }

    /// 内嵌的文案源文件。
    fn source(self) -> &'static str {
        match self {
            Locale::ZhCn => include_str!("../i18n/zh-CN.json"),
            Locale::EnUs => include_str!("../i18n/en-US.json"),
        }
    }
}

/// 一个语言的文案表。
///
/// 键借自内嵌的 JSON（键名不含转义，可以零拷贝借用）；值是解析时构造好的共享字符串，
/// 查询时的克隆只是引用计数自增，不会重新分配。
type Table = HashMap<&'static str, SharedString>;

static ZH_CN: LazyLock<Table> = LazyLock::new(|| load(Locale::ZhCn));
static EN_US: LazyLock<Table> = LazyLock::new(|| load(Locale::EnUs));

/// 取一个语言的文案表，首次查询时才解析内嵌的 JSON。
fn table(locale: Locale) -> &'static Table {
    match locale {
        Locale::ZhCn => &ZH_CN,
        Locale::EnUs => &EN_US,
    }
}

/// 解析内嵌的文案源文件。
fn load(locale: Locale) -> Table {
    serde_json::from_str(locale.source())
        .unwrap_or_else(|error| panic!("语言文件 {} 不是合法的键值 JSON：{error}", locale.code()))
}

/// 取当前语言的一条文案。
pub fn lang(key: &str) -> SharedString {
    lang_in(Locale::DEFAULT, key)
}

/// 取指定语言的一条文案。
///
/// 键缺失时返回键名本身：界面上一眼能看出漏配，且不会因为缺一条文案而崩溃。
/// 调试构建下会同时断言失败，让漏配在开发阶段就暴露。
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
///
/// 占位符数量不足时保留原样（例如 `Main.Title.InstanceSetup` 为 `实例设置 - {0}`）。
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

    /// 各语言的键集必须完全一致：漏配的键会以键名的形式直接出现在界面上。
    #[test]
    fn locales_cover_the_same_keys() {
        let reference = table(Locale::DEFAULT);
        for locale in Locale::ALL {
            let other = table(locale);
            assert_eq!(
                other.len(),
                reference.len(),
                "{} 的键数与缺省语言不一致",
                locale.code()
            );
            for key in reference.keys() {
                assert!(
                    other.contains_key(key),
                    "{} 缺少文案键 {key}",
                    locale.code()
                );
            }
        }
    }

    /// 每种语言都能取到真实文案，而不是回退成键名；两种语言的文案也确实不同。
    #[test]
    fn known_keys_resolve_in_every_locale() {
        for key in [
            "Setup.Left.Item.Ui",
            "Main.PageLaunch.Launch",
            "Setup.Language.Title",
        ] {
            for locale in Locale::ALL {
                let text = lang_in(locale, key);
                assert_ne!(
                    text.as_ref(),
                    key,
                    "{} 的 {key} 没解析成文案",
                    locale.code()
                );
                assert!(
                    !text.trim().is_empty(),
                    "{} 的 {key} 是空文案",
                    locale.code()
                );
            }
            assert_ne!(
                lang_in(Locale::ZhCn, key),
                lang_in(Locale::EnUs, key),
                "{key} 在两种语言下文案相同，en-US 表可能没翻"
            );
        }
    }

    /// 占位符按各语言自己的文案替换。
    #[test]
    fn args_are_substituted_in_every_locale() {
        let key = "Main.Title.InstanceSetup";
        for locale in Locale::ALL {
            assert!(
                lang_in(locale, key).contains("{0}"),
                "{} 的 {key} 应带占位符，测试前提不成立",
                locale.code()
            );
            let text = lang_with_args_in(locale, key, &["1.20.1"]);
            assert!(
                text.contains("1.20.1"),
                "{} 没替换占位符：{text}",
                locale.code()
            );
            assert!(
                !text.contains("{0}"),
                "{} 仍留着占位符：{text}",
                locale.code()
            );
        }
    }

    /// 漏配的键在调试构建下直接断言失败，避免带着键名发布。
    #[test]
    #[cfg_attr(not(debug_assertions), ignore = "只在调试构建下断言")]
    #[should_panic(expected = "缺少文案键")]
    fn missing_key_is_loud_in_debug_builds() {
        lang("No.Such.Key");
    }
}
