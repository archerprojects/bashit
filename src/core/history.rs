// history.rs — read/write .hist files, ID generation, export to ~/.bash_history

use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CommandEntry {
    pub id: String,
    pub command: String,
    pub resident: bool, // locked = keep: survives clean, included in export
}

impl CommandEntry {
    pub fn new(command: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string()[..6].to_string(),
            command,
            resident: false,
        }
    }
}

/// Parse .hist — handles plain lines and `#bhid:` tagged lines,
/// skips bash HISTTIMEFORMAT timestamps and the blank-profile placeholder.
pub fn parse_hist(content: &str) -> Vec<CommandEntry> {
    content
        .lines()
        .filter(|l| {
            let t = l.trim();
            if t.is_empty() { return false; }
            if t == "# BashIt profile" { return false; }
            if let Some(rest) = t.strip_prefix('#') {
                // bash timestamp line: '#' followed by digits only
                if rest.chars().all(|c| c.is_ascii_digit()) { return false; }
                // any other comment line without a bhid tag is not a command
                if !t.contains("#bhid:") { return false; }
            }
            true
        })
        .map(|line| {
            if let Some(idx) = line.rfind("  #bhid:") {
                let command = line[..idx].to_string();
                let id = line[idx + 8..].trim().to_string();
                CommandEntry { id, command, resident: false }
            } else {
                CommandEntry::new(line.to_string())
            }
        })
        .collect()
}

/// Serialize entries to .hist format with inline IDs.
pub fn serialize_hist(entries: &[CommandEntry]) -> String {
    entries
        .iter()
        .map(|e| format!("{}  #bhid:{}", e.command, e.id))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

/// Replace ~/.bash_history with the given entries — IDs stripped.
pub fn export_to_bash_history(entries: &[CommandEntry]) -> std::io::Result<()> {
    let home = dirs::home_dir().expect("cannot locate home directory");
    let path = home.join(".bash_history");
    let content = entries
        .iter()
        .map(|e| e.command.clone())
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    atomic_write(&path, &content)
}

/// Crash-safe write: stage to a sibling temp file, then rename into place.
/// The temp suffix is appended to the *full* filename so that `daily.hist`
/// and `daily.json` produce distinct temps (`daily.hist.tmp`, `daily.json.tmp`)
/// rather than colliding on a shared `daily.tmp`.
pub fn atomic_write(path: &Path, content: &str) -> std::io::Result<()> {
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);
    std::fs::write(&tmp, content)?;
    std::fs::rename(&tmp, path)
}
