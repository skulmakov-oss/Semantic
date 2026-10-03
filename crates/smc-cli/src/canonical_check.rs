//! SSF-09 #1580: the one canonical check path shared by
//! `smc check --format human|json` and `smc lsp`.
//!
//! This module owns no language semantics. It runs exactly the frozen
//! `smc check` dispatch ([`crate::app::check_root_with_attribution`]:
//! Decision E/F surface authority, the #1933 bundling rules, the
//! `sm-sema` project mechanism), then binds each producer diagnostic to a
//! source and projects it through the producer's own canonical adapter
//! (`SemanticDiagnostic::to_canonical`). It never re-parses, re-types,
//! re-classifies, invents a code, or widens a range.
//!
//! Source binding is fixed by the branch the frozen authority took (see
//! [`crate::app::CheckAttribution`]), never guessed from message text.
//! Canonical logical identity (SSF-09 Decision C) is derived only through
//! package admission (`PackageModuleAdmission`: package name plus the
//! module-root-relative module path); a rootless standalone source has no
//! identity. Host paths are kept for transport routing only and are never
//! part of the external schema.

use crate::app::{check_root_with_attribution, cli_profile, CheckAttribution};
use crate::executable_bundle::{prepare_source_text, PrepareSourceError};
use crate::package_manifest::{
    admit_package_entry_module, resolve_package_import_path,
    resolve_project_root_check_entry_portable,
};
use crate::source_access::{CanonicalSources, SourceAccess};
use sm_diagnostic::{Diagnostic, DiagnosticSeverity, SourceId, SourceRegistry};
use sm_sema::{check_source_with_profile, ModuleProvider, SemanticDiagnostic};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// In-memory source texts that take precedence over the filesystem for the
/// paths they cover (open editor documents). Keys are absolute paths,
/// canonicalized when the file exists on disk.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceOverlay {
    texts: BTreeMap<PathBuf, String>,
}

impl SourceOverlay {
    pub fn new() -> Self {
        Self::default()
    }

    /// Installs `text` as the content of `path`.
    pub fn insert(&mut self, path: &Path, text: String) {
        self.texts.insert(overlay_key(path), text);
    }

    /// Removes any overlay for `path`.
    pub fn remove(&mut self, path: &Path) {
        self.texts.remove(&overlay_key(path));
    }

    /// The overlay text for `path`, if any.
    pub fn get(&self, path: &Path) -> Option<&str> {
        self.texts.get(&overlay_key(path)).map(String::as_str)
    }

    pub fn is_empty(&self) -> bool {
        self.texts.is_empty()
    }
}

fn overlay_key(path: &Path) -> PathBuf {
    let canon = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    crate::source_access::strip_verbatim_prefix(&canon)
}

/// Canonical logical source identity (SSF-09 Decision C): the admitting
/// package's name and the module path relative to that package's
/// `module_root`, with `/` separators (for example `util/helper.sm`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceIdentity {
    pub package: String,
    pub module: String,
}

/// One source a canonical check bound at least one diagnostic (or the
/// checked root) to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedSource {
    pub id: SourceId,
    /// Present only when package admission proves it.
    pub identity: Option<SourceIdentity>,
    /// Presentation-only path relative to the request's display base, with
    /// `/` separators. Never an absolute host path.
    pub display_path: Option<String>,
    /// Transport routing only (an editor URI); never serialized.
    pub host_path: Option<PathBuf>,
    /// The exact text the producer analyzed.
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckStatus {
    Passed,
    Failed,
}

/// The canonical result of one check: sources in registry order and
/// diagnostics in producer emission order (never sorted, never deduped).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalCheckReport {
    pub status: CheckStatus,
    pub sources: Vec<CheckedSource>,
    pub diagnostics: Vec<Diagnostic>,
}

impl CanonicalCheckReport {
    pub fn source(&self, id: SourceId) -> Option<&CheckedSource> {
        self.sources.iter().find(|source| source.id == id)
    }

    /// Registry position of `id` (the external schema's source index).
    pub fn source_index(&self, id: SourceId) -> Option<usize> {
        self.sources.iter().position(|source| source.id == id)
    }

    pub fn error_count(&self) -> usize {
        self.count(DiagnosticSeverity::Error)
    }

    pub fn warning_count(&self) -> usize {
        self.count(DiagnosticSeverity::Warning)
    }

    fn count(&self, severity: DiagnosticSeverity) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == severity)
            .count()
    }
}

/// A check that could not run at all (unreadable input, package admission
/// or project-root failure) or whose producer output violated a carrier
/// invariant. Never presented as a canonical diagnostic: no producer
/// assigned it a code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostCheckFailure {
    pub message: String,
}

impl HostCheckFailure {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Inputs of one canonical check.
#[derive(Debug, Clone, Default)]
pub struct CheckRequest {
    /// A `.sm` file or a project root directory.
    pub entry: PathBuf,
    pub overlay: SourceOverlay,
    /// Canonical directory display paths are made relative to.
    pub display_base: Option<PathBuf>,
    /// Display path for the root when it is not under `display_base`
    /// (the command-line argument the user typed).
    pub root_display_fallback: Option<String>,
}

/// Runs the canonical check.
pub fn check_canonical(request: &CheckRequest) -> Result<CanonicalCheckReport, HostCheckFailure> {
    let root = if request.entry.is_dir() {
        resolve_project_root_check_entry_portable(&request.entry).map_err(HostCheckFailure::new)?
    } else {
        request.entry.clone()
    };
    let parser_profile = cli_profile();
    let mut session = Session::new(request.display_base.as_deref());

    // A document open in an editor but not (yet) on disk has no package
    // admission and no canonical identity: it is checked standalone.
    if !root.exists() {
        let text = request.overlay.get(&root).ok_or_else(|| {
            HostCheckFailure::new(format!("failed to read '{}': not found", root.display()))
        })?;
        let root_id = session.register(
            None,
            None,
            text.to_string(),
            request.root_display_fallback.clone(),
        );
        let result = check_source_with_profile(text, &parser_profile);
        return session.finish(result, CheckAttribution::RawRoot, root_id, &BTreeMap::new());
    }

    // SSF-09 #1580 (R3): host failures name the root as the caller gave it
    // and describe the failure by structured code / `io::ErrorKind` only.
    let root_canon = root.canonicalize().map_err(|e| {
        HostCheckFailure::new(format!(
            "failed to resolve '{}': {}",
            root.display(),
            e.kind()
        ))
    })?;
    let root_canon = crate::source_access::strip_verbatim_prefix(&root_canon);
    let anchor = root_canon
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| root_canon.clone());
    let access = CanonicalSources::new(&request.overlay, &anchor);
    admit_package_entry_module(&root).map_err(|e| {
        HostCheckFailure::new(format!(
            "package module admission failed for '{}': {}",
            root.display(),
            access.describe_admission_error(&e)
        ))
    })?;
    let raw_source = match request.overlay.get(&root_canon) {
        Some(text) => text.to_string(),
        None => std::fs::read_to_string(&root_canon).map_err(|e| {
            HostCheckFailure::new(format!("failed to read '{}': {}", root.display(), e.kind()))
        })?,
    };
    let root_id = session.register(
        Some(root_canon.clone()),
        source_identity(&root_canon),
        raw_source.clone(),
        request.root_display_fallback.clone(),
    );

    let provider = OverlayModuleProvider {
        access: &access,
        served: RefCell::new(BTreeMap::new()),
    };
    let (result, attribution) = match prepare_source_text(raw_source) {
        Ok((source, prepared)) => check_root_with_attribution(
            &root_canon,
            &source,
            prepared,
            &provider,
            &parser_profile,
            &access,
        ),
        Err(PrepareSourceError::Lex { source, .. }) => (
            check_source_with_profile(&source, &parser_profile),
            CheckAttribution::RawRoot,
        ),
        Err(PrepareSourceError::Read(message)) => return Err(HostCheckFailure::new(message)),
    };
    let served = provider.served.into_inner();
    session.finish(result, attribution, root_id, &served)
}

/// Canonical check of text that has no file behind it (an editor buffer
/// with a non-`file:` locator): a rootless standalone source with no
/// canonical identity and no project context.
pub fn check_standalone_text(text: &str) -> Result<CanonicalCheckReport, HostCheckFailure> {
    let mut session = Session::new(None);
    let root_id = session.register(None, None, text.to_string(), None);
    let result = check_source_with_profile(text, &cli_profile());
    session.finish(result, CheckAttribution::RawRoot, root_id, &BTreeMap::new())
}

/// Canonical logical identity of an existing source file, via package
/// admission only. `None` for rootless sources or failed admission.
pub fn source_identity(path: &Path) -> Option<SourceIdentity> {
    match admit_package_entry_module(path) {
        Ok(Some(admission)) => Some(SourceIdentity {
            package: admission.package_name,
            module: admission.module_path,
        }),
        _ => None,
    }
}

struct Session {
    registry: SourceRegistry,
    sources: Vec<CheckedSource>,
    by_path: BTreeMap<PathBuf, SourceId>,
    display_base: Option<PathBuf>,
}

impl Session {
    fn new(display_base: Option<&Path>) -> Self {
        Self {
            registry: SourceRegistry::new(),
            sources: Vec::new(),
            by_path: BTreeMap::new(),
            display_base: display_base.map(|base| {
                let canon = base.canonicalize().unwrap_or_else(|_| base.to_path_buf());
                crate::source_access::strip_verbatim_prefix(&canon)
            }),
        }
    }

    fn register(
        &mut self,
        host_path: Option<PathBuf>,
        identity: Option<SourceIdentity>,
        text: String,
        display_fallback: Option<String>,
    ) -> SourceId {
        let id = self.registry.mint();
        let display_path = host_path
            .as_deref()
            .and_then(|path| self.display_path(path))
            .or(display_fallback);
        if let Some(path) = &host_path {
            self.by_path.insert(path.clone(), id);
        }
        self.sources.push(CheckedSource {
            id,
            identity,
            display_path,
            host_path,
            text,
        });
        id
    }

    fn display_path(&self, path: &Path) -> Option<String> {
        let base = self.display_base.as_ref()?;
        let path = crate::source_access::strip_verbatim_prefix(path);
        let relative = path.strip_prefix(base).ok()?;
        let parts = relative
            .components()
            .map(|c| c.as_os_str().to_str().map(str::to_string))
            .collect::<Option<Vec<_>>>()?;
        if parts.is_empty() {
            None
        } else {
            Some(parts.join("/"))
        }
    }

    /// Binds a provider module id (the project mechanism's module key) to a
    /// registered source, registering it on first use with the exact text
    /// the provider served to the producer.
    fn provider_source(
        &mut self,
        module_id: &str,
        served: &BTreeMap<String, String>,
    ) -> Option<SourceId> {
        let path = PathBuf::from(module_id);
        let canonical = path.canonicalize().unwrap_or(path);
        let canonical = crate::source_access::strip_verbatim_prefix(&canonical);
        if let Some(id) = self.by_path.get(&canonical) {
            return Some(*id);
        }
        let text = served.get(module_id)?.clone();
        let identity = source_identity(&canonical);
        Some(self.register(Some(canonical), identity, text, None))
    }

    fn finish(
        mut self,
        result: Result<sm_sema::SemanticReport, sm_sema::SemanticError>,
        attribution: CheckAttribution,
        root_id: SourceId,
        served: &BTreeMap<String, String>,
    ) -> Result<CanonicalCheckReport, HostCheckFailure> {
        let producer: Vec<SemanticDiagnostic> = match result {
            Ok(report) => report.warnings,
            Err(error) => vec![error.diag],
        };
        let mut diagnostics = Vec::with_capacity(producer.len());
        for diag in &producer {
            let source = match attribution {
                CheckAttribution::RawRoot => Some(root_id),
                CheckAttribution::ComposedBundle => None,
                CheckAttribution::ProviderModules => match &diag.provider_module_id {
                    Some(module_id) => self.provider_source(module_id, served),
                    None => None,
                },
            };
            if let (Some(source), Some(range)) = (source, &diag.canonical.range) {
                let text = &self
                    .sources
                    .iter()
                    .find(|s| s.id == source)
                    .expect("registered source")
                    .text;
                if range.start > range.end
                    || range.end > text.len()
                    || !text.is_char_boundary(range.start)
                    || !text.is_char_boundary(range.end)
                {
                    return Err(HostCheckFailure::new(format!(
                        "internal: producer range {}..{} of {} is not a valid range of its source",
                        range.start, range.end, diag.code
                    )));
                }
            }
            let canonical = diag.to_canonical(source).ok_or_else(|| {
                HostCheckFailure::new(format!(
                    "internal: producer diagnostic {} violates canonical carrier invariants",
                    diag.code
                ))
            })?;
            diagnostics.push(canonical);
        }
        let status = if diagnostics
            .iter()
            .any(|d| d.severity == DiagnosticSeverity::Error)
        {
            CheckStatus::Failed
        } else {
            CheckStatus::Passed
        };
        Ok(CanonicalCheckReport {
            status,
            sources: self.sources,
            diagnostics,
        })
    }
}

/// The project mechanism's module provider over the canonical
/// [`SourceAccess`] seam (editor overlay first), recording the exact text
/// served for every module so diagnostics can be bound to the bytes the
/// producer analyzed. Admission and import resolution are the unmodified
/// canonical package authority; only their failure *text* is portable.
struct OverlayModuleProvider<'a> {
    access: &'a CanonicalSources<'a>,
    served: RefCell<BTreeMap<String, String>>,
}

impl ModuleProvider for OverlayModuleProvider<'_> {
    fn read_module(&self, module_id: &str) -> Result<Vec<u8>, String> {
        let path = Path::new(module_id);
        admit_package_entry_module(path).map_err(|e| self.access.describe_admission_error(&e))?;
        let text = self.access.read_source(path)?;
        self.served
            .borrow_mut()
            .insert(module_id.to_string(), text.clone());
        Ok(text.into_bytes())
    }

    fn resolve_import(&self, importer_module_id: &str, spec: &str) -> Result<String, String> {
        resolve_package_import_path(Path::new(importer_module_id), spec)
            .map(|path| {
                let path = crate::source_access::strip_verbatim_prefix(&path);
                let text = path.to_string_lossy();
                if cfg!(windows) {
                    text.replace('\\', "/")
                } else {
                    text.into_owned()
                }
            })
            .map_err(|e| self.access.describe_resolution_error(&e))
    }

    fn display_module(&self, module_id: &str) -> String {
        self.access.display_path(Path::new(module_id))
    }
}

/// 1-based line and 1-based column (in Unicode scalar values) of byte
/// `offset` in `text`. `offset` must be a char boundary within `text`.
pub fn line_column(text: &str, offset: usize) -> (usize, usize) {
    let before = &text[..offset];
    let line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line = before.matches('\n').count() + 1;
    let column = before[line_start..].chars().count() + 1;
    (line, column)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Component, Prefix};

    #[test]
    #[cfg(windows)]
    fn windows_root_and_resolved_import_representation_equivalence() {
        // Create an isolated temp directory following the PID + timestamp convention.
        use std::time::{SystemTime, UNIX_EPOCH};
        let temp_dir = std::env::temp_dir().join(format!(
            "canonical_check_test_{}_{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(temp_dir.join("src")).expect("mkdir");
        let manifest = temp_dir.join("package.toml");
        std::fs::write(&manifest, "[package]\nname = \"test_pkg\"\nversion = \"0.1.0\"\nmodule_root = \"src\"\n").unwrap();
        let file_a = temp_dir.join("src").join("a.sm");
        let file_b = temp_dir.join("src").join("b.sm");
        std::fs::write(&file_a, "import \"b.sm\";").unwrap();
        std::fs::write(&file_b, "import \"a.sm\";").unwrap();

        let root_canon = crate::source_access::strip_verbatim_prefix(&file_a.canonicalize().unwrap());
        let overlay = SourceOverlay::new();
        let access = CanonicalSources::new(&overlay, &temp_dir);
        let provider = OverlayModuleProvider {
            access: &access,
            served: RefCell::new(BTreeMap::new()),
        };

        // When a.sm imports b.sm, and b.sm imports a.sm:
        let importer_a = root_canon.to_string_lossy().replace('\\', "/");
        let resolved_b = provider.resolve_import(&importer_a, "b.sm").unwrap();
        let resolved_a = provider.resolve_import(&resolved_b, "a.sm").unwrap();

        let root_comp = Path::new(&root_canon).components().next();
        let resolved_comp = Path::new(&resolved_a).components().next();

        // R1: Both must have identical prefix representation, and neither may be Prefix::UNC.
        assert_eq!(
            root_comp, resolved_comp,
            "root module and resolved import must share identical prefix component"
        );
        if let Some(Component::Prefix(p)) = resolved_comp {
            assert!(
                !matches!(p.kind(), Prefix::UNC(_, _)),
                "resolved import must not be parsed as a UNC network share: {resolved_a}"
            );
        }

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    #[cfg(windows)]
    fn windows_cycle_boundary_produces_three_hop_chain_and_forbids_four_hop() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let temp_dir = std::env::temp_dir().join(format!(
            "canonical_check_cycle_{}_{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(temp_dir.join("src")).expect("mkdir");
        let manifest = temp_dir.join("package.toml");
        std::fs::write(
            &manifest,
            "[package]\nname = \"cycle_pkg\"\nversion = \"0.1.0\"\nmodule_root = \"src\"\n",
        )
        .unwrap();
        let file_a = temp_dir.join("src").join("a.sm");
        let file_b = temp_dir.join("src").join("b.sm");
        std::fs::write(
            &file_a,
            "Import \"b.sm\"\nLaw \"A\" [priority 1]:\n    When true ->\n        System.recovery()\n",
        )
        .unwrap();
        std::fs::write(
            &file_b,
            "Import \"a.sm\"\nLaw \"B\" [priority 1]:\n    When true ->\n        System.recovery()\n",
        )
        .unwrap();

        let request = CheckRequest {
            entry: file_a.clone(),
            overlay: SourceOverlay::new(),
            display_base: Some(temp_dir.clone()),
            root_display_fallback: None,
        };
        let report = check_canonical(&request).expect("check_canonical succeeds with report");
        assert_eq!(report.status, CheckStatus::Failed);
        let msg = report.diagnostics[0].message.as_str();
        // R2: must produce exactly a.sm -> b.sm -> a.sm and forbid 4 hops:
        assert_eq!(msg, "cyclic import detected: a.sm -> b.sm -> a.sm");
        assert!(!msg.contains("a.sm -> b.sm -> a.sm -> b.sm"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    #[cfg(windows)]
    fn windows_non_cycle_import_loads_cleanly() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let temp_dir = std::env::temp_dir().join(format!(
            "canonical_check_norm_{}_{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(temp_dir.join("src")).expect("mkdir");
        let manifest = temp_dir.join("package.toml");
        std::fs::write(
            &manifest,
            "[package]\nname = \"norm_pkg\"\nversion = \"0.1.0\"\nmodule_root = \"src\"\n",
        )
        .unwrap();
        let file_main = temp_dir.join("src").join("main.sm");
        let file_helper = temp_dir.join("src").join("helper.sm");
        std::fs::write(
            &file_main,
            "Import \"helper.sm\"\nLaw \"Main\" [priority 1]:\n    When true ->\n        System.recovery()\n",
        )
        .unwrap();
        std::fs::write(
            &file_helper,
            "Law \"Helper\" [priority 1]:\n    When true ->\n        System.recovery()\n",
        )
        .unwrap();

        let request = CheckRequest {
            entry: file_main.clone(),
            overlay: SourceOverlay::new(),
            display_base: Some(temp_dir.clone()),
            root_display_fallback: None,
        };
        let report = check_canonical(&request).expect("check_canonical succeeds with report");
        // R3: non-cycle import loads cleanly:
        assert_eq!(report.status, CheckStatus::Passed);
        assert_eq!(report.error_count(), 0);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
