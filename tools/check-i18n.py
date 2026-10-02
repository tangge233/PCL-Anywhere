#!/usr/bin/env python3
"""Check the interface copy tables under `crates/ui/i18n/`.

The tables are hand-maintained JSON objects (`<language code>.json`, a flat
`key -> text` map). This script guards the invariants that the Rust side relies
on:

* every file is a flat JSON object of string values, keys are sorted, unique,
  and contain no character JSON would have to escape (keys are borrowed from
  the embedded source, so an escape would fail to deserialize);
* every key referenced by interface code exists in the tables.

Key set parity between languages is asserted by the Rust tests
(`cargo test -p pcl-ui`), which own the loaded tables.

Usage:
    python3 tools/check-i18n.py --check
"""

from __future__ import annotations

import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
TABLE_DIR = ROOT / "crates/ui/i18n"
UI_SRC = ROOT / "crates/ui/src"

# 键名：以字母开头的点分标识符，例如 `Launch.Button.Start`、`Minecraft.Fool.Description.2013`。
KEY = re.compile(r"^[A-Za-z][A-Za-z0-9_]*(?:\.[A-Za-z0-9_]+)+$")
# 界面源码里出现的文案键字面量：`i18n::lang("…")`、`choices(&["…"])` 等一律按形状识别。
# 键一律以大写字母开头（`Minecraft.Fool.Description.2013` 也是），因此版本号之类的
# 小写字面量（`a1.2.6`）不会被误判成键。
LITERAL = re.compile(r'"([A-Z][A-Za-z0-9_]*(?:\.[A-Za-z0-9_]+)+)"')


def fail(message: str) -> None:
    print(message, file=sys.stderr)


def load_table(path: pathlib.Path) -> dict[str, str]:
    """读入一个文案表，顺带检查 JSON 层面的约束。"""
    pairs: list[tuple[str, str]] = []

    def collect(pairs_in_object: list[tuple[str, str]]) -> dict[str, str]:
        pairs.extend(pairs_in_object)
        return dict(pairs_in_object)

    table = json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=collect)
    keys = [key for key, _ in pairs]
    name = path.relative_to(ROOT)

    duplicates = sorted({key for key in keys if keys.count(key) > 1})
    if duplicates:
        fail(f"{name}: 重复的键 {', '.join(duplicates)}")
    unsorted = [a for a, b in zip(keys, keys[1:]) if a >= b]
    if unsorted:
        fail(f"{name}: 键未按字典序排列，从 {unsorted[0]} 开始")
    for key, value in pairs:
        if not KEY.match(key):
            fail(f"{name}: 键 {key!r} 不符合点分标识符格式（JSON 转义会让键无法零拷贝借用）")
        if not isinstance(value, str):
            fail(f"{name}: 键 {key} 的值不是字符串")
        elif not value.strip():
            fail(f"{name}: 键 {key} 的文案为空")
    return table


def referenced_keys() -> dict[str, set[str]]:
    """界面代码里出现的文案键字面量，以及各自的引用位置。"""
    references: dict[str, set[str]] = {}
    for path in sorted(UI_SRC.rglob("*.rs")):
        for literal in LITERAL.finditer(path.read_text(encoding="utf-8")):
            key = literal.group(1)
            if key in NO_KEYS:
                continue
            references.setdefault(key, set()).add(str(path.relative_to(ROOT)))
    return references


# 形状像键、但并非文案键的字面量：i18n 模块的测试故意取一个不存在的键。
NO_KEYS: set[str] = {"No.Such.Key"}


def main() -> int:
    paths = sorted(TABLE_DIR.glob("*.json"))
    if not paths:
        fail(f"{TABLE_DIR.relative_to(ROOT)} 下没有文案表")
        return 1

    tables = {path.stem: load_table(path) for path in paths}
    keys = set().union(*(table.keys() for table in tables.values()))

    missing = {
        key: paths
        for key, paths in referenced_keys().items()
        if key not in keys
    }
    for key, places in sorted(missing.items()):
        fail(f"缺少文案键 {key}（引用位置：{', '.join(sorted(places))}）")

    for name, table in sorted(tables.items()):
        print(f"{name}: {len(table)} 键")
    print(f"检查完成：文案键缺失 {len(missing)} 个")
    return 1 if missing else 0


if __name__ == "__main__":
    raise SystemExit(main())
