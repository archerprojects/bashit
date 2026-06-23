// profile.rs — the active window and what it is bound to.
//
// Model
// -----
// The window is always "active". It is *bound* to exactly one backing store:
//   - CurrentBash : ~/.bash_history          (shown as "current_bash")
//   - Profile(n)  : ~/.bhistory/profiles/n.* (a named, saved set)
//   - File(p)     : a loose imported file     (scratch — never written back)
//
// Verbs
// -----
//   Save   : write the WHOLE window to its backing store (replace), or to a new
//            profile. Live -> ~/.bash_history, Profile -> profile files,
//            File -> new profile only.
//   Merge  : append the whole window onto a target (current_bash or a profile),
//            lock states preserved, nothing removed.
//   Export : replace ~/.bash_history with the whole window.
//   Clean  : drop unlocked, then write what remains to the backing store
//            (File: memory only — no file touched).
//
// Writes to ~/.bash_history only ever happen on a deliberate Save / Export /
// Merge-to-current_bash / Clean-in-current_bash. Loading or switching never
// writes anything.

use std::path::{Path, PathBuf};
use crate::core::history::{
    CommandEntry, parse_hist, serialize_hist, export_to_bash_history, atomic_write,
};
use crate::core::sidecar::Sidecar;
use crate::core::{clean, backup};

/// The reserved, displayed name for the live ~/.bash_history view.
pub const CURRENT_BASH: &str = "current_bash";

/// What the active window is bound to.
#[derive(Debug, Clone, PartialEq)]
pub enum Binding {
    CurrentBash,
    Profile(String),
    File(PathBuf),
}

impl Binding {
    /// Label shown in the switcher and dialogs.
    pub fn display(&self) -> String {
        match self {
            Binding::CurrentBash => CURRENT_BASH.to_string(),
            Binding::Profile(name) => name.clone(),
            Binding::File(path) => path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("file")
                .to_string(),
        }
    }

    pub fn profile_name(&self) -> Option<&str> {
        match self {
            Binding::Profile(name) => Some(name.as_str()),
            _ => None,
        }
    }
}

pub struct ProfileManager {
    pub storage_dir: PathBuf,
    pub profiles_dir: PathBuf,
    pub binding: Binding,
    pub entries: Vec<CommandEntry>,
    pub sidecar: Option<Sidecar>,
    /// True when lock selections have been changed but not yet saved to the
    /// current backing store. Drives the soft "won't persist" switch reminder.
    pub dirty: bool,
}

impl ProfileManager {
    pub fn new() -> Self {
        let home = dirs::home_dir().expect("cannot locate home directory");
        let storage_dir = home.join(".bhistory");
        let profiles_dir = storage_dir.join("profiles");
        std::fs::create_dir_all(&profiles_dir).ok();
        std::fs::create_dir_all(storage_dir.join("backups")).ok();

        Self {
            storage_dir,
            profiles_dir,
            binding: Binding::CurrentBash,
            entries: vec![],
            sidecar: None,
            dirty: false,
        }
    }

    // --- paths -----------------------------------------------------------

    fn bash_history_path(&self) -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/"))
            .join(".bash_history")
    }

    fn hist_path(&self, name: &str) -> PathBuf {
        self.profiles_dir.join(format!("{name}.hist"))
    }

    fn json_path(&self, name: &str) -> PathBuf {
        self.profiles_dir.join(format!("{name}.json"))
    }

    // --- profile discovery ----------------------------------------------

    pub fn list_profiles(&self) -> Vec<String> {
        let mut names = vec![];
        if let Ok(rd) = std::fs::read_dir(&self.profiles_dir) {
            for entry in rd.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("hist")
                    && std::fs::metadata(&path).map(|m| m.len() > 0).unwrap_or(false)
                {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        names.push(stem.to_string());
                    }
                }
            }
        }
        names.sort();
        names
    }

    pub fn profile_exists(&self, name: &str) -> bool {
        let path = self.hist_path(name);
        path.exists() && std::fs::metadata(&path).map(|m| m.len() > 0).unwrap_or(false)
    }

    /// current_bash, "live", and "live session" are reserved (case-insensitive),
    /// as is an empty name.
    pub fn is_reserved_name(name: &str) -> bool {
        let n = name.trim().to_lowercase();
        n.is_empty() || n == CURRENT_BASH || n == "live" || n == "live session"
    }

    // --- loading (read-only — never writes) -----------------------------

    /// Load live ~/.bash_history into the window.
    pub fn load_current_bash(&mut self) -> std::io::Result<()> {
        let content = std::fs::read_to_string(self.bash_history_path()).unwrap_or_default();
        self.entries = parse_hist(&content);
        self.binding = Binding::CurrentBash;
        self.sidecar = None;
        self.dirty = false;
        Ok(())
    }

    /// Load a named profile into the window. Read-only: no bash write,
    /// no profile rewrite. Lock states come from the sidecar.
    pub fn load_profile(&mut self, name: &str) -> std::io::Result<()> {
        let content = std::fs::read_to_string(self.hist_path(name)).unwrap_or_default();
        let mut entries = parse_hist(&content);
        let sidecar = Sidecar::load(&self.json_path(name));

        for entry in &mut entries {
            if sidecar.residents.contains(&entry.id) {
                entry.resident = true;
            }
        }

        self.entries = entries;
        self.sidecar = Some(sidecar);
        self.binding = Binding::Profile(name.to_string());
        self.dirty = false;
        Ok(())
    }

    /// Load a loose external file into the window as scratch.
    pub fn load_file(&mut self, path: &Path) -> std::io::Result<()> {
        let content = std::fs::read_to_string(path)?;
        self.entries = parse_hist(&content);
        self.binding = Binding::File(path.to_path_buf());
        self.sidecar = None;
        self.dirty = false;
        Ok(())
    }

    // --- helpers ---------------------------------------------------------

    pub fn locked_count(&self) -> usize {
        self.entries.iter().filter(|e| e.resident).count()
    }

    /// Write the whole window to a profile's files, refreshing resident IDs.
    fn write_profile_files(&mut self, name: &str) -> std::io::Result<()> {
        let serialized = serialize_hist(&self.entries);
        atomic_write(&self.hist_path(name), &serialized)?;

        let mut sidecar = self.sidecar.clone().unwrap_or_else(|| Sidecar::new(name));
        sidecar.profile_name = name.to_string();
        sidecar.residents = self
            .entries
            .iter()
            .filter(|e| e.resident)
            .map(|e| e.id.clone())
            .collect();
        sidecar.save(&self.json_path(name))?;
        self.sidecar = Some(sidecar);
        Ok(())
    }

    // --- Save ------------------------------------------------------------

    /// Save the whole window to its backing store (replace). Backs up first.
    /// File binding has no store — callers route File saves to a new profile.
    pub fn save(&mut self) -> std::io::Result<()> {
        match self.binding.clone() {
            Binding::CurrentBash => {
                backup::backup_file(&self.bash_history_path(), &self.storage_dir).ok();
                export_to_bash_history(&self.entries)?;
            }
            Binding::Profile(name) => {
                backup::backup_file(&self.hist_path(&name), &self.storage_dir).ok();
                self.write_profile_files(&name)?;
            }
            Binding::File(_) => return Ok(()),
        }
        self.dirty = false;
        Ok(())
    }

    /// Write the whole window to a brand-new profile. Binding is unchanged —
    /// the snapshot is a separate copy; the user stays where they are.
    pub fn save_as_new_profile(&mut self, name: &str) -> std::io::Result<()> {
        if Self::is_reserved_name(name) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("'{name}' is a reserved name"),
            ));
        }
        if self.profile_exists(name) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!("Profile '{name}' already exists"),
            ));
        }
        let serialized = serialize_hist(&self.entries);
        atomic_write(&self.hist_path(name), &serialized)?;

        let mut sidecar = Sidecar::new(name);
        sidecar.residents = self
            .entries
            .iter()
            .filter(|e| e.resident)
            .map(|e| e.id.clone())
            .collect();
        sidecar.save(&self.json_path(name))?;
        Ok(())
    }

    /// Create a blank profile and switch the window into it.
    pub fn create_blank_profile(&mut self, name: &str) -> std::io::Result<()> {
        if Self::is_reserved_name(name) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("'{name}' is a reserved name"),
            ));
        }
        if self.profile_exists(name) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!("Profile '{name}' already exists"),
            ));
        }
        atomic_write(&self.hist_path(name), "# BashIt profile\n")?;
        let sidecar = Sidecar::new(name);
        sidecar.save(&self.json_path(name))?;

        self.entries = vec![];
        self.sidecar = Some(sidecar);
        self.binding = Binding::Profile(name.to_string());
        self.dirty = false;
        Ok(())
    }

    // --- Merge (append, locks preserved) --------------------------------

    /// Append the whole window onto a target store. Nothing is removed.
    /// File targets are rejected (a loose file is not a store).
    pub fn merge_into(&mut self, target: &Binding) -> std::io::Result<()> {
        match target {
            Binding::CurrentBash => {
                let path = self.bash_history_path();
                backup::backup_file(&path, &self.storage_dir).ok();
                let existing = std::fs::read_to_string(&path).unwrap_or_default();
                let mut combined = parse_hist(&existing);
                combined.extend(self.entries.clone());
                export_to_bash_history(&combined)
            }
            Binding::Profile(name) => {
                backup::backup_file(&self.hist_path(name), &self.storage_dir).ok();
                let existing = std::fs::read_to_string(self.hist_path(name)).unwrap_or_default();
                let mut combined = parse_hist(&existing);
                let mut sidecar = Sidecar::load(&self.json_path(name));

                for entry in &mut combined {
                    if sidecar.residents.contains(&entry.id) {
                        entry.resident = true;
                    }
                }
                combined.extend(self.entries.clone());

                sidecar.residents = combined
                    .iter()
                    .filter(|e| e.resident)
                    .map(|e| e.id.clone())
                    .collect();

                atomic_write(&self.hist_path(name), &serialize_hist(&combined))?;
                sidecar.save(&self.json_path(name))?;
                Ok(())
            }
            Binding::File(_) => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "cannot merge into a loose file",
            )),
        }
    }

    // --- Export ----------------------------------------------------------

    /// Replace ~/.bash_history with the whole window. Backs up first.
    pub fn export_to_bash(&mut self) -> std::io::Result<()> {
        backup::backup_file(&self.bash_history_path(), &self.storage_dir).ok();
        export_to_bash_history(&self.entries)?;
        if self.binding == Binding::CurrentBash {
            self.dirty = false;
        }
        Ok(())
    }

    // --- Clean -----------------------------------------------------------

    /// Drop unlocked entries, then persist what remains to the backing store.
    /// File binding cleans in memory only — the source file is never touched.
    pub fn clean(&mut self) -> std::io::Result<()> {
        match self.binding.clone() {
            Binding::CurrentBash => {
                backup::backup_file(&self.bash_history_path(), &self.storage_dir).ok();
                clean::run(&mut self.entries);
                export_to_bash_history(&self.entries)?;
                self.dirty = false;
            }
            Binding::Profile(name) => {
                backup::backup_file(&self.hist_path(&name), &self.storage_dir).ok();
                clean::run(&mut self.entries);
                self.write_profile_files(&name)?;
                self.dirty = false;
            }
            Binding::File(_) => {
                clean::run(&mut self.entries);
                self.dirty = true; // cleaned view not persisted anywhere
            }
        }
        Ok(())
    }

    // --- lock state ------------------------------------------------------

    pub fn toggle_resident(&mut self, id: &str) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.id == id) {
            entry.resident = !entry.resident;
        }
        if let Some(sidecar) = &mut self.sidecar {
            if sidecar.residents.iter().any(|r| r == id) {
                sidecar.residents.retain(|r| r != id);
            } else {
                sidecar.residents.push(id.to_string());
            }
        }
        self.dirty = true;
    }

    pub fn lock_all(&mut self) {
        for entry in &mut self.entries {
            entry.resident = true;
        }
        if let Some(sidecar) = &mut self.sidecar {
            sidecar.residents = self.entries.iter().map(|e| e.id.clone()).collect();
        }
        self.dirty = true;
    }

    pub fn unlock_all(&mut self) {
        for entry in &mut self.entries {
            entry.resident = false;
        }
        if let Some(sidecar) = &mut self.sidecar {
            sidecar.residents.clear();
        }
        self.dirty = true;
    }
}
