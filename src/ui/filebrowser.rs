// filebrowser.rs — custom egui file browser with hidden-file support.
// Path row sits at the top; Open and Delete are always present and grey out
// until the selection is valid (Open: a real file; Delete: a profile .hist).

use eframe::egui;
use std::path::{Path, PathBuf};

pub enum FileBrowserResult {
    Open(PathBuf),
    Delete(PathBuf),
}

pub struct FileBrowser {
    pub visible: bool,
    pub current_dir: PathBuf,
    pub selected_path: Option<PathBuf>,
    pub path_input: String,
    pub confirm_delete: Option<PathBuf>,
    entries: Vec<DirEntry>,
}

#[derive(Clone)]
struct DirEntry {
    path: PathBuf,
    name: String,
    is_dir: bool,
}

impl FileBrowser {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        let mut browser = Self {
            visible: false,
            current_dir: home.clone(),
            selected_path: None,
            path_input: home.to_string_lossy().to_string(),
            confirm_delete: None,
            entries: vec![],
        };
        browser.refresh();
        browser
    }

    pub fn open(&mut self) {
        self.visible = true;
        self.selected_path = None;
        self.confirm_delete = None;
        self.path_input = self.current_dir.to_string_lossy().to_string();
        self.refresh();
    }

    fn select_file(&mut self, path: PathBuf) {
        if let Some(parent) = path.parent() {
            self.current_dir = parent.to_path_buf();
            self.refresh();
        }
        self.path_input = path.to_string_lossy().to_string();
        self.selected_path = Some(path);
        self.visible = true;
    }

    fn navigate_to(&mut self, path: PathBuf) {
        if path.exists() {
            self.current_dir = path;
        } else {
            self.current_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        }
        self.path_input = self.current_dir.to_string_lossy().to_string();
        self.selected_path = None;
        self.refresh();
        self.visible = true;
    }

    fn refresh(&mut self) {
        self.entries = read_dir_sorted(&self.current_dir);
    }

    fn navigate_up(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            self.current_dir = parent.to_path_buf();
            self.path_input = self.current_dir.to_string_lossy().to_string();
            self.selected_path = None;
            self.refresh();
        }
    }

    /// A profile .hist file living under ~/.bhistory/profiles/.
    fn is_profile(path: &Path) -> bool {
        let home = dirs::home_dir().unwrap_or_default();
        let profiles_dir = home.join(".bhistory").join("profiles");
        path.starts_with(&profiles_dir)
            && path.extension().and_then(|e| e.to_str()) == Some("hist")
    }

    /// Returns an action result and closes. None while still open.
    pub fn show(&mut self, ctx: &egui::Context) -> Option<FileBrowserResult> {
        if !self.visible {
            return None;
        }

        // Delete confirmation dialog
        if let Some(del_path) = self.confirm_delete.clone() {
            let mut result = None;
            let mut cancel = false;

            egui::Window::new("Confirm Delete")
                .collapsible(false)
                .resizable(false)
                .min_width(400.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.set_min_width(400.0);
                    ui.add_space(12.0);
                    ui.label(
                        egui::RichText::new("Delete Profile?")
                            .size(18.0)
                            .color(egui::Color32::from_rgb(227, 93, 79)),
                    );
                    ui.add_space(8.0);
                    let name = del_path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown");
                    ui.label(
                        egui::RichText::new(format!("Permanently delete profile '{name}'?"))
                            .size(14.0),
                    );
                    ui.label(egui::RichText::new("This cannot be undone.").size(13.0));
                    ui.add_space(16.0);
                    ui.horizontal(|ui| {
                        if ui
                            .add_sized([180.0, 36.0], egui::Button::new(
                                egui::RichText::new("Delete")
                                    .size(14.0)
                                    .color(egui::Color32::from_rgb(227, 93, 79)),
                            ))
                            .clicked()
                        {
                            result = Some(FileBrowserResult::Delete(del_path));
                            cancel = true;
                        }
                        ui.add_space(8.0);
                        if ui
                            .add_sized([180.0, 36.0], egui::Button::new(
                                egui::RichText::new("Cancel").size(14.0),
                            ))
                            .clicked()
                        {
                            cancel = true;
                        }
                    });
                    ui.add_space(8.0);
                });

            if cancel {
                self.confirm_delete = None;
            }
            if result.is_some() {
                self.visible = false;
                return result;
            }
            return None;
        }

        let mut result: Option<FileBrowserResult> = None;
        let mut close = false;

        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        let bash_history_path = home.join(".bash_history");
        let profiles_dir_path = home.join(".bhistory").join("profiles");

        egui::Window::new("File")
            .collapsible(false)
            .resizable(true)
            .default_size([700.0, 450.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                // Row 1 — navigation
                ui.horizontal(|ui| {
                    if ui.button(egui::RichText::new("↑ Up").size(14.0)).clicked() {
                        self.navigate_up();
                    }
                    ui.label(
                        egui::RichText::new(self.current_dir.to_string_lossy().as_ref()).size(13.0),
                    );
                });

                ui.separator();

                // Row 2 — path + actions (moved to top)
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Path:").size(14.0));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.path_input)
                            .desired_width(360.0)
                            .font(egui::TextStyle::Body),
                    );

                    let selected_path = PathBuf::from(&self.path_input);
                    let can_open = selected_path.exists() && selected_path.is_file();
                    let can_delete = can_open && Self::is_profile(&selected_path);

                    if ui
                        .add_enabled(
                            can_open,
                            egui::Button::new(egui::RichText::new("Open").size(14.0)),
                        )
                        .clicked()
                    {
                        result = Some(FileBrowserResult::Open(selected_path.clone()));
                        close = true;
                    }

                    // Delete — always present, enabled only for profile .hist files
                    if ui
                        .add_enabled(
                            can_delete,
                            egui::Button::new(
                                egui::RichText::new("Delete")
                                    .size(14.0)
                                    .color(egui::Color32::from_rgb(227, 93, 79)),
                            ),
                        )
                        .clicked()
                    {
                        self.confirm_delete = Some(selected_path);
                    }

                    if ui.button(egui::RichText::new("Cancel").size(14.0)).clicked() {
                        close = true;
                    }
                });

                ui.separator();

                // Row 3 — quick access + listing
                ui.columns(2, |cols| {
                    cols[0].label(egui::RichText::new("Quick Access").strong().size(14.0));
                    cols[0].separator();

                    let accent = egui::Color32::from_rgb(75, 139, 212);

                    if cols[0]
                        .add(
                            egui::Label::new(
                                egui::RichText::new("~/.bash_history").color(accent).size(14.0),
                            )
                            .sense(egui::Sense::click()),
                        )
                        .clicked()
                    {
                        self.select_file(bash_history_path.clone());
                    }

                    if cols[0]
                        .add(
                            egui::Label::new(
                                egui::RichText::new("~/.bhistory/profiles").color(accent).size(14.0),
                            )
                            .sense(egui::Sense::click()),
                        )
                        .clicked()
                    {
                        self.navigate_to(profiles_dir_path.clone());
                    }

                    cols[1].label(
                        egui::RichText::new(
                            self.current_dir
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("/"),
                        )
                        .strong()
                        .size(14.0),
                    );
                    cols[1].separator();

                    let entries = self.entries.clone();
                    let selected = self.selected_path.clone();

                    egui::ScrollArea::vertical()
                        .id_source("file_list")
                        .max_height(300.0)
                        .show(&mut cols[1], |ui| {
                            for entry in &entries {
                                // Hide .json sidecar files — internal only
                                if entry.path.extension().and_then(|e| e.to_str()) == Some("json") {
                                    continue;
                                }

                                // Show .hist files by profile name (no extension)
                                let display_name = if entry
                                    .path
                                    .extension()
                                    .and_then(|e| e.to_str())
                                    == Some("hist")
                                {
                                    entry
                                        .path
                                        .file_stem()
                                        .and_then(|s| s.to_str())
                                        .unwrap_or(&entry.name)
                                        .to_string()
                                } else {
                                    entry.name.clone()
                                };

                                let label = if entry.is_dir {
                                    format!("📁 {display_name}")
                                } else {
                                    format!("📄 {display_name}")
                                };

                                let is_selected =
                                    selected.as_ref().map(|s| s == &entry.path).unwrap_or(false);

                                if ui
                                    .selectable_label(
                                        is_selected,
                                        egui::RichText::new(&label).size(14.0),
                                    )
                                    .clicked()
                                {
                                    if entry.is_dir {
                                        self.current_dir = entry.path.clone();
                                        self.path_input =
                                            entry.path.to_string_lossy().to_string();
                                        self.selected_path = None;
                                        self.entries = read_dir_sorted(&self.current_dir);
                                    } else {
                                        self.selected_path = Some(entry.path.clone());
                                        self.path_input =
                                            entry.path.to_string_lossy().to_string();
                                    }
                                }
                            }
                        });
                });
            });

        if close {
            self.visible = false;
            self.selected_path = None;
        }

        result
    }
}

fn read_dir_sorted(path: &Path) -> Vec<DirEntry> {
    let mut dirs = vec![];
    let mut files = vec![];

    if let Ok(rd) = std::fs::read_dir(path) {
        for entry in rd.flatten() {
            let p = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let is_dir = p.is_dir();
            let de = DirEntry { path: p, name, is_dir };
            if is_dir { dirs.push(de); } else { files.push(de); }
        }
    }

    dirs.sort_by_key(|d| d.name.to_lowercase());
    files.sort_by_key(|f| f.name.to_lowercase());
    dirs.extend(files);
    dirs
}
