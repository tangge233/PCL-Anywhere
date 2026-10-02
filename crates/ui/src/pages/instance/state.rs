//! 实例页的示例数据与相关小工具。
//!
//! 实例扫描、图标解析、存档读取都属于业务逻辑，尚未接入；接入后应替换为视图模型提供的数据，
//! 这里的 `SAMPLE_*` 只用于把界面填成合理内容。

use gpui_kit::*;

use crate::i18n;

pub(super) fn sample_difficulties() -> Vec<SharedString> {
    vec![
        i18n::lang("Instance.Saves.Info.Difficulty.Peaceful"),
        i18n::lang("Instance.Saves.Info.Difficulty.Easy"),
        i18n::lang("Instance.Saves.Info.Difficulty.Normal"),
        i18n::lang("Instance.Saves.Info.Difficulty.Hard"),
    ]
}

// ---- 示例数据 --------------------------------------------------------------
//
// 以下常量只是界面占位：实例扫描、图标解析、存档读取都属于业务逻辑，尚未接入。
// 接入后应替换为视图模型提供的数据。

pub(super) const SAMPLE_JVM_ARGS: &str = "-XX:+UseG1GC -XX:-UseAdaptiveSizePolicy";

pub(super) struct SampleFolder {
    /// 非默认目录的显示名（默认目录用 i18n 的「默认文件夹」）。
    pub(super) name: &'static str,
    pub(super) is_default: bool,
    /// 该目录下的实例数量（示例）。
    pub(super) count: usize,
    #[allow(dead_code)]
    pub(super) path: &'static str,
}

pub(super) struct SampleInstance {
    pub(super) name: &'static str,
    /// 游戏版本（示例）。
    pub(super) version: &'static str,
    /// 加载器（示例）。
    pub(super) loader: &'static str,
    /// 所属文件夹下标。
    pub(super) folder: usize,
    /// lucide 图标名（示例）。
    pub(super) icon: &'static str,
    #[allow(dead_code)]
    pub(super) path: &'static str,
    /// 启动次数（示例）。
    pub(super) launch_count: u32,
    /// 整合包版本（示例）。
    pub(super) modpack: &'static str,
}

pub(super) struct SampleSave {
    pub(super) name: &'static str,
    pub(super) version: &'static str,
    pub(super) game_mode_key: &'static str,
    pub(super) play_time: &'static str,
    pub(super) last_played: &'static str,
    pub(super) seed: &'static str,
    pub(super) spawn: &'static str,
}

pub(super) static SAMPLE_FOLDERS: &[SampleFolder] = &[
    SampleFolder {
        name: "默认文件夹",
        is_default: true,
        count: 3,
        path: ".minecraft",
    },
    SampleFolder {
        name: "HyPixel 整合包",
        is_default: false,
        count: 2,
        path: ".minecraft/versions/HyPixel",
    },
];

pub(super) static SAMPLE_INSTANCES: &[SampleInstance] = &[
    SampleInstance {
        name: "1.20.1-Forge-原版生存",
        version: "1.20.1",
        loader: "Forge 47.2.0",
        folder: 0,
        icon: "box",
        path: ".minecraft/versions/1.20.1-Forge-原版生存",
        launch_count: 42,
        modpack: "原版",
    },
    SampleInstance {
        name: "1.19.2-Fabric-生电服",
        version: "1.19.2",
        loader: "Fabric 0.14.22",
        folder: 0,
        icon: "box",
        path: ".minecraft/versions/1.19.2-Fabric-生电服",
        launch_count: 12,
        modpack: "原版",
    },
    SampleInstance {
        name: "1.16.5-Forge-暮色森林",
        version: "1.16.5",
        loader: "Forge 36.2.34",
        folder: 0,
        icon: "trees",
        path: ".minecraft/versions/1.16.5-Forge-暮色森林",
        launch_count: 5,
        modpack: "Twilight Forest",
    },
    SampleInstance {
        name: "1.20.4-NeoForge-ATM10",
        version: "1.20.4",
        loader: "NeoForge 20.4.237",
        folder: 1,
        icon: "package",
        path: ".minecraft/versions/1.20.4-NeoForge-ATM10",
        launch_count: 3,
        modpack: "All the Mods 10",
    },
    SampleInstance {
        name: "1.18.2-Fabric-空岛",
        version: "1.18.2",
        loader: "Fabric 0.14.9",
        folder: 1,
        icon: "cloud",
        path: ".minecraft/versions/1.18.2-Fabric-空岛",
        launch_count: 0,
        modpack: "Skyblock",
    },
];

pub(super) static SAMPLE_SAVES: &[SampleSave] = &[
    SampleSave {
        name: "新的世界",
        version: "1.20.1",
        game_mode_key: "Instance.Saves.Info.GameMode.Survival",
        play_time: "12 小时 34 分钟",
        last_played: "2026-09-30 21:12",
        seed: "1145141919810",
        spawn: "128 / 71 / -256",
    },
    SampleSave {
        name: "创造试验场",
        version: "1.20.1",
        game_mode_key: "Instance.Saves.Info.GameMode.Creative",
        play_time: "3 小时 8 分钟",
        last_played: "2026-09-28 19:40",
        seed: "-20261002",
        spawn: "0 / 64 / 0",
    },
];

pub(super) fn folder_title(folder: &SampleFolder) -> SharedString {
    if folder.is_default {
        i18n::lang("Instance.Folder.Default")
    } else {
        SharedString::from(folder.name)
    }
}
