//! Presentation rules shared by the compact widget and tray tooltips.
use crate::models::UsageSection;

pub fn has_reading(reset: &str) -> bool {
    !matches!(reset, "!" | "..." | "n/a" | "")
}

pub fn figure(percent: f64, reset: &str) -> String {
    if has_reading(reset) && percent.is_finite() {
        format!("{:.0}%", percent.clamp(0.0, 100.0))
    } else {
        "—".to_owned()
    }
}

pub fn reset_label(reset: &str) -> &str {
    match reset {
        "!" => "Offline",
        "..." | "" => "Waiting",
        "n/a" => "Not reported",
        "--" => "Unknown",
        other => other,
    }
}

pub fn window_label(section: Option<&UsageSection>, fallback: &str) -> String {
    match section.and_then(|s| s.window_minutes).filter(|m| *m > 0) {
        Some(m) if m % 1440 == 0 => format!("{}d", m / 1440),
        Some(m) if m % 60 == 0 => format!("{}h", m / 60),
        Some(m) => format!("{m}m"),
        None => fallback.to_owned(),
    }
}

pub fn tooltip_row(percent: f64, reset: &str) -> String {
    format!("{} used · {}", figure(percent, reset), reset_label(reset))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_is_not_zero() {
        for reset in ["!", "...", "n/a", ""] {
            assert_eq!(figure(0.0, reset), "—");
        }
        assert_eq!(figure(0.0, "2h 0m"), "0%");
        assert_eq!(figure(42.0, "--"), "42%");
        assert_eq!(figure(f64::NAN, "--"), "—");
    }

    #[test]
    fn percentages_are_bounded() {
        assert_eq!(figure(150.0, "now"), "100%");
        assert_eq!(figure(-1.0, "now"), "0%");
    }

    #[test]
    fn window_length_comes_from_the_provider() {
        let mut section = UsageSection::default();
        for (minutes, expected) in [(15, "15m"), (300, "5h"), (10080, "7d")] {
            section.window_minutes = Some(minutes);
            assert_eq!(window_label(Some(&section), "Short"), expected);
        }
        assert_eq!(window_label(None, "Short"), "Short");
    }
}
