use sm_ir::hello_ir::HelloIrModule;
use sm_ir::hello_semcode::{
    emit_hello_conceptual_semcode, render_hello_conceptual_semcode, HelloConceptualSemCode,
};
use sm_ir::{
    compile_program_to_ir, compile_program_to_semcode,
    compile_program_to_semcode_with_options_debug, emit_ir_to_semcode, validate_ir, AccessPath,
    CompilePipelineError, CompileProfile, IrError, IrFunction, IrInstr, OptLevel,
    OwnershipPathEvent, OwnershipPathEventKind,
};

// Compile-time guard: any regression of these signatures back to
// `FrontendError` fails to build, independent of source formatting.
#[test]
fn compile_contract_signatures() {
    let _: fn(&IrFunction) -> Result<(), IrError> = validate_ir;
    let _: fn(&[IrFunction], bool) -> Result<Vec<u8>, IrError> = emit_ir_to_semcode;
    let _: fn(&str) -> Result<Vec<u8>, CompilePipelineError> = compile_program_to_semcode;
    let _: fn(&str) -> Result<Vec<IrFunction>, CompilePipelineError> = compile_program_to_ir;
    let _: fn(&HelloIrModule) -> Result<HelloConceptualSemCode, IrError> =
        emit_hello_conceptual_semcode;
    let _: fn(&HelloIrModule) -> Result<Vec<String>, IrError> = render_hello_conceptual_semcode;
}

// Facade coherence: every facade that re-exports a migrated function also
// names the error type it returns, so consumers need no direct sm_ir import.
#[test]
fn facades_name_the_error_types_their_functions_return() {
    use semantic_language::frontend;

    let _: fn(&str) -> Result<Vec<u8>, sm_emit::CompilePipelineError> =
        sm_emit::compile_program_to_semcode;
    let _: fn(&[IrFunction], bool) -> Result<Vec<u8>, sm_emit::IrError> =
        sm_emit::emit_ir_to_semcode;
    let _: fn(&str) -> Result<Vec<u8>, frontend::CompilePipelineError> =
        frontend::compile_program_to_semcode;
    let _: fn(&str) -> Result<Vec<IrFunction>, frontend::ir::CompilePipelineError> =
        frontend::ir::compile_program_to_ir;
    let _: fn(&IrFunction) -> Result<(), frontend::ir::IrError> = frontend::ir::validate_ir;
    let _: fn(&[IrFunction], bool) -> Result<Vec<u8>, frontend::emit::IrError> =
        frontend::emit::emit_ir_to_semcode;
    let _ = |e: frontend::CompilePipelineError| match e {
        frontend::CompilePipelineError::Frontend(_) => (),
        frontend::CompilePipelineError::InternalIr(frontend::IrError { .. }) => (),
        frontend::CompilePipelineError::Configuration(frontend::ConfigurationError { .. }) => (),
    };
}

#[test]
fn test_a_genuine_frontend_survives_mixed_pipeline() {
    let src = "fn main() {\n    let x: i32 = true;\n    return;\n}\n";
    let err = compile_program_to_semcode(src).expect_err("type error must reject");
    match err {
        CompilePipelineError::Frontend(fe) => {
            assert!(
                fe.message.contains("type mismatch"),
                "expected type mismatch message, got: {}",
                fe.message
            );
        }
        other => panic!("expected CompilePipelineError::Frontend, got {:?}", other),
    }
}

#[test]
fn test_b_configuration_is_not_frontend() {
    let err = compile_program_to_semcode_with_options_debug(
        "fn main() {\n    return;\n}\n",
        CompileProfile::Logos,
        OptLevel::O0,
        false,
    )
    .expect_err("explicit Logos profile must reject for SemCode function IR");
    match err {
        CompilePipelineError::Configuration(ce) => {
            assert!(
                ce.message.contains(
                    "Logos input lowers to LogosIrLaw stream; SemCode function IR requires RustLike frontend"
                ),
                "expected configuration error, got: {}",
                ce.message
            );
        }
        CompilePipelineError::Frontend(fe) => {
            panic!("configuration failure must NOT be FrontendError: {:?}", fe);
        }
        other => panic!(
            "expected CompilePipelineError::Configuration, got {:?}",
            other
        ),
    }
}

#[test]
fn test_c_internal_ir_structural_failure() {
    let malformed_fn = IrFunction {
        name: "malformed_fn".to_string(),
        instrs: vec![],
        ownership_events: vec![],
        params: vec![],
    };
    let err =
        validate_ir(&malformed_fn).expect_err("empty function without RET must fail validation");
    assert!(
        err.message.contains("has no RET"),
        "expected missing RET error, got: {}",
        err.message
    );
}

#[test]
fn test_d_internal_emission_failure() {
    let bad_fn = IrFunction {
        name: "main".to_string(),
        instrs: vec![IrInstr::Ret { src: None }],
        ownership_events: vec![OwnershipPathEvent {
            kind: OwnershipPathEventKind::Write,
            path: AccessPath::new("x".to_string()),
            activation_site: None,
            write_site: None,
        }],
        params: vec![],
    };
    let err = emit_ir_to_semcode(&[bad_fn], false)
        .expect_err("write event without write_site must fail emission");
    assert!(
        err.message.contains("Write event has no WriteSiteId"),
        "expected emission error, got: {}",
        err.message
    );
}

#[test]
fn test_e_ir_error_envelopes_as_internal_ir() {
    // Envelope conversion only: this does not run the optimizer. The
    // production `run_default_opt_passes` mapping is covered in-crate by
    // `optimizer_failure_surfaces_as_internal_ir` in legacy_lowering.rs.
    let opt_err = sm_ir::passes::OptError("optimization invariant violated".to_string());
    let ir_err = IrError { message: opt_err.0 };
    let pipeline_err: CompilePipelineError = ir_err.into();
    match pipeline_err {
        CompilePipelineError::InternalIr(ie) => {
            assert_eq!(ie.message, "optimization invariant violated");
        }
        other => panic!("expected CompilePipelineError::InternalIr, got {:?}", other),
    }
}

#[test]
fn test_f_frozen_frontend_lowering_class() {
    let src = "fn foo<T>(x: T) -> T {\n    return x;\n}\nfn main() {\n    return;\n}\n";
    let err = compile_program_to_semcode(src)
        .expect_err("uninstantiated generic must reject at lowering");
    match err {
        CompilePipelineError::Frontend(fe) => {
            assert!(
                fe.message.contains(
                    "generic function 'foo' is admitted by the frontend but is not executable in the current IR contract because concrete IR monomorphisation is not implemented"
                ),
                "expected generic rejection message, got: {}",
                fe.message
            );
        }
        other => panic!("expected CompilePipelineError::Frontend, got {:?}", other),
    }
}

// Every configuration rejection message constructed in legacy_lowering.rs.
// Feature-disabled paths cannot be exercised at runtime with default
// features, so these sites are guarded at the source level instead.
const CONFIGURATION_MESSAGES: &[&str] = &[
    "RustLike profile is disabled at compile time (enable feature 'profile-rust')",
    "Logos profile is disabled at compile time (enable feature 'profile-logos')",
    "Logos input lowers to LogosIrLaw stream; SemCode function IR requires RustLike frontend",
    "Logos input detected, but Logos profile is disabled at compile time",
    "RustLike lowering is disabled at compile time",
    "debug symbols are disabled at compile time (enable feature 'debug-symbols')",
];

/// Type names of every struct literal whose `message:` field is exactly
/// `message`, e.g. `ConfigurationError` for
/// `ConfigurationError { message: "..." }` (formatting-insensitive, earlier
/// fields such as `pos: 0,` allowed). Other occurrences of the text, such as
/// test assertions or helper functions, are ignored.
fn constructor_types_for_message(source: &str, message: &str) -> Vec<String> {
    let literal = format!("\"{message}\"");
    source
        .match_indices(&literal)
        .filter_map(|(idx, _)| {
            let before = &source[..idx];
            let brace = before.rfind('{')?;
            if !before[brace + 1..].trim_end().ends_with("message:") {
                return None;
            }
            let head = before[..brace].trim_end();
            let start = head
                .rfind(|c: char| !(c.is_alphanumeric() || c == '_'))
                .map_or(0, |i| i + 1);
            Some(head[start..].to_string())
        })
        .collect()
}

#[test]
fn configuration_guard_detects_the_pre_migration_frontend_shape() {
    // The exact pre-P0B shape (base 1a25ce68) must be caught, or the guard
    // below would be vacuous.
    let pre_migration = "return Err(FrontendError {\n                pos: 0,\n                message:\n                    \"RustLike profile is disabled at compile time (enable feature 'profile-rust')\"\n                        .to_string(),\n            });";
    assert_eq!(
        constructor_types_for_message(pre_migration, CONFIGURATION_MESSAGES[0]),
        vec!["FrontendError".to_string()]
    );
}

#[test]
fn configuration_sites_construct_configuration_error() {
    let lowering_source = std::fs::read_to_string("crates/sm-ir/src/legacy_lowering.rs")
        .expect("read legacy_lowering.rs");
    for message in CONFIGURATION_MESSAGES {
        let types = constructor_types_for_message(&lowering_source, message);
        assert!(
            !types.is_empty(),
            "no constructor site found for {message:?}; the guard would be vacuous"
        );
        assert!(
            types.iter().all(|t| t == "ConfigurationError"),
            "{message:?} must be constructed as ConfigurationError, found {types:?}"
        );
    }
}
