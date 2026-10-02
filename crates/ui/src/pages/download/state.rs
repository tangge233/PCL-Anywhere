//! 下载页的示例清单、常量与界面状态枚举。
//!
//! 版本清单来自清单服务（尚未接入），这里用 [`VERSION_SAMPLES`] 占位；
//! 加载器、提示、分类的键与图标次序都取自 `PageDownloadInstall.axaml`。

use gpui_kit::*;
use std::rc::Rc;

use crate::i18n;
pub(super) struct VersionSample {
    /// 版本号。
    pub(super) id: &'static str,
    /// 发布时间（示例值，格式与原版的 `yyyy-MM-dd` 一致）。
    pub(super) released: &'static str,
    /// 清单条目的 `type` 字段。
    pub(super) kind: VersionKind,
}

pub(super) const VERSION_SAMPLES: &[VersionSample] = &[
    VersionSample {
        id: "1.21.4",
        released: "2024-12-03",
        kind: VersionKind::Release,
    },
    VersionSample {
        id: "1.21.3",
        released: "2024-10-23",
        kind: VersionKind::Release,
    },
    VersionSample {
        id: "1.21.1",
        released: "2024-08-08",
        kind: VersionKind::Release,
    },
    VersionSample {
        id: "1.20.6",
        released: "2024-04-29",
        kind: VersionKind::Release,
    },
    VersionSample {
        id: "1.20.4",
        released: "2023-12-07",
        kind: VersionKind::Release,
    },
    VersionSample {
        id: "1.20.1",
        released: "2023-06-12",
        kind: VersionKind::Release,
    },
    VersionSample {
        id: "1.19.4",
        released: "2023-03-14",
        kind: VersionKind::Release,
    },
    VersionSample {
        id: "1.18.2",
        released: "2022-02-28",
        kind: VersionKind::Release,
    },
    VersionSample {
        id: "1.16.5",
        released: "2021-01-14",
        kind: VersionKind::Release,
    },
    VersionSample {
        id: "1.12.2",
        released: "2017-09-18",
        kind: VersionKind::Release,
    },
    VersionSample {
        id: "1.7.10",
        released: "2014-06-26",
        kind: VersionKind::Release,
    },
    VersionSample {
        id: "25w03a",
        released: "2025-01-15",
        kind: VersionKind::Snapshot,
    },
    VersionSample {
        id: "25w02a",
        released: "2025-01-08",
        kind: VersionKind::Snapshot,
    },
    VersionSample {
        id: "24w46a",
        released: "2024-11-13",
        kind: VersionKind::Snapshot,
    },
    VersionSample {
        id: "b1.7.3",
        released: "2011-07-08",
        kind: VersionKind::Old,
    },
    VersionSample {
        id: "a1.2.6",
        released: "2010-12-02",
        kind: VersionKind::Old,
    },
];

/// 版本分类（对应 ViewModel 的 `_AddCategoryCard`）：标题键 + 包含的版本种类。
pub(super) const CATEGORIES: &[(&str, VersionKind)] = &[
    ("Download.Version.Type.Stable", VersionKind::Release),
    ("Download.Version.Type.Snapshot", VersionKind::Snapshot),
    ("Download.Version.Type.BeforeRelease", VersionKind::Old),
];

/// 安装面板里的 11 张加载器卡片，顺序与 `PageDownloadInstall.axaml` 一致（标题键 + lucide 图标）。
/// 安装面板的 11 张加载器卡片：(文案键, 方块图)。
///
/// 方块图与 .NET 版本 `PageDownloadInstall.axaml` 里每张卡片的 `MyImage` 一一对应
/// （资源由 `tools/gen-assets.py` 从 `PCL.Ui/Assets/Images/Blocks` 迁移）。
pub(super) const LOADERS: &[(&str, &str)] = &[
    ("Common.Installation.Forge", "images/Blocks/Anvil.png"),
    (
        "Common.Installation.Cleanroom",
        "images/Blocks/Cleanroom.png",
    ),
    ("Common.Installation.NeoForge", "images/Blocks/NeoForge.png"),
    ("Common.Installation.Fabric", "images/Blocks/Fabric.png"),
    (
        "Common.Installation.LegacyFabric",
        "images/Blocks/Fabric.png",
    ),
    ("Common.Installation.FabricApi", "images/Blocks/Fabric.png"),
    (
        "Common.Installation.LegacyFabricApi",
        "images/Blocks/Fabric.png",
    ),
    ("Common.Installation.LabyMod", "images/Blocks/LabyMod.png"),
    (
        "Common.Installation.OptiFine",
        "images/Blocks/GrassPath.png",
    ),
    (
        "Common.Installation.OptiFabric",
        "images/Blocks/OptiFabric.png",
    ),
    ("Common.Installation.LiteLoader", "images/Blocks/Egg.png"),
];

/// 折叠卡片的参数：标题、标题图标、展开状态与点击标题的切换回调。
pub(super) struct CardSpec<'a> {
    pub(super) id: SharedString,
    pub(super) title: SharedString,
    /// 标题左侧的 18px 方块图路径（`None` 表示无图标）。
    pub(super) icon: Option<&'a str>,
    pub(super) expanded: bool,
    pub(super) toggle: Option<ToggleHandler>,
}

/// 安装面板顶部的兼容性提示（`MyHint`，顺序取自 XAML：红 3 条、黄 3 条）。
/// .NET 版这里也还没接加载器选择，六条提示按 XAML 静态可见；`true` 表示红档提示。
pub(super) const HINTS: &[(&str, bool)] = &[
    ("Download.Install.Warning.FabricApi", true),
    ("Download.Install.Warning.LegacyFabricApi", true),
    ("Download.Install.Warning.OptiFabric", true),
    ("Download.Install.Warning.OptiFabricOld", false),
    ("Download.Install.Warning.LegacyOptiFabric", false),
    ("Download.Install.Warning.ModOptiFine", false),
];

/// 示例安装进度：安装流程未接入，进度条用它固定在一个「进行中」的位置。
pub(super) const SAMPLE_INSTALL_PROGRESS: f32 = 37.0;

/// 版本清单的加载状态（对应 `MyLoading.MyLoadingState`）。
/// .NET 版还有 Error 一态（显示 `Main.PageDownload.LoadFailed`），要等清单服务接入后才可能出现。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum LoadPhase {
    /// 正在请求清单（加载环）。
    Loading,
    /// 清单就绪。
    Ready,
}

/// 内容区当前显示的面板（对应 `IsInSelectPage`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum InstallPanel {
    /// 版本清单。
    List,
    /// 选中版本后的安装面板。
    Select,
}

/// 安装状态（对应 `IsInstalling` 与 `ProgressText`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum InstallState {
    /// 未开始：只显示安装说明。
    Idle,
    /// 安装中：显示进度条与取消按钮。
    Running,
    /// 已取消。
    Cancelled,
}

/// 版本种类（对应清单条目的 `type` 字段，决定图标与所属分类）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum VersionKind {
    Release,
    Snapshot,
    /// 正式版之前的远古版本。
    Old,
}

impl VersionKind {
    /// 版本图标：原版用 Blocks 位图（Grass / CommandBlock / GoldBlock / CobbleStone），
    /// 这里按同样的语义映射到 lucide 图标。
    pub(super) fn icon(self) -> &'static str {
        match self {
            Self::Release => "boxes",
            Self::Snapshot => "square-terminal",
            Self::Old => "gem",
        }
    }

    /// 选中详情里的类型文案（对应 `Main.PageDownload.Release` / `Snapshot`）。
    pub(super) fn label(self) -> SharedString {
        match self {
            Self::Release => i18n::lang("Main.PageDownload.Release"),
            Self::Snapshot => i18n::lang("Main.PageDownload.Snapshot"),
            Self::Old => i18n::lang("Download.Version.Type.BeforeRelease"),
        }
    }
}

/// 卡片标题行的折叠回调。
type ToggleHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;
