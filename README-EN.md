<h1 align="center">PCL-Anywhere</h1>

<p align="center">
  <b>A Minecraft launcher for Linux / XOS</b><br>
  The interface is rebuilt from the interaction and visual language of the <a href="https://github.com/PCL-Community/PCL-CE">PCL launcher</a> (Plain Craft Launcher 2), implemented from scratch in Rust with <a href="https://gpui.rs">GPUI</a> (through <a href="https://gpui-kit.com">gpui-kit</a>).
</p>

<p align="center">
  <a href="https://github.com/tangge233/PCL-Anywhere/stargazers"><img alt="Stars" src="https://img.shields.io/github/stars/tangge233/PCL-Anywhere?style=flat&label=Stars&color=yellow&logo=data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyNCAyNCI+PHBhdGggZmlsbD0iI2Y1YzUxOCIgZD0iTTEyIDIuNGwzLjEgNi4zIDYuOSAxLTUgNC45IDEuMiA2LjlMMTIgMTguMiA1LjggMjEuNSA3IDE0LjZsLTUtNC45IDYuOS0xTDEyIDIuNHoiLz48L3N2Zz4="></a>
  <a href="https://github.com/tangge233/PCL-Anywhere/issues"><img alt="Issues" src="https://img.shields.io/github/issues/tangge233/PCL-Anywhere?style=flat&label=Issues&color=green&logo=github"></a>
  <a href="https://github.com/tangge233/PCL-Anywhere/releases"><img alt="Release" src="https://img.shields.io/github/v/release/tangge233/PCL-Anywhere?style=flat&label=Release&color=blue&logo=github"></a>
  <a href="https://github.com/tangge233/PCL-Anywhere/releases"><img alt="Downloads" src="https://img.shields.io/github/downloads/tangge233/PCL-Anywhere/total?style=flat&label=Downloads&color=orange&logo=github"></a>
</p>

<p align="center">
  <a href="LICENSE"><img alt="License" src="https://img.shields.io/badge/License-GPL--3.0-blue?style=flat"></a>
  <img alt="Platform" src="https://img.shields.io/badge/Platform-Linux%20%2F%20XOS-lightgrey?style=flat&logo=linux">
</p>

---

```bash
cargo run -p pcl-app          # run the UI
cargo test -p pcl-ui          # unit tests + UI integration tests (headless windows)
```

> **UI reference**: layout, colours, control shapes and copy follow PCL's implementation and
> interface conventions (selector column plus content column, card and list-row geometry, colour
> tiers). The code is an independent Rust rewrite and does not copy PCL's source. The .NET
> implementation used as the reference during the port lives on the `dotnet-archive` branch; the
> `PCL/…` and `PCL.Core/…` paths mentioned in code comments refer to it.

## What this is

- **An interface-first rewrite**: all five of PCL's groups are already in place in Rust (theme,
  reusable controls, window shell), and the logic — launching, downloading, accounts, instance
  scanning — is wired in as the future `crates/core` lands.
- **A design language, not a screenshot copy**: colours are computed from PCL's `ToneProfile`
  through OKLCH (the same algorithm as the .NET implementation, checked value by value against
  Wacton.Unicolour 8.0.0), copy keys keep PCL's naming, control geometry comes from PCL's control
  definitions.
- **Verifiable**: formatting, lints, unit tests, headless UI integration tests, plus consistency
  checks for the copy and asset tables — all runnable from `cargo` and two dependency-free scripts.

## Status

All five interface groups are ported; **business logic is not wired up yet** (launching,
downloading, accounts and instance scanning will live in a future `crates/core`, so the screens
currently render sample data):

| Group | Screens |
| --- | --- |
| Launch | Login panel (profile / method picker / Microsoft device code / third-party / offline), launch button with version line, launching panel, community hint and launch log cards |
| Download | Version list (latest / release / snapshot / old, search, collapsible categories) and install panel (compatibility hints, 11 loader cards, install info and progress); other download entries render placeholders |
| Settings | 11 categories (launch / Java / game management / multiplayer / personalisation / language / misc / about / update / feedback / log): 30 groups, 166 items |
| Tools | Multiplayer (two states) and the toolbox page |
| Instance | Instance detail and instance settings (folders / instances / management, three columns with a 250 ms slide) plus save management |

## Build and run

Rust 1.90+ and the Linux desktop development libraries:

```bash
# Debian / Ubuntu
sudo apt install libxkbcommon-dev libxkbcommon-x11-dev libwayland-dev libx11-xcb-dev \
                 libfontconfig-dev libvulkan-dev libssl-dev
# Fedora
sudo dnf install libxkbcommon-devel libxkbcommon-x11-devel wayland-devel libxcb-devel \
                 fontconfig-devel vulkan-loader-devel openssl-devel
```

```bash
cargo run -p pcl-app            # run the UI
cargo test -p pcl-ui            # unit tests + UI integration tests (headless windows)
cargo fmt --all --check         # formatting
cargo clippy -p pcl-ui -p pcl-app --all-targets   # lints
```

Runtime requirements: an X11 or Wayland session, fontconfig and a Vulkan driver (or another
backend wgpu supports).

## Layout

```
crates/
├── app/                    Window entry point: wires gpui-kit and the UI crate
└── ui/                     UI layer
    ├── assets/images/      Image assets (blocks, avatars, icons; committed, embedded at build time)
    ├── i18n/zh-CN.xaml     Interface copy source (keys match PCL's)
    └── src/
        ├── theme/          PCL palette (computed from ToneProfile through OKLCH) and semantic roles
        ├── i18n.rs         String table (generated by tools/gen-i18n.py)
        ├── assets/         Embedded asset table and asset source
        ├── components/     PCL controls: text button / round icon button / checkbox / radio / selector / card / page containers
        ├── shell/          Window shell: route catalog, 48 px custom title bar, main window
        └── pages/          One module per group (launch / download / setup / tools / instance)
tools/
├── gen-i18n.py             Generates the string table; --check verifies every referenced key exists
└── gen-assets.py           Generates the embedded asset table; --check verifies it matches the directory
```

Conventions for new screens:

- Dependency direction: `app` → `ui` → `gpui-kit`. The UI layer never depends on business code;
  models and services will live in their own crates.
- One module per group: `mod.rs` owns state and routing, rendering is split into sibling modules;
  cross-file items use `pub(super)`.
- No literal colours: use `theme::palette(cx)` tiers or semantic roles from `cx.theme()`.
- Do not rebuild controls: reusable ones live in `components/`; gpui-kit components (Settings,
  TabBar, Slider, Select, Input, …) are used directly.
- No hard-coded copy: use `i18n::text("key")` and keep the key present in the table
  (`tools/gen-i18n.py --check`).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[GPL-3.0](LICENSE). The PCL launcher whose interface and copy this project references is developed
by 龙腾猫跃; this project is an independent implementation.
