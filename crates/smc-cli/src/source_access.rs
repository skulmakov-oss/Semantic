//! SSF-09 #1580 (R3): the one source-reading seam behind project checking.
//!
//! Every source read the canonical project mechanisms perform - the root,
//! Logos project modules (`sm-sema`'s `ModuleProvider`), and RustLike
//! executable-helper modules (the bundler) - goes through a
//! [`SourceAccess`]. It owns three host-facing concerns and nothing else:
//!
//! - where source text comes from (disk, or an editor overlay first);
//! - how a module is named in diagnostic text;
//! - how host failures (package admission, import resolution, I/O) are
//!   described in diagnostic text.
//!
//! It never parses, classifies or checks source. [`DiskSources`] is the
//! legacy behaviour of every non-canonical command, byte for byte.
//! [`CanonicalSources`] backs `smc check --format` and `smc lsp`: overlay
//! first, module names relative to a fixed anchor directory (the checked
//! root's directory), and failure text built only from structured error
//! codes and `std::io::ErrorKind` - never an absolute host path or
//! OS-specific error wording - so the same project checked from two
//! checkout locations yields byte-identical canonical diagnostics.

use crate::canonical_check::SourceOverlay;
use crate::package_manifest::{
    PackageImportResolutionCode, PackageImportResolutionError, PackageModuleAdmissionCode,
    PackageModuleAdmissionError,
};
use std::path::{Component, Path, PathBuf};

pub(crate) trait SourceAccess {
    /// The text of the admitted source at canonical path `path`.
    fn read_source(&self, path: &Path) -> Result<String, String>;
    /// How diagnostic text names the source at `path`.
    fn display_path(&self, path: &Path) -> String;
    /// How diagnostic text describes a package admission failure.
    fn describe_admission_error(&self, error: &PackageModuleAdmissionError) -> String;
    /// As [`SourceAccess::describe_admission_error`], for a failure the
    /// surrounding text does not already attribute to `path`.
    fn describe_admission_error_at(
        &self,
        path: &Path,
        error: &PackageModuleAdmissionError,
    ) -> String;
    /// How diagnostic text describes an import resolution failure.
    fn describe_resolution_error(&self, error: &PackageImportResolutionError) -> String;
    /// How diagnostic text describes an I/O failure.
    fn describe_io_error(&self, error: &std::io::Error) -> String;
}

/// Legacy behaviour: disk only, host paths and host error text verbatim.
pub(crate) struct DiskSources;

impl SourceAccess for DiskSources {
    fn read_source(&self, path: &Path) -> Result<String, String> {
        std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read '{}': {}", path.display(), e))
    }

    fn display_path(&self, path: &Path) -> String {
        path.display().to_string()
    }

    fn describe_admission_error(&self, error: &PackageModuleAdmissionError) -> String {
        error.to_string()
    }

    fn describe_admission_error_at(
        &self,
        _path: &Path,
        error: &PackageModuleAdmissionError,
    ) -> String {
        // The legacy admission text already names the module.
        error.to_string()
    }

    fn describe_resolution_error(&self, error: &PackageImportResolutionError) -> String {
        error.to_string()
    }

    fn describe_io_error(&self, error: &std::io::Error) -> String {
        error.to_string()
    }
}

/// Canonical behaviour: editor overlay first, checkout-independent text.
pub(crate) struct CanonicalSources<'a> {
    overlay: &'a SourceOverlay,
    anchor: PathBuf,
}

impl<'a> CanonicalSources<'a> {
    /// `anchor` is the directory module names are made relative to (the
    /// checked root's directory, canonicalized when it exists).
    pub(crate) fn new(overlay: &'a SourceOverlay, anchor: &Path) -> Self {
        Self {
            overlay,
            anchor: anchor.canonicalize().unwrap_or_else(|_| lexical(anchor)),
        }
    }
}

impl SourceAccess for CanonicalSources<'_> {
    fn read_source(&self, path: &Path) -> Result<String, String> {
        if let Some(text) = self.overlay.get(path) {
            return Ok(text.to_string());
        }
        std::fs::read_to_string(path).map_err(|e| {
            format!(
                "failed to read '{}': {}",
                self.display_path(path),
                self.describe_io_error(&e)
            )
        })
    }

    fn display_path(&self, path: &Path) -> String {
        relative_display(&self.anchor, path)
    }

    fn describe_admission_error(&self, error: &PackageModuleAdmissionError) -> String {
        admission_phrase(error.code).to_string()
    }

    fn describe_admission_error_at(
        &self,
        path: &Path,
        error: &PackageModuleAdmissionError,
    ) -> String {
        format!(
            "module '{}': {}",
            self.display_path(path),
            admission_phrase(error.code)
        )
    }

    fn describe_resolution_error(&self, error: &PackageImportResolutionError) -> String {
        resolution_phrase(error.code).to_string()
    }

    fn describe_io_error(&self, error: &std::io::Error) -> String {
        // `ErrorKind`'s description is defined by `std`, not by the OS.
        error.kind().to_string()
    }
}

/// `path` relative to `anchor`, with `..` for each anchor component not
/// shared and `/` separators. Injective for normalized absolute paths, and
/// independent of where the checkout containing both lives.
pub(crate) fn relative_display(anchor: &Path, path: &Path) -> String {
    let path = if path.is_absolute() {
        path.canonicalize().unwrap_or_else(|_| lexical(path))
    } else {
        lexical(&anchor.join(path))
    };
    let anchor_parts: Vec<Component> = anchor.components().collect();
    let path_parts: Vec<Component> = path.components().collect();
    let shared = anchor_parts
        .iter()
        .zip(&path_parts)
        .take_while(|(a, b)| a == b)
        .count();
    let mut out: Vec<String> = Vec::new();
    for _ in shared..anchor_parts.len() {
        out.push("..".to_string());
    }
    for part in &path_parts[shared..] {
        out.push(part.as_os_str().to_string_lossy().into_owned());
    }
    if out.is_empty() {
        ".".to_string()
    } else {
        out.join("/")
    }
}

/// Lexical normalization (`.` dropped, `..` applied) of an absolute path.
fn lexical(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

fn admission_phrase(code: PackageModuleAdmissionCode) -> &'static str {
    match code {
        PackageModuleAdmissionCode::EntryResolutionFailed => {
            "module file does not exist or cannot be resolved"
        }
        PackageModuleAdmissionCode::ManifestReadFailed => "package manifest cannot be read",
        PackageModuleAdmissionCode::ManifestParseFailed => "package manifest is malformed",
        PackageModuleAdmissionCode::ManifestValidationFailed => "package manifest is invalid",
        PackageModuleAdmissionCode::PackageRootResolutionFailed => {
            "package root cannot be resolved"
        }
        PackageModuleAdmissionCode::ModuleRootResolutionFailed => {
            "package module_root cannot be resolved"
        }
        PackageModuleAdmissionCode::NestedManifestInsideModuleRoot => {
            "a nested package manifest lies inside the package module_root"
        }
        PackageModuleAdmissionCode::EntryOutsideModuleRoot => {
            "module lies outside its package module_root"
        }
        PackageModuleAdmissionCode::NonUtf8ModulePath => "module path is not valid UTF-8",
        PackageModuleAdmissionCode::DeclaredDependencyGraphInvalid => {
            "the package's declared dependency graph is invalid"
        }
    }
}

fn resolution_phrase(code: PackageImportResolutionCode) -> &'static str {
    use PackageImportResolutionCode as C;
    match code {
        C::ImporterResolutionFailed => "importing module cannot be resolved",
        C::ImporterManifestMissing => "importing module has no package manifest",
        C::ImporterManifestReadFailed => "importing package manifest cannot be read",
        C::ImporterManifestParseFailed => "importing package manifest is malformed",
        C::ImporterManifestValidationFailed => "importing package manifest is invalid",
        C::ImporterPackageRootResolutionFailed => "importing package root cannot be resolved",
        C::ImporterModuleRootResolutionFailed => "importing package module_root cannot be resolved",
        C::InvalidQualifiedImportSpec => "package-qualified import spec is invalid",
        C::UnknownDependencyAlias => "unknown dependency alias",
        C::DependencyManifestMissing => "dependency package has no manifest",
        C::DependencyManifestReadFailed => "dependency package manifest cannot be read",
        C::DependencyManifestParseFailed => "dependency package manifest is malformed",
        C::DependencyManifestValidationFailed => "dependency package manifest is invalid",
        C::DependencyPackageRootResolutionFailed => "dependency package root cannot be resolved",
        C::DependencyModuleRootResolutionFailed => {
            "dependency package module_root cannot be resolved"
        }
        C::DependencyPackageNameMismatch => "dependency package name does not match",
        C::DependencyManifestFingerprintMismatch => {
            "dependency manifest fingerprint does not match"
        }
        C::DependencyContentFingerprintMismatch => "dependency content fingerprint does not match",
        C::UnsupportedModuleExtension => "imported module has an unsupported extension",
        C::RelativeImportOutsideModuleRoot => "relative import escapes the package module_root",
        C::DependencyImportOutsideModuleRoot => {
            "dependency import escapes the dependency module_root"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_display_is_checkout_independent() {
        let a = relative_display(Path::new("/x/proj/src"), Path::new("/x/proj/src/util/h.sm"));
        let b = relative_display(
            Path::new("/deep/other/checkout/proj/src"),
            Path::new("/deep/other/checkout/proj/src/util/h.sm"),
        );
        assert_eq!(a, "util/h.sm");
        assert_eq!(a, b);
        assert_eq!(
            relative_display(Path::new("/x/proj/src"), Path::new("/x/dep/src/m.sm")),
            "../../dep/src/m.sm"
        );
    }

    #[test]
    fn relative_display_is_injective_for_distinct_paths() {
        let anchor = Path::new("/x/p");
        let one = relative_display(anchor, Path::new("/x/p/a/helper.sm"));
        let two = relative_display(anchor, Path::new("/x/p/b/helper.sm"));
        let up = relative_display(anchor, Path::new("/x/helper.sm"));
        assert_ne!(one, two);
        assert_ne!(one, up);
        assert_eq!(up, "../helper.sm");
    }

    #[test]
    fn canonical_io_text_is_error_kind_only() {
        let overlay = SourceOverlay::new();
        let access = CanonicalSources::new(&overlay, Path::new("/nonexistent-anchor"));
        let error = access
            .read_source(Path::new("/nonexistent-anchor/missing.sm"))
            .unwrap_err();
        assert_eq!(error, "failed to read 'missing.sm': entity not found");
    }
}
