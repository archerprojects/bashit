// toolbar.rs — File, Save, Export, Clean, Merge, New Profile, profile switcher.

use eframe::egui;
use crate::app::{BashItApp, ConfirmAction};
use crate::core::profile::{Binding, CURRENT_BASH};

pub fn show(ctx: &egui::Context, app: &mut BashItApp) {
    egui::TopBottomPanel::top("toolbar")
        .min_height(48.0)
        .show(ctx, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                ui.spacing_mut().button_padding = egui::vec2(14.0, 8.0);

                let btn = |s: &str| egui::RichText::new(s).size(15.0);

                if ui.button(btn("📂 File")).clicked() {
                    app.file_browser.open();
                }

                if ui.button(btn("💾 Save")).clicked() {
                    app.new_profile_name = String::new();
                    app.show_save = true;
                }

                if ui.button(btn("📤 Export")).clicked() {
                    app.confirm_action = Some(ConfirmAction::Export);
                }

                if ui.button(btn("🧹 Clean")).clicked() {
                    app.confirm_action = Some(ConfirmAction::Clean);
                }

                if ui.button(btn("🔀 Merge")).clicked() {
                    if merge_targets(app).is_empty() {
                        app.status_message = "No other target to merge into.".to_string();
                    } else {
                        app.show_merge_picker = true;
                    }
                }

                if ui.button(btn("➕ New Profile")).clicked() {
                    app.new_profile_name = String::new();
                    app.confirm_action = Some(ConfirmAction::NewProfile);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if !app.status_message.is_empty() {
                        ui.label(egui::RichText::new(&app.status_message).size(13.0));
                    }
                });
            });
            ui.add_space(6.0);
        });
}

/// Targets the current window can merge into: current_bash (unless already
/// there) plus every profile except the one currently bound.
pub fn merge_targets(app: &BashItApp) -> Vec<String> {
    let mut targets = vec![];
    if app.profile_manager.binding != Binding::CurrentBash {
        targets.push(CURRENT_BASH.to_string());
    }
    let current_profile = app.profile_manager.binding.profile_name();
    for name in app.profile_manager.list_profiles() {
        if Some(name.as_str()) != current_profile {
            targets.push(name);
        }
    }
    targets
}
