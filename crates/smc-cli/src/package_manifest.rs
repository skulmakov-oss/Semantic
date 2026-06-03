use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};

pub const PACKAGE_MANIFEST_BASELINE_VERSION: u32 = 1;
pub const PACKAGE_MANIFEST_FILE_NAME: &str = "Semantic.package";
pub const SEMANTIC_TOML_FILE_NAME: &str = "semantic.toml";
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProjectRootResolutionCode {
    SemanticTomlReadFailed,
    SemanticTomlManifest(SemanticTomlManifestErrorCode),
    SemanticTomlEntryMissing,
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
        return Ok(root.join("src/main.sm"));
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
        let semantic_toml = dir.join(SEMANTIC_TOML_FILE_NAME);
        if semantic_toml.is_file() {
            return Some(semantic_toml);
        }
        let package_manifest = dir.join(PACKAGE_MANIFEST_FILE_NAME);
        if package_manifest.is_file() {
            return Some(package_manifest);
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
        if path.is_absolute() {
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
        Ok(path.to_string_lossy().replace('\\', "/"))
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
            let value = parent.to_string_lossy().replace('\\', "/");
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
                r#"
[package]
name = "app"

[project]
entry = "C:/abs/main.sm"
"#,
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
