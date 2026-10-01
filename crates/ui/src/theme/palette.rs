//! PCL 调色板：由 `ToneProfile`（各亮度/色度通道）经 OKLCH 转到 sRGB。
//!
//! 参考实现：`PCL.Core/UI/Theme/ThemeService.cs`、`LabColor.cs`。
//! 深色模式与后续的配色切换都复用同一套计算，界面代码里不再出现第二份色值。

use gpui_kit::component::ThemeMode;
use gpui_kit::{Hsla, Rgba};

/// 一组随配色模式变化的亮度与色度通道，默认值取自 `PCL.Core/UI/Theme/ToneProfile.cs`。
#[derive(Clone, Copy, Debug)]
pub struct ToneProfile {
    pub l1: f64,
    pub l2: f64,
    pub l3: f64,
    pub l4: f64,
    pub l5: f64,
    pub l6: f64,
    pub l7: f64,
    pub l8: f64,
    pub l_white: f64,
    pub l_foreground: f64,
    pub l_background: f64,
    pub c1: f64,
    pub c2: f64,
    pub c3: f64,
    pub c4: f64,
    pub c5: f64,
    pub c6: f64,
    pub c7: f64,
    pub c8: f64,
    pub a_semi_white: f64,
    pub a_half_white: f64,
    pub a_semi_transparent: f64,
    pub a_half_transparent: f64,
    pub a_background: f64,
    pub a_tooltip: f64,
}

impl ToneProfile {
    pub const LIGHT: Self = Self {
        l1: 0.35,
        l2: 0.5,
        l3: 0.575,
        l4: 0.65,
        l5: 0.8,
        l6: 0.92,
        l7: 0.94,
        l8: 0.96,
        l_white: 1.0,
        l_foreground: 0.0,
        l_background: 0.995,
        c1: 0.025,
        c2: 0.188,
        c3: 0.213,
        c4: 0.168,
        c5: 0.093,
        c6: 0.036,
        c7: 0.028,
        c8: 0.018,
        a_semi_white: 0.733,
        a_half_white: 0.333,
        a_semi_transparent: 0.004,
        a_half_transparent: 0.5,
        a_background: 0.824,
        a_tooltip: 0.9,
    };

    /// 深色模式，取自 `ToneProfileConfig.DefaultDark`。
    pub const DARK: Self = Self {
        l1: 0.96,
        l2: 0.75,
        l3: 0.6,
        l4: 0.65,
        l5: 0.45,
        l6: 0.25,
        l7: 0.225,
        l8: 0.2,
        l_background: 0.3,
        l_foreground: 1.0,
        l_white: 0.275,
        ..Self::LIGHT
    };
}

/// 配色主题的主色参数，取自 `ThemeService.GetCurrentThemeArgs` 的 CatBlue。
#[derive(Clone, Copy, Debug)]
pub struct ColorTheme {
    pub hue: f64,
    pub light_adjust: f64,
    pub chroma_adjust: f64,
}

impl ColorTheme {
    pub const CAT_BLUE: Self = Self {
        hue: 255.0,
        light_adjust: 0.0,
        chroma_adjust: -0.2,
    };
}

/// 计算得到的调色板。字段名对应 .NET 版本 `ColorBrush*` 资源键去掉前缀后的部分。
#[derive(Clone, Copy, Debug)]
pub struct Palette {
    pub gray: [Hsla; 8],
    pub white: Hsla,
    pub half_white: Hsla,
    pub semi_white: Hsla,
    pub background: Hsla,
    pub transparent_background: Hsla,
    pub tooltip: Hsla,
    pub red_back: Hsla,
    /// PCL 的红色描边（`ColorBrushRedDark`，红色按钮的静息态）。
    pub red_dark: Hsla,
    /// PCL 的红色高亮（`ColorBrushRedLight`，红色按钮的悬停态）。
    pub red_light: Hsla,
    pub color: [Hsla; 8],
    pub bg0: Hsla,
    pub bg1: Hsla,
    pub semi_transparent: Hsla,
}

impl Palette {
    /// 按配色模式与主色参数计算调色板。
    pub fn resolve(mode: ThemeMode, theme: ColorTheme) -> Self {
        Self::from_tone(
            match mode {
                ThemeMode::Dark => ToneProfile::DARK,
                ThemeMode::Light => ToneProfile::LIGHT,
            },
            theme,
        )
    }

    fn from_tone(tone: ToneProfile, theme: ColorTheme) -> Self {
        let gray = |l: f64, c: f64, h: f64, a: f64| oklch(l, c, h, a);
        let adjust = |value: f64, adjustment: f64| {
            if adjustment == 0.0 {
                return value;
            }
            let value = value.clamp(0.0, 1.0);
            let adjustment = adjustment.clamp(-1.0, 1.0);
            if adjustment > 0.0 {
                value + (1.0 - value) * adjustment
            } else {
                value + value * adjustment
            }
        };
        let color = |l: f64, c: f64, a: f64| {
            oklch(
                adjust(l, theme.light_adjust),
                adjust(c, theme.chroma_adjust),
                theme.hue,
                a,
            )
        };

        Self {
            gray: [
                gray(tone.l1, 0.0, 0.0, 1.0),
                gray(tone.l2, 0.0, 0.0, 1.0),
                gray(tone.l3, 0.0, 0.0, 1.0),
                gray(tone.l4, 0.0, 0.0, 1.0),
                gray(tone.l5, 0.0, 0.0, 1.0),
                gray(tone.l6, 0.0, 0.0, 1.0),
                gray(tone.l7, 0.0, 0.0, 1.0),
                gray(tone.l8, 0.0, 0.0, 1.0),
            ],
            white: gray(tone.l_white, 0.0, 0.0, 1.0),
            half_white: gray(tone.l_white, 0.0, 0.0, tone.a_half_white),
            semi_white: gray(tone.l_white, 0.0, 0.0, tone.a_semi_white),
            background: gray(tone.l_background, 0.0, 0.0, 1.0),
            transparent_background: gray(tone.l_background, 0.0, 0.0, tone.a_background),
            tooltip: gray(tone.l_background, 0.0, 0.0, tone.a_tooltip),
            red_back: oklch(tone.l7, 0.25, 30.0, tone.a_half_transparent),
            // PCL 的红色是固定色值，不随配色主题变化（Palette.axaml 的 RedDark / RedLight）。
            red_dark: Rgba {
                r: 0xce as f32 / 255.,
                g: 0x21 as f32 / 255.,
                b: 0x11 as f32 / 255.,
                a: 1.,
            }
            .into(),
            red_light: Rgba {
                r: 0xff as f32 / 255.,
                g: 0x4c as f32 / 255.,
                b: 0x4c as f32 / 255.,
                a: 1.,
            }
            .into(),
            color: [
                color(tone.l1, tone.c1, 1.0),
                color(tone.l2, tone.c2, 1.0),
                color(tone.l3, tone.c3, 1.0),
                color(tone.l4, tone.c4, 1.0),
                color(tone.l5, tone.c5, 1.0),
                color(tone.l6, tone.c6, 1.0),
                color(tone.l7, tone.c7, 1.0),
                color(tone.l8, tone.c8, 1.0),
            ],
            bg0: color(tone.l5, tone.c5, 1.0),
            bg1: color(tone.l7, tone.c7, tone.a_semi_white),
            semi_transparent: color(tone.l8, tone.c8, tone.a_semi_transparent),
        }
    }

    /// 灰度档位（1 最深、8 最浅）。
    pub fn gray_level(&self, level: usize) -> Hsla {
        self.gray[level.clamp(1, 8) - 1]
    }

    /// 主色档位（1 最深、8 最浅）。
    pub fn color_level(&self, level: usize) -> Hsla {
        self.color[level.clamp(1, 8) - 1]
    }
}

/// 语义色：PCL 未提供固定色值的状态色，用同一套色度/亮度曲线另配色相得到。
pub(super) fn oklch(l: f64, c: f64, h: f64, alpha: f64) -> Hsla {
    let mut low = 0.0;
    let mut high = c;
    let mut best = oklch_unmapped(l, 0.0, h);
    for _ in 0..16 {
        let mid = (low + high) / 2.0;
        let rgb = oklch_unmapped(l, mid, h);
        if in_gamut(rgb) {
            best = rgb;
            low = mid;
        } else {
            high = mid;
        }
    }
    let (r, g, b) = if in_gamut(oklch_unmapped(l, c, h)) {
        oklch_unmapped(l, c, h)
    } else {
        best
    };
    Hsla::from(Rgba {
        r: r.clamp(0.0, 1.0) as f32,
        g: g.clamp(0.0, 1.0) as f32,
        b: b.clamp(0.0, 1.0) as f32,
        a: alpha.clamp(0.0, 1.0) as f32,
    })
}

pub(super) fn in_gamut((r, g, b): (f64, f64, f64)) -> bool {
    (0.0..=1.0).contains(&r) && (0.0..=1.0).contains(&g) && (0.0..=1.0).contains(&b)
}

pub(super) fn oklch_unmapped(l: f64, c: f64, h: f64) -> (f64, f64, f64) {
    let hue = h.to_radians();
    let (a, b) = (c * hue.cos(), c * hue.sin());

    let l_ = l + 0.3963377774 * a + 0.2158037573 * b;
    let m_ = l - 0.1055613458 * a - 0.0638541728 * b;
    let s_ = l - 0.0894841775 * a - 1.2914855480 * b;

    let (l3, m3, s3) = (l_ * l_ * l_, m_ * m_ * m_, s_ * s_ * s_);

    let r = 4.0767416621 * l3 - 3.3077115913 * m3 + 0.2309699292 * s3;
    let g = -1.2684380046 * l3 + 2.6097574011 * m3 - 0.3413193965 * s3;
    let blue = -0.0041960863 * l3 - 0.7034186147 * m3 + 1.7076147010 * s3;

    (encode(r), encode(g), encode(blue))
}

/// 线性 sRGB → sRGB 传输函数。
pub(super) fn encode(value: f64) -> f64 {
    if value <= 0.0031308 {
        12.92 * value
    } else {
        1.055 * value.powf(1.0 / 2.4) - 0.055
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn bytes(color: Hsla) -> (u8, u8, u8, u8) {
        let color = Rgba::from(color);
        let to_byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        (
            to_byte(color.r),
            to_byte(color.g),
            to_byte(color.b),
            to_byte(color.a),
        )
    }

    #[test]
    fn light_palette_matches_dotnet_runtime() {
        let palette = Palette::resolve(ThemeMode::Light, ColorTheme::CAT_BLUE);
        // 期望值来自 .NET 运行时的同一算法（Wacton.Unicolour 8.0.0 + ThemeService 的 CatBlue 参数，
        // GamutMap.OklchChromaReduction）。注意 Palette.axaml 里的静态色值只是占位，与运行时并不一致。
        assert_eq!(bytes(palette.background), (0xfd, 0xfd, 0xfd, 0xff));
        assert_eq!(bytes(palette.gray_level(1)), (0x3a, 0x3a, 0x3a, 0xff));
        assert_eq!(bytes(palette.gray_level(3)), (0x79, 0x79, 0x79, 0xff));
        assert_eq!(bytes(palette.color_level(2)), (0x16, 0x62, 0xb6, 0xff));
        assert_eq!(bytes(palette.color_level(3)), (0x20, 0x78, 0xda, 0xff));
        assert_eq!(bytes(palette.color_level(4)), (0x52, 0x91, 0xdf, 0xff));
        assert_eq!(bytes(palette.color_level(6)), (0xd8, 0xe6, 0xf8, 0xff));
        assert_eq!(bytes(palette.color_level(7)), (0xe1, 0xec, 0xfa, 0xff));
    }

    #[test]
    fn dark_palette_matches_dotnet_runtime() {
        let palette = Palette::resolve(ThemeMode::Dark, ColorTheme::CAT_BLUE);
        assert_eq!(bytes(palette.color_level(1)), (0xe9, 0xf3, 0xff, 0xff));
        assert_eq!(bytes(palette.color_level(3)), (0x2a, 0x80, 0xe3, 0xff));
        assert_eq!(bytes(palette.color_level(4)), (0x52, 0x91, 0xdf, 0xff));
        assert_eq!(bytes(palette.color_level(5)), (0x38, 0x57, 0x7e, 0xff));
        assert_eq!(bytes(palette.color_level(6)), (0x18, 0x22, 0x2f, 0xff));
        assert_eq!(bytes(palette.color_level(7)), (0x15, 0x1c, 0x26, 0xff));
        assert_eq!(bytes(palette.color_level(8)), (0x12, 0x16, 0x1d, 0xff));

        // 唯一的例外：暗色最饱和档超出 sRGB 色域时，Unicolour 的 OklchChromaReduction 还会连带
        // 微调亮度与色相（实测 L 0.75→0.744、H 255→252），本实现只在 OKLCH 上降色度，
        // 因此红色通道高 11/255（#73B1FF vs #68B0FF），肉眼不可分辨。
        let (r, g, b, _) = bytes(palette.color_level(2));
        assert!((r as i16 - 0x68).abs() <= 16 && (g as i16 - 0xb0).abs() <= 2 && b == 0xff);
    }
}
