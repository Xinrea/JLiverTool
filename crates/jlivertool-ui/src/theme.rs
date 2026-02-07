//! Theme configuration for JLiverTool

use gpui::*;
use parking_lot::RwLock;
use std::sync::OnceLock;

/// Global theme state
static CURRENT_THEME: OnceLock<RwLock<ThemeColors>> = OnceLock::new();

/// Get the current theme colors
fn current_theme() -> &'static RwLock<ThemeColors> {
    CURRENT_THEME.get_or_init(|| RwLock::new(ThemeColors::dark()))
}

/// Set the current theme by name
pub fn set_theme(name: &str) {
    let theme = match name {
        "light" => ThemeColors::light(),
        "dark" => ThemeColors::dark(),
        "dracula" => ThemeColors::dracula(),
        "catppuccin" => ThemeColors::catppuccin(),
        "blueberry" => ThemeColors::blueberry(),
        "ayu-light" => ThemeColors::ayu_light(),
        "ayu-dark" => ThemeColors::ayu_dark(),
        _ => ThemeColors::dark(),
    };
    *current_theme().write() = theme;
}

/// Update gpui_component theme colors based on current theme
/// This should be called with the GPUI context after set_theme
pub fn update_gpui_component_theme(cx: &mut App) {
    let theme_colors = current_theme().read();
    let gpui_theme = gpui_component::theme::Theme::global_mut(cx);

    // Set background to transparent so window opacity can work
    // The actual background color is applied by each view with opacity
    gpui_theme.colors.background = gpui::transparent_black();

    gpui_theme.colors.foreground = theme_colors.font_color;
    gpui_theme.colors.border = theme_colors.border;
    gpui_theme.colors.primary = theme_colors.uname_color;

    let (r, g, b) = theme_colors.gift_bg;
    gpui_theme.colors.secondary = rgb_to_hsla(r, g, b);

    // Muted color - derived from font color
    let font = theme_colors.font_color;
    gpui_theme.colors.muted = hsla(font.h, font.s * 0.5, font.l * 0.6, 0.6);
    gpui_theme.colors.muted_foreground = hsla(font.h, font.s * 0.3, font.l * 0.8, 0.8);

    // Set caret (cursor) color to be more visible - use the font color for best contrast
    gpui_theme.colors.caret = theme_colors.font_color;

    // List colors for Select dropdown
    let (r, g, b) = theme_colors.main_bg;
    let is_light = (r as u32 + g as u32 + b as u32) > 384; // Light theme detection

    // List active (selected item) - use accent color with proper contrast
    gpui_theme.colors.list_active = theme_colors.uname_color;
    // For selected item text, use white for dark accent, black for light accent
    let accent_lightness = theme_colors.uname_color.l;
    if accent_lightness > 0.5 {
        gpui_theme.colors.foreground = theme_colors.font_color;
    }

    // List hover
    if is_light {
        gpui_theme.colors.list_hover = hsla(0.0, 0.0, 0.9, 1.0);
        gpui_theme.colors.list = hsla(0.0, 0.0, 0.95, 1.0);
    } else {
        gpui_theme.colors.list_hover = hsla(0.0, 0.0, 0.2, 1.0);
        gpui_theme.colors.list = hsla(0.0, 0.0, 0.15, 1.0);
    }

    // Popover colors (for dropdown menus)
    let (r, g, b) = theme_colors.gift_bg;
    gpui_theme.colors.popover = rgb_to_hsla(r, g, b);
    gpui_theme.colors.popover_foreground = theme_colors.font_color;

    // Input colors
    gpui_theme.colors.input = theme_colors.border;

    // Switch colors - derive from theme
    if is_light {
        // Light theme: darker switch background, white thumb
        gpui_theme.colors.switch = hsla(0.0, 0.0, 0.7, 1.0);
        gpui_theme.colors.switch_thumb = hsla(0.0, 0.0, 1.0, 1.0);
    } else {
        // Dark theme: lighter switch background, white thumb
        gpui_theme.colors.switch = hsla(0.0, 0.0, 0.3, 1.0);
        gpui_theme.colors.switch_thumb = hsla(0.0, 0.0, 1.0, 1.0);
    }
}

/// Theme color definitions
#[derive(Clone)]
pub struct ThemeColors {
    /// Title bar background color
    pub titlebar_bg: Hsla,
    /// Title bar text color
    pub titlebar_text: Hsla,
    /// Status bar background color
    pub statusbar_bg: Hsla,
    /// Status bar text color
    pub statusbar_text: Hsla,
    /// Main background color (RGB values 0-255)
    pub main_bg: (u8, u8, u8),
    /// Gift window background (RGB values 0-255)
    pub gift_bg: (u8, u8, u8),
    /// Live indicator color
    pub live_color: Hsla,
    /// Border color
    pub border: Hsla,
    /// Username color
    pub uname_color: Hsla,
    /// General font color
    pub font_color: Hsla,
    /// Special danmu background (RGB values 0-255)
    pub danmu_special_bg: (u8, u8, u8),
    /// Button text color (for text on accent-colored buttons)
    pub button_text: Hsla,
}

impl ThemeColors {
    /// Light theme
    pub fn light() -> Self {
        Self {
            titlebar_bg: rgb_to_hsla(100, 191, 226),      // #64bfe2
            titlebar_text: rgb_to_hsla(255, 255, 255),    // #ffffff
            statusbar_bg: rgb_to_hsla(100, 191, 226),     // #64bfe2
            statusbar_text: rgb_to_hsla(255, 255, 255),   // #ffffff
            main_bg: (241, 242, 243),
            gift_bg: (255, 255, 255),
            live_color: hsla(342.0 / 360.0, 0.82, 0.58, 1.0), // rgba(237, 59, 107, 1)
            border: hsla(0.0, 0.0, 0.7, 0.3),             // gray border
            uname_color: rgb_to_hsla(59, 130, 246),       // #3b82f6 - vibrant blue
            font_color: rgb_to_hsla(46, 56, 77),          // #2e384d
            danmu_special_bg: (229, 241, 249),
            button_text: rgb_to_hsla(255, 255, 255),      // white text on buttons
        }
    }

    /// Dark theme
    pub fn dark() -> Self {
        Self {
            titlebar_bg: rgb_to_hsla(22, 22, 26),         // rgb(22, 22, 26)
            titlebar_text: rgb_to_hsla(232, 232, 232),    // #e8e8e8
            statusbar_bg: rgb_to_hsla(22, 22, 26),        // rgb(22, 22, 26)
            statusbar_text: rgb_to_hsla(232, 232, 232),   // #e8e8e8
            main_bg: (0, 0, 0),
            gift_bg: (20, 20, 20),
            live_color: hsla(342.0 / 360.0, 0.82, 0.58, 1.0), // rgba(237, 59, 107, 1)
            border: hsla(0.0, 0.0, 1.0, 0.31),            // #ffffff50
            uname_color: rgb_to_hsla(153, 153, 153),      // #999
            font_color: rgb_to_hsla(232, 232, 232),       // #e8e8e8
            danmu_special_bg: (20, 31, 39),
            button_text: rgb_to_hsla(255, 255, 255),      // white text on buttons
        }
    }

    /// Dracula theme
    pub fn dracula() -> Self {
        Self {
            titlebar_bg: rgb_to_hsla(53, 55, 70),         // #353746
            titlebar_text: rgb_to_hsla(248, 248, 242),    // #f8f8f2
            statusbar_bg: rgb_to_hsla(53, 55, 70),        // #353746
            statusbar_text: rgb_to_hsla(248, 248, 242),   // #f8f8f2
            main_bg: (25, 26, 33),
            gift_bg: (33, 34, 41),
            live_color: rgb_to_hsla(249, 129, 198),       // #f981c6
            border: rgb_to_hsla(93, 109, 152),            // #5d6d98
            uname_color: rgb_to_hsla(172, 139, 224),      // #ac8be0
            font_color: rgb_to_hsla(234, 236, 233),       // #eaece9
            danmu_special_bg: (20, 31, 39),
            button_text: rgb_to_hsla(255, 255, 255),      // white text on buttons
        }
    }

    /// Catppuccin theme
    pub fn catppuccin() -> Self {
        Self {
            titlebar_bg: rgb_to_hsla(35, 38, 52),         // #232634
            titlebar_text: rgb_to_hsla(248, 248, 242),    // #f8f8f2
            statusbar_bg: rgb_to_hsla(35, 38, 52),        // #232634
            statusbar_text: rgb_to_hsla(248, 248, 242),   // #f8f8f2
            main_bg: (48, 52, 70),
            gift_bg: (54, 59, 73),
            live_color: rgb_to_hsla(239, 159, 118),       // #ef9f76
            border: rgb_to_hsla(93, 109, 152),            // #5d6d98
            uname_color: rgb_to_hsla(166, 209, 137),      // #a6d189
            font_color: rgb_to_hsla(234, 236, 233),       // #eaece9
            danmu_special_bg: (20, 31, 39),
            button_text: rgb_to_hsla(30, 30, 46),         // dark text for light green buttons
        }
    }

    /// Blueberry theme
    pub fn blueberry() -> Self {
        Self {
            titlebar_bg: rgb_to_hsla(25, 29, 40),         // #191d28
            titlebar_text: rgb_to_hsla(220, 236, 251),    // #dcecfb
            statusbar_bg: rgb_to_hsla(25, 29, 40),        // #191d28
            statusbar_text: rgb_to_hsla(220, 236, 251),   // #dcecfb
            main_bg: (29, 33, 48),
            gift_bg: (30, 37, 51),
            live_color: rgb_to_hsla(223, 69, 119),        // #df4577
            border: rgb_to_hsla(57, 62, 90),              // #393e5a
            uname_color: rgb_to_hsla(40, 130, 96),        // #288260
            font_color: rgb_to_hsla(228, 236, 230),       // #e4ece6
            danmu_special_bg: (36, 41, 57),
            button_text: rgb_to_hsla(255, 255, 255),      // white text on buttons
        }
    }

    /// Ayu Light theme
    pub fn ayu_light() -> Self {
        Self {
            titlebar_bg: rgb_to_hsla(252, 172, 41),       // #fcac29
            titlebar_text: rgb_to_hsla(250, 250, 250),    // #fafafa
            statusbar_bg: rgb_to_hsla(250, 250, 250),     // #fafafa
            statusbar_text: rgb_to_hsla(108, 120, 131),   // #6c7883
            main_bg: (250, 250, 250),
            gift_bg: (255, 255, 255),
            live_color: rgb_to_hsla(209, 64, 71),         // #d14047
            border: rgb_to_hsla(220, 222, 225),           // #dcdee1
            uname_color: rgb_to_hsla(255, 135, 30),       // #ff871e
            font_color: rgb_to_hsla(106, 118, 129),       // #6a7681
            danmu_special_bg: (240, 241, 242),
            button_text: rgb_to_hsla(255, 255, 255),      // white text on orange buttons
        }
    }

    /// Ayu Dark theme
    pub fn ayu_dark() -> Self {
        Self {
            titlebar_bg: rgb_to_hsla(9, 14, 21),          // #090e15
            titlebar_text: rgb_to_hsla(255, 255, 255),    // white
            statusbar_bg: rgb_to_hsla(9, 14, 21),         // #090e15
            statusbar_text: rgb_to_hsla(255, 255, 255),   // white
            main_bg: (9, 14, 21),
            gift_bg: (40, 37, 48),
            live_color: rgb_to_hsla(200, 93, 85),         // #c85d55
            border: rgb_to_hsla(57, 62, 90),              // #393e5a
            uname_color: rgb_to_hsla(255, 137, 34),       // #ff8922
            font_color: rgb_to_hsla(179, 177, 173),       // #b3b1ad
            danmu_special_bg: (0, 6, 15),
            button_text: rgb_to_hsla(255, 255, 255),      // white text on orange buttons
        }
    }
}

/// Convert RGB values to Hsla
fn rgb_to_hsla(r: u8, g: u8, b: u8) -> Hsla {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;

    if (max - min).abs() < f32::EPSILON {
        return hsla(0.0, 0.0, l, 1.0);
    }

    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };

    let h = if (max - r).abs() < f32::EPSILON {
        (g - b) / d + (if g < b { 6.0 } else { 0.0 })
    } else if (max - g).abs() < f32::EPSILON {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };

    hsla(h / 6.0, s, l, 1.0)
}

/// Color palette for the application (uses current theme)
pub struct Colors;

impl Colors {
    // Background colors
    pub fn bg_primary() -> Hsla {
        let theme = current_theme().read();
        let (r, g, b) = theme.main_bg;
        rgb_to_hsla(r, g, b)
    }

    /// Background primary with custom opacity
    pub fn bg_primary_with_opacity(opacity: f32) -> Hsla {
        let theme = current_theme().read();
        let (r, g, b) = theme.main_bg;
        let mut color = rgb_to_hsla(r, g, b);
        color.a = opacity;
        color
    }

    pub fn bg_secondary() -> Hsla {
        let theme = current_theme().read();
        let (r, g, b) = theme.gift_bg;
        rgb_to_hsla(r, g, b)
    }

    /// Background secondary with custom opacity
    pub fn bg_secondary_with_opacity(opacity: f32) -> Hsla {
        let theme = current_theme().read();
        let (r, g, b) = theme.gift_bg;
        let mut color = rgb_to_hsla(r, g, b);
        color.a = opacity;
        color
    }

    pub fn bg_hover() -> Hsla {
        let theme = current_theme().read();
        let (r, g, b) = theme.main_bg;
        // Slightly lighter/darker than main bg
        let factor = if r > 128 { 0.9 } else { 1.2 };
        let r = ((r as f32 * factor).min(255.0)) as u8;
        let g = ((g as f32 * factor).min(255.0)) as u8;
        let b = ((b as f32 * factor).min(255.0)) as u8;
        rgb_to_hsla(r, g, b)
    }

    /// Background hover with custom opacity
    pub fn bg_hover_with_opacity(opacity: f32) -> Hsla {
        let theme = current_theme().read();
        let (r, g, b) = theme.main_bg;
        // Slightly lighter/darker than main bg
        let factor = if r > 128 { 0.9 } else { 1.2 };
        let r = ((r as f32 * factor).min(255.0)) as u8;
        let g = ((g as f32 * factor).min(255.0)) as u8;
        let b = ((b as f32 * factor).min(255.0)) as u8;
        let mut color = rgb_to_hsla(r, g, b);
        color.a = opacity;
        color
    }

    /// Sidebar background (darker than primary)
    pub fn bg_sidebar_with_opacity(opacity: f32) -> Hsla {
        let theme = current_theme().read();
        let (r, g, b) = theme.main_bg;
        // Slightly darker than main bg
        let r = (r as f32 * 0.8) as u8;
        let g = (g as f32 * 0.8) as u8;
        let b = (b as f32 * 0.8) as u8;
        let mut color = rgb_to_hsla(r, g, b);
        color.a = opacity;
        color
    }

    /// Special danmu background
    pub fn bg_special() -> Hsla {
        let theme = current_theme().read();
        let (r, g, b) = theme.danmu_special_bg;
        rgb_to_hsla(r, g, b)
    }

    // Text colors
    pub fn text_primary() -> Hsla {
        current_theme().read().font_color
    }

    pub fn text_secondary() -> Hsla {
        let color = current_theme().read().font_color;
        hsla(color.h, color.s, color.l * 0.8, 0.8)
    }

    pub fn text_muted() -> Hsla {
        let color = current_theme().read().font_color;
        hsla(color.h, color.s * 0.5, color.l * 0.6, 0.6)
    }

    // Accent colors (username color from theme)
    pub fn accent() -> Hsla {
        current_theme().read().uname_color
    }

    // Button text color (for text on accent-colored buttons)
    pub fn button_text() -> Hsla {
        current_theme().read().button_text
    }

    // Title bar colors
    pub fn titlebar_bg() -> Hsla {
        current_theme().read().titlebar_bg
    }

    pub fn titlebar_text() -> Hsla {
        current_theme().read().titlebar_text
    }

    // Status bar colors
    pub fn statusbar_bg() -> Hsla {
        current_theme().read().statusbar_bg
    }

    pub fn statusbar_text() -> Hsla {
        current_theme().read().statusbar_text
    }

    // Border color
    pub fn border() -> Hsla {
        current_theme().read().border
    }

    // Status colors
    pub fn success() -> Hsla {
        hsla(120.0 / 360.0, 0.6, 0.5, 1.0)
    }

    pub fn warning() -> Hsla {
        hsla(45.0 / 360.0, 0.9, 0.5, 1.0)
    }

    pub fn error() -> Hsla {
        hsla(0.0, 0.8, 0.5, 1.0)
    }

    /// SuperChat theme color (#297EA1)
    pub fn superchat() -> Hsla {
        hsla(194.0 / 360.0, 0.58, 0.40, 1.0)
    }

    /// LIVE status color
    pub fn live() -> Hsla {
        current_theme().read().live_color
    }

    /// LIVE status color with custom opacity
    pub fn live_with_opacity(opacity: f32) -> Hsla {
        let mut color = current_theme().read().live_color;
        color.a = opacity;
        color
    }

    // Guard colors
    pub fn guard_1() -> Hsla {
        hsla(45.0 / 360.0, 0.9, 0.6, 1.0) // 总督 - Gold
    }

    pub fn guard_2() -> Hsla {
        hsla(280.0 / 360.0, 0.7, 0.6, 1.0) // 提督 - Purple
    }

    pub fn guard_3() -> Hsla {
        hsla(200.0 / 360.0, 0.8, 0.6, 1.0) // 舰长 - Blue
    }

    /// Get guard color by level
    pub fn guard_color(level: u8) -> Hsla {
        match level {
            1 => Self::guard_1(),
            2 => Self::guard_2(),
            3 => Self::guard_3(),
            _ => Self::text_muted(),
        }
    }

    /// Get medal background color based on medal level
    /// Returns (background_color, border_color)
    pub fn medal_colors(level: u8) -> (Hsla, Hsla) {
        match level {
            1..=10 => (
                hsla(230.0 / 360.0, 0.32, 0.50, 0.6),  // rgba(87, 98, 167, 0.6)
                hsla(230.0 / 360.0, 0.32, 0.50, 0.6),  // #5762A799
            ),
            11..=20 => (
                hsla(320.0 / 360.0, 0.44, 0.61, 0.6),  // rgba(199, 112, 164, 0.6)
                hsla(320.0 / 360.0, 0.44, 0.61, 0.6),  // #C770A499
            ),
            21..=30 => (
                hsla(200.0 / 360.0, 0.91, 0.61, 0.6),  // rgba(63, 180, 246, 0.6)
                hsla(200.0 / 360.0, 0.91, 0.61, 0.6),  // #3FB4F699
            ),
            31..=40 => (
                hsla(222.0 / 360.0, 1.0, 0.65, 0.6),   // rgba(76, 125, 255, 0.6)
                hsla(222.0 / 360.0, 1.0, 0.65, 0.6),   // #4C7DFF99
            ),
            _ => (
                hsla(270.0 / 360.0, 0.78, 0.70, 0.6),  // rgba(167, 115, 241, 0.6)
                hsla(285.0 / 360.0, 1.0, 0.74, 1.0),   // #D47AFF
            ),
        }
    }

    /// Convert integer color (from Bilibili API) to Hsla
    pub fn from_int(color: u32) -> Hsla {
        let r = ((color >> 16) & 0xFF) as u8;
        let g = ((color >> 8) & 0xFF) as u8;
        let b = (color & 0xFF) as u8;
        rgb_to_hsla(r, g, b)
    }
}
