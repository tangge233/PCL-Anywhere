//! 界面文案。
//!
//! 键名与 PCL.Core 的语言文件一致；数值位于 [`zh_cn`]，由 `tools/gen-i18n.py` 从
//! `PCL.Core/App/Localization/Languages/zh-CN.xaml` 生成。后续独立的本地化 crate 接管后，
//! 只需替换本模块的实现。

mod zh_cn;

pub use zh_cn::ZH_CN;

use gpui_kit::SharedString;

/// 取一条中文文案。
///
/// 键缺失时返回键名本身：界面上一眼能看出漏配，且不会因为缺一条文案而崩溃。
pub fn text(key: &str) -> SharedString {
    match ZH_CN.binary_search_by(|(candidate, _)| candidate.cmp(&key)) {
        Ok(index) => SharedString::from(ZH_CN[index].1),
        Err(_) => {
            debug_assert!(false, "缺少文案键: {key}");
            SharedString::from(key)
        }
    }
}

/// 取一条带占位符的文案，把 `{0}`、`{1}`… 依次替换为 `args`。
///
/// 占位符数量不足时保留原样（例如 `Main.Title.InstanceSetup` 为 `实例设置 - {0}`）。
pub fn text_args(key: &str, args: &[&str]) -> SharedString {
    let template = text(key);
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

    #[test]
    fn table_is_sorted_for_binary_search() {
        assert!(ZH_CN.windows(2).all(|pair| pair[0].0 < pair[1].0));
    }

    #[test]
    fn known_key_resolves() {
        assert_eq!(text("Setup.Left.Item.Ui"), SharedString::from("个性化"));
    }
}
