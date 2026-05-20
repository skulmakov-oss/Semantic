#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
mod app;
#[cfg(feature = "std")]
mod api_contract;
#[cfg(feature = "std")]
mod config;
#[cfg(feature = "std")]
mod executable_bundle;
#[cfg(feature = "std")]
mod formatter;
#[cfg(feature = "std")]
mod incremental;
#[cfg(feature = "std")]
mod package_manifest;
#[cfg(feature = "std")]
mod schema_versioning;
#[cfg(feature = "std")]
mod wire_contract;

#[cfg(feature = "std")]
use sm_emit::compile_program_to_semcode_with_options_debug;
#[cfg(feature = "std")]
use sm_ir::{compile_program_to_ir_with_options, CompileProfile, OptLevel};
#[cfg(feature = "std")]
use sm_sema::{check_file_with_provider, check_source, ModuleProvider, SemanticReport};
#[cfg(feature = "std")]
use std::path::Path;
#[cfg(feature = "std")]
use ton618_core::diagnostics::diagnostic_catalog;

#[cfg(feature = "std")]
pub struct CliPipeline;

#[cfg(feature = "std")]
pub use app::{main_entry, run};
#[cfg(feature = "std")]
pub use api_contract::{build_generated_api_contract, format_generated_api_contract, GeneratedApiContractArtifact, GeneratedApiContractBuildError, GeneratedApiField, GeneratedApiSchema, GeneratedApiSchemaRole, GeneratedApiSchemaShape, GeneratedApiVariant, GENERATED_API_CONTRACT_FORMAT_VERSION, GENERATED_API_CONTRACT_GENERATOR, GENERATED_API_CONTRACT_GENERATOR_VERSION};
#[cfg(feature = "std")]
pub use config::{build_config_contract, parse_config_document, validate_config_document, ConfigContract, ConfigContractBuildError, ConfigDocument, ConfigEntry, ConfigNumber, ConfigNumberKind, ConfigParseError, ConfigValidationDiagnostic, ConfigValidationError, ConfigValue};
#[cfg(feature = "std")]
pub use formatter::{format_path, format_source_text, FormatterMode, FormatterSummary};
#[cfg(feature = "std")]
pub use package_manifest::{admit_package_entry_module, parse_package_manifest_baseline, resolve_package_import_path, validate_package_manifest_baseline, PackageDependency, PackageDependencySource, PackageIdentity, PackageImportResolutionCode, PackageImportResolutionError, PackageManifest, PackageManifestParseCode, PackageManifestParseError, PackageManifestValidationCode, PackageManifestValidationError, PackageModuleAdmission, PackageModuleAdmissionCode, PackageModuleAdmissionError, PackageRoot, PACKAGE_IMPORT_SEPARATOR, PACKAGE_MANIFEST_BASELINE_VERSION, PACKAGE_MANIFEST_FILE_NAME};
#[cfg(feature = "std")]
pub use schema_versioning::{build_schema_migration_metadata, classify_record_schema_compatibility, classify_tagged_union_schema_compatibility, format_schema_migration_metadata, RecordSchemaCompatibilityReport, SchemaCompatibilityBuildError, SchemaCompatibilityKind, SchemaFieldChange, SchemaFieldChangeKind, SchemaMigrationChangeSet, SchemaMigrationMetadataArtifact, SchemaMigrationReviewKind, SchemaMigrationShapeKind, SchemaVariantChangeKind, TaggedUnionSchemaCompatibilityReport, TaggedUnionSchemaVariantChange};
#[cfg(feature = "std")]
pub use wire_contract::{build_generated_wire_contract, format_generated_wire_contract, GeneratedWireContractArtifact, GeneratedWireContractBuildError, TaggedWireUnionContract, TaggedWireUnionField, TaggedWireUnionVariant, WirePatchField, WirePatchTypeContract, GENERATED_WIRE_CONTRACT_FORMAT_VERSION, GENERATED_WIRE_CONTRACT_GENERATOR, GENERATED_WIRE_CONTRACT_GENERATOR_VERSION};

#[cfg(feature = "std")]
struct CliFsProvider;

#[cfg(feature = "std")]
impl ModuleProvider for CliFsProvider {
    fn read_module(&self, module_id: &str) -> Result<Vec<u8>, String> {
        package_manifest::admit_package_entry_module(Path::new(module_id))
            .map(|_| ())
            .map_err(|e| e.to_string())?;
        std::fs::read(module_id).map_err(|e| e.to_string())
    }

    fn resolve_import(&self, importer_module_id: &str, spec: &str) -> Result<String, String> {
        package_manifest::resolve_package_import_path(Path::new(importer_module_id), spec)
            .map(|path| path.to_string_lossy().replace('\\', "/"))
            .map_err(|e| e.to_string())
    }
}

#[cfg(feature = "std")]
impl CliPipeline {
    pub fn compile_source(
        src: &str,
        profile: CompileProfile,
        opt: OptLevel,
        debug_symbols: bool,
    ) -> Result<Vec<u8>, String> {
        compile_program_to_semcode_with_options_debug(src, profile, opt, debug_symbols)
            .map_err(|e| e.to_string())
    }

    pub fn build_ir(
        src: &str,
        profile: CompileProfile,
        opt: OptLevel,
    ) -> Result<Vec<sm_ir::IrFunction>, String> {
        compile_program_to_ir_with_options(src, profile, opt).map_err(|e| e.to_string())
    }

    pub fn semantic_check_source(src: &str) -> Result<SemanticReport, String> {
        check_source(src).map_err(|e| e.to_string())
    }

    pub fn semantic_check_file(path: &Path) -> Result<SemanticReport, String> {
        let provider = CliFsProvider;
        let root = path
            .canonicalize()
            .map_err(|e| format!("failed to resolve '{}': {}", path.display(), e))?;
        check_file_with_provider(&root, &provider).map_err(|e| e.to_string())
    }

    pub fn explain(code: &str) -> Option<&'static str> {
        let upper = code.trim().to_ascii_uppercase();
        diagnostic_catalog()
            .iter()
            .find(|(c, _)| *c == upper)
            .map(|(_, msg)| *msg)
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn mk_temp_dir(prefix: &str) -> std::path::PathBuf {
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
    fn compile_pipeline_smoke() {
        let src = "fn main() { return; }";
        let bytes = CliPipeline::compile_source(src, CompileProfile::RustLike, OptLevel::O0, false)
            .expect("compile");
        assert_eq!(&bytes[0..8], b"SEMCODE0");
    }

    #[test]
    fn semantic_pipeline_smoke() {
        let src = r#"
Law "L" [priority 1]:
    When true ->
        System.recovery()
"#;
        let rep = CliPipeline::semantic_check_source(src).expect("check");
        assert_eq!(rep.scheduled_laws.len(), 1);
    }

    #[test]
    fn explain_smoke() {
        let text = CliPipeline::explain("E0101").expect("known code");
        assert!(text.contains("indent"));
    }

    #[test]
    fn semantic_check_file_admits_entry_within_package_module_root() {
        let dir = mk_temp_dir("smc_cli_pkg_admit_ok");
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
        let entry = src_dir.join("main.sm");
        std::fs::write(
            &entry,
            r#"
Law "L" [priority 1]:
    When true ->
        System.recovery()
"#,
        )
        .expect("write source");

        let report = CliPipeline::semantic_check_file(&entry).expect("check");
        assert_eq!(report.scheduled_laws.len(), 1);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn semantic_check_file_resolves_local_path_package_dependency() {
        let dir = mk_temp_dir("smc_cli_pkg_dep_ok");
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
        let entry = app_src.join("main.sm");
        std::fs::write(
            &entry,
            r#"
Import "math::core.sm"
Law "App" [priority 1]:
    When true ->
        System.recovery()
"#,
        )
        .expect("write app source");
        std::fs::write(
            math_src.join("core.sm"),
            r#"
Law "Core" [priority 1]:
    When true ->
        System.recovery()
"#,
        )
        .expect("write dep source");

        let report = CliPipeline::semantic_check_file(&entry).expect("check");
        assert_eq!(report.scheduled_laws.len(), 2);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
