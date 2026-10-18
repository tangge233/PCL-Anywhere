//! 应用级配置。

use std::sync::Arc;

use lite_config::{Config as ConfigFile, Error, Schema};
use serde::{Deserialize, Serialize};
use url::Url;

/// 全局配置。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct AppConfig {
    /// 游戏相关配置。同时接受 PCL 风格的 `Game` 键。
    #[serde(alias = "Game")]
    pub(crate) game: GameConfig,
}

impl Schema for AppConfig {
    const VERSION: u32 = 1;
}

/// 游戏配置。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct GameConfig {
    /// Minecraft 下载源。
    #[serde(alias = "DownloadSource")]
    pub(crate) download_source: Vec<DownloadSource>,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            download_source: vec![
                DownloadSource::default(),
                DownloadSource {
                    id: "bmclapi".to_owned(),
                    enabled: false,
                    version: DownloadSourceVersion::LauncherMetaV1,
                    assets_url: Url::parse("https://bmclapi2.bangbang93.com/assets")
                        .expect("BMCLAPI Assets URL"),
                    libraries_url: Url::parse("https://bmclapi2.bangbang93.com/maven")
                        .expect("BMCLAPI Libraries URL"),
                    version_url: Url::parse(
                        "https://bmclapi2.bangbang93.com/mc/game/version_manifest.json",
                    )
                    .expect("BMCLAPI 版本信息 URL"),
                },
            ],
        }
    }
}

/// 一个 Minecraft 下载源。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct DownloadSource {
    /// 稳定 ID，例如 `official` / `bmclapi`。
    pub(crate) id: String,

    /// 是否启用。
    pub(crate) enabled: bool,

    /// 版本清单 API 结构版本。
    pub(crate) version: DownloadSourceVersion,

    /// Assets 基础 URL。
    pub(crate) assets_url: Url,

    /// Libraries 基础 URL。
    pub(crate) libraries_url: Url,

    /// 版本信息（版本清单）URL。
    pub(crate) version_url: Url,
}

impl Default for DownloadSource {
    fn default() -> Self {
        Self {
            id: "official".to_owned(),
            enabled: true,
            version: DownloadSourceVersion::PistonMetaV2,
            assets_url: Url::parse("https://resources.download.minecraft.net")
                .expect("官方 Assets URL"),
            libraries_url: Url::parse("https://libraries.minecraft.net")
                .expect("官方 Libraries URL"),
            version_url: Url::parse(
                "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json",
            )
            .expect("官方版本信息 URL"),
        }
    }
}

/// 版本清单 API 结构版本。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum DownloadSourceVersion {
    /// launchermeta v1 结构。
    #[default]
    #[serde(rename = "launchermeta-v1")]
    LauncherMetaV1,

    /// piston-meta v2 结构。
    #[serde(rename = "piston-meta-v2")]
    PistonMetaV2,
}

/// 打开全局配置文件；不存在时写入默认值，损坏时替换并保留备份。
pub(crate) fn load() -> Result<Arc<ConfigFile<AppConfig>>, Error> {
    let path = lite_config::global_config_file()?;
    ConfigFile::open_or_default_replace(path)
}
