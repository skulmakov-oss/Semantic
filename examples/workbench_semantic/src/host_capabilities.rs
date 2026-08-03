//! Capability-gated host process boundary.
//!
//! This is the *only* place in the application allowed to spawn a process:
//! every spawn must resolve to an allow-listed executable and a cwd inside
//! the open project root, or it is denied before anything runs. Generic
//! host responsibility -- this module knows nothing about job kinds,
//! Workbench screens, or any other Workbench-specific concept, only
//! "which real executable paths and directories may be spawned into."

use std::env;
use std::path::{Path, PathBuf};

pub struct HostCapabilities {
    allowed_executables: Vec<PathBuf>,
    project_root: PathBuf,
}

impl HostCapabilities {
    pub fn new(project_root: PathBuf) -> Self {
        // `cargo run` places this binary directly in target/<profile>/, but
        // `cargo test` places the test binary in target/<profile>/deps/ --
        // check both so the same allowlist resolution works in either context.
        let exe_dir = env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()));
        let mut search_dirs = Vec::new();
        if let Some(dir) = &exe_dir {
            search_dirs.push(dir.clone());
            if dir.file_name().map(|n| n == "deps").unwrap_or(false) {
                if let Some(parent) = dir.parent() {
                    search_dirs.push(parent.to_path_buf());
                }
            }
        }

        let mut allowed = Vec::new();
        for dir in &search_dirs {
            for name in ["smc.exe", "smc", "svm.exe", "svm"] {
                let candidate = dir.join(name);
                if candidate.is_file() {
                    allowed.push(candidate);
                }
            }
        }
        if let Some(cargo) = resolve_on_path("cargo") {
            allowed.push(cargo);
        }
        if let Some(pwsh) = resolve_on_path("pwsh") {
            allowed.push(pwsh);
        }

        Self {
            allowed_executables: allowed,
            project_root,
        }
    }

    /// Returns Ok(canonical_exe_path) if this spawn is admitted, or Err(reason) if denied.
    pub fn check_spawn(&self, exe_name: &str, cwd: &Path) -> Result<PathBuf, String> {
        let resolved = self
            .allowed_executables
            .iter()
            .find(|p| {
                p.file_stem()
                    .map(|s| s.to_string_lossy().eq_ignore_ascii_case(exe_name))
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| {
                format!(
                    "capability denied: '{exe_name}' is not on the resolved executable allowlist"
                )
            })?;

        let cwd_canon = cwd
            .canonicalize()
            .map_err(|e| format!("capability denied: cwd does not resolve ({e})"))?;
        let root_canon = self
            .project_root
            .canonicalize()
            .map_err(|e| format!("capability denied: project root does not resolve ({e})"))?;
        if !cwd_canon.starts_with(&root_canon) {
            return Err("capability denied: cwd is outside the open project root".to_string());
        }

        Ok(resolved)
    }
}

pub fn resolve_on_path(name: &str) -> Option<PathBuf> {
    let path_var = env::var_os("PATH")?;
    let candidates: &[&str] = if cfg!(windows) {
        &[".exe", ".cmd", ".bat", ""]
    } else {
        &[""]
    };
    for dir in env::split_paths(&path_var) {
        for suffix in candidates {
            let candidate = dir.join(format!("{name}{suffix}"));
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}
