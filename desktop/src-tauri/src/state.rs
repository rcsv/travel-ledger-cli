use std::path::PathBuf;
use std::sync::Mutex;

/// Managed desktop state — selected DB path + settings file location.
pub struct DesktopState {
    pub selected_db_path: Mutex<Option<PathBuf>>,
    /// Absolute path to `desktop-settings.json` (injectable for tests).
    pub settings_path: PathBuf,
}

impl DesktopState {
    pub fn new(settings_path: PathBuf) -> Self {
        Self {
            selected_db_path: Mutex::new(None),
            settings_path,
        }
    }
}
