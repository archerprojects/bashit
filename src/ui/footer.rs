// footer.rs — Lock All, Unlock All, profile selector, and the live count.

use eframe::egui;
use crate::app::{BashItApp, ConfirmAction};
use crate::core::profile::{Binding, CURRENT_BASH};

pub fn show(ctx: &egui::Context, app: &mut BashItApp) {
    egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;
            ui.spacing_mut().button_padding = egui::vec2(12.0, 6.0);

            let btn = |s: &str| egui::RichText::new(s).size(14.0);
            let accent = egui::Color32::from_rgb(75, 139, 212);

            if ui.button(btn("🔒 Lock All")).clicked() {
                app.profile_manager.lock_all();
            }

            if ui.button(btn("🔓 Unlock All")).clicked() {
                app.confirm_action = Some(ConfirmAction::UnlockAll);
            }

            ui.separator();

            ui.label(egui::RichText::new("Profile:").size(14.0));

            let profiles = app.profile_manager.list_profiles();
            let current = app.profile_manager.binding.display();
            let dirty = app.profile_manager.dirty;

            egui::ComboBox::from_id_source("profile_switcher")
                .selected_text(egui::RichText::new(&current).size(14.0))
                .width(170.0)
                .show_ui(ui, |ui| {
                    let on_current_bash =
                        app.profile_manager.binding == Binding::CurrentBash;
                    if ui
                        .selectable_label(
                            on_current_bash,
                            egui::RichText::new(CURRENT_BASH).size(14.0),
                        )
                        .clicked()
                        && !on_current_bash
                    {
                        if dirty {
                            app.confirm_action = Some(ConfirmAction::SwitchToCurrentBash);
                        } else {
                            match app.profile_manager.load_current_bash() {
                                Ok(_) => app.status_message = "Switched to current_bash.".to_string(),
                                Err(e) => app.status_message = format!("Error: {e}"),
                            }
                        }
                    }

                    for name in &profiles {
                        let is_current =
                            app.profile_manager.binding == Binding::Profile(name.clone());
                        if ui
                            .selectable_label(is_current, egui::RichText::new(name).size(14.0))
                            .clicked()
                            && !is_current
                        {
                            if dirty {
                                app.confirm_action =
                                    Some(ConfirmAction::SwitchProfile(name.clone()));
                            } else {
                                match app.profile_manager.load_profile(name) {
                                    Ok(_) => app.status_message = format!("Loaded '{name}'"),
                                    Err(e) => app.status_message = format!("Error: {e}"),
                                }
                            }
                        }
                    }
                });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let total = app.profile_manager.entries.len();
                let locked = app.profile_manager.locked_count();
                ui.label(
                    egui::RichText::new(format!(
                        "{locked} locked  •  {} unlocked  •  {total} total",
                        total - locked
                    ))
                    .size(13.0)
                    .color(accent),
                );
            });
        });
        ui.add_space(4.0);
    });
}
