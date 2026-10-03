//! 界面层的资源：完整 lucide 图标集与沿用 PCL 启动器的图片资源。
//!
//! gpui-kit 默认只嵌入少量常用图标，PCL 的界面用到了完整 lucide 名称，因此改用 `AllAssets`；
//! 图片资源放在 `crates/ui/assets/images`，内嵌表由 `build.rs` 扫描该目录生成，这里只做查表。

/// 内嵌图片资源表（键名即 `img("images/…")` 用的路径，按键排序）。
mod index {
    include!(concat!(env!("OUT_DIR"), "/assets.rs"));
}

use gpui_kit::assets::AllAssets;
use gpui_kit::{AssetSource, Result, SharedString};
use std::borrow::Cow;
use std::sync::{Arc, LazyLock};

pub struct Assets;

/// 窗口图标（PNG 字节，与 PCL 启动器的窗口图标是同一张图）。
pub const WINDOW_ICON_PNG: &[u8] = include_bytes!("../../assets/images/icon.png");

/// 解码后的窗口图标，交给 `WindowOptions::icon`；进程内只解码一次。
pub fn window_icon() -> Arc<image::RgbaImage> {
    static ICON: LazyLock<Arc<image::RgbaImage>> = LazyLock::new(|| {
        Arc::new(
            image::load_from_memory(WINDOW_ICON_PNG)
                .expect("窗口图标应当可以解码")
                .into_rgba8(),
        )
    });
    Arc::clone(&ICON)
}

impl Assets {
    /// 按路径取内嵌图片；路径沿用 PCL 启动器的资源命名。
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
