// BashIt — Bash command file curator and profile manager
// Developer: archerprojects <archer.projects@proton.me>

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod core;
mod icons;
mod theme;
mod ui;

use eframe::egui;

fn main() -> eframe::Result<()> {
    check_first_run();

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("BashIt")
        .with_inner_size([800.0, 600.0])
        .with_min_inner_size([600.0, 400.0]);

    if let Some(icon) = icons::load_window_icon() {
        viewport = viewport.with_icon(std::sync::Arc::new(icon));
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "BashIt",
        options,
        Box::new(|cc| Box::new(app::BashItApp::new(cc))),
    )
}

/// On release installs, hand off to the Lean first-run framework once.
/// The framework script owns the flag-file check and the welcome popup;
/// this only triggers it when the flag is absent and the script exists.
fn check_first_run() {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return,
    };
    let flag = home.join(".config/lean/bashit-welcomed");
    let script = std::path::Path::new("/usr/share/lean/firstrun/bashit.sh");
    if !flag.exists() && script.exists() {
        std::process::Command::new("bash")
            .arg(script)
            .spawn()
            .ok();
    }
}
