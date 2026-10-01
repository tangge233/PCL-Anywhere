#!/usr/bin/env python3
"""生成界面层的内嵌图片资源表。

界面层不额外依赖资源内嵌库：`crates/ui/src/assets/index.rs` 里每个路径对应一条
`include_bytes!`，由 `crates/ui/src/assets/mod.rs` 的 AssetSource 查表返回。
资源本身（`crates/ui/assets/images/**`）直接进版本库，本脚本只负责同步这张表。

用法：
    python3 tools/gen-assets.py          # 按图片目录重新生成资源表
    python3 tools/gen-assets.py --check  # 只校验资源表与图片目录是否一致
"""

from __future__ import annotations

import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
SOURCE = ROOT / "crates/ui/assets/images"
TARGET = ROOT / "crates/ui/src/assets/index.rs"
# 资源路径前缀：界面用 `img("images/Blocks/Anvil.png")` 引用。
PREFIX = "images"

HEADER = """//! 内嵌图片资源表，由 `tools/gen-assets.py` 按 `crates/ui/assets/images` 生成，请勿手工编辑。
//!
//! 键名即资源路径（`images/Blocks/Anvil.png` 等），界面用 `img("images/Blocks/Anvil.png")` 引用；
//! 查表见 `assets::Assets`。

#[rustfmt::skip]
pub static EMBEDDED: &[(&str, &[u8])] = &[
"""


def collect() -> list[pathlib.Path]:
    return sorted(path for path in SOURCE.rglob("*") if path.is_file())


def relative(path: pathlib.Path) -> str:
    return f"{PREFIX}/{path.relative_to(SOURCE).as_posix()}"


def main() -> int:
    files = collect()
    if not files:
        print(f"没有找到图片资源：{SOURCE}", file=sys.stderr)
        return 1

    if "--check" in sys.argv[1:]:
        if not TARGET.exists():
            print(f"缺少生成文件：{TARGET}", file=sys.stderr)
            return 1
        text = TARGET.read_text(encoding="utf-8")
        missing = [relative(path) for path in files if f'("{relative(path)}"' not in text]
        for path in missing:
            print(f"资源表缺少 {path}", file=sys.stderr)
        print(f"检查完成：缺少 {len(missing)} 项")
        return 1 if missing else 0

    lines = [HEADER]
    for path in files:
        key = relative(path)
        # 生成文件位于 crates/ui/src/assets/，相对路径指回 crates/ui/assets/。
        lines.append(f'    ("{key}", include_bytes!("../../assets/images/{path.relative_to(SOURCE).as_posix()}")),\n')
    lines.append("];\n")

    TARGET.parent.mkdir(parents=True, exist_ok=True)
    TARGET.write_text("".join(lines), encoding="utf-8")
    print(f"生成 {TARGET.relative_to(ROOT)}：{len(files)} 个图片资源")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
