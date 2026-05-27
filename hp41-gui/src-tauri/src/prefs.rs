//! GUI preference persistence for hp41-gui.
//!
//! Stores user-facing preferences in `~/.hp41/prefs.json` — completely separate from
//! `~/.hp41/autosave.json` which holds the calculator state. This separation is a hard
//! constraint (P59 / THEME-05): theme preference MUST NEVER appear in the autosave file.
//!
//! Design mirrors `persistence.rs` but with two key differences:
//! 1. No `StateFile` version wrapper — prefs.json is simpler (just the struct, pretty-printed).
//! 2. `load_prefs` returns `GuiPrefs` directly (not `Result`) — a missing file is the normal
//!    first-run case, not an error. Corrupt JSON also returns `GuiPrefs::default()`.
//!
//! Phase 49 will add `pub onboarding_done: bool` with `#[serde(default)]` to this struct.
//! The `#[serde(default)]` on each field ensures forward/backward compatibility: a prefs.json
//! written by an older version will load cleanly into a newer struct with new fields defaulted.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// User-facing GUI preferences. Stored in `~/.hp41/prefs.json`.
///
/// THEME-05: This struct must NEVER be embedded in the autosave file or shared with
/// the core calculator state — preferences are fully orthogonal to calculator memory.
///
/// # Field notes
/// - `theme`: one of "dark" | "light" | "classic-beige" | "high-contrast" (D-48.8).
///   Defaults to "dark" on missing prefs.json (first-run default).
///
/// # Future fields (Phase 49)
/// ```rust,ignore
/// #[serde(default)]
/// pub onboarding_done: bool,
/// ```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GuiPrefs {
    #[serde(default = "default_theme")]
    pub theme: String,
}

/// Default theme value — "dark" per D-48.8.
fn default_theme() -> String {
    "dark".to_string()
}

impl Default for GuiPrefs {
    fn default() -> Self {
        GuiPrefs {
            theme: default_theme(),
        }
    }
}

/// Resolve the default preferences file path: `~/.hp41/prefs.json`.
///
/// Fallback: `./.hp41/prefs.json` if `home_dir()` returns None (mirrors `persistence.rs`
/// `default_state_path()` — same pattern, different target file).
pub fn default_prefs_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".hp41")
        .join("prefs.json")
}

/// Persist `prefs` to `path` as pretty-printed JSON.
///
/// Creates the parent directory if it does not exist (mirrors `save_state`).
/// Returns `Err` on I/O failure; the caller should log and continue.
pub fn save_prefs(path: &Path, prefs: &GuiPrefs) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let file = fs::File::create(path)?;
    serde_json::to_writer_pretty(file, prefs).map_err(std::io::Error::other)
}

/// Load preferences from `path`, returning `GuiPrefs::default()` on any failure.
///
/// Unlike `load_state`, this function never returns `Err`:
/// - Missing file → `GuiPrefs::default()` (normal first-run case).
/// - Corrupt JSON → `GuiPrefs::default()` (safe fallback; logs nothing, caller is silent).
///
/// P59 / THEME-05: this function is fully isolated from the calculator state and autosave.json.
pub fn load_prefs(path: &Path) -> GuiPrefs {
    fs::File::open(path)
        .ok()
        .and_then(|file| serde_json::from_reader(file).ok())
        .unwrap_or_default()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir()
            .join(format!("hp41_test_{name}"))
            .join("prefs.json")
    }

    #[test]
    fn test_roundtrip() {
        let path = temp_path("prefs_roundtrip");
        let prefs = GuiPrefs {
            theme: "light".to_string(),
        };
        save_prefs(&path, &prefs).unwrap();
        let loaded = load_prefs(&path);
        assert_eq!(loaded.theme, "light", "roundtrip must preserve theme value");
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn test_missing_file_returns_default() {
        let path = temp_path("prefs_missing_should_not_exist_xyz987");
        // Ensure the path does not exist
        let _ = fs::remove_dir_all(path.parent().unwrap());
        let loaded = load_prefs(&path);
        assert_eq!(
            loaded.theme, "dark",
            "missing prefs.json must return default theme 'dark'"
        );
    }

    #[test]
    fn test_corrupt_json_returns_default() {
        let path = temp_path("prefs_corrupt");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, b"this is not valid json {{ garbage").unwrap();
        let loaded = load_prefs(&path);
        assert_eq!(
            loaded.theme, "dark",
            "corrupt prefs.json must return default theme 'dark'"
        );
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn test_default_theme_is_dark() {
        let prefs = GuiPrefs::default();
        assert_eq!(prefs.theme, "dark", "GuiPrefs::default() must have theme 'dark'");
    }

    #[test]
    fn test_default_prefs_path_ends_with_prefs_json() {
        let path = default_prefs_path();
        assert!(
            path.ends_with(".hp41/prefs.json"),
            "default_prefs_path() must end with .hp41/prefs.json, got: {}",
            path.display()
        );
        assert!(
            !path.to_string_lossy().contains("autosave"),
            "default_prefs_path() must NOT reference autosave.json (THEME-05)"
        );
    }
}
