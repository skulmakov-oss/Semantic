//! S15-C2: executable qualification of the atomic strict execution boundary
//! (`sm_vm::run_host_math_free_*`) for `HOST_MATH_FREE@draft1`.
//!
//! Every case runs a real verifier-admitted artifact through the strict API and,
//! where it matters, through the unchanged LOCAL sibling on the same entry token.
//! Test names carry the S15-C2 matrix id.

use std::collections::BTreeSet;
use std::marker::PhantomData;

use sm_format::semcode_format::{CallableValueFamily, Opcode, MAGIC1};
use sm_ir::{
    compile_program_to_semcode_with_options_debug, emit_ir_to_semcode, CompileProfile, IrFunction,
    IrInstr, OptLevel,
};
use sm_runtime_core::{ExecutionConfig, ExecutionContext, QuotaKind, RuntimeTrap};
use sm_verify::{
    analyze_host_math_free_draft1, verify_semcode_token, HostMathAnalysis, HostMathAnalysisLimits,
    HostMathBuiltin, HostMathIndeterminate, HostMathVerdict, VerifiedEntrySemCode,
    HOST_MATH_FREE_DRAFT1_PROFILE,
};
use sm_vm::{
    run_host_math_free_entry_semcode_with_config,
    run_host_math_free_function_semcode_with_args_and_config,
    run_verified_entry_semcode_with_config, run_verified_function_semcode_with_args_and_config,
    ClosureValue, HostMathFreeGrant, QuadVal, RuntimeError, StrictExecutionOutcome, StrictRunError,
    Value,
};

use HostMathBuiltin::{Abs, Cos, Pow, Sin, Sqrt, Tan};

// ---------------------------------------------------------------- helpers

fn compile(src: &str, opt: OptLevel, debug: bool) -> Vec<u8> {
    compile_program_to_semcode_with_options_debug(src, CompileProfile::RustLike, opt, debug)
        .unwrap_or_else(|err| panic!("source compiles: {err:?}\n{src}"))
}

fn o0(src: &str) -> Vec<u8> {
    compile(src, OptLevel::O0, false)
}

fn ir_fn(name: &str, params: Vec<CallableValueFamily>, instrs: Vec<IrInstr>) -> IrFunction {
    IrFunction {
        name: name.to_string(),
        instrs,
        ownership_events: Vec::new(),
        params,
    }
}

fn ret() -> IrInstr {
    IrInstr::Ret { src: None }
}

fn emit(functions: &[IrFunction]) -> Vec<u8> {
    emit_ir_to_semcode(functions, false).expect("IR emits")
}

fn cfg() -> ExecutionConfig {
    ExecutionConfig::for_context(ExecutionContext::VerifiedLocal)
}

fn default_limits() -> HostMathAnalysisLimits {
    HostMathAnalysisLimits::draft1_default()
}

fn grant() -> HostMathFreeGrant {
    HostMathFreeGrant::draft1()
}

/// Strict P16 and LOCAL P16 on the same entry token.
fn run_entry_with(
    bytes: &[u8],
    entry: &str,
    limits: HostMathAnalysisLimits,
    config: ExecutionConfig,
) -> (StrictExecutionOutcome<()>, Result<(), RuntimeError>) {
    let token = verify_semcode_token(bytes).expect("artifact is verifier-admitted");
    let entry = token.require_entry(entry).expect("entry exists");
    let strict = run_host_math_free_entry_semcode_with_config(&entry, &grant(), &limits, config);
    let local = run_verified_entry_semcode_with_config(&entry, config);
    (strict, local)
}

fn run_entry(bytes: &[u8], entry: &str) -> (StrictExecutionOutcome<()>, Result<(), RuntimeError>) {
    run_entry_with(bytes, entry, default_limits(), cfg())
}

/// Strict P21 and LOCAL P21 on the same entry token with the same arguments.
fn run_fn_with(
    bytes: &[u8],
    entry: &str,
    args: Vec<Value>,
    limits: HostMathAnalysisLimits,
    config: ExecutionConfig,
) -> (StrictExecutionOutcome<Value>, Result<Value, RuntimeError>) {
    let token = verify_semcode_token(bytes).expect("artifact is verifier-admitted");
    let entry = token.require_entry(entry).expect("entry exists");
    let strict = run_host_math_free_function_semcode_with_args_and_config(
        &entry,
        &grant(),
        &limits,
        args.clone(),
        config,
    );
    let local = run_verified_function_semcode_with_args_and_config(&entry, args, config);
    (strict, local)
}

fn run_fn(
    bytes: &[u8],
    entry: &str,
    args: Vec<Value>,
) -> (StrictExecutionOutcome<Value>, Result<Value, RuntimeError>) {
    run_fn_with(bytes, entry, args, default_limits(), cfg())
}

fn standalone_analysis(
    bytes: &[u8],
    entry: &str,
    limits: HostMathAnalysisLimits,
) -> HostMathAnalysis {
    let token = verify_semcode_token(bytes).expect("artifact is verifier-admitted");
    let entry = token.require_entry(entry).expect("entry exists");
    analyze_host_math_free_draft1(&entry, &limits)
}

/// Admitted: the strict result is exactly the LOCAL result on the same entry.
fn assert_admitted_as_local<T: PartialEq + std::fmt::Debug + Clone>(
    strict: &StrictExecutionOutcome<T>,
    local: &Result<T, RuntimeError>,
) {
    assert_eq!(
        strict.analysis().verdict(),
        HostMathVerdict::Admit,
        "{strict:?}"
    );
    assert!(strict.execution_attempted(), "{strict:?}");
    let expected = local.clone().map_err(StrictRunError::Runtime);
    assert_eq!(
        strict.result(),
        &expected,
        "strict must not alter LOCAL semantics"
    );
}

/// Refused: no execution, typed refusal, never a runtime error.
fn assert_refused<T: std::fmt::Debug>(strict: &StrictExecutionOutcome<T>) {
    assert_eq!(
        strict.analysis().verdict(),
        HostMathVerdict::Refuse,
        "{strict:?}"
    );
    assert!(
        matches!(strict.result(), Err(StrictRunError::AdmissionRefused)),
        "{strict:?}"
    );
    assert!(!strict.execution_attempted(), "{strict:?}");
}

fn assert_refused_may<T: std::fmt::Debug>(
    strict: &StrictExecutionOutcome<T>,
    may: &[HostMathBuiltin],
) {
    assert_refused(strict);
    assert_eq!(strict.analysis().may(), may, "{strict:?}");
    assert!(strict.analysis().indeterminate().is_empty(), "{strict:?}");
}

fn assert_refused_indet<T: std::fmt::Debug>(
    strict: &StrictExecutionOutcome<T>,
    indet: &[HostMathIndeterminate],
) {
    assert_refused(strict);
    assert_eq!(strict.analysis().indeterminate(), indet, "{strict:?}");
}

type Math = (
    HostMathVerdict,
    Vec<HostMathBuiltin>,
    Vec<HostMathIndeterminate>,
    u64,
);

fn math(a: &HostMathAnalysis) -> Math {
    (
        a.verdict(),
        a.may().to_vec(),
        a.indeterminate().to_vec(),
        a.work_units_used(),
    )
}

/// `has_default!(T)` is `true` iff `T: Default` (inherent method beats the
/// autoref trait fallback only when its bound holds).
struct DefaultProbe<T>(PhantomData<T>);
trait NoDefault {
    fn has_default(&self) -> bool {
        false
    }
}
impl<T> NoDefault for &DefaultProbe<T> {}
impl<T: Default> DefaultProbe<T> {
    fn has_default(&self) -> bool {
        true
    }
}
macro_rules! has_default {
    ($t:ty) => {
        (&DefaultProbe::<$t>(PhantomData)).has_default()
    };
}

// ---------------------------------------------------------------- sources

const CLEAN: &str = "fn main() {\n    let a: i32 = 2;\n    let b: i32 = a * 3;\n    return;\n}\n";
const SAFE_INT: &str =
    "fn mix(a: i32, b: i32) -> i32 {\n    return a * b + 3;\n}\nfn main() {\n    return;\n}\n";
const QUAD: &str = "fn decide(x: quad) -> quad {\n    let y: quad = x && T;\n    return y;\n}\nfn main() {\n    return;\n}\n";
const SHADOW_SAFE: &str = "fn sqrt(x: f64) -> f64 {\n    return x + 100.0;\n}\nfn root(v: f64) -> f64 {\n    return sqrt(v);\n}\nfn main() {\n    let y: f64 = root(16.0);\n    return;\n}\n";
const UNREACHABLE_FN: &str =
    "fn unused() {\n    let y: f64 = sqrt(2.0);\n    return;\n}\nfn main() {\n    return;\n}\n";
const RECURSIVE: &str = "fn count(n: i32) -> i32 {\n    if n == 0 {\n        return 0;\n    }\n    return 1 + count(n - 1);\n}\nfn main() {\n    let r: i32 = count(3);\n    return;\n}\n";
const ENTRIES: &str = "fn clean() {\n    let a: i32 = 4;\n    return;\n}\nfn hot() {\n    let y: f64 = sqrt(2.0);\n    return;\n}\nfn main() {\n    let y: f64 = cos(0.0);\n    return;\n}\n";

const DIRECT_SQRT: &str = "fn main() {\n    let y: f64 = sqrt(16.0);\n    return;\n}\n";
const DIRECT_ABS: &str = "fn main() {\n    let y: f64 = abs(-2.5);\n    return;\n}\n";
const TAN_POW: &str =
    "fn main() {\n    let a: f64 = tan(1.0);\n    let b: f64 = pow(a, 2.0);\n    return;\n}\n";
const VIA_HELPER: &str = "fn helper(x: f64) -> f64 {\n    return cos(x);\n}\nfn main() {\n    let y: f64 = helper(1.0);\n    return;\n}\n";
const SHADOW_HAZARD: &str = "fn sqrt(x: f64) -> f64 {\n    return tan(x);\n}\nfn main() {\n    let y: f64 = sqrt(16.0);\n    return;\n}\n";
const CLOSURE_HAZARD: &str = "fn main() {\n    let f: Closure(f64 -> f64) = (x => sin(x));\n    let y: f64 = f(1.0);\n    return;\n}\n";
const HAZARD_FN: &str =
    "fn root(v: f64) -> f64 {\n    return sqrt(v);\n}\nfn main() {\n    return;\n}\n";
const SAFE_FN: &str =
    "fn twice(v: f64) -> f64 {\n    return v + v;\n}\nfn main() {\n    return;\n}\n";

/// Each of these would fail or be charged at run time before its hazard.
const TRAP_BEFORE: &str = "fn main() {\n    let z: i32 = 0;\n    let q: i32 = 7 / z;\n    let y: f64 = sqrt(2.0);\n    return;\n}\n";
const ASSERT_BEFORE: &str =
    "fn main() {\n    assert(1 == 2);\n    let y: f64 = sqrt(2.0);\n    return;\n}\n";

/// `main: Ret; Call sqrt; Ret` - a prohibited call after an unconditional return.
fn after_return() -> Vec<u8> {
    emit(&[ir_fn(
        "main",
        Vec::new(),
        vec![
            IrInstr::LoadF64 { dst: 0, val: 2.0 },
            ret(),
            IrInstr::Call {
                dst: None,
                name: "sqrt".to_string(),
                args: vec![0],
            },
            ret(),
        ],
    )])
}

/// `main: ClosureCall` with no `MakeClosure` anywhere.
fn closure_call_without_origin() -> Vec<u8> {
    emit(&[ir_fn(
        "main",
        Vec::new(),
        vec![
            IrInstr::LoadF64 { dst: 0, val: 2.0 },
            IrInstr::ClosureCall {
                dst: None,
                closure: 0,
                arg: 0,
            },
            ret(),
        ],
    )])
}

fn entry_with_params(params: Vec<CallableValueFamily>) -> Vec<u8> {
    emit(&[ir_fn("main", params, vec![ret()])])
}

/// A rev-2 artifact without canonical signatures (SIG0 starts at rev 20).
fn legacy_without_signature() -> Vec<u8> {
    let code = [0u8, 0, Opcode::Ret as u8, 0];
    let mut bytes = MAGIC1.to_vec();
    bytes.extend_from_slice(&4u16.to_le_bytes());
    bytes.extend_from_slice(b"main");
    bytes.extend_from_slice(&(code.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&code);
    bytes
}

// ------------------------------------------------------- 9.1 positive controls

#[test]
fn p01_clean_entry_executes_exactly_as_local() {
    let (strict, local) = run_entry(&o0(CLEAN), "main");
    assert_eq!(local, Ok(()));
    assert_admitted_as_local(&strict, &local);
}

#[test]
fn p02_safe_integer_arithmetic_returns_the_local_value() {
    let (strict, local) = run_fn(&o0(SAFE_INT), "mix", vec![Value::I32(6), Value::I32(7)]);
    assert_eq!(local, Ok(Value::I32(45)));
    assert_admitted_as_local(&strict, &local);
}

#[test]
fn p03_permitted_quad_operations_return_the_local_value() {
    let bytes = o0(QUAD);
    for q in [QuadVal::T, QuadVal::F, QuadVal::N, QuadVal::S] {
        let (strict, local) = run_fn(&bytes, "decide", vec![Value::Quad(q)]);
        assert!(local.is_ok(), "{local:?}");
        assert_admitted_as_local(&strict, &local);
    }
}

#[test]
fn p04_user_shadowed_sqrt_runs_the_user_function() {
    let bytes = o0(SHADOW_SAFE);
    let (strict, local) = run_fn(&bytes, "root", vec![Value::F64(16.0)]);
    assert_eq!(local, Ok(Value::F64(116.0)));
    assert_admitted_as_local(&strict, &local);
    let (strict, local) = run_entry(&bytes, "main");
    assert_admitted_as_local(&strict, &local);
}

#[test]
fn p05_prohibited_call_only_in_cfg_unreachable_code_executes() {
    let (strict, local) = run_entry(&after_return(), "main");
    assert_eq!(local, Ok(()));
    assert_admitted_as_local(&strict, &local);
}

#[test]
fn p06_prohibited_call_only_in_an_unreachable_function_executes() {
    let (strict, local) = run_entry(&o0(UNREACHABLE_FN), "main");
    assert_admitted_as_local(&strict, &local);
}

#[test]
fn p07_safe_recursion_returns_the_local_value() {
    let bytes = o0(RECURSIVE);
    let (strict, local) = run_fn(&bytes, "count", vec![Value::I32(5)]);
    assert_eq!(local, Ok(Value::I32(5)));
    assert_admitted_as_local(&strict, &local);
    let (strict, local) = run_entry(&bytes, "main");
    assert_admitted_as_local(&strict, &local);
}

#[test]
fn p08_distinct_valid_entries_are_analysed_and_run_independently() {
    let bytes = o0(SHADOW_SAFE);
    let (main, _) = run_entry(&bytes, "main");
    let (root, _) = run_fn(&bytes, "root", vec![Value::F64(1.0)]);
    let (shadow, _) = run_fn(&bytes, "sqrt", vec![Value::F64(1.0)]);
    assert_eq!(main.analysis().entry(), "main");
    assert_eq!(root.analysis().entry(), "root");
    assert_eq!(shadow.analysis().entry(), "sqrt");
    assert!(main.analysis().work_units_used() > root.analysis().work_units_used());
    assert!(root.analysis().work_units_used() > shadow.analysis().work_units_used());
    assert_eq!(shadow.result(), &Ok(Value::F64(101.0)));
}

// ------------------------------------------------------- 9.2 negative controls

#[test]
fn n01_reachable_sqrt_is_refused_where_local_runs_it() {
    let (strict, local) = run_entry(&o0(DIRECT_SQRT), "main");
    assert_eq!(local, Ok(()), "LOCAL stays LOCAL: it executes the builtin");
    assert_refused_may(&strict, &[Sqrt]);
}

#[test]
fn n02_reachable_abs_is_refused() {
    let (strict, _) = run_entry(&o0(DIRECT_ABS), "main");
    assert_refused_may(&strict, &[Abs]);
}

#[test]
fn n03_reachable_tan_and_pow_are_refused() {
    let (strict, _) = run_entry(&o0(TAN_POW), "main");
    assert_refused_may(&strict, &[Pow, Tan]);
}

#[test]
fn n04_hazard_reached_through_a_normal_call_is_refused() {
    let (strict, _) = run_entry(&o0(VIA_HELPER), "main");
    assert_refused_may(&strict, &[Cos]);
}

#[test]
fn n05_user_shadowed_function_reaching_a_real_builtin_is_refused() {
    let (strict, _) = run_entry(&o0(SHADOW_HAZARD), "main");
    assert_refused_may(&strict, &[Tan]);
}

#[test]
fn n06_hazardous_closure_target_is_refused() {
    let (strict, _) = run_entry(&o0(CLOSURE_HAZARD), "main");
    assert_refused_may(&strict, &[Sin]);
}

#[test]
fn n07_unknown_closure_origin_is_refused() {
    let (strict, _) = run_entry(&closure_call_without_origin(), "main");
    assert_refused_indet(&strict, &[HostMathIndeterminate::ClosureOriginUnknown]);
}

#[test]
fn n08_callable_carrying_entry_parameter_is_refused_regardless_of_the_value() {
    let bytes = entry_with_params(vec![CallableValueFamily::Closure]);
    let closure = Value::Closure(ClosureValue {
        function_name: "main".to_string(),
        captures: Vec::new(),
    });
    for args in [vec![closure], vec![Value::I32(1)], Vec::new()] {
        let (strict, _) = run_fn(&bytes, "main", args);
        assert_refused_indet(
            &strict,
            &[HostMathIndeterminate::CallableEntryParameter { index: 0 }],
        );
    }
}

#[test]
fn n09_missing_canonical_entry_signature_is_refused() {
    let (strict, local) = run_entry(&legacy_without_signature(), "main");
    assert_eq!(local, Ok(()));
    assert_refused_indet(&strict, &[HostMathIndeterminate::EntrySignatureUnavailable]);
}

#[test]
fn n10_analysis_resource_exhaustion_is_a_deterministic_refusal() {
    let bytes = o0(CLEAN);
    let needed = standalone_analysis(&bytes, "main", default_limits()).work_units_used();
    let tight = HostMathAnalysisLimits::new(needed).expect("valid");
    let short = HostMathAnalysisLimits::new(needed - 1).expect("valid");
    let (ok, local) = run_entry_with(&bytes, "main", tight, cfg());
    assert_admitted_as_local(&ok, &local);
    for _ in 0..2 {
        let (strict, local) = run_entry_with(&bytes, "main", short, cfg());
        assert_eq!(local, Ok(()), "the analysis budget is not a runtime quota");
        assert_refused_indet(&strict, &[HostMathIndeterminate::AnalysisLimit]);
        assert_eq!(strict.analysis().work_units_used(), needed - 1);
        // The P21 sibling honours the host's limits too.
        let (strict, local) = run_fn_with(&bytes, "main", Vec::new(), short, cfg());
        assert_eq!(local, Ok(Value::Unit));
        assert_refused_indet(&strict, &[HostMathIndeterminate::AnalysisLimit]);
    }
    let (ok, local) = run_fn_with(&bytes, "main", Vec::new(), tight, cfg());
    assert_admitted_as_local(&ok, &local);
}

#[test]
fn n11_strict_analysis_equals_the_c1_analysis_of_the_same_entry_on_both_siblings() {
    let clean = o0(CLEAN);
    let short = HostMathAnalysisLimits::new(
        standalone_analysis(&clean, "main", default_limits()).work_units_used() - 1,
    )
    .expect("valid");
    let cases: Vec<(Vec<u8>, &str, HostMathAnalysisLimits, HostMathVerdict)> = vec![
        (
            o0(DIRECT_SQRT),
            "main",
            default_limits(),
            HostMathVerdict::Refuse,
        ),
        (
            o0(TAN_POW),
            "main",
            default_limits(),
            HostMathVerdict::Refuse,
        ),
        (
            o0(SHADOW_HAZARD),
            "main",
            default_limits(),
            HostMathVerdict::Refuse,
        ),
        (
            o0(CLOSURE_HAZARD),
            "main",
            default_limits(),
            HostMathVerdict::Refuse,
        ),
        (
            closure_call_without_origin(),
            "main",
            default_limits(),
            HostMathVerdict::Refuse,
        ),
        (
            legacy_without_signature(),
            "main",
            default_limits(),
            HostMathVerdict::Refuse,
        ),
        (
            entry_with_params(vec![CallableValueFamily::Closure]),
            "main",
            default_limits(),
            HostMathVerdict::Refuse,
        ),
        (clean.clone(), "main", short, HostMathVerdict::Refuse),
        (clean, "main", default_limits(), HostMathVerdict::Admit),
        (
            o0(SHADOW_SAFE),
            "root",
            default_limits(),
            HostMathVerdict::Admit,
        ),
    ];
    for (bytes, entry, limits, verdict) in cases {
        let expected = standalone_analysis(&bytes, entry, limits);
        assert_eq!(expected.verdict(), verdict, "{entry}: {expected:?}");
        let (p16, _) = run_entry_with(&bytes, entry, limits, cfg());
        let (p21, _) = run_fn_with(&bytes, entry, Vec::new(), limits, cfg());
        assert_eq!(p16.analysis(), &expected, "P16 analyses exactly its entry");
        assert_eq!(p21.analysis(), &expected, "P21 analyses exactly its entry");
        let admitted = verdict == HostMathVerdict::Admit;
        assert_eq!(p16.execution_attempted(), admitted);
        assert_eq!(p21.execution_attempted(), admitted);
        if !admitted {
            assert_refused(&p16);
            assert_refused(&p21);
        }
    }
}

// ----------------------------------------------- 9.3 refusal before execution
//
// These tests observe precedence in the returned result: where LOCAL fails at
// run time before the hazard, strict returns the refusal instead. They cannot
// see an execution whose result is discarded (no host effect or observation is
// available on P16/P21); that no execution path runs on refusal is shown
// structurally by `semcode_vm::s15c2_strict_gate_tests` (mutant M02).

#[test]
fn x01_refusal_precedes_a_runtime_trap_before_the_hazard() {
    let (strict, local) = run_entry(&o0(TRAP_BEFORE), "main");
    assert_eq!(local, Err(RuntimeError::Trap(RuntimeTrap::DivisionByZero)));
    assert_refused_may(&strict, &[Sqrt]);
}

#[test]
fn x02_refusal_precedes_a_failing_assertion_before_the_hazard() {
    let (strict, local) = run_entry(&o0(ASSERT_BEFORE), "main");
    assert!(
        local.is_err(),
        "LOCAL executes the failing assert: {local:?}"
    );
    assert_refused_may(&strict, &[Sqrt]);
}

#[test]
fn x03_refusal_precedes_runtime_quota_errors_on_both_siblings() {
    let mut tight = cfg();
    tight.quotas.max_steps = 1;
    let (strict, local) = run_entry_with(&o0(DIRECT_SQRT), "main", default_limits(), tight);
    assert!(
        matches!(&local, Err(RuntimeError::QuotaExceeded(q)) if q.kind == QuotaKind::Steps),
        "LOCAL is charged steps: {local:?}"
    );
    assert_refused_may(&strict, &[Sqrt]);
    // The same config is passed through unchanged once admitted.
    let (strict, local) = run_entry_with(&o0(CLEAN), "main", default_limits(), tight);
    assert!(
        matches!(&local, Err(RuntimeError::QuotaExceeded(_))),
        "{local:?}"
    );
    assert_admitted_as_local(&strict, &local);
    // P21: refusal precedes the quota error; an admitted run gets the host's config.
    let hazard = run_fn_with(
        &o0(HAZARD_FN),
        "root",
        vec![Value::F64(4.0)],
        default_limits(),
        tight,
    );
    assert!(
        matches!(&hazard.1, Err(RuntimeError::QuotaExceeded(_))),
        "{:?}",
        hazard.1
    );
    assert_refused_may(&hazard.0, &[Sqrt]);
    let (strict, local) = run_fn_with(
        &o0(SAFE_FN),
        "twice",
        vec![Value::F64(4.0)],
        default_limits(),
        tight,
    );
    assert!(
        matches!(&local, Err(RuntimeError::QuotaExceeded(_))),
        "{local:?}"
    );
    assert_admitted_as_local(&strict, &local);
}

#[test]
fn x04_refusal_precedes_frame_admission_errors() {
    let mut no_frames = cfg();
    no_frames.quotas.max_frames = 0;
    let (strict, local) = run_entry_with(&o0(DIRECT_SQRT), "main", default_limits(), no_frames);
    assert!(
        local.is_err(),
        "LOCAL fails at entry-frame admission: {local:?}"
    );
    assert_refused_may(&strict, &[Sqrt]);
}

// ----------------------------------------------- 9.4 argument precedence (D-4)

#[test]
fn a01_hazardous_entry_with_wrong_family_arguments_is_refused_first() {
    let bytes = o0(HAZARD_FN);
    for args in [
        vec![Value::I32(4)],
        Vec::new(),
        vec![Value::F64(4.0), Value::F64(1.0)],
    ] {
        let (strict, local) = run_fn(&bytes, "root", args);
        assert!(
            matches!(local, Err(RuntimeError::TypeMismatchRuntime(_))),
            "LOCAL rejects the arguments: {local:?}"
        );
        assert_refused_may(&strict, &[Sqrt]);
    }
}

#[test]
fn a02_safe_entry_with_wrong_family_arguments_gives_the_local_argument_error() {
    let bytes = o0(SAFE_FN);
    for args in [
        vec![Value::I32(4)],
        Vec::new(),
        vec![Value::F64(4.0), Value::F64(1.0)],
    ] {
        let (strict, local) = run_fn(&bytes, "twice", args);
        assert!(
            matches!(local, Err(RuntimeError::TypeMismatchRuntime(_))),
            "{local:?}"
        );
        assert_admitted_as_local(&strict, &local);
    }
}

#[test]
fn a03_safe_entry_with_correct_arguments_returns_the_local_value() {
    let (strict, local) = run_fn(&o0(SAFE_FN), "twice", vec![Value::F64(2.25)]);
    assert_eq!(local, Ok(Value::F64(4.5)));
    assert_admitted_as_local(&strict, &local);
}

#[test]
fn a04_runtime_argument_values_do_not_influence_the_analysis() {
    for (src, entry) in [(SAFE_FN, "twice"), (HAZARD_FN, "root")] {
        let bytes = o0(src);
        let analyses: BTreeSet<String> = [
            vec![Value::F64(0.0)],
            vec![Value::F64(-1.0e300)],
            vec![Value::F64(f64::NAN)],
            vec![Value::I32(3)],
            Vec::new(),
        ]
        .into_iter()
        .map(|args| format!("{:?}", math(run_fn(&bytes, entry, args).0.analysis())))
        .collect();
        assert_eq!(analyses.len(), 1, "{entry}: {analyses:?}");
    }
}

// ----------------------------------------------- 9.5 cross-entry safety

#[test]
fn e01_one_artifact_clean_entry_runs_and_hazardous_entries_are_refused() {
    let bytes = o0(ENTRIES);
    let token = verify_semcode_token(&bytes).expect("verifies");
    let clean = token.require_entry("clean").expect("entry");
    let hot = token.require_entry("hot").expect("entry");
    let main = token.require_entry("main").expect("entry");
    let limits = default_limits();

    let hot_run = run_host_math_free_entry_semcode_with_config(&hot, &grant(), &limits, cfg());
    let clean_run = run_host_math_free_entry_semcode_with_config(&clean, &grant(), &limits, cfg());
    let main_run = run_host_math_free_entry_semcode_with_config(&main, &grant(), &limits, cfg());
    // Order does not matter: re-running the hazardous entry after the clean
    // one is analysed afresh and refused again.
    let hot_again = run_host_math_free_entry_semcode_with_config(&hot, &grant(), &limits, cfg());

    assert_eq!(clean_run.result(), &Ok(()));
    assert_eq!(clean_run.analysis().entry(), "clean");
    assert_refused_may(&hot_run, &[Sqrt]);
    assert_eq!(hot_run.analysis().entry(), "hot");
    assert_refused_may(&main_run, &[Cos]);
    assert_eq!(hot_again.analysis(), hot_run.analysis());
    assert_refused(&hot_again);
    assert_eq!(
        clean_run.analysis().artifact_hash(),
        hot_run.analysis().artifact_hash()
    );
}

// ----------------------------------------------- 9.6 LOCAL compatibility

#[test]
fn l01_admitted_corpus_matches_local_and_local_still_runs_hazards() {
    let admitted: &[(&str, &str)] = &[
        (CLEAN, "main"),
        (SHADOW_SAFE, "main"),
        (UNREACHABLE_FN, "main"),
        (RECURSIVE, "main"),
        (SAFE_INT, "main"),
        (QUAD, "main"),
        (ENTRIES, "clean"),
    ];
    for opt in [OptLevel::O0, OptLevel::O1] {
        for (src, entry) in admitted {
            let bytes = compile(src, opt, false);
            let (strict, local) = run_entry(&bytes, entry);
            assert_admitted_as_local(&strict, &local);
            let (strict, local) = run_fn(&bytes, entry, Vec::new());
            assert_admitted_as_local(&strict, &local);
        }
        for src in [
            DIRECT_SQRT,
            TAN_POW,
            VIA_HELPER,
            SHADOW_HAZARD,
            CLOSURE_HAZARD,
        ] {
            let bytes = compile(src, opt, false);
            let (strict, local) = run_entry(&bytes, "main");
            assert_eq!(local, Ok(()), "LOCAL behaviour is unchanged");
            assert_refused(&strict);
        }
    }
}

#[test]
fn l02_local_signatures_are_unchanged_and_never_return_a_strict_outcome() {
    // Compile-time: the LOCAL P16/P21 still have their exact LOCAL types.
    let _p16: fn(&VerifiedEntrySemCode<'_, '_>, ExecutionConfig) -> Result<(), RuntimeError> =
        run_verified_entry_semcode_with_config;
    let _p21: fn(
        &VerifiedEntrySemCode<'_, '_>,
        Vec<Value>,
        ExecutionConfig,
    ) -> Result<Value, RuntimeError> = run_verified_function_semcode_with_args_and_config;

    // Snapshot (`public_api_contracts` regenerates it from the current source and
    // fails on any difference, so it reflects the implementation): only the two
    // strict functions return `StrictExecutionOutcome`,
    // its fields are private, and nothing else in the public surface mentions it.
    let snapshot = include_str!("golden_snapshots/public_api/sm_vm_semcode_vm.txt");
    let mut returning: Vec<&str> = snapshot
        .lines()
        .filter(|l| l.starts_with("pub fn ") && l.contains("StrictExecutionOutcome"))
        .map(|l| l["pub fn ".len()..].split('(').next().unwrap())
        .collect();
    returning.sort();
    assert_eq!(
        returning,
        [
            "run_host_math_free_entry_semcode_with_config",
            "run_host_math_free_function_semcode_with_args_and_config",
        ]
    );
    // The snapshot lists public fields; the outcome's struct line is followed
    // directly by its accessors, so it has none (and no public constructor).
    let lines: Vec<&str> = snapshot.lines().collect();
    let struct_at = lines
        .iter()
        .position(|l| *l == "pub struct StrictExecutionOutcome<T> {")
        .expect("struct in snapshot");
    assert_eq!(
        &lines[struct_at + 1..struct_at + 5],
        [
            "pub fn analysis(&self) -> &HostMathAnalysis {",
            "pub fn result(&self) -> &Result<T, StrictRunError> {",
            "pub fn into_result(self) -> Result<T, StrictRunError> {",
            "pub fn execution_attempted(&self) -> bool {",
        ]
    );
    assert!(
        !snapshot.contains("-> StrictExecutionOutcome<T>"),
        "no generic constructor"
    );
}

#[test]
fn l03_grant_and_outcome_cannot_be_defaulted() {
    assert!(!has_default!(HostMathFreeGrant));
    assert!(!has_default!(StrictExecutionOutcome<()>));
    assert!(!has_default!(StrictExecutionOutcome<Value>));
    assert!(has_default!(Vec<u8>), "the probe itself detects Default");
}

#[test]
fn l04_grant_names_the_profile_and_carries_no_artifact_identity() {
    let g = HostMathFreeGrant::draft1();
    assert_eq!(g.profile(), HOST_MATH_FREE_DRAFT1_PROFILE);
    assert_eq!(
        g,
        HostMathFreeGrant::draft1(),
        "every draft1 grant is the same selection"
    );
    assert_eq!(std::mem::size_of::<HostMathFreeGrant>(), 0);
}

// ----------------------------------------------- 10 determinism and Q9a

#[test]
fn q01_repeated_invocations_are_identical() {
    for (src, entry) in [(CLEAN, "main"), (DIRECT_SQRT, "main"), (TAN_POW, "main")] {
        let bytes = o0(src);
        let (first, _) = run_entry(&bytes, entry);
        for _ in 0..3 {
            let (again, _) = run_entry(&bytes, entry);
            assert_eq!(again.analysis(), first.analysis());
            assert_eq!(again.result(), first.result());
        }
    }
}

#[test]
fn q02_debug_metadata_does_not_change_the_strict_decision() {
    for src in [
        CLEAN,
        DIRECT_SQRT,
        SHADOW_SAFE,
        SHADOW_HAZARD,
        CLOSURE_HAZARD,
        RECURSIVE,
    ] {
        for opt in [OptLevel::O0, OptLevel::O1] {
            let plain = compile(src, opt, false);
            let debug = compile(src, opt, true);
            assert_ne!(plain, debug, "debug metadata changes the bytes");
            let (a, _) = run_entry(&plain, "main");
            let (b, _) = run_entry(&debug, "main");
            assert_ne!(a.analysis().artifact_hash(), b.analysis().artifact_hash());
            assert_eq!(math(a.analysis()), math(b.analysis()), "{src}");
            assert_eq!(a.result(), b.result(), "{src}");
        }
    }
}
