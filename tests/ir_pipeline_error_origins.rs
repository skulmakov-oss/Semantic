use sm_ir::{
    compile_program_to_ir, compile_program_to_semcode,
    compile_program_to_semcode_with_options_debug, emit_ir_to_semcode, validate_ir, AccessPath,
    CompilePipelineError, CompileProfile, IrError, IrFunction, IrInstr, OptLevel,
    OwnershipPathEvent, OwnershipPathEventKind,
};

#[test]
fn compile_contract_signatures() {
    let _: fn(&IrFunction) -> Result<(), IrError> = validate_ir;
    let _: fn(&[IrFunction], bool) -> Result<Vec<u8>, IrError> = emit_ir_to_semcode;
    let _: fn(&str) -> Result<Vec<u8>, CompilePipelineError> = compile_program_to_semcode;
    let _: fn(&str) -> Result<Vec<IrFunction>, CompilePipelineError> = compile_program_to_ir;
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
fn test_e_optimizer_origin_mapping() {
    // sm_ir::OptError maps to IrError and then to CompilePipelineError::InternalIr.
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

#[test]
fn test_source_level_negative_guard_migrated_sites_do_not_fabricate_frontend_error() {
    let lowering_source = std::fs::read_to_string("crates/sm-ir/src/legacy_lowering.rs")
        .expect("read legacy_lowering.rs");
    let hello_source = std::fs::read_to_string("crates/sm-ir/src/hello_semcode.rs")
        .expect("read hello_semcode.rs");

    // Pure IR APIs must not return Result<_, FrontendError>
    assert!(
        !lowering_source
            .contains("pub fn validate_ir(func: &IrFunction) -> Result<(), FrontendError>"),
        "validate_ir must not return FrontendError"
    );
    assert!(
        !lowering_source.contains("pub fn emit_ir_to_semcode(funcs: &[IrFunction], debug_symbols: bool) -> Result<Vec<u8>, FrontendError>"),
        "emit_ir_to_semcode must not return FrontendError"
    );
    assert!(
        !hello_source.contains("pub fn emit_hello_conceptual_semcode(module: &HelloIrModule) -> Result<Vec<u8>, FrontendError>"),
        "emit_hello_conceptual_semcode must not return FrontendError"
    );
    assert!(
        !hello_source.contains("pub fn render_hello_conceptual_semcode(module: &HelloIrModule) -> Result<String, FrontendError>"),
        "render_hello_conceptual_semcode must not return FrontendError"
    );

    // Configuration sites must not construct FrontendError
    assert!(
        !lowering_source.contains("\"RustLike profile is disabled at compile time (enable feature 'profile-rust')\"\n                        .to_string(),\n                });"),
        "profile-rust configuration failure must not be FrontendError"
    );
    assert!(
        !lowering_source.contains("\"Logos input lowers to LogosIrLaw stream; SemCode function IR requires RustLike frontend\"\n                        .to_string(),\n                });"),
        "Logos redirect configuration failure must not be FrontendError"
    );
}
