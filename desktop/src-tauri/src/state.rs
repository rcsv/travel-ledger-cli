use std::path::PathBuf;
use std::sync::Mutex;

/// Managed desktop state — selected DB path only (no long-lived connection).
pub struct DesktopState {
    pub selected_db_path: Mutex<Option<PathBuf>>,
}

impl Default for DesktopState {
    fn default() -> Self {
        Self {
            selected_db_path: Mutex::new(None),
        }
    }
}
