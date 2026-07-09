//! Appearance color palette and tint helpers for custom themes.

/// Default text — warm stone, WCAG AA on [`DEFAULT_BG_COLOR`].
pub const DEFAULT_TEXT_COLOR: &str = "#1c1917";
/// Default popup background — warm paper white.
pub const DEFAULT_BG_COLOR: &str = "#faf8f5";
/// Holiday accent — deep Persian crimson.
pub const DEFAULT_HOLIDAY_COLOR: &str = "#a61e2e";
/// Today accent — Persian tile blue-teal.
pub const DEFAULT_TODAY_COLOR: &str = "#0f6e9c";
/// Prayer accent — calm emerald.
pub const DEFAULT_PRAYER_COLOR: &str = "#1a7a4c";

const TODAY_BG_ALPHA: f32 = 0.22;
const HOLIDAY_BG_ALPHA: f32 = 0.14;

#[derive(Debug, Clone, PartialEq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// Parsed appearance colors with derived background tints for day cells.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppearanceTints {
    pub today_background: String,
    pub holiday_background: String,
}

/// Returns true for `#RRGGBB` hex strings.
pub fn is_valid_hex(color: &str) -> bool {
    let bytes = color.as_bytes();
    bytes.len() == 7
        && bytes[0] == b'#'
        && bytes[1..]
            .iter()
            .all(|b| b.is_ascii_hexdigit())
}

pub fn parse_hex(color: &str) -> Option<Rgb> {
    if !is_valid_hex(color) {
        return None;
    }
    let r = u8::from_str_radix(&color[1..3], 16).ok()?;
    let g = u8::from_str_radix(&color[3..5], 16).ok()?;
    let b = u8::from_str_radix(&color[5..7], 16).ok()?;
    Some(Rgb { r, g, b })
}

/// CSS `rgba(r, g, b, a)` string for translucent overlays.
pub fn rgba_css(color: &str, alpha: f32) -> Option<String> {
    let rgb = parse_hex(color)?;
    let alpha = alpha.clamp(0.0, 1.0);
    Some(format!(
        "rgba({}, {}, {}, {:.2})",
        rgb.r, rgb.g, rgb.b, alpha
    ))
}

/// Background tints aligned with GNOME Shell stylesheet alpha values.
pub fn appearance_tints(today_color: &str, holiday_color: &str) -> AppearanceTints {
    AppearanceTints {
        today_background: rgba_css(today_color, TODAY_BG_ALPHA)
            .unwrap_or_else(|| "rgba(15, 110, 156, 0.22)".into()),
        holiday_background: rgba_css(holiday_color, HOLIDAY_BG_ALPHA)
            .unwrap_or_else(|| "rgba(166, 30, 46, 0.14)".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_hex_colors() {
        assert!(is_valid_hex("#1c1917"));
        assert!(is_valid_hex("#A61E2E"));
        assert!(!is_valid_hex("red"));
        assert!(!is_valid_hex("#abc"));
    }

    #[test]
    fn rgba_css_formats_alpha() {
        assert_eq!(
            rgba_css("#0f6e9c", 0.22),
            Some("rgba(15, 110, 156, 0.22)".into())
        );
    }

    #[test]
    fn appearance_tints_use_defaults_on_invalid_input() {
        let tints = appearance_tints("invalid", "also-bad");
        assert_eq!(tints.today_background, "rgba(15, 110, 156, 0.22)");
        assert_eq!(tints.holiday_background, "rgba(166, 30, 46, 0.14)");
    }
}
