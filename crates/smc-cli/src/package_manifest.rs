use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};

pub const PACKAGE_MANIFEST_BASELINE_VERSION: u32 = 1;
pub const PACKAGE_MANIFEST_FILE_NAME: &str = "Semantic.package";
pub const PACKAGE_IMPORT_SEPARATOR: &str = "::";

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
    LocalPath { path: String },
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
    EntryOutsideModuleRoot,
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
                if tokens.len() != 4 {
                    return Err(parse_error(
                        PackageManifestParseCode::InvalidDirective,
                        line_no,
                        "dep directive must be: dep <alias> <package_name> <local_path>",
                    ));
                }
                dependencies.push(PackageDependency {
                    alias: tokens[1].clone(),
                    package_name: tokens[2].clone(),
                    source: PackageDependencySource::LocalPath {
                        path: tokens[3].clone(),
                    },
                });
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

    if manifest.package.root.module_root.trim().is_empty() {
        return Err(PackageManifestValidationError {
            code: PackageManifestValidationCode::EmptyModuleRoot,
            message: "package module_root must not be empty".to_string(),
        });
    }

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
        match &dependency.source {
            PackageDependencySource::LocalPath { path } if path.trim().is_empty() => {
                return Err(PackageManifestValidationError {
                    code: PackageManifestValidationCode::EmptyLocalDependencyPath,
                    message: format!(
                        "package dependency '{}' requires a non-empty local path",
                        dependency.alias
                    ),
                });
            }
            PackageDependencySource::LocalPath { .. } => {}
        }
    }

    Ok(())
}

pub fn admit_package_entry_module(
    entry: &Path,
) -> Result<Option<PackageModuleAdmission>, PackageModuleAdmissionError> {
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
    let manifest = load_and_validate_manifest(&manifest_path)?;
    let manifest_dir = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    let package_root = manifest_dir
        .join(&manifest.package.root.manifest_dir)
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
    let module_root = package_root
        .join(&manifest.package.root.module_root)
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

    Ok(Some(PackageModuleAdmission {
        manifest_path: normalize_path(&manifest_path),
        package_name: manifest.package.name,
        module_path: normalize_relative_path(module_relative),
    }))
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
        return resolve_dependency_import(&importer_canonical, alias, module_spec, spec);
    }

    let base = importer_canonical
        .parent()
        .unwrap_or_else(|| Path::new("."));
    Ok(resolve_relative_import_path(base, spec))
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

fn find_nearest_manifest(entry_canonical: &Path) -> Option<PathBuf> {
    let mut current = entry_canonical.parent();
    while let Some(dir) = current {
        let candidate = dir.join(PACKAGE_MANIFEST_FILE_NAME);
        if candidate.is_file() {
            return Some(candidate);
        }
        current = dir.parent();
    }
    None
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
    let manifest =
        parse_package_manifest_baseline(&source).map_err(|e| PackageModuleAdmissionError {
            code: PackageModuleAdmissionCode::ManifestParseFailed,
            message: format!("failed to parse '{}': {}", manifest_path.display(), e),
        })?;
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
    if alias.trim().is_empty() || module_spec.trim().is_empty() {
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

    let dependency_path = match &dependency.source {
        PackageDependencySource::LocalPath { path } => path,
    };
    let dependency_manifest_path = importer_ctx
        .package_root
        .join(dependency_path)
        .join(PACKAGE_MANIFEST_FILE_NAME);
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
    let source =
        std::fs::read_to_string(manifest_path).map_err(|e| PackageImportResolutionError {
            code: read_code,
            message: format!("failed to read '{}': {}", manifest_path.display(), e),
        })?;
    let manifest =
        parse_package_manifest_baseline(&source).map_err(|e| PackageImportResolutionError {
            code: parse_code,
            message: format!("failed to parse '{}': {}", manifest_path.display(), e),
        })?;
    validate_package_manifest_baseline(&manifest).map_err(|e| PackageImportResolutionError {
        code: validate_code,
        message: format!("failed to validate '{}': {}", manifest_path.display(), e),
    })?;

    let manifest_dir = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    let package_root = manifest_dir
        .join(&manifest.package.root.manifest_dir)
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
    let module_root = package_root
        .join(&manifest.package.root.module_root)
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

    Ok(ResolvedPackageContext {
        manifest,
        package_root,
        module_root,
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
    let normalized = path.to_string_lossy().replace('\\', "/");
    normalized
        .strip_prefix("//?/")
        .unwrap_or(&normalized)
        .to_string()
}

fn normalize_relative_path(path: &Path) -> String {
    let value = normalize_path(path);
    if value.is_empty() {
        ".".to_string()
    } else {
        value
    }
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
"#,
        )
        .expect("parse");
        assert_eq!(manifest.package.name, "app");
        assert_eq!(manifest.dependencies.len(), 2);
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
dep math math ../math
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
        assert_eq!(normalize_path(&resolved), normalize_path(&child));

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
        assert_eq!(normalize_path(&resolved), normalize_path(&dep));

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
}
