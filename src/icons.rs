// icons.rs — runtime window icon loader.
// Developer: archerprojects <archer.projects@proton.me>
//
// Loads bashit-32.png from the installed hicolor icon theme at startup and
// returns it as an eframe IconData for the viewport builder.
//
// Fallback chain: hicolor → gnome → Adwaita
// If no icon is found the app starts normally with no window icon — no crash.

/// Attempt to load the 32px app icon from the system hicolor tree.
/// Returns None silently if the icon is not installed or cannot be decoded.
pub fn load_window_icon() -> Option<egui::IconData> {
    let themes = ["hicolor", "gnome", "Adwaita"];
    let size = "32x32";

    for theme in &themes {
        let path = format!("/usr/share/icons/{theme}/{size}/apps/bashit.png");
        if let Some(icon) = try_load(&path) {
            return Some(icon);
        }
    }
    None
}

fn try_load(path: &str) -> Option<egui::IconData> {
    let data = std::fs::read(path).ok()?;
    let img = image::load_from_memory_with_format(&data, image::ImageFormat::Png).ok()?;
    let rgba = img.into_rgba8();
    let (width, height) = rgba.dimensions();
    Some(egui::IconData {
        rgba: rgba.into_raw(),
        width,
        height,
    })
}
