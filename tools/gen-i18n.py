#!/usr/bin/env python3
"""Generate the Rust zh-CN string table from the project's localization source.

`crates/ui/i18n/zh-CN.xaml` is the source of truth for interface copy (the keys
and text carried over from the launcher this rewrite is based on). A later
localization crate will replace this table without touching界面 code.

Usage:
    python3 tools/gen-i18n.py          # 重新生成 crates/ui/src/i18n/zh_cn.rs
    python3 tools/gen-i18n.py --check  # 只校验界面代码引用的键是否都存在
"""

from __future__ import annotations

import html
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
SOURCE = ROOT / "crates/ui/i18n/zh-CN.xaml"
TARGET = ROOT / "crates/ui/src/i18n/zh_cn.rs"
UI_SRC = ROOT / "crates/ui/src"

ENTRY = re.compile(r'<sys:String x:Key="(?P<key>[^"]+)">(?P<value>.*?)</sys:String>', re.S)
REFERENCE = re.compile(r'(?:i18n::text|text_args)\(\s*"([^"]+)"')

HEADER = """//! zh-CN 文案表，由 `tools/gen-i18n.py` 从 `crates/ui/i18n/zh-CN.xaml` 生成，请勿手工编辑。
//!
//! 键名沿用 PCL 启动器的命名（`Launch.*` / `Setup.*` / `Instance.*` 等），便于后续由独立的本地化 crate 接管。

"""


def rust_string(value: str) -> str:
    out = ['"']
    for ch in value:
        if ch == "\\":
            out.append("\\\\")
        elif ch == '"':
            out.append('\\"')
        elif ch == "\n":
            out.append("\\n")
        elif ch == "\r":
            out.append("\\r")
        elif ch == "\t":
            out.append("\\t")
        elif ch < " " or ch == "\u007f":
            out.append(f"\\u{{{ord(ch):x}}}")
        else:
            out.append(ch)
    out.append('"')
    return "".join(out)


def check_referenced_keys(keys: set[str]) -> int:
    """校验界面代码引用的文案键都存在（键名写错会在界面上直接暴露，这里提前拦住）。"""
    missing: dict[str, set[str]] = {}
    for path in sorted(UI_SRC.rglob("*.rs")):
        if path.name == "zh_cn.rs":
            continue
        for match in REFERENCE.finditer(path.read_text(encoding="utf-8")):
            key = match.group(1)
            if key not in keys:
                missing.setdefault(key, set()).add(str(path.relative_to(ROOT)))
    for key, paths in sorted(missing.items()):
        print(f"缺少文案键 {key}（引用位置：{', '.join(sorted(paths))}）", file=sys.stderr)
    print(f"检查完成：引用键缺失 {len(missing)} 个")
    return 1 if missing else 0


def main() -> int:
    text = SOURCE.read_text(encoding="utf-8")
    entries: dict[str, str] = {}
    for match in ENTRY.finditer(text):
        key = match.group("key")
        value = html.unescape(match.group("value"))
        entries[key] = value

    if not entries:
        print(f"no entries parsed from {SOURCE}", file=sys.stderr)
        return 1

    if "--check" in sys.argv[1:]:
        return check_referenced_keys(set(entries))

    lines = [HEADER]
    lines.append("#[rustfmt::skip]\n")
    lines.append("pub static ZH_CN: &[(&str, &str)] = &[\n")
    for key in sorted(entries):
        lines.append(f'    ({rust_string(key)}, {rust_string(entries[key])}),\n')
    lines.append("];\n")

    TARGET.parent.mkdir(parents=True, exist_ok=True)
    TARGET.write_text("".join(lines), encoding="utf-8")
    print(f"wrote {len(entries)} keys to {TARGET.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
