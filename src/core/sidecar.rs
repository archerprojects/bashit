// sidecar.rs — read/write .json sidecar (profile metadata + resident IDs)
// Developer: archerprojects <archer.projects@proton.me>

use serde::{Deserialize, Serialize};
use chrono::Utc;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sidecar {
    pub profile_name: String,
    pub created: String,
    pub last_loaded: String,
    pub residents: Vec<String>, // list of bhid strings
}

impl Sidecar {
    pub fn new(profile_name: &str) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            profile_name: profile_name.to_string(),
            created: now.clone(),
            last_loaded: now,
            residents: vec![],
        }
    }

    pub fn load(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| Self::new(
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown"),
            ))
    }

    /// Update last_loaded to now — call when a profile is opened.
    #[allow(dead_code)]
    pub fn touch(&mut self) {
        self.last_loaded = Utc::now().to_rfc3339();
    }

    /// Crash-safe via the shared atomic writer.
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let content = serde_json::to_string_pretty(self)
            .map_err(std::io::Error::other)?;
        crate::core::history::atomic_write(path, &content)
    }
}
