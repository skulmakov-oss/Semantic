use std::cell::RefCell;
use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

pub const PACKAGE_MANIFEST_BASELINE_VERSION: u32 = 1;
pub const PACKAGE_MANIFEST_FILE_NAME: &str = "Semantic.package";
pub const SEMANTIC_TOML_FILE_NAME: &str = "semantic.toml";
pub const PACKAGE_IMPORT_SEPARATOR: &str = "::";

thread_local! {
    /// Memoizes which pinned dependency manifests have already had their content
    /// fingerprint verified, so a dependency imported through many package-qualified
    /// modules is hashed once per bundle pass instead of once per import (DL-020).
    /// `smc watch` (a long-lived process) must clear this via
    /// `reset_pinned_dependency_fingerprint_cache` before each rebuild pass, since the
    /// filesystem can change between passes; one-shot commands (check/compile/run) never
    /// need to, since a fresh process already starts empty (same reasoning as DL-017's
    /// nested-manifest admission cache, which was a `static` and unsafe for exactly this
    /// reason -- this one is explicitly resettable instead).
    static PINNED_DEPENDENCY_CONTENT_VERIFIED: RefCell<HashSet<PathBuf>> = RefCell::new(HashSet::new());
}

/// Clears the pinned-dependency content-fingerprint cache. Must be called by any
/// long-lived caller (currently only `smc watch`, see `cmd_watch` in `app.rs`) before each
/// rebuild pass; one-shot commands never need to call this.
pub(crate) fn reset_pinned_dependency_fingerprint_cache() {
    PINNED_DEPENDENCY_CONTENT_VERIFIED.with(|cache| cache.borrow_mut().clear());
}

thread_local! {
    /// Memoizes which manifests have already had their full DECLARED dependency graph
    /// validated (missing/cyclic/name-mismatched/stale-pinned), so admitting many modules
    /// from the same manifest only walks that graph once per bundle pass (DL-021). Same
    /// reset discipline as `PINNED_DEPENDENCY_CONTENT_VERIFIED` above: `smc watch` must
    /// clear it before each rebuild pass via `reset_declared_dependency_graph_cache`.
    static DECLARED_DEPENDENCY_GRAPH_VALIDATED: RefCell<HashSet<PathBuf>> =
        RefCell::new(HashSet::new());
}

/// Clears the declared-dependency-graph validation cache. Must be called by any long-lived
/// caller (currently only `smc watch`) before each rebuild pass; one-shot commands never
/// need to call this.
pub(crate) fn reset_declared_dependency_graph_cache() {
    DECLARED_DEPENDENCY_GRAPH_VALIDATED.with(|cache| cache.borrow_mut().clear());
}

/// `smc package inspect` rejects a missing/cyclic/name-mismatched/stale-pinned dependency
/// declared anywhere in a package's graph, regardless of whether any module actually
/// imports it, by walking the full graph via `PackageGraphBuilder::visit`. Reuse that same
/// walk here so admission (`check`/`compile`/`run`) enforces the identical contract instead
/// of only validating dependencies lazily as they happen to be imported (DL-021).
fn validate_declared_dependency_graph(
    manifest_path: &Path,
) -> Result<(), PackageModuleAdmissionError> {
    let canonical = manifest_path
        .canonicalize()
        .map_err(|error| PackageModuleAdmissionError {
            code: PackageModuleAdmissionCode::DeclaredDependencyGraphInvalid,
            message: format!("failed to resolve '{}': {error}", manifest_path.display()),
        })?;
    if DECLARED_DEPENDENCY_GRAPH_VALIDATED.with(|cache| cache.borrow().contains(&canonical)) {
        return Ok(());
    }
    PackageGraphBuilder::default()
        .visit(manifest_path)
        .map_err(|message| PackageModuleAdmissionError {
            code: PackageModuleAdmissionCode::DeclaredDependencyGraphInvalid,
            message,
        })?;
    DECLARED_DEPENDENCY_GRAPH_VALIDATED.with(|cache| cache.borrow_mut().insert(canonical));
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageRoot {
    pub manifest_dir: String,
    pub module_root: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageIdentity {
    pub name: String,
    pub root: PackageRoot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageDependencySource {
    LocalPath {
        path: String,
    },
    PinnedLocalPath {
        path: String,
        manifest_fingerprint: String,
        content_fingerprint: String,
    },
}

impl PackageDependencySource {
    fn path(&self) -> &str {
        match self {
            Self::LocalPath { path } | Self::PinnedLocalPath { path, .. } => path,
        }
    }

    fn expected_fingerprints(&self) -> (Option<&str>, Option<&str>) {
        match self {
            Self::LocalPath { .. } => (None, None),
            Self::PinnedLocalPath {
                manifest_fingerprint,
                content_fingerprint,
                ..
            } => (Some(manifest_fingerprint), Some(content_fingerprint)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageDependency {
    pub alias: String,
    pub package_name: String,
    pub source: PackageDependencySource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageManifest {
    pub format_version: u32,
    pub package: PackageIdentity,
    pub dependencies: Vec<PackageDependency>,
    pub capability_requests: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageManifestParseCode {
    MissingFormatDirective,
    MissingPackageDirective,
    MissingManifestDirDirective,
    MissingModuleRootDirective,
    DuplicateDirective,
    InvalidFormatVersion,
    InvalidDirective,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageManifestParseError {
    pub code: PackageManifestParseCode,
    pub line: usize,
    pub message: String,
}

impl fmt::Display for PackageManifestParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "package manifest parse error on line {}: {}",
            self.line, self.message
        )
    }
}

impl Error for PackageManifestParseError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageManifestValidationCode {
    UnsupportedFormatVersion,
    EmptyPackageName,
    EmptyManifestDir,
    EmptyModuleRoot,
    EmptyDependencyAlias,
    DuplicateDependencyAlias,
    EmptyDependencyPackageName,
    EmptyLocalDependencyPath,
    LocalDependencyPathMustBeRelative,
    InvalidDependencyFingerprint,
    EmptyCapabilityRequest,
    DuplicateCapabilityRequest,
    PackageRootMustBeRelative,
    PackageRootMustNotEscapeManifest,
    ModuleRootMustBeRelative,
    ModuleRootMustNotEscapePackage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageManifestValidationError {
    pub code: PackageManifestValidationCode,
    pub message: String,
}

impl fmt::Display for PackageManifestValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "package manifest validation error: {}", self.message)
    }
}

impl Error for PackageManifestValidationError {}

impl PackageManifest {
    pub fn new(package: PackageIdentity, dependencies: Vec<PackageDependency>) -> Self {
        Self {
            format_version: PACKAGE_MANIFEST_BASELINE_VERSION,
            package,
            dependencies,
            capability_requests: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageModuleAdmission {
    pub manifest_path: String,
    pub package_name: String,
    pub module_path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageModuleAdmissionCode {
    EntryResolutionFailed,
    ManifestReadFailed,
    ManifestParseFailed,
    ManifestValidationFailed,
    PackageRootResolutionFailed,
    ModuleRootResolutionFailed,
    NestedManifestInsideModuleRoot,
    EntryOutsideModuleRoot,
    NonUtf8ModulePath,
    DeclaredDependencyGraphInvalid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageModuleAdmissionError {
    pub code: PackageModuleAdmissionCode,
    pub message: String,
}

impl fmt::Display for PackageModuleAdmissionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "package module admission error: {}", self.message)
    }
}

impl Error for PackageModuleAdmissionError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageImportResolutionCode {
    ImporterResolutionFailed,
    ImporterManifestMissing,
    ImporterManifestReadFailed,
    ImporterManifestParseFailed,
    ImporterManifestValidationFailed,
    ImporterPackageRootResolutionFailed,
    ImporterModuleRootResolutionFailed,
    InvalidQualifiedImportSpec,
    UnknownDependencyAlias,
    DependencyManifestMissing,
    DependencyManifestReadFailed,
    DependencyManifestParseFailed,
    DependencyManifestValidationFailed,
    DependencyPackageRootResolutionFailed,
    DependencyModuleRootResolutionFailed,
    DependencyPackageNameMismatch,
    DependencyManifestFingerprintMismatch,
    DependencyContentFingerprintMismatch,
    UnsupportedModuleExtension,
    RelativeImportOutsideModuleRoot,
    DependencyImportOutsideModuleRoot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageImportResolutionError {
    pub code: PackageImportResolutionCode,
    pub message: String,
}

impl fmt::Display for PackageImportResolutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "package import resolution error: {}", self.message)
    }
}

impl Error for PackageImportResolutionError {}

pub fn parse_package_manifest_baseline(
    source: &str,
) -> Result<PackageManifest, PackageManifestParseError> {
    let mut format_version = None::<u32>;
    let mut package_name = None::<String>;
    let mut manifest_dir = None::<String>;
    let mut module_root = None::<String>;
    let mut dependencies = Vec::<PackageDependency>::new();
    let mut capability_requests = std::collections::BTreeSet::<String>::new();

    for (index, raw_line) in source.lines().enumerate() {
        let line_no = index + 1;
        let tokens = split_manifest_tokens(raw_line, line_no)?;
        if tokens.is_empty() {
            continue;
        }
        match tokens[0].as_str() {
            "format" => {
                ensure_unique_directive("format", &format_version, line_no)?;
                if tokens.len() != 2 {
                    return Err(parse_error(
                        PackageManifestParseCode::InvalidDirective,
                        line_no,
                        "format directive must be: format <u32>",
                    ));
                }
                let parsed = tokens[1].parse::<u32>().map_err(|_| {
                    parse_error(
                        PackageManifestParseCode::InvalidFormatVersion,
                        line_no,
                        "format directive requires a valid u32 version",
                    )
                })?;
                format_version = Some(parsed);
            }
            "package" => {
                ensure_unique_directive("package", &package_name, line_no)?;
                if tokens.len() != 2 {
                    return Err(parse_error(
                        PackageManifestParseCode::InvalidDirective,
                        line_no,
                        "package directive must be: package <name>",
                    ));
                }
                package_name = Some(tokens[1].clone());
            }
            "manifest_dir" => {
                ensure_unique_directive("manifest_dir", &manifest_dir, line_no)?;
                if tokens.len() != 2 {
                    return Err(parse_error(
                        PackageManifestParseCode::InvalidDirective,
                        line_no,
                        "manifest_dir directive must be: manifest_dir <path>",
                    ));
                }
                manifest_dir = Some(tokens[1].clone());
            }
            "module_root" => {
                ensure_unique_directive("module_root", &module_root, line_no)?;
                if tokens.len() != 2 {
                    return Err(parse_error(
                        PackageManifestParseCode::InvalidDirective,
                        line_no,
                        "module_root directive must be: module_root <path>",
                    ));
                }
                module_root = Some(tokens[1].clone());
            }
            "dep" => {
                if tokens.len() != 4 && tokens.len() != 6 {
                    return Err(parse_error(
                        PackageManifestParseCode::InvalidDirective,
                        line_no,
                        "dep directive must be: dep <alias> <package_name> <local_path> [<manifest_fingerprint> <content_fingerprint>]",
                    ));
                }
                let source = if tokens.len() == 4 {
                    PackageDependencySource::LocalPath {
                        path: tokens[3].clone(),
                    }
                } else {
                    PackageDependencySource::PinnedLocalPath {
                        path: tokens[3].clone(),
                        manifest_fingerprint: tokens[4].clone(),
                        content_fingerprint: tokens[5].clone(),
                    }
                };
                dependencies.push(PackageDependency {
                    alias: tokens[1].clone(),
                    package_name: tokens[2].clone(),
                    source,
                });
            }
            "capability" => {
                if tokens.len() != 2 {
                    return Err(parse_error(
                        PackageManifestParseCode::InvalidDirective,
                        line_no,
                        "capability directive must be: capability <request-id>",
                    ));
                }
                if tokens[1].trim().is_empty() {
                    return Err(parse_error(
                        PackageManifestParseCode::InvalidDirective,
                        line_no,
                        "capability request id must not be empty",
                    ));
                }
                if !capability_requests.insert(tokens[1].clone()) {
                    return Err(parse_error(
                        PackageManifestParseCode::InvalidDirective,
                        line_no,
                        &format!("duplicate package capability request '{}'", tokens[1]),
                    ));
                }
            }
            other => {
                return Err(parse_error(
                    PackageManifestParseCode::InvalidDirective,
                    line_no,
                    &format!("unknown package manifest directive '{}'", other),
                ))
            }
        }
    }

    let format_version = format_version.ok_or_else(|| {
        parse_error(
            PackageManifestParseCode::MissingFormatDirective,
            0,
            "package manifest requires an explicit format directive",
        )
    })?;
    let package_name = package_name.ok_or_else(|| {
        parse_error(
            PackageManifestParseCode::MissingPackageDirective,
            0,
            "package manifest requires an explicit package directive",
        )
    })?;
    let manifest_dir = manifest_dir.ok_or_else(|| {
        parse_error(
            PackageManifestParseCode::MissingManifestDirDirective,
            0,
            "package manifest requires an explicit manifest_dir directive",
        )
    })?;
    let module_root = module_root.ok_or_else(|| {
        parse_error(
            PackageManifestParseCode::MissingModuleRootDirective,
            0,
            "package manifest requires an explicit module_root directive",
        )
    })?;

    Ok(PackageManifest {
        format_version,
        package: PackageIdentity {
            name: package_name,
            root: PackageRoot {
                manifest_dir,
                module_root,
            },
        },
        dependencies,
        capability_requests: capability_requests.into_iter().collect(),
    })
}

pub fn validate_package_manifest_baseline(
    manifest: &PackageManifest,
) -> Result<(), PackageManifestValidationError> {
    if manifest.format_version != PACKAGE_MANIFEST_BASELINE_VERSION {
        return Err(PackageManifestValidationError {
            code: PackageManifestValidationCode::UnsupportedFormatVersion,
            message: format!(
                "unsupported package manifest format version {}; expected {}",
                manifest.format_version, PACKAGE_MANIFEST_BASELINE_VERSION
            ),
        });
    }

    if manifest.package.name.trim().is_empty() {
        return Err(PackageManifestValidationError {
            code: PackageManifestValidationCode::EmptyPackageName,
            message: "package name must not be empty".to_string(),
        });
    }

    if manifest.package.root.manifest_dir.trim().is_empty() {
        return Err(PackageManifestValidationError {
            code: PackageManifestValidationCode::EmptyManifestDir,
            message: "package manifest_dir must not be empty".to_string(),
        });
    }
    validate_contained_relative_path(
        &manifest.package.root.manifest_dir,
        PackageManifestValidationCode::PackageRootMustBeRelative,
        PackageManifestValidationCode::PackageRootMustNotEscapeManifest,
        "package manifest_dir",
    )?;

    if manifest.package.root.module_root.trim().is_empty() {
        return Err(PackageManifestValidationError {
            code: PackageManifestValidationCode::EmptyModuleRoot,
            message: "package module_root must not be empty".to_string(),
        });
    }
    validate_contained_relative_path(
        &manifest.package.root.module_root,
        PackageManifestValidationCode::ModuleRootMustBeRelative,
        PackageManifestValidationCode::ModuleRootMustNotEscapePackage,
        "package module_root",
    )?;

    let mut seen_aliases = std::collections::BTreeSet::new();
    for dependency in &manifest.dependencies {
        if dependency.alias.trim().is_empty() {
            return Err(PackageManifestValidationError {
                code: PackageManifestValidationCode::EmptyDependencyAlias,
                message: "package dependency alias must not be empty".to_string(),
            });
        }
        if !seen_aliases.insert(dependency.alias.as_str()) {
            return Err(PackageManifestValidationError {
                code: PackageManifestValidationCode::DuplicateDependencyAlias,
                message: format!("duplicate package dependency alias '{}'", dependency.alias),
            });
        }
        if dependency.package_name.trim().is_empty() {
            return Err(PackageManifestValidationError {
                code: PackageManifestValidationCode::EmptyDependencyPackageName,
                message: "package dependency package_name must not be empty".to_string(),
            });
        }
        let path = dependency.source.path();
        if path.trim().is_empty() {
            return Err(PackageManifestValidationError {
                code: PackageManifestValidationCode::EmptyLocalDependencyPath,
                message: format!(
                    "package dependency '{}' requires a non-empty local path",
                    dependency.alias
                ),
            });
        }
        if Path::new(path).is_absolute()
            || Path::new(path).components().any(|component| {
                matches!(
                    component,
                    std::path::Component::Prefix(_) | std::path::Component::RootDir
                )
            })
        {
            return Err(PackageManifestValidationError {
                code: PackageManifestValidationCode::LocalDependencyPathMustBeRelative,
                message: format!(
                    "package dependency '{}' local path '{}' must be relative",
                    dependency.alias, path
                ),
            });
        }
        let (manifest_fingerprint, content_fingerprint) = dependency.source.expected_fingerprints();
        for fingerprint in [manifest_fingerprint, content_fingerprint]
            .into_iter()
            .flatten()
        {
            if !is_valid_package_fingerprint(fingerprint) {
                return Err(PackageManifestValidationError {
                    code: PackageManifestValidationCode::InvalidDependencyFingerprint,
                    message: format!(
                        "package dependency '{}' has invalid fingerprint '{}'; expected fnv1a64:<16 lowercase hex digits>",
                        dependency.alias, fingerprint
                    ),
                });
            }
        }
    }

    let mut seen_capabilities = std::collections::BTreeSet::new();
    for capability in &manifest.capability_requests {
        if capability.trim().is_empty() {
            return Err(PackageManifestValidationError {
                code: PackageManifestValidationCode::EmptyCapabilityRequest,
                message: "package capability request must not be empty".to_string(),
            });
        }
        if !seen_capabilities.insert(capability.as_str()) {
            return Err(PackageManifestValidationError {
                code: PackageManifestValidationCode::DuplicateCapabilityRequest,
                message: format!("duplicate package capability request '{}'", capability),
            });
        }
    }

    Ok(())
}

fn validate_contained_relative_path(
    value: &str,
    absolute_code: PackageManifestValidationCode,
    escape_code: PackageManifestValidationCode,
    field: &str,
) -> Result<(), PackageManifestValidationError> {
    let path = Path::new(value);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                std::path::Component::Prefix(_) | std::path::Component::RootDir
            )
        })
    {
        return Err(PackageManifestValidationError {
            code: absolute_code,
            message: format!("{} '{}' must be relative", field, value),
        });
    }
    if path
        .components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(PackageManifestValidationError {
            code: escape_code,
            message: format!("{} '{}' must not escape its declared root", field, value),
        });
    }
    Ok(())
}

fn is_valid_package_fingerprint(value: &str) -> bool {
    value.strip_prefix("fnv1a64:").is_some_and(|hex| {
        hex.len() == 16
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    })
}

pub fn admit_package_entry_module(
    entry: &Path,
) -> Result<Option<PackageModuleAdmission>, PackageModuleAdmissionError> {
    reject_reparse_path(entry).map_err(|message| PackageModuleAdmissionError {
        code: PackageModuleAdmissionCode::EntryResolutionFailed,
        message,
    })?;
    let entry_canonical = entry
        .canonicalize()
        .map_err(|e| PackageModuleAdmissionError {
            code: PackageModuleAdmissionCode::EntryResolutionFailed,
            message: format!(
                "failed to resolve entry module '{}': {}",
                entry.display(),
                e
            ),
        })?;
    let manifest_path = match find_nearest_manifest(&entry_canonical) {
        Some(path) => path,
        None => return Ok(None),
    };
    reject_reparse_path(&manifest_path).map_err(|message| PackageModuleAdmissionError {
        code: PackageModuleAdmissionCode::ManifestReadFailed,
        message,
    })?;
    reject_nested_manifest_authority(&manifest_path)?;
    let manifest = load_and_validate_manifest(&manifest_path)?;
    let manifest_dir = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    let package_root = manifest_dir.join(&manifest.package.root.manifest_dir);
    reject_reparse_path(&package_root).map_err(|message| PackageModuleAdmissionError {
        code: PackageModuleAdmissionCode::PackageRootResolutionFailed,
        message,
    })?;
    let package_root = package_root
        .canonicalize()
        .map_err(|e| PackageModuleAdmissionError {
            code: PackageModuleAdmissionCode::PackageRootResolutionFailed,
            message: format!(
                "failed to resolve package root '{}' relative to '{}': {}",
                manifest.package.root.manifest_dir,
                manifest_path.display(),
                e
            ),
        })?;
    let module_root = package_root.join(&manifest.package.root.module_root);
    reject_reparse_path(&module_root).map_err(|message| PackageModuleAdmissionError {
        code: PackageModuleAdmissionCode::ModuleRootResolutionFailed,
        message,
    })?;
    let module_root = module_root
        .canonicalize()
        .map_err(|e| PackageModuleAdmissionError {
            code: PackageModuleAdmissionCode::ModuleRootResolutionFailed,
            message: format!(
                "failed to resolve package module_root '{}' relative to '{}': {}",
                manifest.package.root.module_root,
                package_root.display(),
                e
            ),
        })?;
    if module_root.strip_prefix(&package_root).is_err() {
        return Err(PackageModuleAdmissionError {
            code: PackageModuleAdmissionCode::EntryOutsideModuleRoot,
            message: format!(
                "package module_root '{}' is outside package root '{}'",
                module_root.display(),
                package_root.display()
            ),
        });
    }
    // A nested manifest inside module_root is rejected regardless of whether any module
    // actually imports from it -- `package inspect` already rejects it via
    // package_content_fingerprint's tree walk, so admission must reject it too, or
    // `check`/`compile`/`run` would silently accept a tree that `inspect` rejects. Uses the
    // lightweight, memoized reject_nested_manifests rather than the full content-hashing
    // walk, since admission (called once per imported module) doesn't need file content,
    // only the structural nested-manifest fact (see DL-016).
    reject_nested_manifests(&module_root, &manifest_path).map_err(|message| {
        PackageModuleAdmissionError {
            code: PackageModuleAdmissionCode::NestedManifestInsideModuleRoot,
            message,
        }
    })?;
    let module_relative =
        entry_canonical
            .strip_prefix(&module_root)
            .map_err(|_| PackageModuleAdmissionError {
                code: PackageModuleAdmissionCode::EntryOutsideModuleRoot,
                message: format!(
                    "module '{}' is outside admitted package module_root '{}'",
                    entry_canonical.display(),
                    module_root.display()
                ),
            })?;

    let module_path = normalize_relative_path(module_relative).map_err(|message| {
        PackageModuleAdmissionError {
            code: PackageModuleAdmissionCode::NonUtf8ModulePath,
            message,
        }
    })?;
    // `smc package inspect` walks the FULL declared dependency graph via
    // PackageGraphBuilder::visit, regardless of which modules are actually imported, and
    // rejects a missing/cyclic/name-mismatched/stale-pinned dependency deterministically
    // (docs/spec/package_baseline_v0.md:71-77). Admission previously validated dependencies
    // only lazily, as each one was actually resolved through an import -- so `check`,
    // `compile`, and `run` could accept and execute a package with an unused-but-invalid
    // declared dependency that `inspect` would reject on the same tree. Run the same
    // full-graph walk here too, last, so a more specific failure already caught above
    // (e.g. a nested manifest, which this walk would also encounter while hashing content)
    // reports under its own precise error code rather than this broader one (see DL-021).
    validate_declared_dependency_graph(&manifest_path)?;
    Ok(Some(PackageModuleAdmission {
        manifest_path: normalize_path(&manifest_path),
        package_name: manifest.package.name,
        module_path,
    }))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProjectRootResolutionCode {
    SemanticTomlReadFailed,
    SemanticTomlManifest(SemanticTomlManifestErrorCode),
    SemanticTomlEntryMissing,
    EntrySymlinkOrReparse,
    MissingProjectManifest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProjectRootResolutionError {
    code: ProjectRootResolutionCode,
    message: String,
}

impl fmt::Display for ProjectRootResolutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

fn project_root_resolution_error(
    code: ProjectRootResolutionCode,
    message: impl Into<String>,
) -> ProjectRootResolutionError {
    ProjectRootResolutionError {
        code,
        message: message.into(),
    }
}

fn resolve_project_root_check_entry_structured(
    root: &Path,
) -> Result<PathBuf, ProjectRootResolutionError> {
    let semantic_toml = root.join(SEMANTIC_TOML_FILE_NAME);
    if semantic_toml.is_file() {
        let source = std::fs::read_to_string(&semantic_toml).map_err(|e| {
            project_root_resolution_error(
                ProjectRootResolutionCode::SemanticTomlReadFailed,
                format!("failed to read '{}': {}", semantic_toml.display(), e),
            )
        })?;
        let project_manifest =
            parse_semantic_toml_manifest(&semantic_toml, &source).map_err(|e| {
                project_root_resolution_error(
                    ProjectRootResolutionCode::SemanticTomlManifest(e.code),
                    format!("failed to parse '{}': {}", semantic_toml.display(), e),
                )
            })?;
        let entry_path = root.join(&project_manifest.entry);
        reject_reparse_path(&entry_path).map_err(|message| {
            project_root_resolution_error(ProjectRootResolutionCode::EntrySymlinkOrReparse, message)
        })?;
        if !entry_path.is_file() {
            return Err(project_root_resolution_error(
                ProjectRootResolutionCode::SemanticTomlEntryMissing,
                format!(
                    "semantic.toml manifest '{}' entry '{}' resolves to missing file '{}'",
                    semantic_toml.display(),
                    project_manifest.entry,
                    entry_path.display()
                ),
            ));
        }
        return Ok(entry_path);
    }

    let package_manifest = root.join(PACKAGE_MANIFEST_FILE_NAME);
    if package_manifest.is_file() {
        let entry_path = root.join("src/main.sm");
        reject_reparse_path(&entry_path).map_err(|message| {
            project_root_resolution_error(ProjectRootResolutionCode::EntrySymlinkOrReparse, message)
        })?;
        return Ok(entry_path);
    }

    Err(project_root_resolution_error(
        ProjectRootResolutionCode::MissingProjectManifest,
        format!(
            "project root '{}' must contain '{}' or '{}'",
            root.display(),
            SEMANTIC_TOML_FILE_NAME,
            PACKAGE_MANIFEST_FILE_NAME
        ),
    ))
}

pub(crate) fn resolve_project_root_check_entry(root: &Path) -> Result<PathBuf, String> {
    resolve_project_root_check_entry_structured(root).map_err(|e| e.to_string())
}
pub fn resolve_package_import_path(
    importer_module: &Path,
    spec: &str,
) -> Result<PathBuf, PackageImportResolutionError> {
    let importer_canonical =
        importer_module
            .canonicalize()
            .map_err(|e| PackageImportResolutionError {
                code: PackageImportResolutionCode::ImporterResolutionFailed,
                message: format!(
                    "failed to resolve importer module '{}': {}",
                    importer_module.display(),
                    e
                ),
            })?;
    if let Some((alias, module_spec)) = spec.split_once(PACKAGE_IMPORT_SEPARATOR) {
        validate_package_import_extension(module_spec, spec)?;
        return resolve_dependency_import(&importer_canonical, alias, module_spec, spec);
    }

    if is_rooted_import_spec(spec) {
        return Err(PackageImportResolutionError {
            code: PackageImportResolutionCode::RelativeImportOutsideModuleRoot,
            message: format!("relative package import '{}' must be relative", spec),
        });
    }

    let base = importer_canonical
        .parent()
        .unwrap_or_else(|| Path::new("."));
    let resolved = resolve_relative_import_path(base, spec);
    if let Some(manifest_path) = find_nearest_manifest(&importer_canonical) {
        let importer_ctx = resolve_manifest_context(
            &manifest_path,
            PackageImportResolutionCode::ImporterManifestReadFailed,
            PackageImportResolutionCode::ImporterManifestParseFailed,
            PackageImportResolutionCode::ImporterManifestValidationFailed,
            PackageImportResolutionCode::ImporterPackageRootResolutionFailed,
            PackageImportResolutionCode::ImporterModuleRootResolutionFailed,
        )?;
        validate_package_import_extension(spec, spec)?;
        if resolved.strip_prefix(&importer_ctx.module_root).is_err() {
            return Err(PackageImportResolutionError {
                code: PackageImportResolutionCode::RelativeImportOutsideModuleRoot,
                message: format!(
                    "relative package import '{}' escapes package module_root '{}'",
                    spec,
                    importer_ctx.module_root.display()
                ),
            });
        }
    }
    Ok(resolved)
}

fn validate_package_import_extension(
    module_spec: &str,
    original_spec: &str,
) -> Result<(), PackageImportResolutionError> {
    match Path::new(module_spec)
        .extension()
        .and_then(|value| value.to_str())
    {
        None | Some("sm" | "exo") => Ok(()),
        Some(_) => Err(PackageImportResolutionError {
            code: PackageImportResolutionCode::UnsupportedModuleExtension,
            message: format!(
                "package import '{}' must use a .sm or .exo module extension",
                original_spec
            ),
        }),
    }
}

fn parse_error(
    code: PackageManifestParseCode,
    line: usize,
    message: &str,
) -> PackageManifestParseError {
    PackageManifestParseError {
        code,
        line,
        message: message.to_string(),
    }
}

fn ensure_unique_directive<T>(
    name: &str,
    slot: &Option<T>,
    line: usize,
) -> Result<(), PackageManifestParseError> {
    if slot.is_some() {
        return Err(parse_error(
            PackageManifestParseCode::DuplicateDirective,
            line,
            &format!("duplicate package manifest directive '{}'", name),
        ));
    }
    Ok(())
}

fn split_manifest_tokens(
    raw_line: &str,
    line_no: usize,
) -> Result<Vec<String>, PackageManifestParseError> {
    let mut out = Vec::<String>::new();
    let chars: Vec<char> = raw_line.chars().collect();
    let mut i = 0usize;
    while i < chars.len() {
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        if i >= chars.len() || chars[i] == '#' {
            break;
        }
        if chars[i] == '"' {
            i += 1;
            let start = i;
            while i < chars.len() && chars[i] != '"' {
                i += 1;
            }
            if i >= chars.len() {
                return Err(parse_error(
                    PackageManifestParseCode::InvalidDirective,
                    line_no,
                    "unterminated quoted value in package manifest",
                ));
            }
            out.push(chars[start..i].iter().collect());
            i += 1;
            continue;
        }
        let start = i;
        while i < chars.len() && !chars[i].is_whitespace() && chars[i] != '#' {
            i += 1;
        }
        out.push(chars[start..i].iter().collect());
        if i < chars.len() && chars[i] == '#' {
            break;
        }
    }
    Ok(out)
}

/// `semantic.toml` wins over a co-located `Semantic.package` in the same directory
/// (`docs/spec/project_model_v0.md`). Shared by `find_nearest_manifest`'s upward search
/// and `inspect_local_package_graph`'s root selection so both apply the same precedence
/// (see DL-019 -- `package inspect` used to always select `Semantic.package` directly,
/// disagreeing with what `check`/`compile`/`run` would admit for the same directory).
fn select_manifest_in_directory(dir: &Path) -> Option<PathBuf> {
    let semantic_toml = dir.join(SEMANTIC_TOML_FILE_NAME);
    if semantic_toml.is_file() {
        return Some(semantic_toml);
    }
    let package_manifest = dir.join(PACKAGE_MANIFEST_FILE_NAME);
    if package_manifest.is_file() {
        return Some(package_manifest);
    }
    None
}

fn find_nearest_manifest(entry_canonical: &Path) -> Option<PathBuf> {
    let mut current = entry_canonical.parent();
    while let Some(dir) = current {
        if let Some(manifest) = select_manifest_in_directory(dir) {
            return Some(manifest);
        }
        current = dir.parent();
    }
    None
}

fn reject_nested_manifest_authority(
    manifest_path: &Path,
) -> Result<(), PackageModuleAdmissionError> {
    let manifest_canonical =
        manifest_path
            .canonicalize()
            .map_err(|error| PackageModuleAdmissionError {
                code: PackageModuleAdmissionCode::ManifestReadFailed,
                message: format!(
                    "failed to resolve package manifest '{}': {error}",
                    manifest_path.display()
                ),
            })?;
    let mut current = manifest_canonical
        .parent()
        .and_then(|directory| directory.parent());
    while let Some(directory) = current {
        let semantic_toml = directory.join(SEMANTIC_TOML_FILE_NAME);
        let package_manifest = directory.join(PACKAGE_MANIFEST_FILE_NAME);
        let outer_manifest = if semantic_toml.is_file() {
            Some(semantic_toml)
        } else if package_manifest.is_file() {
            Some(package_manifest)
        } else {
            None
        };
        if let Some(outer_manifest) = outer_manifest {
            let outer = resolve_manifest_context(
                &outer_manifest,
                PackageImportResolutionCode::ImporterManifestReadFailed,
                PackageImportResolutionCode::ImporterManifestParseFailed,
                PackageImportResolutionCode::ImporterManifestValidationFailed,
                PackageImportResolutionCode::ImporterPackageRootResolutionFailed,
                PackageImportResolutionCode::ImporterModuleRootResolutionFailed,
            )
            .map_err(|error| PackageModuleAdmissionError {
                code: PackageModuleAdmissionCode::ManifestValidationFailed,
                message: error.to_string(),
            })?;
            if manifest_canonical.strip_prefix(&outer.module_root).is_ok() {
                return Err(PackageModuleAdmissionError {
                    code: PackageModuleAdmissionCode::NestedManifestInsideModuleRoot,
                    message: format!(
                        "nested package manifest '{}' is inside outer package module_root '{}'",
                        manifest_canonical.display(),
                        outer.module_root.display()
                    ),
                });
            }
        }
        current = directory.parent();
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct ResolvedPackageContext {
    manifest: PackageManifest,
    package_root: PathBuf,
    module_root: PathBuf,
}

fn load_and_validate_manifest(
    manifest_path: &Path,
) -> Result<PackageManifest, PackageModuleAdmissionError> {
    let source =
        std::fs::read_to_string(manifest_path).map_err(|e| PackageModuleAdmissionError {
            code: PackageModuleAdmissionCode::ManifestReadFailed,
            message: format!("failed to read '{}': {}", manifest_path.display(), e),
        })?;
    let manifest = if manifest_path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == SEMANTIC_TOML_FILE_NAME)
    {
        parse_semantic_toml_manifest(manifest_path, &source)
            .map(|project_manifest| project_manifest.manifest)
            .map_err(|e| PackageModuleAdmissionError {
                code: PackageModuleAdmissionCode::ManifestParseFailed,
                message: format!("failed to parse '{}': {}", manifest_path.display(), e),
            })?
    } else {
        parse_package_manifest_baseline(&source).map_err(|e| PackageModuleAdmissionError {
            code: PackageModuleAdmissionCode::ManifestParseFailed,
            message: format!("failed to parse '{}': {}", manifest_path.display(), e),
        })?
    };
    validate_package_manifest_baseline(&manifest).map_err(|e| PackageModuleAdmissionError {
        code: PackageModuleAdmissionCode::ManifestValidationFailed,
        message: format!("failed to validate '{}': {}", manifest_path.display(), e),
    })?;
    Ok(manifest)
}

fn resolve_dependency_import(
    importer_canonical: &Path,
    alias: &str,
    module_spec: &str,
    original_spec: &str,
) -> Result<PathBuf, PackageImportResolutionError> {
    if alias.trim().is_empty()
        || module_spec.trim().is_empty()
        || module_spec.contains(PACKAGE_IMPORT_SEPARATOR)
    {
        return Err(PackageImportResolutionError {
            code: PackageImportResolutionCode::InvalidQualifiedImportSpec,
            message: format!(
                "qualified package import '{}' must be '<alias>{}<module_path>'",
                original_spec, PACKAGE_IMPORT_SEPARATOR
            ),
        });
    }

    let importer_manifest_path =
        find_nearest_manifest(importer_canonical).ok_or_else(|| PackageImportResolutionError {
            code: PackageImportResolutionCode::ImporterManifestMissing,
            message: format!(
                "qualified package import '{}' requires an enclosing {} for '{}'",
                original_spec,
                PACKAGE_MANIFEST_FILE_NAME,
                importer_canonical.display()
            ),
        })?;
    let importer_ctx = resolve_manifest_context(
        &importer_manifest_path,
        PackageImportResolutionCode::ImporterManifestReadFailed,
        PackageImportResolutionCode::ImporterManifestParseFailed,
        PackageImportResolutionCode::ImporterManifestValidationFailed,
        PackageImportResolutionCode::ImporterPackageRootResolutionFailed,
        PackageImportResolutionCode::ImporterModuleRootResolutionFailed,
    )?;

    let dependency = importer_ctx
        .manifest
        .dependencies
        .iter()
        .find(|dep| dep.alias == alias)
        .ok_or_else(|| PackageImportResolutionError {
            code: PackageImportResolutionCode::UnknownDependencyAlias,
            message: format!(
                "package '{}' does not declare dependency alias '{}'",
                importer_ctx.manifest.package.name, alias
            ),
        })?;

    let dependency_path = dependency.source.path();
    let dependency_manifest_path = importer_ctx
        .package_root
        .join(dependency_path)
        .join(PACKAGE_MANIFEST_FILE_NAME);
    reject_reparse_path(&dependency_manifest_path).map_err(|message| {
        PackageImportResolutionError {
            code: PackageImportResolutionCode::DependencyManifestMissing,
            message,
        }
    })?;
    if !dependency_manifest_path.is_file() {
        return Err(PackageImportResolutionError {
            code: PackageImportResolutionCode::DependencyManifestMissing,
            message: format!(
                "dependency alias '{}' expected {} at '{}'",
                alias,
                PACKAGE_MANIFEST_FILE_NAME,
                dependency_manifest_path.display()
            ),
        });
    }

    let dependency_ctx = resolve_manifest_context(
        &dependency_manifest_path,
        PackageImportResolutionCode::DependencyManifestReadFailed,
        PackageImportResolutionCode::DependencyManifestParseFailed,
        PackageImportResolutionCode::DependencyManifestValidationFailed,
        PackageImportResolutionCode::DependencyPackageRootResolutionFailed,
        PackageImportResolutionCode::DependencyModuleRootResolutionFailed,
    )?;
    if dependency_ctx.manifest.package.name != dependency.package_name {
        return Err(PackageImportResolutionError {
            code: PackageImportResolutionCode::DependencyPackageNameMismatch,
            message: format!(
                "dependency alias '{}' expected package '{}' but manifest declares '{}'",
                alias, dependency.package_name, dependency_ctx.manifest.package.name
            ),
        });
    }
    let (expected_manifest, expected_content) = dependency.source.expected_fingerprints();
    if let Some(expected) = expected_manifest {
        let actual = manifest_fingerprint(&dependency_manifest_path).map_err(|message| {
            PackageImportResolutionError {
                code: PackageImportResolutionCode::DependencyManifestReadFailed,
                message,
            }
        })?;
        if actual != expected {
            return Err(PackageImportResolutionError {
                code: PackageImportResolutionCode::DependencyManifestFingerprintMismatch,
                message: format!(
                    "dependency alias '{}' manifest fingerprint mismatch: expected '{}', found '{}'",
                    alias, expected, actual
                ),
            });
        }
    }
    if let Some(expected) = expected_content {
        let canonical_dependency_manifest =
            dependency_manifest_path.canonicalize().map_err(|error| {
                PackageImportResolutionError {
                    code: PackageImportResolutionCode::DependencyContentFingerprintMismatch,
                    message: format!(
                        "failed to resolve '{}': {error}",
                        dependency_manifest_path.display()
                    ),
                }
            })?;
        // A pinned dependency imported through multiple package-qualified modules would
        // otherwise re-read and re-hash its entire content tree once per import -- O(N*M)
        // for N importing modules and M dependency files. Cache per dependency manifest
        // for the life of this thread-local, scoped the same way DL-017 scoped the
        // nested-manifest admission cache: `smc watch`'s loop explicitly clears it via
        // reset_pinned_dependency_fingerprint_cache before each rebuild pass, since that
        // command is long-lived and the filesystem can change between passes; one-shot
        // commands never need to clear it, since a fresh process already starts empty
        // (see DL-020).
        let already_verified = PINNED_DEPENDENCY_CONTENT_VERIFIED
            .with(|cache| cache.borrow().contains(&canonical_dependency_manifest));
        if !already_verified {
            let actual =
                package_content_fingerprint(&dependency_ctx.module_root, &dependency_manifest_path)
                    .map_err(|message| PackageImportResolutionError {
                        code: PackageImportResolutionCode::DependencyContentFingerprintMismatch,
                        message,
                    })?;
            if actual != expected {
                return Err(PackageImportResolutionError {
                    code: PackageImportResolutionCode::DependencyContentFingerprintMismatch,
                    message: format!(
                        "dependency alias '{}' content fingerprint mismatch: expected '{}', found '{}'",
                        alias, expected, actual
                    ),
                });
            }
            PINNED_DEPENDENCY_CONTENT_VERIFIED.with(|cache| {
                cache.borrow_mut().insert(canonical_dependency_manifest);
            });
        }
    }

    let resolved = normalize_lexical(
        &dependency_ctx
            .module_root
            .join(append_default_module_extension(module_spec)),
    );
    if resolved.strip_prefix(&dependency_ctx.module_root).is_err() {
        return Err(PackageImportResolutionError {
            code: PackageImportResolutionCode::DependencyImportOutsideModuleRoot,
            message: format!(
                "qualified package import '{}' escapes dependency module_root '{}'",
                original_spec,
                dependency_ctx.module_root.display()
            ),
        });
    }
    Ok(resolved)
}

fn resolve_manifest_context(
    manifest_path: &Path,
    read_code: PackageImportResolutionCode,
    parse_code: PackageImportResolutionCode,
    validate_code: PackageImportResolutionCode,
    package_root_code: PackageImportResolutionCode,
    module_root_code: PackageImportResolutionCode,
) -> Result<ResolvedPackageContext, PackageImportResolutionError> {
    reject_reparse_path(manifest_path).map_err(|message| PackageImportResolutionError {
        code: read_code,
        message,
    })?;
    let source =
        std::fs::read_to_string(manifest_path).map_err(|e| PackageImportResolutionError {
            code: read_code,
            message: format!("failed to read '{}': {}", manifest_path.display(), e),
        })?;
    let manifest = if manifest_path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == SEMANTIC_TOML_FILE_NAME)
    {
        parse_semantic_toml_manifest(manifest_path, &source)
            .map(|project_manifest| project_manifest.manifest)
            .map_err(|e| PackageImportResolutionError {
                code: parse_code,
                message: format!("failed to parse '{}': {}", manifest_path.display(), e),
            })?
    } else {
        parse_package_manifest_baseline(&source).map_err(|e| PackageImportResolutionError {
            code: parse_code,
            message: format!("failed to parse '{}': {}", manifest_path.display(), e),
        })?
    };
    validate_package_manifest_baseline(&manifest).map_err(|e| PackageImportResolutionError {
        code: validate_code,
        message: format!("failed to validate '{}': {}", manifest_path.display(), e),
    })?;

    let manifest_dir = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    let package_root = manifest_dir.join(&manifest.package.root.manifest_dir);
    reject_reparse_path(&package_root).map_err(|message| PackageImportResolutionError {
        code: package_root_code,
        message,
    })?;
    let package_root = package_root
        .canonicalize()
        .map_err(|e| PackageImportResolutionError {
            code: package_root_code,
            message: format!(
                "failed to resolve package root '{}' relative to '{}': {}",
                manifest.package.root.manifest_dir,
                manifest_path.display(),
                e
            ),
        })?;
    let module_root = package_root.join(&manifest.package.root.module_root);
    reject_reparse_path(&module_root).map_err(|message| PackageImportResolutionError {
        code: module_root_code,
        message,
    })?;
    let module_root = module_root
        .canonicalize()
        .map_err(|e| PackageImportResolutionError {
            code: module_root_code,
            message: format!(
                "failed to resolve package module_root '{}' relative to '{}': {}",
                manifest.package.root.module_root,
                package_root.display(),
                e
            ),
        })?;
    if module_root.strip_prefix(&package_root).is_err() {
        return Err(PackageImportResolutionError {
            code: module_root_code,
            message: format!(
                "package module_root '{}' resolves outside package root '{}'",
                module_root.display(),
                package_root.display()
            ),
        });
    }

    Ok(ResolvedPackageContext {
        manifest,
        package_root,
        module_root,
    })
}

#[derive(Debug)]
struct ParsedSemanticTomlManifest {
    manifest: PackageManifest,
    entry: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SemanticTomlManifestErrorCode {
    MalformedSectionHeader,
    EntryOutsideSection,
    UnsupportedSection,
    UnsupportedPackageField,
    UnsupportedProjectField,
    MissingPackageName,
    EmptyPackageName,
    EmptyProjectEntry,
    ProjectEntryMustBeRelative,
    ProjectEntryMustNotEscapeRoot,
    InvalidStringValue,
    InvalidKeyValueEntry,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SemanticTomlManifestError {
    code: SemanticTomlManifestErrorCode,
    message: String,
}

impl fmt::Display for SemanticTomlManifestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

fn semantic_toml_error(
    code: SemanticTomlManifestErrorCode,
    message: impl Into<String>,
) -> SemanticTomlManifestError {
    SemanticTomlManifestError {
        code,
        message: message.into(),
    }
}

fn parse_semantic_toml_manifest(
    manifest_path: &Path,
    source: &str,
) -> Result<ParsedSemanticTomlManifest, SemanticTomlManifestError> {
    #[derive(Debug, Default)]
    struct ParsedSemanticTomlFields {
        package_name: Option<String>,
        project_entry: Option<String>,
    }

    fn parse_toml_string(
        value: &str,
        line_no: usize,
        field: &str,
    ) -> Result<String, SemanticTomlManifestError> {
        let trimmed = value.trim();
        if trimmed.len() < 2 || !trimmed.starts_with('"') || !trimmed.ends_with('"') {
            return Err(semantic_toml_error(
                SemanticTomlManifestErrorCode::InvalidStringValue,
                format!("line {}: {} must be a double-quoted string", line_no, field),
            ));
        }
        Ok(trimmed[1..trimmed.len() - 1].to_string())
    }

    fn normalize_project_entry(entry: &str) -> Result<String, SemanticTomlManifestError> {
        let path = Path::new(entry);
        if path.is_absolute()
            || path.components().any(|component| {
                matches!(
                    component,
                    std::path::Component::Prefix(_) | std::path::Component::RootDir
                )
            })
        {
            return Err(semantic_toml_error(
                SemanticTomlManifestErrorCode::ProjectEntryMustBeRelative,
                format!("project entry '{}' must be relative", entry),
            ));
        }
        if path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
        {
            return Err(semantic_toml_error(
                SemanticTomlManifestErrorCode::ProjectEntryMustNotEscapeRoot,
                format!("project entry '{}' must not escape the project root", entry),
            ));
        }
        // `entry` is already a valid-UTF-8 `&str` (parsed from semantic.toml text), so
        // `to_string_lossy` never loses information here; only fold '\' into '/' on
        // Windows, where it is the actual separator -- on Unix it is an ordinary filename
        // character and folding it unconditionally would change which file this entry
        // names (same defect class as DL-012/DL-014).
        let text = path.to_string_lossy();
        Ok(if cfg!(windows) {
            text.replace('\\', "/")
        } else {
            text.into_owned()
        })
    }

    let mut parsed = ParsedSemanticTomlFields::default();
    let mut section: Option<String> = None;

    for (index, raw_line) in source.lines().enumerate() {
        let line_no = index + 1;
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') {
            if !line.ends_with(']') {
                return Err(semantic_toml_error(
                    SemanticTomlManifestErrorCode::MalformedSectionHeader,
                    format!("line {}: malformed TOML section header", line_no),
                ));
            }
            section = Some(line[1..line.len() - 1].trim().to_string());
            continue;
        }
        let (key, value) = line.split_once('=').ok_or_else(|| {
            semantic_toml_error(
                SemanticTomlManifestErrorCode::InvalidKeyValueEntry,
                format!("line {}: expected 'key = \"value\"' entry", line_no),
            )
        })?;
        let key = key.trim();
        let value = value.trim();
        match section.as_deref() {
            Some("package") => match key {
                "name" => {
                    parsed.package_name = Some(parse_toml_string(value, line_no, "package.name")?);
                }
                "version" => {}
                other => {
                    return Err(semantic_toml_error(
                        SemanticTomlManifestErrorCode::UnsupportedPackageField,
                        format!(
                            "line {}: unsupported semantic.toml package field '{}'",
                            line_no, other
                        ),
                    ));
                }
            },
            Some("project") => match key {
                "entry" => {
                    parsed.project_entry = Some(normalize_project_entry(&parse_toml_string(
                        value,
                        line_no,
                        "project.entry",
                    )?)?);
                }
                other => {
                    return Err(semantic_toml_error(
                        SemanticTomlManifestErrorCode::UnsupportedProjectField,
                        format!(
                            "line {}: unsupported semantic.toml project field '{}'",
                            line_no, other
                        ),
                    ));
                }
            },
            Some(other) => {
                return Err(semantic_toml_error(
                    SemanticTomlManifestErrorCode::UnsupportedSection,
                    format!(
                        "line {}: unsupported semantic.toml section '{}'",
                        line_no, other
                    ),
                ));
            }
            None => {
                return Err(semantic_toml_error(
                    SemanticTomlManifestErrorCode::EntryOutsideSection,
                    format!(
                        "line {}: semantic.toml entries must appear inside [package] or [project]",
                        line_no
                    ),
                ));
            }
        }
    }

    let package_name = parsed.package_name.ok_or_else(|| {
        semantic_toml_error(
            SemanticTomlManifestErrorCode::MissingPackageName,
            format!(
                "semantic.toml manifest '{}' is missing required [package].name",
                manifest_path.display()
            ),
        )
    })?;
    if package_name.trim().is_empty() {
        return Err(semantic_toml_error(
            SemanticTomlManifestErrorCode::EmptyPackageName,
            format!(
                "semantic.toml manifest '{}' has empty [package].name",
                manifest_path.display()
            ),
        ));
    }
    let entry = parsed
        .project_entry
        .unwrap_or_else(|| "src/main.sm".to_string());
    if entry.trim().is_empty() {
        return Err(semantic_toml_error(
            SemanticTomlManifestErrorCode::EmptyProjectEntry,
            format!(
                "semantic.toml manifest '{}' has empty [project].entry",
                manifest_path.display()
            ),
        ));
    }
    let entry_path = Path::new(&entry);
    let module_root = entry_path
        .parent()
        .map(|parent| {
            // `entry` is already valid UTF-8 text; only fold '\' into '/' on Windows, for
            // the same reason as normalize_project_entry above (DL-012/DL-014 shape).
            let text = parent.to_string_lossy();
            let value = if cfg!(windows) {
                text.replace('\\', "/")
            } else {
                text.into_owned()
            };
            if value.is_empty() {
                ".".to_string()
            } else {
                value
            }
        })
        .unwrap_or_else(|| ".".to_string());
    let manifest = PackageManifest::new(
        PackageIdentity {
            name: package_name,
            root: PackageRoot {
                manifest_dir: ".".to_string(),
                module_root,
            },
        },
        Vec::new(),
    );
    Ok(ParsedSemanticTomlManifest { manifest, entry })
}
// Same check DL-010 added to `validate_contained_relative_path` et al.: a Windows
// root-relative spec like `\outside` has a `RootDir` component but no `Prefix`, so
// `Path::is_absolute()` reports it as NOT absolute -- joining it to `base` resets to the
// current drive's root (`PathBuf::push`'s documented Windows semantics), escaping
// containment. `is_absolute()` alone also isn't enough on its own: a fully absolute spec
// (`C:\outside` or, on Unix, `/outside`) previously resolved straight through with no
// containment check at all when the importer had no enclosing manifest (see DL-017). A
// relative `Import` spec must never be rooted or absolute, manifest or not.
fn is_rooted_import_spec(spec: &str) -> bool {
    let path = Path::new(spec);
    path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                std::path::Component::Prefix(_) | std::path::Component::RootDir
            )
        })
}

fn resolve_relative_import_path(base: &Path, spec: &str) -> PathBuf {
    let path = append_default_module_extension(spec);
    if path.is_absolute() {
        normalize_lexical(&path)
    } else {
        normalize_lexical(&base.join(path))
    }
}

fn append_default_module_extension(spec: &str) -> PathBuf {
    let mut path = PathBuf::from(spec);
    if path.extension().is_none() {
        path.set_extension("exo");
    }
    path
}

fn normalize_lexical(path: &Path) -> PathBuf {
    use std::path::Component;
    let mut out = PathBuf::new();
    for comp in path.components() {
        match comp {
            Component::Prefix(prefix) => out.push(prefix.as_os_str()),
            Component::RootDir => out.push(Path::new("/")),
            Component::CurDir => {}
            Component::ParentDir => {
                let _ = out.pop();
            }
            Component::Normal(part) => out.push(part),
        }
    }
    out
}

fn normalize_path(path: &Path) -> String {
    // Only fold '\' into '/' on Windows, where it is the actual path separator (same
    // reasoning as normalize_relative_path's DL-012 fix); Unix keeps the raw text.
    let text = path.to_string_lossy();
    let normalized = if cfg!(windows) {
        text.replace('\\', "/")
    } else {
        text.into_owned()
    };
    normalized
        .strip_prefix("//?/")
        .unwrap_or(&normalized)
        .to_string()
}

/// Unlike `normalize_path`, this is used to key package content/provenance by relative
/// path (module paths fed into content fingerprinting, admitted module identity), where
/// a lossy conversion could let two distinct non-UTF-8 filenames collapse onto the same
/// key -- silently hiding a real path/content change behind an unchanged fingerprint.
/// Reject non-UTF-8 relative paths outright instead.
fn normalize_relative_path(path: &Path) -> Result<String, String> {
    let text = path
        .to_str()
        .ok_or_else(|| format!("package path '{}' is not valid UTF-8", path.display()))?;
    // `\` is only a path separator on Windows -- on Unix it is an ordinary filename
    // character, so folding it into `/` unconditionally would collapse a real
    // subdirectory path (`dir/helper.sm`) and a file literally named `dir\helper.sm`
    // onto the same normalized string.
    let normalized = if cfg!(windows) {
        text.replace('\\', "/")
    } else {
        text.to_string()
    };
    let normalized = normalized.strip_prefix("//?/").unwrap_or(&normalized);
    Ok(if normalized.is_empty() {
        ".".to_string()
    } else {
        normalized.to_string()
    })
}

#[derive(Debug, Clone)]
struct PackageGraphDependencyRecord {
    alias: String,
    package_name: String,
    local_path: String,
    expected_manifest_fingerprint: Option<String>,
    expected_content_fingerprint: Option<String>,
}

#[derive(Debug, Clone)]
struct PackageGraphNode {
    name: String,
    manifest_fingerprint: String,
    content_fingerprint: String,
    capability_requests: Vec<String>,
    dependencies: Vec<PackageGraphDependencyRecord>,
}

#[derive(Default)]
struct PackageGraphBuilder {
    names: std::collections::BTreeMap<String, PathBuf>,
    nodes: std::collections::BTreeMap<String, PackageGraphNode>,
    visiting: Vec<(PathBuf, String)>,
}

pub(crate) fn inspect_local_package_graph(root: &Path) -> Result<String, String> {
    let manifest_path = select_manifest_in_directory(root).ok_or_else(|| {
        format!(
            "package inspect requires '{}' or '{}' at '{}'",
            SEMANTIC_TOML_FILE_NAME,
            PACKAGE_MANIFEST_FILE_NAME,
            root.display()
        )
    })?;
    reject_reparse_path(&manifest_path)?;
    let mut builder = PackageGraphBuilder::default();
    let root_package = builder.visit(&manifest_path)?;

    let mut graph_material = String::new();
    let packages = builder
        .nodes
        .values()
        .map(|node| {
            graph_material.push_str(&node.name);
            graph_material.push('\n');
            graph_material.push_str(&node.manifest_fingerprint);
            graph_material.push('\n');
            graph_material.push_str(&node.content_fingerprint);
            graph_material.push('\n');
            for capability in &node.capability_requests {
                graph_material.push_str("capability\t");
                graph_material.push_str(capability);
                graph_material.push('\n');
            }
            let dependencies = node
                .dependencies
                .iter()
                .map(|dependency| {
                    graph_material.push_str("dependency\t");
                    graph_material.push_str(&dependency.alias);
                    graph_material.push('\t');
                    graph_material.push_str(&dependency.package_name);
                    graph_material.push('\t');
                    graph_material.push_str(&dependency.local_path);
                    graph_material.push('\n');
                    serde_json::json!({
                        "alias": dependency.alias,
                        "package": dependency.package_name,
                        "local_path": dependency.local_path,
                        "expected_manifest_fingerprint": dependency.expected_manifest_fingerprint,
                        "expected_content_fingerprint": dependency.expected_content_fingerprint,
                    })
                })
                .collect::<Vec<_>>();
            serde_json::json!({
                "name": node.name,
                "manifest_fingerprint": node.manifest_fingerprint,
                "content_fingerprint": node.content_fingerprint,
                "capability_requests": node.capability_requests,
                "dependencies": dependencies,
            })
        })
        .collect::<Vec<_>>();
    let graph_fingerprint = fingerprint(graph_material.as_bytes());
    serde_json::to_string_pretty(&serde_json::json!({
        "schema": "semantic.foundation.package.provenance/0.1",
        "fingerprint_algorithm": "fnv1a64-non-cryptographic",
        "root_package": root_package,
        "graph_fingerprint": graph_fingerprint,
        "packages": packages,
    }))
    .map_err(|error| format!("failed to serialize package provenance record: {error}"))
}

impl PackageGraphBuilder {
    fn visit(&mut self, manifest_path: &Path) -> Result<String, String> {
        reject_reparse_path(manifest_path)?;
        let manifest_path = manifest_path.canonicalize().map_err(|error| {
            format!(
                "failed to resolve dependency manifest '{}': {error}",
                manifest_path.display()
            )
        })?;
        if let Some(index) = self
            .visiting
            .iter()
            .position(|(path, _)| path == &manifest_path)
        {
            let mut cycle = self.visiting[index..]
                .iter()
                .map(|(_, name)| name.clone())
                .collect::<Vec<_>>();
            cycle.push(self.visiting[index].1.clone());
            return Err(format!("package dependency cycle: {}", cycle.join(" -> ")));
        }

        let source = fs::read_to_string(&manifest_path)
            .map_err(|error| format!("failed to read '{}': {error}", manifest_path.display()))?;
        // Dispatch by filename, matching load_and_validate_manifest's rule for
        // check/compile/run: semantic.toml wins when both manifest kinds are present in
        // the same directory (docs/spec/project_model_v0.md). `visit` used to always parse
        // via the Semantic.package line format regardless of which file was selected --
        // fixed together with inspect_local_package_graph's selection below (DL-019).
        let manifest = if manifest_path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name == SEMANTIC_TOML_FILE_NAME)
        {
            parse_semantic_toml_manifest(&manifest_path, &source)
                .map(|project_manifest| project_manifest.manifest)
                .map_err(|error| {
                    format!("failed to parse '{}': {error}", manifest_path.display())
                })?
        } else {
            parse_package_manifest_baseline(&source).map_err(|error| {
                format!("failed to parse '{}': {error}", manifest_path.display())
            })?
        };
        validate_package_manifest_baseline(&manifest).map_err(|error| {
            format!("failed to validate '{}': {error}", manifest_path.display())
        })?;
        let package_name = manifest.package.name.clone();
        if let Some(existing_path) = self.names.get(&package_name) {
            if existing_path != &manifest_path {
                return Err(format!(
                    "duplicate package identity '{}': '{}' and '{}'",
                    package_name,
                    existing_path.display(),
                    manifest_path.display()
                ));
            }
            if self.nodes.contains_key(&package_name) {
                return Ok(package_name);
            }
        } else {
            self.names
                .insert(package_name.clone(), manifest_path.clone());
        }

        let manifest_dir = manifest_path.parent().unwrap_or_else(|| Path::new("."));
        let package_root_path = manifest_dir.join(&manifest.package.root.manifest_dir);
        reject_reparse_path(&package_root_path)?;
        let package_root = package_root_path.canonicalize().map_err(|error| {
            format!(
                "failed to resolve package root '{}' for '{}': {error}",
                package_root_path.display(),
                package_name
            )
        })?;
        let module_root_path = package_root.join(&manifest.package.root.module_root);
        reject_reparse_path(&module_root_path)?;
        let module_root = module_root_path.canonicalize().map_err(|error| {
            format!(
                "failed to resolve module root '{}' for '{}': {error}",
                module_root_path.display(),
                package_name
            )
        })?;
        if module_root.strip_prefix(&package_root).is_err() {
            return Err(format!(
                "package '{}' module_root '{}' escapes package root '{}'",
                package_name,
                module_root.display(),
                package_root.display()
            ));
        }

        self.visiting
            .push((manifest_path.clone(), package_name.clone()));
        let mut declared_dependencies = manifest.dependencies.clone();
        declared_dependencies.sort_by(|left, right| {
            left.alias
                .cmp(&right.alias)
                .then_with(|| left.package_name.cmp(&right.package_name))
        });
        let mut dependencies = Vec::with_capacity(declared_dependencies.len());
        for dependency in declared_dependencies {
            let local_path = dependency.source.path().to_string();
            let (expected_manifest_fingerprint, expected_content_fingerprint) =
                dependency.source.expected_fingerprints();
            let dependency_root = package_root.join(&local_path);
            reject_reparse_path(&dependency_root)?;
            let dependency_manifest = dependency_root.join(PACKAGE_MANIFEST_FILE_NAME);
            if !dependency_manifest.is_file() {
                return Err(format!(
                    "dependency alias '{}' expected {} at '{}'",
                    dependency.alias,
                    PACKAGE_MANIFEST_FILE_NAME,
                    dependency_manifest.display()
                ));
            }
            let resolved_name = self.visit(&dependency_manifest)?;
            if resolved_name != dependency.package_name {
                return Err(format!(
                    "dependency alias '{}' expected package '{}' but manifest declares '{}'",
                    dependency.alias, dependency.package_name, resolved_name
                ));
            }
            let resolved = self
                .nodes
                .get(&resolved_name)
                .ok_or_else(|| format!("internal package graph error for '{}'", resolved_name))?;
            if let Some(expected) = expected_manifest_fingerprint {
                if resolved.manifest_fingerprint != expected {
                    return Err(format!(
                        "dependency alias '{}' manifest fingerprint mismatch: expected '{}', found '{}'",
                        dependency.alias, expected, resolved.manifest_fingerprint
                    ));
                }
            }
            if let Some(expected) = expected_content_fingerprint {
                if resolved.content_fingerprint != expected {
                    return Err(format!(
                        "dependency alias '{}' content fingerprint mismatch: expected '{}', found '{}'",
                        dependency.alias, expected, resolved.content_fingerprint
                    ));
                }
            }
            // Only fold '\' into '/' on Windows, where it is the actual path separator --
            // on Unix it is an ordinary filename character, so an unconditional replacement
            // would let the provenance record identify a different real directory than the
            // one that was actually loaded (same defect class fixed for content
            // fingerprinting; see DL-012).
            let provenance_local_path = if cfg!(windows) {
                local_path.replace('\\', "/")
            } else {
                local_path
            };
            dependencies.push(PackageGraphDependencyRecord {
                alias: dependency.alias,
                package_name: dependency.package_name,
                local_path: provenance_local_path,
                expected_manifest_fingerprint: expected_manifest_fingerprint.map(str::to_string),
                expected_content_fingerprint: expected_content_fingerprint.map(str::to_string),
            });
        }
        let _ = self.visiting.pop();

        let capability_requests = manifest.capability_requests.clone();
        let node = PackageGraphNode {
            name: package_name.clone(),
            manifest_fingerprint: fingerprint(normalize_line_endings(&source).as_bytes()),
            content_fingerprint: package_content_fingerprint(&module_root, &manifest_path)?,
            capability_requests,
            dependencies,
        };
        self.nodes.insert(package_name.clone(), node);
        Ok(package_name)
    }
}

fn manifest_fingerprint(path: &Path) -> Result<String, String> {
    let source = fs::read_to_string(path)
        .map_err(|error| format!("failed to read '{}': {error}", path.display()))?;
    Ok(fingerprint(normalize_line_endings(&source).as_bytes()))
}

fn package_content_fingerprint(
    module_root: &Path,
    authority_manifest: &Path,
) -> Result<String, String> {
    let mut files = Vec::<(String, Vec<u8>)>::new();
    let authority_manifest = authority_manifest.canonicalize().map_err(|error| {
        format!(
            "failed to resolve '{}': {error}",
            authority_manifest.display()
        )
    })?;
    collect_package_files(module_root, module_root, &authority_manifest, &mut files)?;
    files.sort_by(|left, right| left.0.cmp(&right.0));
    let mut material = Vec::new();
    for (path, bytes) in files {
        material.extend_from_slice(&(path.len() as u64).to_le_bytes());
        material.extend_from_slice(path.as_bytes());
        material.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
        material.extend_from_slice(&bytes);
    }
    Ok(fingerprint(&material))
}

fn collect_package_files(
    module_root: &Path,
    directory: &Path,
    authority_manifest: &Path,
    files: &mut Vec<(String, Vec<u8>)>,
) -> Result<(), String> {
    let mut entries = fs::read_dir(directory)
        .map_err(|error| {
            format!(
                "failed to read package directory '{}': {error}",
                directory.display()
            )
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            format!(
                "failed to enumerate package directory '{}': {error}",
                directory.display()
            )
        })?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        if path_is_reparse(&path)? {
            return Err(format!(
                "package content path '{}' is a symlink or reparse point",
                path.display()
            ));
        }
        let file_type = entry
            .file_type()
            .map_err(|error| format!("failed to inspect '{}': {error}", path.display()))?;
        if file_type.is_dir() {
            collect_package_files(module_root, &path, authority_manifest, files)?;
        } else if file_type.is_file()
            && path_is_disallowed_nested_manifest(&path, authority_manifest)
        {
            return Err(format!(
                "nested package manifest '{}' is inside package module_root '{}'",
                path.display(),
                module_root.display()
            ));
        } else if file_type.is_file()
            && path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| matches!(extension, "sm" | "exo"))
        {
            let relative = path
                .strip_prefix(module_root)
                .map_err(|_| format!("package content '{}' escaped module root", path.display()))?;
            let bytes = fs::read(&path)
                .map_err(|error| format!("failed to read '{}': {error}", path.display()))?;
            let bytes = match String::from_utf8(bytes) {
                Ok(text) => normalize_line_endings(&text).into_bytes(),
                Err(error) => error.into_bytes(),
            };
            files.push((normalize_relative_path(relative)?, bytes));
        }
    }
    Ok(())
}

/// A manifest is only "nested" (forbidden) when it sits in a directory below the authority
/// manifest's own directory. A manifest of the other type co-located in the exact same
/// directory as the authority manifest is a permitted compatibility pairing
/// (`docs/spec/project_model_v0.md`: `semantic.toml` wins when both are present), not
/// nested content -- a pre-existing bug in the original nested-manifest check, exposed
/// once admission started exercising this same check (see DL-016).
fn path_is_disallowed_nested_manifest(path: &Path, authority_manifest: &Path) -> bool {
    path.parent() != authority_manifest.parent()
        && path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| {
                name == PACKAGE_MANIFEST_FILE_NAME || name == SEMANTIC_TOML_FILE_NAME
            })
}

/// Lightweight counterpart to `package_content_fingerprint`'s tree walk, used by admission
/// (`check`/`compile`/`run`) purely to reject a nested manifest. It never reads `.sm`/`.exo`
/// file bytes -- the dominant cost the original DL-016 finding cited -- so a package with
/// many imported modules pays only repeated cheap directory listings, not repeated full
/// content reads. DL-016 also added a process-lifetime memoization cache here, but that is
/// unsafe for `smc watch` (a genuinely long-lived process, see `cmd_watch` in `app.rs`):
/// once a module_root was scanned, a nested manifest added later would never be
/// re-detected for the life of the watch process. Deliberately NOT re-added -- see
/// DL-017. A future caller that specifically needs cross-call memoization within one
/// bounded bundle operation (not a whole process) should thread an explicit cache
/// parameter through that operation instead of reintroducing global state here.
fn reject_nested_manifests(module_root: &Path, authority_manifest: &Path) -> Result<(), String> {
    let authority_manifest = authority_manifest.canonicalize().map_err(|error| {
        format!(
            "failed to resolve '{}': {error}",
            authority_manifest.display()
        )
    })?;
    reject_nested_manifests_under(module_root, module_root, &authority_manifest)
}

fn reject_nested_manifests_under(
    module_root: &Path,
    directory: &Path,
    authority_manifest: &Path,
) -> Result<(), String> {
    let mut entries = fs::read_dir(directory)
        .map_err(|error| {
            format!(
                "failed to read package directory '{}': {error}",
                directory.display()
            )
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            format!(
                "failed to enumerate package directory '{}': {error}",
                directory.display()
            )
        })?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        if path_is_reparse(&path)? {
            return Err(format!(
                "package content path '{}' is a symlink or reparse point",
                path.display()
            ));
        }
        let file_type = entry
            .file_type()
            .map_err(|error| format!("failed to inspect '{}': {error}", path.display()))?;
        if file_type.is_dir() {
            reject_nested_manifests_under(module_root, &path, authority_manifest)?;
        } else if file_type.is_file()
            && path_is_disallowed_nested_manifest(&path, authority_manifest)
        {
            return Err(format!(
                "nested package manifest '{}' is inside package module_root '{}'",
                path.display(),
                module_root.display()
            ));
        }
    }
    Ok(())
}

fn reject_reparse_path(path: &Path) -> Result<(), String> {
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component.as_os_str());
        if current.exists() && path_is_reparse(&current)? {
            return Err(format!(
                "package path '{}' contains symlink or reparse point '{}'",
                path.display(),
                current.display()
            ));
        }
    }
    Ok(())
}

fn path_is_reparse(path: &Path) -> Result<bool, String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        format!(
            "failed to inspect package path '{}': {error}",
            path.display()
        )
    })?;
    if metadata.file_type().is_symlink() {
        return Ok(true);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
        Ok(metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0)
    }
    #[cfg(not(windows))]
    Ok(false)
}

fn normalize_line_endings(source: &str) -> String {
    source.replace("\r\n", "\n").replace('\r', "\n")
}

fn fingerprint(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn package_root() -> PackageRoot {
        PackageRoot {
            manifest_dir: ".".to_string(),
            module_root: "src".to_string(),
        }
    }

    fn normalize_canonical_path(path: &Path) -> String {
        normalize_path(&path.canonicalize().expect("canonicalize"))
    }

    fn mk_temp_dir(prefix: &str) -> PathBuf {
        let base = std::env::temp_dir().join(format!(
            "{}_{}_{}",
            prefix,
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(&base).expect("mkdir");
        base
    }

    #[test]
    fn new_manifest_uses_canonical_baseline_version() {
        let manifest = PackageManifest::new(
            PackageIdentity {
                name: "app".to_string(),
                root: package_root(),
            },
            Vec::new(),
        );
        assert_eq!(manifest.format_version, PACKAGE_MANIFEST_BASELINE_VERSION);
    }

    #[test]
    fn validate_package_manifest_accepts_local_path_dependency_inventory() {
        let manifest = PackageManifest::new(
            PackageIdentity {
                name: "app".to_string(),
                root: package_root(),
            },
            vec![
                PackageDependency {
                    alias: "math".to_string(),
                    package_name: "math".to_string(),
                    source: PackageDependencySource::LocalPath {
                        path: "../math".to_string(),
                    },
                },
                PackageDependency {
                    alias: "ui".to_string(),
                    package_name: "ui".to_string(),
                    source: PackageDependencySource::LocalPath {
                        path: "../ui".to_string(),
                    },
                },
            ],
        );
        assert_eq!(
            manifest
                .dependencies
                .iter()
                .map(|dep| dep.alias.as_str())
                .collect::<Vec<_>>(),
            vec!["math", "ui"]
        );
        validate_package_manifest_baseline(&manifest).expect("valid local path manifest");
    }

    #[test]
    fn validate_package_manifest_rejects_duplicate_dependency_alias() {
        let manifest = PackageManifest::new(
            PackageIdentity {
                name: "app".to_string(),
                root: package_root(),
            },
            vec![
                PackageDependency {
                    alias: "shared".to_string(),
                    package_name: "first".to_string(),
                    source: PackageDependencySource::LocalPath {
                        path: "../first".to_string(),
                    },
                },
                PackageDependency {
                    alias: "shared".to_string(),
                    package_name: "second".to_string(),
                    source: PackageDependencySource::LocalPath {
                        path: "../second".to_string(),
                    },
                },
            ],
        );
        let err = validate_package_manifest_baseline(&manifest).expect_err("must reject");
        assert_eq!(
            err.code,
            PackageManifestValidationCode::DuplicateDependencyAlias
        );
    }

    #[test]
    fn validate_package_manifest_rejects_empty_package_root_fields() {
        let manifest = PackageManifest::new(
            PackageIdentity {
                name: "app".to_string(),
                root: PackageRoot {
                    manifest_dir: "".to_string(),
                    module_root: "src".to_string(),
                },
            },
            Vec::new(),
        );
        let err = validate_package_manifest_baseline(&manifest).expect_err("must reject");
        assert_eq!(err.code, PackageManifestValidationCode::EmptyManifestDir);
    }

    #[test]
    fn parse_package_manifest_accepts_first_wave_directives() {
        let manifest = parse_package_manifest_baseline(
            r#"
format 1
package "app"
manifest_dir "."
module_root "src"
dep math math "../math"
dep ui ui "../ui"
capability fs.read
capability stdout.write
"#,
        )
        .expect("parse");
        assert_eq!(manifest.package.name, "app");
        assert_eq!(manifest.dependencies.len(), 2);
        assert_eq!(
            manifest.capability_requests,
            vec!["fs.read".to_string(), "stdout.write".to_string()]
        );
        let without_capabilities = parse_package_manifest_baseline(
            "format 1\npackage app\nmanifest_dir .\nmodule_root src\ndep math math ../math\ndep ui ui ../ui\n",
        )
        .expect("parse without capabilities");
        assert_ne!(manifest, without_capabilities);
        validate_package_manifest_baseline(&manifest).expect("validate");
    }

    #[test]
    fn parse_package_manifest_rejects_duplicate_package_directive() {
        let err = parse_package_manifest_baseline(
            r#"
format 1
package app
package other
manifest_dir .
module_root src
"#,
        )
        .expect_err("must reject");
        assert_eq!(err.code, PackageManifestParseCode::DuplicateDirective);
        assert_eq!(err.line, 4);
    }

    #[test]
    fn parse_package_manifest_rejects_missing_module_root() {
        let err = parse_package_manifest_baseline(
            r#"
format 1
package app
manifest_dir .
"#,
        )
        .expect_err("must reject");
        assert_eq!(
            err.code,
            PackageManifestParseCode::MissingModuleRootDirective
        );
    }

    #[test]
    fn parse_semantic_toml_manifest_preserves_default_entry_and_manifest_shape() {
        let manifest = parse_semantic_toml_manifest(
            Path::new("semantic.toml"),
            r#"
[package]
name = "app"

[project]
entry = "src/main.sm"
"#,
        )
        .expect("parse semantic.toml");
        assert_eq!(manifest.manifest.package.name, "app");
        assert_eq!(manifest.entry, "src/main.sm");
        assert_eq!(manifest.manifest.package.root.manifest_dir, ".");
        assert_eq!(manifest.manifest.package.root.module_root, "src");
        assert!(manifest.manifest.dependencies.is_empty());
    }

    #[test]
    fn parse_semantic_toml_manifest_preserves_explicit_nested_entry_shape() {
        let manifest = parse_semantic_toml_manifest(
            Path::new("semantic.toml"),
            r#"
[package]
name = "app"

[project]
entry = "examples/main.sm"
"#,
        )
        .expect("parse semantic.toml with explicit nested entry");
        assert_eq!(manifest.manifest.package.name, "app");
        assert_eq!(manifest.entry, "examples/main.sm");
        assert_eq!(manifest.manifest.package.root.manifest_dir, ".");
        assert_eq!(manifest.manifest.package.root.module_root, "examples");
        assert!(manifest.manifest.dependencies.is_empty());
    }

    #[test]
    fn parse_semantic_toml_manifest_preserves_root_level_entry_shape() {
        let manifest = parse_semantic_toml_manifest(
            Path::new("semantic.toml"),
            r#"
[package]
name = "app"

[project]
entry = "main.sm"
"#,
        )
        .expect("parse semantic.toml with root-level entry");
        assert_eq!(manifest.manifest.package.name, "app");
        assert_eq!(manifest.entry, "main.sm");
        assert_eq!(manifest.manifest.package.root.manifest_dir, ".");
        assert_eq!(manifest.manifest.package.root.module_root, ".");
        assert!(manifest.manifest.dependencies.is_empty());
    }

    #[test]
    fn parse_semantic_toml_manifest_defaults_entry_and_rejects_escape() {
        let manifest = parse_semantic_toml_manifest(
            Path::new("semantic.toml"),
            r#"
[package]
name = "app"
"#,
        )
        .expect("parse semantic.toml with default entry");
        assert_eq!(manifest.entry, "src/main.sm");
        assert_eq!(manifest.manifest.package.root.module_root, "src");
        assert_eq!(manifest.manifest.package.name, "app");
        assert_eq!(manifest.manifest.package.root.manifest_dir, ".");
        assert!(manifest.manifest.dependencies.is_empty());

        let err = parse_semantic_toml_manifest(
            Path::new("semantic.toml"),
            r#"
[package]
name = "app"

[project]
entry = "../escape.sm"
"#,
        )
        .expect_err("must reject entry escape");
        assert_eq!(
            err.code,
            SemanticTomlManifestErrorCode::ProjectEntryMustNotEscapeRoot
        );
        assert!(err.message.contains("must not escape the project root"));
    }

    #[test]
    fn parse_semantic_toml_manifest_rejects_empty_package_name() {
        let err = parse_semantic_toml_manifest(
            Path::new("semantic.toml"),
            r#"
[package]
name = ""
"#,
        )
        .expect_err("must reject empty package name");
        assert_eq!(err.code, SemanticTomlManifestErrorCode::EmptyPackageName);
        assert!(err.message.contains("empty [package].name"));
    }

    #[test]
    fn parse_semantic_toml_manifest_rejects_path_escape_entry() {
        let err = parse_semantic_toml_manifest(
            Path::new("semantic.toml"),
            r#"
[package]
name = "app"

[project]
entry = "../escape.sm"
"#,
        )
        .expect_err("must reject entry escape");
        assert_eq!(
            err.code,
            SemanticTomlManifestErrorCode::ProjectEntryMustNotEscapeRoot
        );
        assert!(err.message.contains("must not escape the project root"));
    }

    #[test]
    fn parse_semantic_toml_manifest_reports_structured_error_codes() {
        let abs_entry_path = if cfg!(windows) {
            "C:/abs/main.sm"
        } else {
            "/abs/main.sm"
        };
        let abs_entry_source = format!(
            r#"
[package]
name = "app"

[project]
entry = "{abs_entry_path}"
"#
        );

        let cases: &[(&str, SemanticTomlManifestErrorCode, &str)] = &[
            (
                r#"
[package
name = "app"
"#,
                SemanticTomlManifestErrorCode::MalformedSectionHeader,
                "malformed TOML section header",
            ),
            (
                r#"
name = "app"
"#,
                SemanticTomlManifestErrorCode::EntryOutsideSection,
                "semantic.toml entries must appear inside [package] or [project]",
            ),
            (
                r#"
[package]
name = "app"

[tool]
foo = "bar"
"#,
                SemanticTomlManifestErrorCode::UnsupportedSection,
                "unsupported semantic.toml section 'tool'",
            ),
            (
                r#"
[package]
name = "app"
extra = "nope"
"#,
                SemanticTomlManifestErrorCode::UnsupportedPackageField,
                "unsupported semantic.toml package field 'extra'",
            ),
            (
                r#"
[package]
name = "app"

[project]
extra = "nope"
"#,
                SemanticTomlManifestErrorCode::UnsupportedProjectField,
                "unsupported semantic.toml project field 'extra'",
            ),
            (
                r#"
[package]
"#,
                SemanticTomlManifestErrorCode::MissingPackageName,
                "missing required [package].name",
            ),
            (
                r#"
[package]
name = ""
"#,
                SemanticTomlManifestErrorCode::EmptyPackageName,
                "has empty [package].name",
            ),
            (
                r#"
[package]
name = "app"

[project]
entry = ""
"#,
                SemanticTomlManifestErrorCode::EmptyProjectEntry,
                "has empty [project].entry",
            ),
            (
                &abs_entry_source,
                SemanticTomlManifestErrorCode::ProjectEntryMustBeRelative,
                "must be relative",
            ),
            (
                r#"
[package]
name = "app"

[project]
entry = "../escape.sm"
"#,
                SemanticTomlManifestErrorCode::ProjectEntryMustNotEscapeRoot,
                "must not escape the project root",
            ),
            (
                r#"
[package]
name = app
"#,
                SemanticTomlManifestErrorCode::InvalidStringValue,
                "must be a double-quoted string",
            ),
            (
                r#"
[package]
name "app"
"#,
                SemanticTomlManifestErrorCode::InvalidKeyValueEntry,
                "expected 'key = \"value\"' entry",
            ),
        ];

        for (source, code, needle) in cases {
            let err = parse_semantic_toml_manifest(Path::new("semantic.toml"), source)
                .expect_err("must reject");
            assert_eq!(err.code, *code, "wrong code for:\n{source}");
            assert!(
                err.message.contains(needle),
                "missing '{needle}' in error message: {}",
                err.message
            );
        }
    }

    // A Windows root-relative path such as `\rooted\main.sm` has a `RootDir` component
    // but no `Prefix`, so `Path::is_absolute()` reports it as NOT absolute. Joining it to
    // a canonical base resets to the current drive's root, escaping containment. This
    // must be rejected the same as a fully absolute (drive-prefixed) path.
    #[cfg(windows)]
    #[test]
    fn parse_semantic_toml_manifest_rejects_rooted_project_entry_without_drive_prefix() {
        let source = r#"
[package]
name = "app"

[project]
entry = "\rooted\main.sm"
"#;
        let err = parse_semantic_toml_manifest(Path::new("semantic.toml"), source)
            .expect_err("rooted entry without a drive prefix must reject");
        assert_eq!(
            err.code,
            SemanticTomlManifestErrorCode::ProjectEntryMustBeRelative
        );
        assert!(err.message.contains("must be relative"));
    }

    // normalize_project_entry and the module_root-from-entry-parent step both used to
    // fold '\' into '/' unconditionally. On Unix '\' is an ordinary filename character in
    // the entry text (this manifest parser does not interpret TOML escapes; see
    // parse_toml_string), so a literal backslash there must survive unchanged instead of
    // being folded into a directory separator that changes which file/directory it names
    // (same defect class as DL-012/DL-014, found by self-review while closing DL-014).
    #[cfg(unix)]
    #[test]
    fn parse_semantic_toml_manifest_preserves_literal_backslash_in_project_entry_on_unix() {
        let source = r#"
[package]
name = "app"

[project]
entry = "a\b/main.sm"
"#;
        let parsed = parse_semantic_toml_manifest(Path::new("semantic.toml"), source)
            .expect("literal backslash entry must parse on Unix");
        assert_eq!(parsed.entry, "a\\b/main.sm");
        assert_eq!(parsed.manifest.package.root.module_root, "a\\b");
    }

    #[cfg(windows)]
    #[test]
    fn validate_contained_relative_path_rejects_rooted_path_without_drive_prefix() {
        let err = validate_contained_relative_path(
            r"\rooted\dep",
            PackageManifestValidationCode::PackageRootMustBeRelative,
            PackageManifestValidationCode::PackageRootMustNotEscapeManifest,
            "package root",
        )
        .expect_err("rooted path without a drive prefix must reject");
        assert_eq!(
            err.code,
            PackageManifestValidationCode::PackageRootMustBeRelative
        );
        assert!(err.message.contains("must be relative"));
    }

    #[test]
    fn resolve_project_root_check_entry_preserves_structured_diagnostic_codes() {
        let semantic_toml_root = mk_temp_dir("project_root_semantic_toml_codes");
        let semantic_toml = semantic_toml_root.join(SEMANTIC_TOML_FILE_NAME);

        std::fs::write(
            &semantic_toml,
            r#"
[package
name = "app"
"#,
        )
        .expect("write malformed semantic.toml");
        let err = resolve_project_root_check_entry_structured(&semantic_toml_root)
            .expect_err("must reject malformed semantic.toml");
        assert_eq!(
            err.code,
            ProjectRootResolutionCode::SemanticTomlManifest(
                SemanticTomlManifestErrorCode::MalformedSectionHeader
            )
        );
        assert!(err.message.contains("failed to parse"));

        std::fs::write(
            &semantic_toml,
            r#"
[package]
name = ""
"#,
        )
        .expect("write empty package semantic.toml");
        let err = resolve_project_root_check_entry_structured(&semantic_toml_root)
            .expect_err("must reject empty package name");
        assert_eq!(
            err.code,
            ProjectRootResolutionCode::SemanticTomlManifest(
                SemanticTomlManifestErrorCode::EmptyPackageName
            )
        );
        assert!(err.message.contains("empty [package].name"));

        std::fs::write(
            &semantic_toml,
            r#"
[package]
name = "app"

[project]
entry = "../escape.sm"
"#,
        )
        .expect("write escaping semantic.toml");
        let err = resolve_project_root_check_entry_structured(&semantic_toml_root)
            .expect_err("must reject entry escape");
        assert_eq!(
            err.code,
            ProjectRootResolutionCode::SemanticTomlManifest(
                SemanticTomlManifestErrorCode::ProjectEntryMustNotEscapeRoot
            )
        );
        assert!(err.message.contains("must not escape the project root"));
        let _ = std::fs::remove_dir_all(&semantic_toml_root);
    }

    #[test]
    fn resolve_project_root_check_entry_reports_missing_entry_and_manifest_codes() {
        let semantic_toml_root = mk_temp_dir("project_root_missing_entry");
        std::fs::write(
            semantic_toml_root.join(SEMANTIC_TOML_FILE_NAME),
            r#"
[package]
name = "app"

[project]
entry = "src/main.sm"
"#,
        )
        .expect("write semantic.toml");
        let err = resolve_project_root_check_entry_structured(&semantic_toml_root)
            .expect_err("must reject missing entry file");
        assert_eq!(
            err.code,
            ProjectRootResolutionCode::SemanticTomlEntryMissing
        );
        assert!(err.message.contains("resolves to missing file"));
        let _ = std::fs::remove_dir_all(&semantic_toml_root);

        let missing_root = mk_temp_dir("project_root_missing_manifest");
        let err = resolve_project_root_check_entry_structured(&missing_root)
            .expect_err("must reject missing manifests");
        assert_eq!(err.code, ProjectRootResolutionCode::MissingProjectManifest);
        assert!(err.message.contains("must contain"));
        let _ = std::fs::remove_dir_all(&missing_root);
    }

    #[test]
    fn admit_package_entry_module_maps_entry_into_package_context() {
        let dir = mk_temp_dir("pkg_admit_ok");
        let src_dir = dir.join("src");
        std::fs::create_dir_all(src_dir.join("nested")).expect("mkdir src");
        std::fs::write(
            dir.join(PACKAGE_MANIFEST_FILE_NAME),
            r#"
format 1
package app
manifest_dir .
module_root src
"#,
        )
        .expect("write manifest");
        let entry = src_dir.join("nested").join("main.sm");
        std::fs::write(&entry, "fn main() { return; }").expect("write entry");

        let admitted = admit_package_entry_module(&entry)
            .expect("admit")
            .expect("manifest must exist");
        assert_eq!(admitted.package_name, "app");
        assert!(admitted.manifest_path.ends_with("/Semantic.package"));
        assert_eq!(admitted.module_path, "nested/main.sm");

        let _ = std::fs::remove_dir_all(&dir);
    }

    // `package inspect` already rejects a nested manifest anywhere under module_root via
    // package_content_fingerprint's tree walk, regardless of whether any module imports
    // from it. Admission (used by check/compile/run) must reject the same tree instead of
    // silently accepting it just because nothing happens to import the nested package.
    #[test]
    fn admit_package_entry_module_rejects_unused_nested_manifest() {
        let dir = mk_temp_dir("pkg_admit_unused_nested");
        let src_dir = dir.join("src");
        std::fs::create_dir_all(src_dir.join("nested")).expect("mkdir src/nested");
        std::fs::write(
            dir.join(PACKAGE_MANIFEST_FILE_NAME),
            r#"
format 1
package app
manifest_dir .
module_root src
"#,
        )
        .expect("write manifest");
        std::fs::write(
            src_dir.join("nested").join(PACKAGE_MANIFEST_FILE_NAME),
            "format 1\npackage nested\nmanifest_dir .\nmodule_root .\n",
        )
        .expect("write nested manifest");
        let entry = src_dir.join("main.sm");
        std::fs::write(&entry, "fn main() { return; }").expect("write entry");

        let err = admit_package_entry_module(&entry)
            .expect_err("unused nested manifest must be rejected");
        assert_eq!(
            err.code,
            PackageModuleAdmissionCode::NestedManifestInsideModuleRoot
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    // `smc package inspect` rejects a package whose declared dependency graph is invalid
    // (missing/cyclic/name-mismatched/stale-pinned) regardless of whether any module
    // actually imports the broken dependency, by walking the full graph via
    // PackageGraphBuilder::visit. Admission used to validate dependencies only lazily, as
    // each was actually resolved through an import, so `check`/`compile`/`run` accepted
    // and executed a package with an unused-but-missing declared dependency that `inspect`
    // would reject on the identical tree -- found by @codex review (DL-021).
    #[test]
    fn admit_package_entry_module_rejects_unused_missing_declared_dependency() {
        let dir = mk_temp_dir("pkg_admit_missing_unused_dep");
        let src_dir = dir.join("src");
        std::fs::create_dir_all(&src_dir).expect("mkdir src");
        std::fs::write(
            dir.join(PACKAGE_MANIFEST_FILE_NAME),
            r#"
format 1
package app
manifest_dir .
module_root src
dep math math ../math
"#,
        )
        .expect("write manifest");
        let entry = src_dir.join("main.sm");
        std::fs::write(&entry, "fn main() { return; }").expect("write entry");
        // Deliberately do not create ../math -- the declared dependency is missing, and
        // nothing in `entry` actually imports it.

        let err = admit_package_entry_module(&entry)
            .expect_err("an unused but missing declared dependency must still be rejected");
        assert_eq!(
            err.code,
            PackageModuleAdmissionCode::DeclaredDependencyGraphInvalid
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    // The declared-dependency-graph validation is cached per manifest so admitting many
    // modules from the same manifest only walks the graph once per bundle pass. Prove the
    // cache is real (a dependency removed after the first admission is NOT caught by a
    // second admission against the same cache) and that
    // reset_declared_dependency_graph_cache correctly re-enables detection, mirroring
    // DL-020's proof for the sibling pinned-content cache.
    #[test]
    fn admit_package_entry_module_declared_dependency_graph_cache_is_resettable() {
        let dir = mk_temp_dir("pkg_admit_dep_graph_cache");
        let app_src = dir.join("app").join("src");
        let math_src = dir.join("math").join("src");
        std::fs::create_dir_all(&app_src).expect("mkdir app src");
        std::fs::create_dir_all(&math_src).expect("mkdir math src");
        std::fs::write(
            dir.join("math").join(PACKAGE_MANIFEST_FILE_NAME),
            "format 1\npackage math\nmanifest_dir .\nmodule_root src\n",
        )
        .expect("write math manifest");
        std::fs::write(
            dir.join("app").join(PACKAGE_MANIFEST_FILE_NAME),
            "format 1\npackage dep_graph_cache_app\nmanifest_dir .\nmodule_root src\ndep math math ../math\n",
        )
        .expect("write app manifest");
        let first_entry = app_src.join("first.sm");
        let second_entry = app_src.join("second.sm");
        std::fs::write(&first_entry, "fn first() { return; }").expect("write first entry");
        std::fs::write(&second_entry, "fn second() { return; }").expect("write second entry");

        reset_declared_dependency_graph_cache();
        admit_package_entry_module(&first_entry)
            .expect("first admission must succeed and populate the cache")
            .expect("manifest must exist");

        let _ = std::fs::remove_dir_all(dir.join("math"));

        admit_package_entry_module(&second_entry)
            .expect("second admission must succeed from the cache despite the removed dependency")
            .expect("manifest must exist");

        reset_declared_dependency_graph_cache();
        let err = admit_package_entry_module(&second_entry)
            .expect_err("after reset, the missing dependency must be re-detected");
        assert_eq!(
            err.code,
            PackageModuleAdmissionCode::DeclaredDependencyGraphInvalid
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    // docs/spec/project_model_v0.md permits `semantic.toml` and `Semantic.package` to
    // co-exist in the same directory for compatibility, with `semantic.toml` winning as
    // the authority manifest. The DL-013 fix above wrongly treated the losing, co-located
    // manifest as forbidden "nested" content and rejected it -- found by @codex review
    // (DL-016). A manifest is only nested when it sits *below* the authority manifest's
    // own directory, not when it merely sits beside it.
    #[test]
    fn admit_package_entry_module_permits_co_located_semantic_toml_and_package_manifest() {
        let dir = mk_temp_dir("pkg_admit_co_located_manifests");
        std::fs::write(
            dir.join(SEMANTIC_TOML_FILE_NAME),
            "[package]\nname = \"app\"\n\n[project]\nentry = \"main.sm\"\n",
        )
        .expect("write semantic.toml");
        std::fs::write(
            dir.join(PACKAGE_MANIFEST_FILE_NAME),
            "format 1\npackage app\nmanifest_dir .\nmodule_root .\n",
        )
        .expect("write co-located Semantic.package");
        let entry = dir.join("main.sm");
        std::fs::write(&entry, "fn main() { return; }").expect("write entry");

        let admitted = admit_package_entry_module(&entry)
            .expect("admission must not error")
            .expect("semantic.toml manifest must exist");
        assert_eq!(admitted.package_name, "app");

        let _ = std::fs::remove_dir_all(&dir);
    }

    // DL-016 originally memoized the nested-manifest scan per module_root for the life of
    // the process. `smc watch` (see `cmd_watch` in app.rs) is a genuinely long-lived
    // process that re-admits the same module_root repeatedly as files change, so that
    // cache would have permanently hidden a nested manifest added after the first scan --
    // found by @codex review and fixed in DL-017 by removing the process-wide cache.
    // Prove there is no stale caching: a nested manifest added between two admissions of
    // the same module_root must be detected on the second call, not silently accepted
    // because the first call already "validated" that module_root.
    #[test]
    fn admit_package_entry_module_detects_nested_manifest_added_after_prior_admission() {
        let dir = mk_temp_dir("pkg_admit_no_stale_cache");
        let src_dir = dir.join("src");
        std::fs::create_dir_all(&src_dir).expect("mkdir src");
        std::fs::write(
            dir.join(PACKAGE_MANIFEST_FILE_NAME),
            "format 1\npackage no_stale_cache_app\nmanifest_dir .\nmodule_root src\n",
        )
        .expect("write manifest");
        let first_entry = src_dir.join("first.sm");
        std::fs::write(&first_entry, "fn first() { return; }").expect("write first entry");
        admit_package_entry_module(&first_entry)
            .expect("first admission must not error")
            .expect("manifest must exist");

        std::fs::create_dir_all(src_dir.join("later_nested")).expect("mkdir later_nested");
        std::fs::write(
            src_dir
                .join("later_nested")
                .join(PACKAGE_MANIFEST_FILE_NAME),
            "format 1\npackage later_nested\nmanifest_dir .\nmodule_root .\n",
        )
        .expect("write later nested manifest");

        let second_entry = src_dir.join("second.sm");
        std::fs::write(&second_entry, "fn second() { return; }").expect("write second entry");
        let err = admit_package_entry_module(&second_entry)
            .expect_err("nested manifest added after a prior admission must still be caught");
        assert_eq!(
            err.code,
            PackageModuleAdmissionCode::NestedManifestInsideModuleRoot
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn admit_package_entry_module_rejects_entry_outside_module_root() {
        let dir = mk_temp_dir("pkg_admit_outside");
        let src_dir = dir.join("src");
        std::fs::create_dir_all(&src_dir).expect("mkdir src");
        std::fs::write(
            dir.join(PACKAGE_MANIFEST_FILE_NAME),
            r#"
format 1
package app
manifest_dir .
module_root src
"#,
        )
        .expect("write manifest");
        let entry = dir.join("main.sm");
        std::fs::write(&entry, "fn main() { return; }").expect("write entry");

        let err = admit_package_entry_module(&entry).expect_err("must reject");
        assert_eq!(err.code, PackageModuleAdmissionCode::EntryOutsideModuleRoot);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_package_import_path_keeps_relative_import_behavior() {
        let dir = mk_temp_dir("pkg_import_relative");
        let src_dir = dir.join("src");
        std::fs::create_dir_all(&src_dir).expect("mkdir src");
        std::fs::write(
            dir.join(PACKAGE_MANIFEST_FILE_NAME),
            r#"
format 1
package app
manifest_dir .
module_root src
"#,
        )
        .expect("write manifest");
        let importer = src_dir.join("main.sm");
        let child = src_dir.join("child.sm");
        std::fs::write(&importer, "Import \"child.sm\"\nfn main() { return; }\n")
            .expect("write importer");
        std::fs::write(&child, "fn child() { return; }\n").expect("write child");

        let resolved = resolve_package_import_path(&importer, "child.sm").expect("resolve");
        assert_eq!(
            normalize_canonical_path(&resolved),
            normalize_canonical_path(&child)
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_package_import_path_maps_local_path_dependency_alias() {
        let dir = mk_temp_dir("pkg_import_alias");
        let app_src = dir.join("app").join("src");
        let math_src = dir.join("math").join("src");
        std::fs::create_dir_all(&app_src).expect("mkdir app src");
        std::fs::create_dir_all(&math_src).expect("mkdir math src");
        std::fs::write(
            dir.join("app").join(PACKAGE_MANIFEST_FILE_NAME),
            r#"
format 1
package app
manifest_dir .
module_root src
dep math math ../math
"#,
        )
        .expect("write app manifest");
        std::fs::write(
            dir.join("math").join(PACKAGE_MANIFEST_FILE_NAME),
            r#"
format 1
package math
manifest_dir .
module_root src
"#,
        )
        .expect("write math manifest");
        let importer = app_src.join("main.sm");
        let dep = math_src.join("core.sm");
        std::fs::write(
            &importer,
            "Import \"math::core.sm\"\nfn main() { return; }\n",
        )
        .expect("write importer");
        std::fs::write(&dep, "fn core() { return; }\n").expect("write dep");

        let resolved = resolve_package_import_path(&importer, "math::core.sm").expect("resolve");
        assert_eq!(
            normalize_canonical_path(&resolved),
            normalize_canonical_path(&dep)
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_package_import_path_rejects_unknown_dependency_alias() {
        let dir = mk_temp_dir("pkg_import_missing_alias");
        let src_dir = dir.join("src");
        std::fs::create_dir_all(&src_dir).expect("mkdir src");
        std::fs::write(
            dir.join(PACKAGE_MANIFEST_FILE_NAME),
            r#"
format 1
package app
manifest_dir .
module_root src
"#,
        )
        .expect("write manifest");
        let importer = src_dir.join("main.sm");
        std::fs::write(
            &importer,
            "Import \"math::core.sm\"\nfn main() { return; }\n",
        )
        .expect("write importer");

        let err = resolve_package_import_path(&importer, "math::core.sm").expect_err("must reject");
        assert_eq!(
            err.code,
            PackageImportResolutionCode::UnknownDependencyAlias
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_package_import_path_rejects_dependency_package_name_mismatch() {
        let dir = mk_temp_dir("pkg_import_name_mismatch");
        let app_src = dir.join("app").join("src");
        let math_src = dir.join("math").join("src");
        std::fs::create_dir_all(&app_src).expect("mkdir app src");
        std::fs::create_dir_all(&math_src).expect("mkdir math src");
        std::fs::write(
            dir.join("app").join(PACKAGE_MANIFEST_FILE_NAME),
            r#"
format 1
package app
manifest_dir .
module_root src
dep math math ../math
"#,
        )
        .expect("write app manifest");
        std::fs::write(
            dir.join("math").join(PACKAGE_MANIFEST_FILE_NAME),
            r#"
format 1
package other_math
manifest_dir .
module_root src
"#,
        )
        .expect("write math manifest");
        let importer = app_src.join("main.sm");
        std::fs::write(
            &importer,
            "Import \"math::core.sm\"\nfn main() { return; }\n",
        )
        .expect("write importer");

        let err = resolve_package_import_path(&importer, "math::core.sm").expect_err("must reject");
        assert_eq!(
            err.code,
            PackageImportResolutionCode::DependencyPackageNameMismatch
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    // A pinned dependency imported through multiple package-qualified modules would
    // otherwise be re-read and re-hashed on every single import -- O(N*M) for N importing
    // modules and M dependency files -- found by @codex review as a direct consequence of
    // DL-016's admission-side memoization fix. Cache the content-fingerprint verification
    // per dependency manifest, but (learning DL-017's lesson: a process-lifetime cache is
    // unsafe for `smc watch`) make it explicitly resettable rather than a bare `static`,
    // and prove both halves: the cache is real (a tamper made after the first resolution
    // is NOT caught by a second resolution against the same cache), and reset correctly
    // re-enables detection (DL-020).
    #[test]
    fn resolve_package_import_path_caches_pinned_dependency_content_verification() {
        let dir = mk_temp_dir("pkg_import_pinned_cache");
        let app_src = dir.join("app").join("src");
        let math_src = dir.join("math").join("src");
        std::fs::create_dir_all(&app_src).expect("mkdir app src");
        std::fs::create_dir_all(&math_src).expect("mkdir math src");
        let math_manifest_path = dir.join("math").join(PACKAGE_MANIFEST_FILE_NAME);
        std::fs::write(
            &math_manifest_path,
            "format 1\npackage math\nmanifest_dir .\nmodule_root src\n",
        )
        .expect("write math manifest");
        std::fs::write(math_src.join("core.sm"), "fn core() { return; }\n")
            .expect("write dep content");

        let manifest_fp = manifest_fingerprint(&math_manifest_path).expect("manifest fp");
        let content_fp =
            package_content_fingerprint(&math_src, &math_manifest_path).expect("content fp");
        std::fs::write(
            dir.join("app").join(PACKAGE_MANIFEST_FILE_NAME),
            format!(
                "format 1\npackage app\nmanifest_dir .\nmodule_root src\ndep math math ../math {manifest_fp} {content_fp}\n"
            ),
        )
        .expect("write app manifest");
        let importer = app_src.join("main.sm");
        std::fs::write(
            &importer,
            "Import \"math::core.sm\"\nfn main() { return; }\n",
        )
        .expect("write importer");

        reset_pinned_dependency_fingerprint_cache();
        resolve_package_import_path(&importer, "math::core.sm")
            .expect("first resolution must succeed and populate the cache");

        std::fs::write(
            math_src.join("core.sm"),
            "fn core() { return; } // tampered\n",
        )
        .expect("tamper dep content without updating the pin");

        resolve_package_import_path(&importer, "math::core.sm").expect(
            "second resolution must succeed from the cache despite the tampered content \
             -- this is the caching behavior under test, not a security hole: the pin was \
             already verified once this pass",
        );

        reset_pinned_dependency_fingerprint_cache();
        let err = resolve_package_import_path(&importer, "math::core.sm")
            .expect_err("after an explicit reset, the tampered content must be re-detected");
        assert_eq!(
            err.code,
            PackageImportResolutionCode::DependencyContentFingerprintMismatch
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    // A lossy relative-path conversion would let two distinct non-UTF-8 filenames
    // collapse onto the same fingerprint key, so renaming between them while keeping
    // file bytes unchanged would leave the content fingerprint unchanged too --
    // silently hiding a real source-tree change from a pinned dependency's provenance
    // record. Reject non-UTF-8 relative paths outright instead of hashing a lossy
    // string.
    #[cfg(unix)]
    #[test]
    fn package_content_fingerprint_rejects_non_utf8_module_paths() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let dir = mk_temp_dir("package_content_fingerprint_non_utf8");
        let module_root = dir.join("src");
        std::fs::create_dir_all(&module_root).expect("mkdir src");
        let manifest = dir.join(PACKAGE_MANIFEST_FILE_NAME);
        std::fs::write(
            &manifest,
            "format 1\npackage app\nmanifest_dir .\nmodule_root src\n",
        )
        .expect("write manifest");
        let invalid_name = OsStr::from_bytes(&[0xffu8, b'.', b's', b'm']);
        std::fs::write(module_root.join(invalid_name), "fn main() { return; }\n")
            .expect("write non-utf8 module");

        let err = package_content_fingerprint(&module_root, &manifest)
            .expect_err("non-UTF-8 module path must reject");
        assert!(err.contains("not valid UTF-8"), "unexpected error: {err}");

        let _ = std::fs::remove_dir_all(&dir);
    }

    // On Unix, `\` is an ordinary filename character, not a separator. Blindly replacing
    // `\` with `/` when normalizing a relative path therefore collapses a real subdirectory
    // path (`dir/helper.sm`) and a single file literally named `dir\helper.sm` onto the same
    // fingerprint key, so renaming between those two forms while keeping the bytes unchanged
    // would leave `content_fingerprint` unchanged too. Only fold `\` into `/` on Windows,
    // where it is the actual path separator.
    #[cfg(unix)]
    #[test]
    fn package_content_fingerprint_does_not_collapse_literal_backslash_with_separator() {
        let content = "fn main() { return; }\n";

        let real_dir = mk_temp_dir("package_content_fingerprint_backslash_real");
        let real_module_root = real_dir.join("src");
        std::fs::create_dir_all(real_module_root.join("dir")).expect("mkdir src/dir");
        let real_manifest = real_dir.join(PACKAGE_MANIFEST_FILE_NAME);
        std::fs::write(
            &real_manifest,
            "format 1\npackage app\nmanifest_dir .\nmodule_root src\n",
        )
        .expect("write manifest");
        std::fs::write(real_module_root.join("dir").join("helper.sm"), content)
            .expect("write dir/helper.sm");
        let real_fingerprint = package_content_fingerprint(&real_module_root, &real_manifest)
            .expect("real subdirectory path must fingerprint");

        let literal_dir = mk_temp_dir("package_content_fingerprint_backslash_literal");
        let literal_module_root = literal_dir.join("src");
        std::fs::create_dir_all(&literal_module_root).expect("mkdir src");
        let literal_manifest = literal_dir.join(PACKAGE_MANIFEST_FILE_NAME);
        std::fs::write(
            &literal_manifest,
            "format 1\npackage app\nmanifest_dir .\nmodule_root src\n",
        )
        .expect("write manifest");
        std::fs::write(literal_module_root.join("dir\\helper.sm"), content)
            .expect("write literal dir\\helper.sm");
        let literal_fingerprint =
            package_content_fingerprint(&literal_module_root, &literal_manifest)
                .expect("literal backslash filename must fingerprint");

        assert_ne!(
            real_fingerprint, literal_fingerprint,
            "a real dir/helper.sm subdirectory path and a file literally named \
             dir\\helper.sm must not collapse onto the same content fingerprint"
        );

        let _ = std::fs::remove_dir_all(&real_dir);
        let _ = std::fs::remove_dir_all(&literal_dir);
    }

    // `smc check`/`compile`/`run` select `semantic.toml` over a co-located `Semantic.package`
    // (docs/spec/project_model_v0.md:18-20), but `inspect_local_package_graph` used to
    // always select `Semantic.package` directly, so its provenance record could describe a
    // different package identity/module_root/dependency graph than the code the project
    // commands actually admit -- found by @codex review (DL-019).
    #[test]
    fn inspect_local_package_graph_honors_semantic_toml_precedence_over_package_manifest() {
        let dir = mk_temp_dir("pkg_inspect_semantic_toml_precedence");
        std::fs::write(
            dir.join(SEMANTIC_TOML_FILE_NAME),
            "[package]\nname = \"winner_from_semantic_toml\"\n\n[project]\nentry = \"main.sm\"\n",
        )
        .expect("write semantic.toml");
        std::fs::write(
            dir.join(PACKAGE_MANIFEST_FILE_NAME),
            "format 1\npackage loser_from_package_manifest\nmanifest_dir .\nmodule_root .\n",
        )
        .expect("write co-located Semantic.package");
        std::fs::write(dir.join("main.sm"), "fn main() { return; }\n").expect("write entry");

        let provenance = inspect_local_package_graph(&dir).expect("inspect must succeed");
        let value: serde_json::Value =
            serde_json::from_str(&provenance).expect("provenance must be valid JSON");
        assert_eq!(value["root_package"], "winner_from_semantic_toml");

        let _ = std::fs::remove_dir_all(&dir);
    }

    // Same defect class as the content-fingerprint collision above, but for the
    // dependency-edge provenance record: on Unix a declared local dependency path is not a
    // separator-delimited OS path, it is raw manifest text, so a literal '\' in it must be
    // preserved rather than folded into '/' when emitted into the provenance JSON.
    #[cfg(unix)]
    #[test]
    fn inspect_local_package_graph_preserves_literal_backslash_in_dependency_local_path() {
        let dir = mk_temp_dir("pkg_inspect_backslash_dep_path");
        let app_dir = dir.join("app");
        let app_src = app_dir.join("src");
        std::fs::create_dir_all(&app_src).expect("mkdir app src");
        let dep_dir = dir.join("dep\\lib");
        std::fs::create_dir_all(&dep_dir).expect("mkdir literal-backslash dep dir");
        std::fs::write(
            dep_dir.join(PACKAGE_MANIFEST_FILE_NAME),
            "format 1\npackage libx\nmanifest_dir .\nmodule_root .\n",
        )
        .expect("write dependency manifest");
        std::fs::write(
            app_dir.join(PACKAGE_MANIFEST_FILE_NAME),
            "format 1\npackage app\nmanifest_dir .\nmodule_root src\ndep libx libx ../dep\\lib\n",
        )
        .expect("write app manifest");
        std::fs::write(app_src.join("main.sm"), "fn main() { return; }\n").expect("write entry");

        let provenance = inspect_local_package_graph(&app_dir).expect("inspect must succeed");
        let value: serde_json::Value =
            serde_json::from_str(&provenance).expect("provenance must be valid JSON");
        let app_node = value["packages"]
            .as_array()
            .expect("packages array")
            .iter()
            .find(|node| node["name"] == "app")
            .expect("app node present");
        let local_path = app_node["dependencies"][0]["local_path"]
            .as_str()
            .expect("local_path string");
        assert_eq!(
            local_path, "../dep\\lib",
            "a literal backslash in a declared dependency path must survive into provenance \
             unchanged on Unix, not be folded into a separator"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    // normalize_path (used for the PackageModuleAdmission.manifest_path display field) had
    // the same unconditional backslash-fold bug as the other normalize_* helpers in this
    // file; only Windows actually uses '\' as a separator.
    #[cfg(unix)]
    #[test]
    fn normalize_path_preserves_literal_backslash_on_unix() {
        assert_eq!(normalize_path(Path::new("dir\\file.sm")), "dir\\file.sm");
    }
}
