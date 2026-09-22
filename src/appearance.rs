use crate::native_interop::Color;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Light,
    Dark,
    Custom,
    #[default]
    #[serde(other)]
    System,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Appearance {
    pub mode: Mode,
    /// RGB: background, text, Claude, Codex, Antigravity.
    pub colors: [u32; 5],
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            mode: Mode::System,
            colors: PRESETS[0].1,
        }
    }
}

pub const PRESETS: [(&str, [u32; 5]); 4] = [
    (
        "Midnight",
        [0x151923, 0xe7edf7, 0xf0aa87, 0xa6b9ff, 0x74d8d3],
    ),
    (
        "Porcelain",
        [0xf6f3ed, 0x292d36, 0x9e4d32, 0x3859a8, 0x226c65],
    ),
    (
        "Evergreen",
        [0x122522, 0xe0eee6, 0xeac098, 0x88d9b4, 0x8fcada],
    ),
    (
        "Afterglow",
        [0x281d2c, 0xf8e5ee, 0xffb98f, 0xd3afff, 0xf19abf],
    ),
];

pub fn color(rgb: u32) -> Color {
    Color::new((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

impl Appearance {
    pub fn is_dark(&self, system_dark: bool) -> bool {
        match self.mode {
            Mode::System => system_dark,
            Mode::Light => false,
            Mode::Dark => true,
            Mode::Custom => {
                let c = color(self.colors[0]);
                (299 * c.r as u32 + 587 * c.g as u32 + 114 * c.b as u32) < 128000
            }
        }
    }

    pub fn palette(&self, dark: bool) -> [Color; 5] {
        if self.mode == Mode::Custom {
            return self.colors.map(color);
        }
        if dark {
            [0x1c1c1c, 0xe8ecef, 0xeaa082, 0xf5f5f5, 0x8ab4f8].map(color)
        } else {
            [0xf3f3f3, 0x252b32, 0x9a452a, 0x1f1f1f, 0x1967d2].map(color)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_themes_ignore_system_changes() {
        let mut a = Appearance::default();
        assert!(a.is_dark(true));
        assert!(!a.is_dark(false));
        a.mode = Mode::Light;
        assert!(!a.is_dark(true));
        a.mode = Mode::Dark;
        assert!(a.is_dark(false));
    }
    #[test]
    fn custom_colors_survive_round_trip_and_determine_contrast() {
        for (_, colors) in PRESETS {
            let a = Appearance {
                mode: Mode::Custom,
                colors,
            };
            let b: Appearance = serde_json::from_str(&serde_json::to_string(&a).unwrap()).unwrap();
            assert_eq!(a, b);
            assert_eq!(
                a.palette(false)[0].to_colorref(),
                color(colors[0]).to_colorref()
            );
        }
        assert!(!Appearance {
            mode: Mode::Custom,
            colors: PRESETS[1].1
        }
        .is_dark(true));
    }
    #[test]
    fn missing_preferences_default_safely() {
        assert_eq!(
            serde_json::from_str::<Appearance>("{}").unwrap(),
            Appearance::default()
        );
    }
}
