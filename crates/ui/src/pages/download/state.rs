//! 下载页的示例清单、常量与界面状态枚举。
//!
//! 版本清单来自清单服务（尚未接入），这里用 [`VERSION_SAMPLES`] 占位；
//! 加载器与兼容性提示的条目次序对应 PCL 安装面板的可选加载器与提示列表。

use gpui_kit::*;

use crate::components::HintLevel;
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
    // 以下两条是占位样例：真实存在的特殊版本（愚人节发布），日期为实际发布日。
    VersionSample {
        id: "25w14craftmine",
        released: "2025-04-01",
        kind: VersionKind::Special,
    },
    VersionSample {
        id: "1.RV-Pre1",
        released: "2016-03-31",
        kind: VersionKind::Special,
    },
];

/// 版本分类多选按钮组的条目：分类文案键 + 对应的版本种类，次序即按钮次序。
///
/// 四类按用户口径给出：预览版收纳快照（`25w*`），远古版收纳正式版之前的旧版本，
/// 特殊版本收纳愚人节等非标准发布。
pub(super) const CATEGORIES: &[(&str, VersionKind)] = &[
    ("Download.Version.Type.Release", VersionKind::Release),
    ("Download.Version.Type.Development", VersionKind::Snapshot),
    ("Download.Version.Type.BeforeRelease", VersionKind::Old),
    ("Download.Version.Type.Special", VersionKind::Special),
];

/// 安装面板的加载器条目：(文案键, 方块图)。
///
/// 方块图取自 `images/Blocks` 资产表（`tools/gen-assets.py` 迁移），
/// 同一加载器的多个条目（Fabric 系列）共用同一张图。
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

/// 各加载器可选的版本（示例清单；接入加载器清单服务后按所选的 Minecraft 版本请求）。
///
/// 次序与 [`LOADERS`] 一一对应，卡片上显示的就是这里的字符串。加载器与 Minecraft 版本的
/// 兼容性要靠真实数据判定，现在还没有，所以每个加载器的版本清单都对所有 MC 版本可见。
pub(super) const LOADER_VERSIONS: &[&[&str]] = &[
    &["47.3.0", "47.2.0", "47.1.3"],
    &["0.2.4", "0.2.3"],
    &["21.4.12", "21.1.72"],
    &["0.16.10", "0.16.9"],
    &["0.14.24", "0.14.23"],
    &["0.119.2", "0.115.6"],
    &["1.13.5", "1.13.4"],
    &["4.2.5", "4.1.0"],
    &["HD U J6", "HD U I6"],
    &["1.21.4", "1.20.1"],
    &["1.12.2-SNAPSHOT-r", "1.12.2"],
];

/// 安装面板顶部的兼容性提示：红档 3 条在前、黄档 3 条在后，进入面板后静态可见。
pub(super) const HINTS: &[(&str, HintLevel)] = &[
    ("Download.Install.Warning.FabricApi", HintLevel::Red),
    ("Download.Install.Warning.LegacyFabricApi", HintLevel::Red),
    ("Download.Install.Warning.OptiFabric", HintLevel::Red),
    ("Download.Install.Warning.OptiFabricOld", HintLevel::Yellow),
    (
        "Download.Install.Warning.LegacyOptiFabric",
        HintLevel::Yellow,
    ),
    ("Download.Install.Warning.ModOptiFine", HintLevel::Yellow),
];

/// 示例安装进度：安装流程未接入，进度条用它固定在一个「进行中」的位置。
pub(super) const SAMPLE_INSTALL_PROGRESS: f32 = 37.0;

/// 版本清单的加载状态（对应 `MyLoading.MyLoadingState`）。
/// 另有 Error 一态（显示 `Main.PageDownload.LoadFailed`），要等清单服务接入后才可能出现。
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

/// 版本种类（决定图标与所属分类）：接入清单后按条目的 `type` 字段归并，
/// 愚人节等非标准发布另归为特殊版本。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum VersionKind {
    Release,
    Snapshot,
    /// 正式版之前的远古版本。
    Old,
    /// 愚人节等非标准发布的特殊版本。
    Special,
}

impl VersionKind {
    /// 版本图标：原版用 Blocks 位图（Grass / CommandBlock / GoldBlock / CobbleStone），
    /// 这里按同样的语义映射到 lucide 图标。
    pub(super) fn icon(self) -> &'static str {
        match self {
            Self::Release => "boxes",
            Self::Snapshot => "square-terminal",
            Self::Old => "gem",
            Self::Special => "sparkles",
        }
    }

    /// 选中详情里的类型文案：正式版与快照取 `Main.PageDownload.Release` / `Snapshot`，
    /// 远古版与特殊版本取分类文案。
    pub(super) fn label(self) -> SharedString {
        match self {
            Self::Release => i18n::lang("Main.PageDownload.Release"),
            Self::Snapshot => i18n::lang("Main.PageDownload.Snapshot"),
            Self::Old => i18n::lang("Download.Version.Type.BeforeRelease"),
            Self::Special => i18n::lang("Download.Version.Type.Special"),
        }
    }
}
