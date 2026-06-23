// dialog.rs — all dialogs. Consistent sizing; every destructive action warns.
// Developer: archerprojects <archer.projects@proton.me>

use eframe::egui;
use crate::app::{BashItApp, ConfirmAction};
use crate::core::profile::{Binding, CURRENT_BASH};
use crate::ui::toolbar::merge_targets;

const W: f32 = 500.0;
const BTN_FULL: [f32; 2] = [480.0, 44.0];
const BTN_HALF: [f32; 2] = [228.0, 40.0];

fn accent() -> egui::Color32 { egui::Color32::from_rgb(75, 139, 212) }
fn danger() -> egui::Color32 { egui::Color32::from_rgb(227, 93, 79) }
fn warn()   -> egui::Color32 { egui::Color32::from_rgb(242, 126, 63) }

fn h(s: &str)    -> egui::RichText { egui::RichText::new(s).size(20.0).strong() }
fn body(s: &str) -> egui::RichText { egui::RichText::new(s).size(15.0) }
fn note(s: &str) -> egui::RichText { egui::RichText::new(s).size(13.0) }

fn window<'a>(title: &str) -> egui::Window<'a> {
    egui::Window::new(title.to_string())
        .collapsible(false)
        .resizable(false)
        .min_width(W)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
}

// --- startup ------------------------------------------------------------

pub fn show_startup(ctx: &egui::Context, app: &mut BashItApp) {
    let profiles = app.profile_manager.list_profiles();

    window("BashIt").show(ctx, |ui| {
        ui.set_min_width(W);
        ui.add_space(12.0);
        ui.label(h("What would you like to do?"));
        ui.add_space(16.0);
        ui.separator();
        ui.add_space(12.0);

        if ui
            .add_sized(BTN_FULL, egui::Button::new(
                body("⚡  Edit current_bash  (~/.bash_history)"),
            ))
            .clicked()
        {
            app.show_startup = false;
        }

        if !profiles.is_empty() {
            ui.add_space(12.0);
            ui.separator();
            ui.add_space(12.0);

            ui.label(body("Load existing profile:").color(accent()));
            ui.add_space(6.0);
            egui::ComboBox::from_id_source("startup_profile_select")
                .width(W - 20.0)
                .selected_text(body(&app.startup_profile_selection))
                .show_ui(ui, |ui| {
                    for name in &profiles {
                        ui.selectable_value(
                            &mut app.startup_profile_selection,
                            name.clone(),
                            body(name),
                        );
                    }
                });
            ui.add_space(8.0);
            if ui
                .add_sized(BTN_FULL, egui::Button::new(body("✅  Load Selected Profile")))
                .clicked()
            {
                let name = app.startup_profile_selection.clone();
                match app.profile_manager.load_profile(&name) {
                    Ok(_) => app.status_message = format!("Loaded '{name}'"),
                    Err(e) => app.status_message = format!("Error: {e}"),
                }
                app.show_startup = false;
            }
        }

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(12.0);

        if ui
            .add_sized(BTN_FULL, egui::Button::new(body("📂  Open a File")))
            .clicked()
        {
            app.file_browser.open();
            app.show_startup = false;
        }

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(8.0);
        ui.label(note("Lock the commands you want to keep.").color(egui::Color32::GRAY));
        ui.label(note("Save or Export to preserve your work.").color(egui::Color32::GRAY));
        ui.add_space(12.0);
        ui.separator();
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(note("Developed by archerprojects  ·  ").color(egui::Color32::GRAY));
            ui.hyperlink_to(
                note("github.com/archerprojects/bashit").color(accent()),
                "https://github.com/archerprojects/bashit",
            );
        });
        ui.add_space(8.0);
    });
}

// --- save ---------------------------------------------------------------

pub fn show_save(ctx: &egui::Context, app: &mut BashItApp) {
    let binding = app.profile_manager.binding.clone();
    let count = app.profile_manager.entries.len();

    window("Save").show(ctx, |ui| {
        ui.set_min_width(W);
        ui.add_space(12.0);
        ui.label(h("Save"));
        ui.add_space(8.0);
        ui.label(body(&format!("{count} commands in the current window.")));
        ui.add_space(12.0);
        ui.separator();
        ui.add_space(12.0);

        match &binding {
            Binding::CurrentBash => {
                ui.label(body("Save to current_bash:").color(accent()));
                ui.add_space(4.0);
                ui.label(note("⚠  Replaces ALL of ~/.bash_history. A backup is made first.")
                    .color(warn()));
                ui.add_space(6.0);
                if ui
                    .add_sized(BTN_FULL, egui::Button::new(body("💾  Save to current_bash")))
                    .clicked()
                {
                    match app.profile_manager.save() {
                        Ok(_) => app.status_message = "Saved to current_bash.".to_string(),
                        Err(e) => app.status_message = format!("Save error: {e}"),
                    }
                    app.show_save = false;
                }
                ui.add_space(12.0);
                ui.separator();
                ui.add_space(12.0);
            }
            Binding::Profile(name) => {
                ui.label(body(&format!("Save to profile '{name}':")).color(accent()));
                ui.add_space(4.0);
                ui.label(note(&format!("⚠  Replaces the contents of '{name}'. A backup is made first."))
                    .color(warn()));
                ui.add_space(6.0);
                if ui
                    .add_sized(BTN_FULL, egui::Button::new(body(&format!("💾  Save to {name}"))))
                    .clicked()
                {
                    match app.profile_manager.save() {
                        Ok(_) => app.status_message = format!("Saved to '{name}'."),
                        Err(e) => app.status_message = format!("Save error: {e}"),
                    }
                    app.show_save = false;
                }
                ui.add_space(12.0);
                ui.separator();
                ui.add_space(12.0);
            }
            Binding::File(_) => {
                ui.label(body("This file is not a profile yet — save it as a new profile.")
                    .color(accent()));
                ui.add_space(12.0);
            }
        }

        ui.label(body("Save as new profile:").color(accent()));
        ui.add_space(6.0);
        ui.add_sized(
            [W - 20.0, 36.0],
            egui::TextEdit::singleline(&mut app.new_profile_name)
                .hint_text("profile name")
                .font(egui::TextStyle::Body),
        );
        ui.add_space(8.0);
        if ui
            .add_sized(BTN_FULL, egui::Button::new(body("💾  Save as New Profile")))
            .clicked()
        {
            let name = app.new_profile_name.trim().to_string();
            match app.profile_manager.save_as_new_profile(&name) {
                Ok(_) => {
                    app.status_message = format!("Saved as '{name}'.");
                    app.show_save = false;
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    app.status_message = format!("'{name}' already exists — choose another name.");
                }
                Err(e) if e.kind() == std::io::ErrorKind::InvalidInput => {
                    app.status_message = format!("'{name}' is reserved — choose another name.");
                }
                Err(e) => {
                    app.status_message = format!("Save error: {e}");
                    app.show_save = false;
                }
            }
        }

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(8.0);
        if ui
            .add_sized(BTN_FULL, egui::Button::new(body("Cancel")))
            .clicked()
        {
            app.show_save = false;
        }
        ui.add_space(8.0);
    });
}

// --- merge picker -------------------------------------------------------

pub fn show_merge_picker(ctx: &egui::Context, app: &mut BashItApp) {
    let targets = merge_targets(app);

    if targets.is_empty() {
        app.show_merge_picker = false;
        app.status_message = "No other target to merge into.".to_string();
        return;
    }

    if !targets.contains(&app.merge_target_selection) {
        app.merge_target_selection = targets.first().cloned().unwrap_or_default();
    }

    window("Merge").show(ctx, |ui| {
        ui.set_min_width(W);
        ui.add_space(12.0);
        ui.label(h("Merge"));
        ui.add_space(8.0);
        ui.label(body("Append all commands in the current window onto the target."));
        ui.label(note("Lock states are preserved. Nothing is removed.").color(accent()));
        ui.add_space(16.0);

        egui::ComboBox::from_id_source("merge_target_select")
            .width(W - 20.0)
            .selected_text(body(&app.merge_target_selection))
            .show_ui(ui, |ui| {
                for name in &targets {
                    ui.selectable_value(&mut app.merge_target_selection, name.clone(), body(name));
                }
            });

        ui.add_space(16.0);
        ui.horizontal(|ui| {
            if ui
                .add_sized(BTN_HALF, egui::Button::new(body("Merge").color(accent())))
                .clicked()
            {
                let sel = app.merge_target_selection.clone();
                let target = if sel == CURRENT_BASH {
                    Binding::CurrentBash
                } else {
                    Binding::Profile(sel.clone())
                };
                match app.profile_manager.merge_into(&target) {
                    Ok(_) => app.status_message = format!("Merged into '{sel}'."),
                    Err(e) => app.status_message = format!("Merge error: {e}"),
                }
                app.show_merge_picker = false;
            }
            ui.add_space(8.0);
            if ui
                .add_sized(BTN_HALF, egui::Button::new(body("Cancel")))
                .clicked()
            {
                app.show_merge_picker = false;
            }
        });
        ui.add_space(8.0);
    });
}

// --- confirmations ------------------------------------------------------

pub fn show_confirm(ctx: &egui::Context, app: &mut BashItApp) {
    let action = match &app.confirm_action {
        Some(a) => a.clone(),
        None => return,
    };

    window("Confirm").show(ctx, |ui| {
        ui.set_min_width(W);
        ui.add_space(12.0);

        match &action {
            ConfirmAction::SwitchProfile(name) => {
                ui.label(h("Switch Profile"));
                ui.add_space(12.0);
                ui.label(body(&format!("Switch to profile '{name}'?")));
                ui.label(note("Selections you've made won't persist unless you save the file first.")
                    .color(egui::Color32::GRAY));
                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    if ui.add_sized(BTN_HALF, egui::Button::new(body("Switch"))).clicked() {
                        match app.profile_manager.load_profile(name) {
                            Ok(_) => app.status_message = format!("Loaded '{name}'"),
                            Err(e) => app.status_message = format!("Error: {e}"),
                        }
                        app.confirm_action = None;
                    }
                    ui.add_space(8.0);
                    if ui.add_sized(BTN_HALF, egui::Button::new(body("Cancel"))).clicked() {
                        app.confirm_action = None;
                    }
                });
                ui.add_space(8.0);
            }

            ConfirmAction::SwitchToCurrentBash => {
                ui.label(h("Switch to current_bash"));
                ui.add_space(12.0);
                ui.label(body("Switch to the live ~/.bash_history view?"));
                ui.label(note("Selections you've made won't persist unless you save the file first.")
                    .color(egui::Color32::GRAY));
                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    if ui.add_sized(BTN_HALF, egui::Button::new(body("Switch"))).clicked() {
                        match app.profile_manager.load_current_bash() {
                            Ok(_) => app.status_message = "Switched to current_bash.".to_string(),
                            Err(e) => app.status_message = format!("Error: {e}"),
                        }
                        app.confirm_action = None;
                    }
                    ui.add_space(8.0);
                    if ui.add_sized(BTN_HALF, egui::Button::new(body("Cancel"))).clicked() {
                        app.confirm_action = None;
                    }
                });
                ui.add_space(8.0);
            }

            ConfirmAction::Clean => {
                let total = app.profile_manager.entries.len();
                let locked = app.profile_manager.locked_count();
                let removing = total - locked;
                let target = app.profile_manager.binding.display();
                let is_file = matches!(app.profile_manager.binding, Binding::File(_));

                ui.label(h("Clean").color(danger()));
                ui.add_space(12.0);

                if locked == 0 {
                    ui.label(body("⚠  No commands are locked.").color(danger()));
                    ui.label(body(&format!("All {total} commands will be removed.")));
                } else {
                    ui.label(body(&format!("{removing} unlocked commands will be removed.")));
                    ui.label(body(&format!("{locked} locked commands will remain."))
                        .color(accent()));
                }

                ui.add_space(8.0);
                if is_file {
                    ui.label(note("Removed from the window only — the source file is not changed.")
                        .color(egui::Color32::GRAY));
                } else {
                    ui.label(note(&format!("Result is written to '{target}'. A backup is made first."))
                        .color(egui::Color32::GRAY));
                }

                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    if ui.add_sized(BTN_HALF, egui::Button::new(body("Clean").color(danger()))).clicked() {
                        match app.profile_manager.clean() {
                            Ok(_) => app.status_message = "Cleaned.".to_string(),
                            Err(e) => app.status_message = format!("Clean error: {e}"),
                        }
                        app.confirm_action = None;
                    }
                    ui.add_space(8.0);
                    if ui.add_sized(BTN_HALF, egui::Button::new(body("Cancel"))).clicked() {
                        app.confirm_action = None;
                    }
                });
                ui.add_space(8.0);
            }

            ConfirmAction::UnlockAll => {
                ui.label(h("Unlock All Commands").color(warn()));
                ui.add_space(12.0);
                ui.label(body("All commands will be unlocked."));
                ui.label(body("They will be removed on the next Clean."));
                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    if ui.add_sized(BTN_HALF, egui::Button::new(body("Unlock All").color(warn()))).clicked() {
                        app.profile_manager.unlock_all();
                        app.status_message = "All commands unlocked.".to_string();
                        app.confirm_action = None;
                    }
                    ui.add_space(8.0);
                    if ui.add_sized(BTN_HALF, egui::Button::new(body("Cancel"))).clicked() {
                        app.confirm_action = None;
                    }
                });
                ui.add_space(8.0);
            }

            ConfirmAction::Export => {
                let count = app.profile_manager.entries.len();
                ui.label(h("Export to current_bash").color(warn()));
                ui.add_space(12.0);
                ui.label(body("⚠  This replaces ALL of ~/.bash_history with the current window."));
                ui.label(body(&format!("{count} commands will be written.")));
                ui.label(note("A backup is made first.").color(egui::Color32::GRAY));
                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    if ui.add_sized(BTN_HALF, egui::Button::new(body("Export").color(warn()))).clicked() {
                        match app.profile_manager.export_to_bash() {
                            Ok(_) => app.status_message = "Exported to ~/.bash_history.".to_string(),
                            Err(e) => app.status_message = format!("Export error: {e}"),
                        }
                        app.confirm_action = None;
                    }
                    ui.add_space(8.0);
                    if ui.add_sized(BTN_HALF, egui::Button::new(body("Cancel"))).clicked() {
                        app.confirm_action = None;
                    }
                });
                ui.add_space(8.0);
            }

            ConfirmAction::NewProfile => {
                ui.label(h("New Profile"));
                ui.add_space(12.0);
                ui.label(body("Create a blank profile to build a command set from scratch."));
                ui.add_space(8.0);
                ui.label(body("Profile name:"));
                ui.add_space(6.0);
                ui.add_sized(
                    [W - 20.0, 36.0],
                    egui::TextEdit::singleline(&mut app.new_profile_name)
                        .hint_text("profile name")
                        .font(egui::TextStyle::Body),
                );
                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    if ui.add_sized(BTN_HALF, egui::Button::new(body("Create").color(accent()))).clicked() {
                        let name = app.new_profile_name.trim().to_string();
                        match app.profile_manager.create_blank_profile(&name) {
                            Ok(_) => {
                                app.status_message = format!("Created profile '{name}'");
                                app.confirm_action = None;
                            }
                            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                                app.status_message = format!("'{name}' already exists.");
                            }
                            Err(e) if e.kind() == std::io::ErrorKind::InvalidInput => {
                                app.status_message = format!("'{name}' is reserved — choose another name.");
                            }
                            Err(e) => {
                                app.status_message = format!("Error: {e}");
                                app.confirm_action = None;
                            }
                        }
                    }
                    ui.add_space(8.0);
                    if ui.add_sized(BTN_HALF, egui::Button::new(body("Cancel"))).clicked() {
                        app.confirm_action = None;
                    }
                });
                ui.add_space(8.0);
            }
        }
    });
}
