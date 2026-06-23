// list.rs — command list. Lock toggle only, no checkboxes.
// Uniform single-line rows; long pipelines reachable by horizontal scroll.

use eframe::egui;
use crate::app::BashItApp;

pub fn show(ctx: &egui::Context, app: &mut BashItApp) {
    egui::CentralPanel::default().show(ctx, |ui| {
        if app.profile_manager.entries.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(
                    egui::RichText::new("No commands loaded. Open a file or load a profile.")
                        .size(15.0),
                );
            });
            return;
        }

        let accent = egui::Color32::from_rgb(75, 139, 212);

        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // Keep rows on one line — the horizontal scrollbar handles
                // long commands rather than wrapping into ragged heights.
                ui.style_mut().wrap = Some(false);

                let ids: Vec<String> = app
                    .profile_manager
                    .entries
                    .iter()
                    .map(|e| e.id.clone())
                    .collect();

                for id in ids {
                    let (resident, command) = {
                        let e = app
                            .profile_manager
                            .entries
                            .iter()
                            .find(|e| e.id == id)
                            .unwrap();
                        (e.resident, e.command.clone())
                    };

                    ui.horizontal(|ui| {
                        let lock_label = if resident { "🔒" } else { "🔓" };
                        let lock_color = if resident { accent } else { egui::Color32::DARK_GRAY };

                        let lock_btn = egui::Button::new(
                            egui::RichText::new(lock_label).color(lock_color).size(15.0),
                        )
                        .frame(false)
                        .fill(egui::Color32::TRANSPARENT);

                        if ui
                            .add(lock_btn)
                            .on_hover_text("🔒 Lock to keep  |  🔓 Unlock to remove on clean")
                            .clicked()
                        {
                            app.profile_manager.toggle_resident(&id);
                        }

                        if resident {
                            ui.label(egui::RichText::new(&command).size(14.0).color(accent));
                        } else {
                            ui.label(egui::RichText::new(&command).size(14.0));
                        }
                    });
                }
            });
    });
}
