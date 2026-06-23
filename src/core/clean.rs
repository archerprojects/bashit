// clean.rs — filter logic: keep only locked (resident) commands.

use crate::core::history::CommandEntry;

/// Remove all unlocked entries from the list in place.
pub fn run(entries: &mut Vec<CommandEntry>) {
    entries.retain(|e| e.resident);
}
