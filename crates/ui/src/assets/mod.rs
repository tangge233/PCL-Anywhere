//! 界面层的资源源：完整 lucide 图标集 + 迁移过来的图片资源。
//!
//! gpui-kit 默认只嵌入少量常用图标，PCL 的界面用到了完整 lucide 名称，因此改用 `AllAssets`；
//! 图片资源（`images/**`）放在 `crates/ui/assets/images`，内嵌表由 `tools/gen-assets.py` 生成。

mod index;

use gpui_kit::assets::AllAssets;
use gpui_kit::{AssetSource, Result, SharedString};
use std::borrow::Cow;

pub struct Assets;

/// 窗口图标（PNG 字节，与 .NET 版本 `Assets/Images/icon.ico` 同一张图）。
pub const WINDOW_ICON_PNG: &[u8] = include_bytes!("../../assets/images/icon.png");

/// 解码后的窗口图标，交给 `WindowOptions::icon`。
pub fn window_icon() -> std::sync::Arc<image::RgbaImage> {
    let image = image::load_from_memory(WINDOW_ICON_PNG)
        .expect("窗口图标应当可以解码")
        .into_rgba8();
    std::sync::Arc::new(image)
}

impl Assets {
    /// 按路径取内嵌图片；路径与 .NET 版本的 `avares://PCL.Ui/Assets/…` 同名。
    fn embedded_image(path: &str) -> Option<&'static [u8]> {
        index::EMBEDDED
            .binary_search_by(|(key, _)| (*key).cmp(path))
            .ok()
            .map(|found| index::EMBEDDED[found].1)
    }
}

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if let Some(bytes) = Self::embedded_image(path) {
            return Ok(Some(Cow::Borrowed(bytes)));
        }
        AllAssets.load(path)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut paths = AllAssets.list(path)?;
        paths.extend(
            index::EMBEDDED
                .iter()
                .map(|(key, _)| *key)
                .filter(|key| key.starts_with(path))
                .map(SharedString::from),
        );
        Ok(paths)
    }
}
