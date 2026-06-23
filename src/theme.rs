// theme.rs — Lean Linux theming per lean-app-directive-v2
// Detects dark/light via gsettings, applies the Lean palette to egui visuals.
// Re-queried each frame — gsettings output is OS-cached, so the cost is negligible.

use eframe::egui;

pub fn is_dark() -> bool {
    std::process::Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "gtk-theme"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("dark"))
        .unwrap_or(true) // default to dark if undetectable
}

pub fn apply(ctx: &egui::Context) {
    let dark = is_dark();
    let mut visuals = if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };

    // Fixed accent — never conditional per directive (#4b8bd4)
    let accent = egui::Color32::from_rgb(75, 139, 212);

    if dark {
        visuals.window_fill         = egui::Color32::from_rgb(56, 56, 56);   // #383838
        visuals.panel_fill          = egui::Color32::from_rgb(46, 46, 46);   // #2e2e2e
        visuals.faint_bg_color      = egui::Color32::from_rgb(64, 64, 64);   // #404040
        visuals.override_text_color = Some(egui::Color32::from_rgb(240, 240, 240)); // #f0f0f0
    } else {
        visuals.window_fill         = egui::Color32::from_rgb(245, 245, 245); // #f5f5f5
        visuals.panel_fill          = egui::Color32::from_rgb(235, 235, 235); // #ebebeb
        visuals.faint_bg_color      = egui::Color32::from_rgb(255, 255, 255);
        visuals.override_text_color = Some(egui::Color32::from_rgb(26, 26, 26)); // #1a1a1a
    }

    visuals.selection.bg_fill       = egui::Color32::from_rgba_premultiplied(75, 139, 212, 64);
    visuals.hyperlink_color         = accent;
    visuals.widgets.active.bg_fill  = accent;
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(92, 92, 92); // #5c5c5c

    ctx.set_visuals(visuals);
}
