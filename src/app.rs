// app.rs — top-level app state and event loop.

use eframe::egui;
use crate::core::profile::{Binding, ProfileManager};
use crate::theme;
use crate::ui;
use crate::ui::filebrowser::{FileBrowser, FileBrowserResult};

/// Destructive or confirmable actions routed through the confirm dialog.
#[derive(Debug, Clone)]
pub enum ConfirmAction {
    SwitchProfile(String),
    SwitchToCurrentBash,
    Clean,
    UnlockAll,
    Export,
    NewProfile,
}

pub struct BashItApp {
    pub profile_manager: ProfileManager,
    pub confirm_action: Option<ConfirmAction>,
    pub status_message: String,
    pub file_browser: FileBrowser,

    // dialog visibility
    pub show_startup: bool,
    pub show_save: bool,
    pub show_merge_picker: bool,

    // dialog field state
    pub startup_profile_selection: String,
    pub new_profile_name: String,
    pub merge_target_selection: String,
}

impl BashItApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply(&cc.egui_ctx);

        let mut profile_manager = ProfileManager::new();
        profile_manager.load_current_bash().ok();

        let profiles = profile_manager.list_profiles();
        let startup_profile_selection = profiles.first().cloned().unwrap_or_default();
        let merge_target_selection = profiles.first().cloned().unwrap_or_default();

        Self {
            profile_manager,
            confirm_action: None,
            status_message: String::new(),
            file_browser: FileBrowser::new(),
            show_startup: true,
            show_save: false,
            show_merge_picker: false,
            startup_profile_selection,
            new_profile_name: String::new(),
            merge_target_selection,
        }
    }
}

impl eframe::App for BashItApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        theme::apply(ctx);

        if self.show_startup {
            ui::dialog::show_startup(ctx, self);
            return;
        }

        if self.show_save {
            ui::dialog::show_save(ctx, self);
            return;
        }

        if self.show_merge_picker {
            ui::dialog::show_merge_picker(ctx, self);
            return;
        }

        if let Some(result) = self.file_browser.show(ctx) {
            match result {
                FileBrowserResult::Open(path) => {
                    match self.profile_manager.load_file(&path) {
                        Ok(_) => self.status_message =
                            "File loaded. Lock commands to keep, then Save, Merge or Export.".to_string(),
                        Err(e) => self.status_message = format!("Open error: {e}"),
                    }
                }
                FileBrowserResult::Delete(path) => {
                    let json_path = path.with_extension("json");
                    std::fs::remove_file(&path).ok();
                    std::fs::remove_file(&json_path).ok();
                    let deleted = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_string();
                    if self.profile_manager.binding == Binding::Profile(deleted.clone()) {
                        self.profile_manager.load_current_bash().ok();
                    }
                    self.status_message = format!("Deleted profile '{deleted}'");
                }
            }
        }

        if self.confirm_action.is_some() {
            ui::dialog::show_confirm(ctx, self);
            return;
        }

        ui::toolbar::show(ctx, self);
        ui::footer::show(ctx, self);
        ui::list::show(ctx, self);
    }
}
