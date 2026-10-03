//! 设置 · 个性化的六张外观卡片（对应 PCL 的 `PageSetupUI` 设置页：外观 / 字体 / 背景 / 音乐 /
//! 图标 / 主页）。
//!
//! 每张卡片一个函数：读受控值、产出 `SettingItem` 列表；文件操作类按钮只保留排版。

use gpui_kit::component::setting::SettingItem;
use gpui_kit::component::slider::SliderState;
use gpui_kit::*;

use super::super::{Fields, NO_ENABLE, choices};
use crate::components::ButtonColor;

/// 主题色下拉项（`ColorTheme` 的顺序：SkyBlue / CatBlue / DeathBlue）。
const COLOR_KEYS: &[&str] = &[
    "Setup.Ui.Theme.Color.SkyBlue",
    "Setup.Ui.Theme.Color.CatBlue",
    "Setup.Ui.Theme.Color.CrashBlue",
];

pub(super) fn basic(
    fields: &Fields,
    opacity: &Entity<SliderState>,
    blur_radius: &Entity<SliderState>,
    blur_sampling: &Entity<SliderState>,
) -> Vec<SettingItem> {
    vec![
        fields.slider("Setup.Ui.Basic.Opacity", None, opacity, "", NO_ENABLE),
        fields.dropdown(
            "Setup.Ui.Theme.Title",
            None,
            choices(&[
                "Setup.Ui.Theme.Light",
                "Setup.Ui.Theme.Dark",
                "Setup.Ui.Theme.FollowSystem",
            ]),
            |state| state.ui.theme_mode,
            |state, index| state.ui.theme_mode = index,
        ),
        fields.dropdown(
            "Setup.Ui.Theme.LightColor",
            None,
            choices(COLOR_KEYS),
            |state| state.ui.light_color,
            |state, index| state.ui.light_color = index,
        ),
        fields.dropdown(
            "Setup.Ui.Theme.DarkColor",
            None,
            choices(COLOR_KEYS),
            |state| state.ui.dark_color,
            |state, index| state.ui.dark_color = index,
        ),
        fields.checkbox(
            "ui-show-logo",
            "Setup.Ui.Basic.ShowLogo",
            None,
            |state| state.ui.show_logo,
            |state, value| state.ui.show_logo = value,
        ),
        fields.checkbox(
            "ui-lock-window",
            "Setup.Ui.Basic.LockWindow",
            None,
            |state| state.ui.lock_window,
            |state, value| state.ui.lock_window = value,
        ),
        fields.checkbox(
            "ui-trivia-hint",
            "Setup.Ui.Basic.ShowTriviaHint",
            None,
            |state| state.ui.trivia_hint,
            |state, value| state.ui.trivia_hint = value,
        ),
        fields.checkbox(
            "ui-advanced-material",
            "Setup.Ui.Basic.AdvancedMaterial",
            Some("Setup.Ui.Basic.AdvancedMaterial.ToolTip"),
            |state| state.ui.advanced_material,
            |state, value| state.ui.advanced_material = value,
        ),
        fields.slider(
            "Setup.Ui.Basic.BlurRadius",
            Some("Setup.Ui.Basic.BlurRadius.ToolTip"),
            blur_radius,
            "",
            NO_ENABLE,
        ),
        fields.slider(
            "Setup.Ui.Basic.BlurSamplingRate",
            Some("Setup.Ui.Basic.BlurSamplingRate.ToolTip"),
            blur_sampling,
            "",
            NO_ENABLE,
        ),
        fields.dropdown(
            "Setup.Ui.Basic.BlurMethod",
            Some("Setup.Ui.Basic.BlurMethod.ToolTip"),
            choices(&[
                "Setup.Ui.Basic.BlurMethod.Gaussian",
                "Setup.Ui.Basic.BlurMethod.Box",
            ]),
            |state| state.ui.blur_method,
            |state, index| state.ui.blur_method = index,
        ),
    ]
}

pub(super) fn font(fields: &Fields) -> Vec<SettingItem> {
    vec![
        fields.input(
            "Setup.Ui.Font.Global",
            Some("Setup.Ui.Font.Global.ToolTip"),
            |state| state.ui.font_global.clone(),
            |state, value| state.ui.font_global = value,
        ),
        fields.input(
            "Setup.Ui.Font.Motd",
            Some("Setup.Ui.Font.Motd.ToolTip"),
            |state| state.ui.font_motd.clone(),
            |state, value| state.ui.font_motd = value,
        ),
    ]
}

pub(super) fn background(
    fields: &Fields,
    opacity: &Entity<SliderState>,
    blur: &Entity<SliderState>,
) -> Vec<SettingItem> {
    vec![
        fields.dropdown(
            "Setup.Ui.Background.ContentAdapt",
            None,
            choices(&[
                "Setup.Ui.Background.Suit.Smart",
                "Setup.Ui.Background.Suit.Center",
                "Setup.Ui.Background.Suit.Fit",
                "Setup.Ui.Background.Suit.Stretch",
                "Setup.Ui.Background.Suit.Tile",
                "Setup.Ui.Background.Suit.TopLeft",
                "Setup.Ui.Background.Suit.TopRight",
                "Setup.Ui.Background.Suit.BottomLeft",
                "Setup.Ui.Background.Suit.BottomRight",
            ]),
            |state| state.ui.background_suit,
            |state, index| state.ui.background_suit = index,
        ),
        fields.slider("Setup.Ui.Basic.Opacity2", None, opacity, "", NO_ENABLE),
        fields.slider(
            "Setup.Ui.Background.Blur",
            Some("Setup.Ui.Background.Blur.ToolTip"),
            blur,
            "",
            NO_ENABLE,
        ),
        fields.checkbox(
            "ui-pause-video",
            "Setup.Ui.Background.PauseVideo",
            Some("Setup.Ui.Background.PauseVideo.ToolTip"),
            |state| state.ui.pause_video,
            |state, value| state.ui.pause_video = value,
        ),
        fields.checkbox(
            "ui-colorful",
            "Setup.Ui.Background.ColorOverlay",
            None,
            |state| state.ui.colorful,
            |state, value| state.ui.colorful = value,
        ),
        fields.stub_button(
            "ui-background-open",
            "Common.Action.OpenFolder",
            Some("Setup.Ui.Background.OpenFolder.ToolTip"),
            ButtonColor::Normal,
        ),
        fields.stub_button(
            "ui-background-refresh",
            "Setup.Ui.Background.Refresh",
            Some("Setup.Ui.Background.Refresh.ToolTip"),
            ButtonColor::Normal,
        ),
        fields.stub_button(
            "ui-background-clear",
            "Setup.Ui.Background.Clear",
            None,
            ButtonColor::Normal,
        ),
    ]
}

pub(super) fn music(fields: &Fields, volume: &Entity<SliderState>) -> Vec<SettingItem> {
    vec![
        fields.slider("Setup.Ui.Music.Volume", None, volume, "", NO_ENABLE),
        fields.checkbox(
            "ui-music-shuffle",
            "Setup.Ui.Music.Shuffle",
            None,
            |state| state.ui.music_shuffle,
            |state, value| state.ui.music_shuffle = value,
        ),
        fields.checkbox(
            "ui-music-auto",
            "Setup.Ui.Music.AutoStart",
            None,
            |state| state.ui.music_auto,
            |state, value| state.ui.music_auto = value,
        ),
        fields.checkbox(
            "ui-music-start",
            "Setup.Ui.Music.StartInGame",
            None,
            |state| state.ui.music_start,
            |state, value| state.ui.music_start = value,
        ),
        fields.checkbox(
            "ui-music-stop",
            "Setup.Ui.Music.StopInGame",
            None,
            |state| state.ui.music_stop,
            |state, value| state.ui.music_stop = value,
        ),
        fields.checkbox(
            "ui-music-smtc",
            "Setup.Ui.Music.Smtc",
            Some("Setup.Ui.Music.Smtc.ToolTip"),
            |state| state.ui.music_smtc,
            |state, value| state.ui.music_smtc = value,
        ),
        fields.stub_button(
            "ui-music-open",
            "Common.Action.OpenFolder",
            Some("Setup.Ui.Music.OpenFolder.ToolTip"),
            ButtonColor::Normal,
        ),
        fields.stub_button(
            "ui-music-refresh",
            "Setup.Ui.Music.Refresh",
            Some("Setup.Ui.Music.Refresh.ToolTip"),
            ButtonColor::Normal,
        ),
        fields.stub_button(
            "ui-music-clear",
            "Setup.Ui.Music.Clear",
            None,
            ButtonColor::Red,
        ),
    ]
}

pub(super) fn logo(fields: &Fields) -> Vec<SettingItem> {
    vec![
        fields.radio_option(
            "ui-logo-none",
            "Setup.Ui.Logo.None",
            None,
            0,
            |state| state.ui.logo_type,
            |state, index| state.ui.logo_type = index,
        ),
        fields.radio_option(
            "ui-logo-default",
            "Common.Option.Default",
            None,
            1,
            |state| state.ui.logo_type,
            |state, index| state.ui.logo_type = index,
        ),
        fields.radio_option(
            "ui-logo-text",
            "Setup.Ui.Logo.Text",
            None,
            2,
            |state| state.ui.logo_type,
            |state, index| state.ui.logo_type = index,
        ),
        fields.radio_option(
            "ui-logo-image",
            "Setup.Ui.Logo.Image",
            None,
            3,
            |state| state.ui.logo_type,
            |state, index| state.ui.logo_type = index,
        ),
        fields.checkbox(
            "ui-logo-left",
            "Setup.Ui.Logo.LeftAlign",
            None,
            |state| state.ui.logo_left,
            |state, value| state.ui.logo_left = value,
        ),
        fields.input(
            "Setup.Ui.Logo.TextLabel",
            None,
            |state| state.ui.logo_text.clone(),
            |state, value| state.ui.logo_text = value,
        ),
        fields.stub_button(
            "ui-logo-change",
            "Setup.Ui.Logo.ChangeImage",
            None,
            ButtonColor::Normal,
        ),
        fields.stub_button(
            "ui-logo-clear-image",
            "Setup.Ui.Logo.ClearImage",
            None,
            ButtonColor::Normal,
        ),
    ]
}

pub(super) fn homepage(fields: &Fields) -> Vec<SettingItem> {
    vec![
        fields.radio_option(
            "ui-homepage-blank",
            "Setup.Ui.Homepage.Blank",
            None,
            0,
            |state| state.ui.homepage_type,
            |state, index| state.ui.homepage_type = index,
        ),
        fields.radio_option(
            "ui-homepage-preset",
            "Setup.Ui.Homepage.Preset",
            None,
            1,
            |state| state.ui.homepage_type,
            |state, index| state.ui.homepage_type = index,
        ),
        fields.radio_option(
            "ui-homepage-local",
            "Setup.Ui.Homepage.LocalFile",
            None,
            2,
            |state| state.ui.homepage_type,
            |state, index| state.ui.homepage_type = index,
        ),
        fields.radio_option(
            "ui-homepage-net",
            "Setup.Ui.Homepage.NetUpdate",
            None,
            3,
            |state| state.ui.homepage_type,
            |state, index| state.ui.homepage_type = index,
        ),
        fields.paragraph("Setup.Ui.Homepage.HintReferToModSetup", false),
        fields.hint("Setup.Ui.Homepage.NetWarning"),
        fields.stub_button(
            "ui-homepage-refresh",
            "Setup.Ui.Homepage.Refresh",
            None,
            ButtonColor::Normal,
        ),
        fields.stub_button(
            "ui-homepage-tutorial",
            "Setup.Ui.Homepage.Tutorial",
            None,
            ButtonColor::Normal,
        ),
        fields.input(
            "Setup.Ui.Homepage.DownloadUrl",
            None,
            |state| state.ui.homepage_url.clone(),
            |state, value| state.ui.homepage_url = value,
        ),
        fields.dropdown(
            "Setup.Ui.Homepage.PresetLabel",
            None,
            choices(&[
                "Setup.Ui.Homepage.Preset.Trivia",
                "Setup.Ui.Homepage.Preset.McNews",
                "Setup.Ui.Homepage.Preset.DailyModpack",
                "Setup.Ui.Homepage.Preset.McSkin",
                "Setup.Ui.Homepage.Preset.OpenBmclapi",
                "Setup.Ui.Homepage.Preset.PclManual",
                "Setup.Ui.Homepage.Preset.Magazine",
                "Setup.Ui.Homepage.Preset.GitHub",
                "Setup.Ui.Homepage.Preset.McUpdateSummary",
                "Setup.Ui.Homepage.Preset.NewsToday",
                "Setup.Ui.Homepage.Preset.MinecraftKnowledge",
                "Setup.Ui.Homepage.Preset.ModpackRecommendation",
                "Setup.Ui.Homepage.Preset.Bangumi",
                "Setup.Ui.Homepage.Preset.PclAnnouncement",
                "Setup.Ui.Homepage.Preset.McOfficialFeed",
            ]),
            |state| state.ui.homepage_preset,
            |state, index| state.ui.homepage_preset = index,
        ),
    ]
}
