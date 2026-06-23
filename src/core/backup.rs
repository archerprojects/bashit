// backup.rs — silent timestamped backup of a file before a destructive op.
// Backs up the artifact about to be mutated (~/.bash_history or a profile .hist),
// written to ~/.bhistory/backups/. Fails gracefully; logs nothing to the UI.

use std::path::Path;
use chrono::Local;

/// Copy `source` to ~/.bhistory/backups/<filename>_<timestamp>.bak.
/// No-op if the source does not exist.
pub fn backup_file(source: &Path, storage_dir: &Path) -> std::io::Result<()> {
    if !source.exists() {
        return Ok(());
    }

    let backups_dir = storage_dir.join("backups");
    std::fs::create_dir_all(&backups_dir)?;

    let name = source
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("backup");
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let dest = backups_dir.join(format!("{name}_{timestamp}.bak"));

    std::fs::copy(source, &dest)?;
    Ok(())
}
