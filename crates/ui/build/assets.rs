//! 图片资源表生成：递归收集 `assets/images` 下的文件，写出内嵌图片表 `assets.rs`；不做文案键校验。

use std::fmt::Write as _;
use std::path::Path;

use crate::{collect_files, fail, write};

/// 生成内嵌图片资源表，键名即界面 `img("images/…")` 用的路径。
pub(super) fn write_asset_table(manifest: &Path, out: &Path) {
    let root = manifest.join("assets/images");
    let mut files = Vec::new();
    collect_files(&root, &mut files);
    files.sort();
    if files.is_empty() {
        fail(format!("{} 下没有图片资源", root.display()));
    }

    let mut source = String::from(
        "// 由 build.rs 依据 assets/images 生成，请勿手工编辑。\n\n#[rustfmt::skip]\npub static EMBEDDED: &[(&str, &[u8])] = &[\n",
    );
    for path in &files {
        let relative = path
            .strip_prefix(&root)
            .expect("图片在 images 下")
            .to_str()
            .expect("图片路径是 UTF-8");
        code_ln!(
            source,
            "    (\"images/{relative}\", include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/assets/images/{relative}\"))),"
        );
    }
    source.push_str("];\n");

    write(out, "assets.rs", source);
}
