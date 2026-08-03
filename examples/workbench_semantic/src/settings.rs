//! Local Workbench settings and persistence.
//!
//! Generic host responsibility (local file read/write), deliberately kept
//! separate from Workbench screen/command composition: this module knows
//! nothing about jobs, tabs, diagnostics, or any other Workbench-specific
//! concept -- only "a small local JSON file of recent projects."

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;

use crate::RECENT_PROJECT_SLOT_COUNT;

#[derive(Debug, Clone)]
pub struct Settings {
    pub recent_projects: Vec<PathBuf>,
    pub last_project: Option<PathBuf>,
}

impl Settings {
    #[cfg(not(test))]
    pub fn settings_path() -> PathBuf {
        let base = env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(env::temp_dir);
        base.join("semantic_workbench").join("settings.json")
    }

    // The real settings file is intentionally one shared path per machine
    // (that's the point -- it's how "last open project" persists across
    // launches). But the Rust test harness runs every #[test] concurrently
    // on its own thread within one process, and many tests each construct a
    // WorkbenchApp that reads-modifies-writes that same shared file --
    // without isolation that is a genuine lost-update race between
    // unrelated tests, not a bug in the settings logic itself. Test builds
    // get a path unique per test *thread* instead, which keeps each test's
    // view of "the settings file" consistent for real without changing any
    // production behavior (this function does not exist in non-test builds).
    #[cfg(test)]
    pub fn settings_path() -> PathBuf {
        thread_local! {
            static UNIQUE: PathBuf = {
                let id = format!("{:?}", thread::current().id());
                let safe: String = id.chars().filter(|c| c.is_alphanumeric()).collect();
                env::temp_dir()
                    .join("semantic_workbench_test_settings")
                    .join(format!("{safe}.json"))
            };
        }
        UNIQUE.with(|p| p.clone())
    }

    pub fn load() -> Self {
        let path = Self::settings_path();
        let Ok(text) = fs::read_to_string(&path) else {
            return Self {
                recent_projects: Vec::new(),
                last_project: None,
            };
        };
        // Corrupt-file recovery: any parse failure is treated as "no
        // settings yet", never a crash.
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) else {
            return Self {
                recent_projects: Vec::new(),
                last_project: None,
            };
        };
        let recent_projects = json
            .get("recent_projects")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(PathBuf::from)
                    .collect()
            })
            .unwrap_or_default();
        let last_project = json
            .get("last_project")
            .and_then(|v| v.as_str())
            .map(PathBuf::from);
        Self {
            recent_projects,
            last_project,
        }
    }

    pub fn save(&self) {
        let path = Self::settings_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let recent: Vec<String> = self
            .recent_projects
            .iter()
            .map(|p| p.display().to_string())
            .collect();
        let json = serde_json::json!({
            "recent_projects": recent,
            "last_project": self.last_project.as_ref().map(|p| p.display().to_string()),
        });
        let body = serde_json::to_string_pretty(&json).unwrap_or_default();
        // Atomic write (temp file + rename) so a concurrent `load()` -- e.g.
        // from another WorkbenchApp instance in a parallel test thread, or a
        // future multi-window session -- can never observe a torn/partial
        // write of this shared file.
        let unique = format!("{:?}-{}", thread::current().id(), std::process::id());
        let tmp_path = path.with_extension(format!("json.{unique}.tmp"));
        if fs::write(&tmp_path, body).is_ok() {
            let _ = fs::rename(&tmp_path, &path);
        }
    }

    pub fn record_project(&mut self, path: &Path) {
        self.recent_projects.retain(|p| p != path);
        self.recent_projects.insert(0, path.to_path_buf());
        self.recent_projects.truncate(RECENT_PROJECT_SLOT_COUNT);
        self.last_project = Some(path.to_path_buf());
        self.save();
    }

    pub fn clear(&mut self) {
        self.recent_projects.clear();
        self.last_project = None;
        self.save();
    }
}
