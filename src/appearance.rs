use crate::native_interop::Color;
use serde::{Deserialize, Deserializer, Serialize};

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
    /// WIDGET SIZE as a whole percent of the default size, on top of DPI.
    #[serde(deserialize_with = "deserialize_widget_size")]
    pub widget_size: u32,
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            mode: Mode::System,
            colors: PRESETS[0].1,
            widget_size: WIDGET_SIZE_DEFAULT,
        }
    }
}

pub const WIDGET_SIZE_DEFAULT: u32 = 100;
pub const WIDGET_SIZE_MIN: u32 = 75;
pub const WIDGET_SIZE_MAX: u32 = 200;
pub const WIDGET_SIZE_STEP: u32 = 5;

/// Bring any WIDGET SIZE into range: snapped to the nearest 5% step (112
/// becomes 110, 113 becomes 115, and an exact half such as 112.5 rounds up),
/// then clamped to 75..=200. Not-a-number falls back to 100%.
pub fn snap_widget_size(value: f64) -> u32 {
    if value.is_nan() {
        return WIDGET_SIZE_DEFAULT;
    }
    let step = WIDGET_SIZE_STEP as f64;
    let snapped = (value / step).round() * step;
    snapped.clamp(WIDGET_SIZE_MIN as f64, WIDGET_SIZE_MAX as f64) as u32
}

/// Any JSON number is accepted and snapped, so a hand-edited value such as
/// -10 or 1000 cannot reset the rest of the settings file or break layout.
fn deserialize_widget_size<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u32, D::Error> {
    f64::deserialize(deserializer).map(snap_widget_size)
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

    /// The stored WIDGET SIZE, re-snapped in case it was set out of range.
    pub fn clamped_widget_size(&self) -> u32 {
        snap_widget_size(self.widget_size as f64)
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
                ..Appearance::default()
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
            colors: PRESETS[1].1,
            ..Appearance::default()
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
    #[test]
    fn widget_size_snaps_to_five_percent_steps_within_range() {
        assert_eq!(snap_widget_size(60.0), 75);
        assert_eq!(snap_widget_size(203.0), 200);
        assert_eq!(snap_widget_size(112.0), 110);
        assert_eq!(snap_widget_size(113.0), 115);
        assert_eq!(snap_widget_size(112.5), 115);
        assert_eq!(snap_widget_size(125.0), 125);
        assert_eq!(snap_widget_size(-40.0), 75);
        assert_eq!(snap_widget_size(f64::NAN), 100);
        let a = Appearance {
            widget_size: 999,
            ..Appearance::default()
        };
        assert_eq!(a.clamped_widget_size(), 200);
    }
    #[test]
    fn widget_size_defaults_and_hand_edits_are_snapped_on_load() {
        let old: Appearance =
            serde_json::from_str(r#"{"mode":"dark","colors":[1,2,3,4,5]}"#).unwrap();
        assert_eq!(old.widget_size, 100);
        let edited: Appearance = serde_json::from_str(r#"{"widget_size":-12}"#).unwrap();
        assert_eq!(edited.widget_size, 75);
        let edited: Appearance = serde_json::from_str(r#"{"widget_size":187.4}"#).unwrap();
        assert_eq!(edited.widget_size, 185);
        let a = Appearance {
            widget_size: 150,
            ..Appearance::default()
        };
        let b: Appearance = serde_json::from_str(&serde_json::to_string(&a).unwrap()).unwrap();
        assert_eq!(b.widget_size, 150);
    }
}
