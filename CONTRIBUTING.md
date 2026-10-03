# 贡献指南

## 环境

Rust 1.90+ 与 Linux 桌面开发库。界面只支持 Linux / OSX，
不支持 Windows。

## 提交前必须通过的检查

```bash
cargo fmt --all --check
cargo clippy -p pcl-ui -p pcl-app --all-targets   # 0 warning
cargo test -p pcl-ui                              # 单元测试 + 界面集成测试
```

文案表与图片资源表由 `crates/ui/build.rs` 在构建期从 `crates/ui/i18n/*.json` 与
`crates/ui/assets/images` 生成，并顺带校验：键写错、各语言漏翻或多写、JSON 重复键、空文案
都会直接构建失败，不需要额外的检查命令。

界面改动还要真的跑一遍：`cargo run -p pcl-app`，改动到哪一屏就操作到哪一屏；交互类改动补一条
`crates/ui/tests/` 下的界面集成测试（headless 窗口 + 真实事件，参考 `instance_bar.rs`）。

## 代码约定

- **依赖方向**：`app` → `ui` → `gpui-kit`。界面层不得依赖具体业务实现，也不得绕过 gpui-kit 直接用
  gpui / gpui-base / gpui-component。
- **分组即模块**：一个分组一个目录，`mod.rs` 持有状态与路由，渲染按区域拆到同页子模块（例如
  `pages/instance/{mod,merged,manage,saves,state}.rs`）。跨文件共享的项用 `pub(super)`；页面状态
  字段保持私有（私有项对后代模块可见）。
- **颜色与尺寸**：颜色取 `theme::palette(cx)` 的档位色（Gray1–8 / Color1–8）或 `cx.theme()` 的语义色，
  不写十六进制；尺寸用 rem 档位助手（`p_4`、`h_9`、`text_sm` 等），只有资产尺寸、1px 分隔线这类
  物理边界才写 `px(..)` 并注明原因。
- **控件复用**：通用控件在 `components/`（`AppButton` / `IconButton` / `AppCheckBox` / `AppRadio` /
  `Selector` / `Card` / `PageScroll` 等）；gpui-kit 已有的组件直接用，不重造。新增通用控件要先想清楚
  它的契约（构造、受控值、回调、状态、可访问性），不要为单个页面开特例。
- **文案**：`i18n::lang("键")`，键来自 `crates/ui/i18n/<语言代码>.json`（改文案直接改 JSON；新增一种语言
  就是加一份 JSON，`Locale` 枚举与文案表由 `build.rs` 生成，各语言键集必须一致）；按指定语言取文案用
  `i18n::lang_in`，语言切换与配置系统尚未实现。示例数据必须注释说明它是占位。
- **注释**：说明「对应 PCL 的哪个界面 / 为什么这么写」，不要复述代码。

## 提交信息

沿用 Conventional Commits（`feat:` / `fix:` / `refactor:` / `test:` / `docs:` / `ci:`），主题用中文，
与现有历史一致。发版说明由 git-cliff 按 `cliff.toml` 生成。
