//! Tray icon loading utilities

use tray_icon::Icon;

/// Embedded default icon (32x32 PNG)
const ICON_DEFAULT: &[u8] = include_bytes!("../../assets/tray/icon.png");

/// Embedded live icon (32x32 PNG)
const ICON_LIVE: &[u8] = include_bytes!("../../assets/tray/icon_live.png");

/// Load the default tray icon
pub fn load_icon() -> Result<Icon, Box<dyn std::error::Error>> {
    load_icon_from_bytes(ICON_DEFAULT)
}

/// Load icon based on live status
pub fn load_icon_for_status(is_live: bool) -> Result<Icon, Box<dyn std::error::Error>> {
    if is_live {
        load_icon_from_bytes(ICON_LIVE)
    } else {
        load_icon_from_bytes(ICON_DEFAULT)
    }
}

/// Load icon from PNG bytes
fn load_icon_from_bytes(bytes: &[u8]) -> Result<Icon, Box<dyn std::error::Error>> {
    // Decode PNG image
    let image = image::load_from_memory(bytes)?;
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();

    Icon::from_rgba(rgba.into_raw(), width, height).map_err(|e| e.into())
}
