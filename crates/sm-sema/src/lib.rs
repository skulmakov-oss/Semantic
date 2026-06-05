#![allow(clippy::len_without_is_empty, clippy::unnecessary_sort_by)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(any(feature = "alloc", feature = "std"))]
extern crate alloc;

#[cfg(any(feature = "alloc", feature = "std"))]
pub mod alloc_core;
#[cfg(feature = "std")]
pub use alloc_core::ModuleProvider;
#[cfg(all(feature = "alloc", not(feature = "std")))]
pub use alloc_core::*;
#[cfg(any(feature = "alloc", feature = "std"))]
pub use alloc_core::{
    build_export_sets_core, collect_local_exports_core, default_import_alias, diagnostic_help_core,
    evaluate_law_header_policy_core, fold_fx_const_call_core, has_magic_number_core,
    infer_atom_type_core, infer_law_entity_core, infer_when_condition_type_core, insert_name_core,
    insert_scoped_name_core, is_assignment_compatible, is_compatible_cmp, is_dead_when_condition,
    is_large_law_core, is_law_name_style_ok, is_valid_when_result_type_core,
    parse_import_directives, parse_law_local_decl, parse_select_items,
    track_entity_field_usage_core, validate_import_bindings_core, validate_import_namespace_rules,
    validate_select_imports_core, validate_when_non_empty_core, ConditionInferError,
    ExportBuildError, ExportBuildModule, ExportItem, ExportKind, ExportOrigin, ExportSet,
    GateInstr, ImmutableIr, ImportDirective, ImportPolicyError, LawHeaderPolicy, LawScheduler,
    LocalExportDecl, ScopeKind, SelectImportModule, SelectImportPolicyError, SemanticType, Symbol,
    SymbolError, SymbolTable, TypeId, TypeRegistry, WhenValidationError,
};

#[cfg(feature = "std")]
mod frontend {
    #[cfg(test)]
    pub use sm_front::parse_logos_program;
    pub use sm_front::{
        parse_logos_program_with_profile, parse_program_with_profile, type_check_program,
        LogosEntity, LogosEntityFieldKind, LogosProgram, Type,
    };
    pub use sm_profile::ParserProfile;
    pub use ton618_core::SourceMark;
}

#[cfg(feature = "std")]
mod std_adapters;

#[cfg(feature = "std")]
pub use std_adapters::*;

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    #[test]
    fn sema_accepts_fx_passthrough_surface() {
        let src = r#"
            fn id(x: fx) -> fx {
                let y: fx = x;
                return y;
            }

            fn main() {
                return;
            }
        "#;

        check_source(src).expect("fx passthrough surface should analyze");
    }

    #[test]
    fn sema_accepts_fx_literal_surface() {
        let src = r#"
            fn id(x: fx) -> fx {
                return x;
            }

            fn value() -> fx {
                return -1.25;
            }

            fn main() {
                let x: fx = 1.0;
                let y: fx = id(2);
                let z: fx = value();
                let same = x == x;
                let also_same = y == z;
                if same == also_same { return; } else { return; }
            }
        "#;

        check_source(src).expect("fx literal/call/return surface should analyze");
    }
}
