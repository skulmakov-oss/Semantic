//! S15-B2: the typed `A0001` admission-refusal contract of `HOST_MATH_FREE@draft1`.
//!
//! `sm_vm::HostMathAdmissionRefusal` is derived only from a strict invocation
//! refused at S3b (`StrictRunError::AdmissionRefused`). Every case runs a real
//! verifier-admitted artifact through the strict API. Test names carry the
//! S15-B2 matrix id; the `b1v` comments name the S15-B1 design vectors.
//! External (JSON) projection of the record is deferred to S15-B3. `j01` only
//! checks that `smc check` emits no admission code; byte identity of
//! `semantic.diagnostics` v1 rests on the unchanged `ssf09_editor_baseline`
//! JSON goldens.

use std::marker::PhantomData;
use std::path::Path;
use std::process::Command;

use sm_format::semcode_decode::{MAX_SIGNATURE_PARAMETERS_PER_FUNCTION, MAX_STRING_LEN};
use sm_format::semcode_format::{CallableValueFamily, Opcode, MAGIC1};
use sm_format::sha256::sha256;
use sm_ir::{
    compile_program_to_semcode_with_options_debug, emit_ir_to_semcode, CompileProfile, IrFunction,
    IrInstr, OptLevel,
};
use sm_runtime_core::{ExecutionConfig, ExecutionContext, RuntimeTrap};
use sm_verify::{
    analyze_host_math_free_draft1, verify_semcode_token, HostMathAnalysis, HostMathAnalysisLimits,
    HostMathBuiltin, HostMathIndeterminate, HostMathVerdict, HOST_MATH_ANALYSER_IDENTITY,
    HOST_MATH_FREE_DRAFT1_PROFILE,
};
use sm_vm::diagnostic_admission::RUNTIME_DIAGNOSTIC_CODES;
use sm_vm::{
    run_host_math_free_entry_semcode_with_config,
    run_host_math_free_function_semcode_with_args_and_config,
    run_verified_entry_semcode_with_config, run_verified_function_semcode_with_args_and_config,
    run_verified_semcode, HostMathAdmissionRefusal, HostMathFreeGrant, RuntimeError,
    StrictExecutionOutcome, StrictRunError, Value, ADMISSION_DIAGNOSTIC_CODES,
};
use ton618_core::diagnostics::diagnostic_catalog;

use HostMathBuiltin::{Abs, Cos, Pow, Sin, Sqrt, Tan};
use HostMathIndeterminate::{
    AnalysisLimit, CallableEntryParameter, ClosureOriginUnknown, EntrySignatureUnavailable,
    ResolutionMismatch,
};

// ---------------------------------------------------------------- helpers

fn o0(src: &str) -> Vec<u8> {
    compile_program_to_semcode_with_options_debug(
        src,
        CompileProfile::RustLike,
        OptLevel::O0,
        false,
    )
    .unwrap_or_else(|err| panic!("source compiles: {err:?}\n{src}"))
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

fn call(name: &str) -> IrInstr {
    IrInstr::Call {
        dst: None,
        name: name.to_string(),
        args: vec![0],
    }
}

fn emit(functions: &[IrFunction]) -> Vec<u8> {
    emit_ir_to_semcode(functions, false).expect("IR emits")
}

fn cfg() -> ExecutionConfig {
    ExecutionConfig::for_context(ExecutionContext::VerifiedLocal)
}

fn limits() -> HostMathAnalysisLimits {
    HostMathAnalysisLimits::draft1_default()
}

fn strict_entry_with(
    bytes: &[u8],
    entry: &str,
    limits: HostMathAnalysisLimits,
    config: ExecutionConfig,
) -> StrictExecutionOutcome<()> {
    let token = verify_semcode_token(bytes).expect("artifact is verifier-admitted");
    let entry = token.require_entry(entry).expect("entry exists");
    run_host_math_free_entry_semcode_with_config(
        &entry,
        &HostMathFreeGrant::draft1(),
        &limits,
        config,
    )
}

fn strict_entry(bytes: &[u8], entry: &str) -> StrictExecutionOutcome<()> {
    strict_entry_with(bytes, entry, limits(), cfg())
}

fn strict_fn_with(
    bytes: &[u8],
    entry: &str,
    args: Vec<Value>,
    limits: HostMathAnalysisLimits,
    config: ExecutionConfig,
) -> StrictExecutionOutcome<Value> {
    let token = verify_semcode_token(bytes).expect("artifact is verifier-admitted");
    let entry = token.require_entry(entry).expect("entry exists");
    run_host_math_free_function_semcode_with_args_and_config(
        &entry,
        &HostMathFreeGrant::draft1(),
        &limits,
        args,
        config,
    )
}

fn strict_fn(bytes: &[u8], entry: &str, args: Vec<Value>) -> StrictExecutionOutcome<Value> {
    strict_fn_with(bytes, entry, args, limits(), cfg())
}

fn local_entry(bytes: &[u8], entry: &str, config: ExecutionConfig) -> Result<(), RuntimeError> {
    let token = verify_semcode_token(bytes).expect("artifact is verifier-admitted");
    let entry = token.require_entry(entry).expect("entry exists");
    run_verified_entry_semcode_with_config(&entry, config)
}

fn standalone(bytes: &[u8], entry: &str, limits: HostMathAnalysisLimits) -> HostMathAnalysis {
    let token = verify_semcode_token(bytes).expect("artifact is verifier-admitted");
    let entry = token.require_entry(entry).expect("entry exists");
    analyze_host_math_free_draft1(&entry, &limits)
}

/// The refusal record of a refused strict outcome; panics if there is none.
fn refusal<T: std::fmt::Debug>(outcome: &StrictExecutionOutcome<T>) -> HostMathAdmissionRefusal {
    assert!(
        matches!(outcome.result(), Err(StrictRunError::AdmissionRefused)),
        "{outcome:?}"
    );
    outcome
        .admission_refusal()
        .unwrap_or_else(|| panic!("a refused strict invocation yields a record: {outcome:?}"))
}

fn tokens(record: &HostMathAdmissionRefusal) -> (Vec<&'static str>, Vec<&'static str>) {
    (
        record.may().iter().map(|b| b.name()).collect(),
        record
            .indeterminate()
            .iter()
            .map(|r| r.reason_token())
            .collect(),
    )
}

/// The record's fixed identity and its faithfulness to the outcome's analysis.
fn assert_record_contract<T: std::fmt::Debug>(
    outcome: &StrictExecutionOutcome<T>,
    bytes: &[u8],
    entry: &str,
) -> HostMathAdmissionRefusal {
    let record = refusal(outcome);
    let analysis = outcome.analysis();
    assert_eq!(record.code(), "A0001");
    assert_eq!(record.class(), "admission_refusal");
    assert_eq!(record.authority(), "host_math_admission");
    assert_eq!(record.profile(), HOST_MATH_FREE_DRAFT1_PROFILE);
    assert_eq!(record.analyser(), HOST_MATH_ANALYSER_IDENTITY);
    assert_eq!(record.entry(), entry);
    assert_eq!(record.artifact_sha256(), sha256(bytes));
    assert_eq!(record.artifact_sha256(), analysis.artifact_hash());
    assert_eq!(record.verdict(), HostMathVerdict::Refuse);
    assert_eq!(record.may(), analysis.may());
    assert_eq!(record.indeterminate(), analysis.indeterminate());
    assert_eq!(
        record.hazards(),
        if analysis.indeterminate().is_empty() {
            "known"
        } else {
            "unknown"
        }
    );
    assert_eq!(record.may_exhaustive(), analysis.indeterminate().is_empty());
    assert_eq!(record.limits(), analysis.limits());
    assert_eq!(record.work_units_used(), analysis.work_units_used());
    assert!(!record.execution_attempted());
    assert!(!outcome.execution_attempted());
    assert_eq!(record.message(), FIXED_MESSAGE);
    assert!(!record.message().contains(entry) || entry.is_empty());
    // Producer ownership: the record code is the single declared admission code.
    assert_eq!(record.code(), ADMISSION_DIAGNOSTIC_CODES[0]);
    record
}

const FIXED_MESSAGE: &str = "HOST_MATH_FREE@draft1 admission refused the entry before execution; \
the structured reasons are in the typed refusal record.";

/// Autoref probes: `has_default!(T)` / `has_from_analysis!(T)` are `true`
/// iff the bound holds (the inherent method wins only then).
struct Probe<T>(PhantomData<T>);
trait NoImpl {
    fn has_default(&self) -> bool {
        false
    }
}
impl<T> NoImpl for &Probe<T> {}
impl<T: Default> Probe<T> {
    fn has_default(&self) -> bool {
        true
    }
}
struct FromProbe<T>(PhantomData<T>);
trait NoFrom {
    fn has_from_analysis(&self) -> bool {
        false
    }
}
impl<T> NoFrom for &FromProbe<T> {}
impl<T: From<HostMathAnalysis>> FromProbe<T> {
    fn has_from_analysis(&self) -> bool {
        true
    }
}
macro_rules! has_default {
    ($t:ty) => {
        (&Probe::<$t>(PhantomData)).has_default()
    };
}
macro_rules! has_from_analysis {
    ($t:ty) => {
        (&FromProbe::<$t>(PhantomData)).has_from_analysis()
    };
}

// ---------------------------------------------------------------- sources

const CLEAN: &str = "fn main() {\n    let a: i32 = 2;\n    let b: i32 = a * 3;\n    return;\n}\n";
const DIRECT_SQRT: &str = "fn main() {\n    let y: f64 = sqrt(16.0);\n    return;\n}\n";
const TAN_POW: &str =
    "fn main() {\n    let a: f64 = tan(1.0);\n    let b: f64 = pow(a, 2.0);\n    return;\n}\n";
const VIA_HELPER: &str = "fn helper(x: f64) -> f64 {\n    return cos(x);\n}\nfn main() {\n    let y: f64 = helper(1.0);\n    return;\n}\n";
const CLOSURE_HAZARD: &str = "fn main() {\n    let f: Closure(f64 -> f64) = (x => sin(x));\n    let y: f64 = f(1.0);\n    return;\n}\n";
const SQRT_THEN_COS: &str =
    "fn main() {\n    let a: f64 = sqrt(2.0);\n    let b: f64 = cos(1.0);\n    return;\n}\n";
const HAZARD_FN: &str =
    "fn root(v: f64) -> f64 {\n    return sqrt(v);\n}\nfn main() {\n    return;\n}\n";
const TRAP_SAFE: &str =
    "fn main() {\n    let z: i32 = 0;\n    let q: i32 = 7 / z;\n    return;\n}\n";
const TRAP_BEFORE: &str = "fn main() {\n    let z: i32 = 0;\n    let q: i32 = 7 / z;\n    let y: f64 = sqrt(2.0);\n    return;\n}\n";
const RECURSIVE: &str = "fn count(n: i32) -> i32 {\n    if n == 0 {\n        return 0;\n    }\n    return 1 + count(n - 1);\n}\nfn main() {\n    let r: i32 = count(30);\n    return;\n}\n";

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

/// S15-B1 probe PR1: a callable-carrying entry parameter and a reachable `sqrt`.
fn closure_param_and_sqrt() -> Vec<u8> {
    emit(&[ir_fn(
        "main",
        vec![CallableValueFamily::Closure],
        vec![IrInstr::LoadF64 { dst: 0, val: 2.0 }, call("sqrt"), ret()],
    )])
}

/// Every Indet kind a verified artifact with a signature can raise at once,
/// with repeated May sites: params [Closure, I32, Map, Tuple], a ClosureCall
/// with no origin, and `sqrt` twice plus `abs`.
fn mixed_reasons() -> Vec<u8> {
    emit(&[ir_fn(
        "main",
        vec![
            CallableValueFamily::Closure,
            CallableValueFamily::I32,
            CallableValueFamily::Map,
            CallableValueFamily::Tuple,
        ],
        vec![
            IrInstr::LoadF64 { dst: 0, val: 2.0 },
            call("sqrt"),
            call("abs"),
            call("sqrt"),
            IrInstr::ClosureCall {
                dst: None,
                closure: 0,
                arg: 0,
            },
            ret(),
        ],
    )])
}

// ------------------------------------------------- T: reason vocabulary (sm-verify)

#[test]
fn t01_every_indeterminate_reason_has_its_stable_token() {
    let table = [
        (
            CallableEntryParameter { index: 0 },
            "callable_entry_parameter",
        ),
        (
            CallableEntryParameter { index: 4095 },
            "callable_entry_parameter",
        ),
        (EntrySignatureUnavailable, "entry_signature_unavailable"),
        (ClosureOriginUnknown, "closure_origin_unknown"),
        (ResolutionMismatch, "resolution_mismatch"),
        (AnalysisLimit, "analysis_limit"),
    ];
    for (reason, token) in table {
        assert_eq!(reason.reason_token(), token, "{reason:?}");
        assert!(token.bytes().all(|b| b.is_ascii_lowercase() || b == b'_'));
        // Never derived from Debug or the Rust identifier (INV-14).
        assert_ne!(format!("{reason:?}"), token);
        assert!(!token.contains("unsupported_opcode"));
    }
    // The structured index is a separate field, never part of the token.
    assert!(!CallableEntryParameter { index: 7 }
        .reason_token()
        .contains('7'));
}

#[test]
fn t02_prohibited_builtin_identities_are_the_six_names() {
    let names: Vec<_> = [Abs, Cos, Pow, Sin, Sqrt, Tan]
        .iter()
        .map(|b| b.name())
        .collect();
    assert_eq!(names, ["abs", "cos", "pow", "sin", "sqrt", "tan"]);
    // Enum order is token order, so a sorted May is sorted by token.
    let mut sorted = names.clone();
    sorted.sort_unstable();
    assert_eq!(names, sorted);
}

// --------------------------------------------- R: refusal records (b1v-04..12, 23)

#[test]
fn r01_direct_forbidden_builtin_yields_a_complete_a0001_record() {
    let bytes = o0(DIRECT_SQRT);
    for record in [
        assert_record_contract(&strict_entry(&bytes, "main"), &bytes, "main"),
        assert_record_contract(&strict_fn(&bytes, "main", Vec::new()), &bytes, "main"),
    ] {
        assert_eq!(tokens(&record), (vec!["sqrt"], vec![]));
        assert_eq!(record.hazards(), "known");
        assert!(record.may_exhaustive());
    }
}

#[test]
fn r02_every_prohibited_builtin_appears_by_its_token() {
    let bytes = o0(TAN_POW);
    let record = assert_record_contract(&strict_entry(&bytes, "main"), &bytes, "main");
    assert_eq!(tokens(&record).0, ["pow", "tan"]);
    for (src, token) in [
        ("abs(-2.5)", "abs"),
        ("cos(1.0)", "cos"),
        ("sin(1.0)", "sin"),
        ("sqrt(4.0)", "sqrt"),
    ] {
        let bytes = o0(&format!(
            "fn main() {{\n    let y: f64 = {src};\n    return;\n}}\n"
        ));
        let record = assert_record_contract(&strict_entry(&bytes, "main"), &bytes, "main");
        assert_eq!(tokens(&record).0, [token]);
    }
}

#[test]
fn r03_transitively_reachable_builtin_is_recorded() {
    let bytes = o0(VIA_HELPER);
    let record = assert_record_contract(&strict_entry(&bytes, "main"), &bytes, "main");
    assert_eq!(tokens(&record), (vec!["cos"], vec![]));
}

#[test]
fn r04_indirect_callables_closure_target_and_unknown_origin() {
    let bytes = o0(CLOSURE_HAZARD);
    let record = assert_record_contract(&strict_entry(&bytes, "main"), &bytes, "main");
    assert_eq!(tokens(&record), (vec!["sin"], vec![]));

    let bytes = closure_call_without_origin();
    let record = assert_record_contract(&strict_entry(&bytes, "main"), &bytes, "main");
    assert_eq!(tokens(&record), (vec![], vec!["closure_origin_unknown"]));
    assert_eq!(record.hazards(), "unknown");
    assert!(!record.may_exhaustive());
}

#[test]
fn r05_entry_signature_unavailable_is_recorded() {
    let bytes = legacy_without_signature();
    let record = assert_record_contract(&strict_entry(&bytes, "main"), &bytes, "main");
    assert_eq!(
        tokens(&record),
        (vec![], vec!["entry_signature_unavailable"])
    );
    assert_eq!(record.indeterminate(), [EntrySignatureUnavailable]);
}

#[test]
fn r06_callable_entry_parameters_keep_their_structured_indices() {
    let bytes = emit(&[ir_fn(
        "main",
        vec![
            CallableValueFamily::I32,
            CallableValueFamily::Record,
            CallableValueFamily::F64,
            CallableValueFamily::Adt,
            CallableValueFamily::Sequence,
        ],
        vec![ret()],
    )]);
    for args in [Vec::new(), vec![Value::I32(1)]] {
        let record = assert_record_contract(&strict_fn(&bytes, "main", args), &bytes, "main");
        assert_eq!(
            record.indeterminate(),
            [
                CallableEntryParameter { index: 1 },
                CallableEntryParameter { index: 3 },
                CallableEntryParameter { index: 4 },
            ]
        );
        assert_eq!(tokens(&record).1, ["callable_entry_parameter"; 3]);
    }
}

#[test]
fn r07_analysis_limit_reports_the_limit_and_the_work_used() {
    let bytes = o0(CLEAN);
    let needed = standalone(&bytes, "main", limits()).work_units_used();
    let short = HostMathAnalysisLimits::new(needed - 1).expect("valid");
    let outcome = strict_entry_with(&bytes, "main", short, cfg());
    let record = assert_record_contract(&outcome, &bytes, "main");
    assert_eq!(record.indeterminate(), [AnalysisLimit]);
    assert_eq!(record.limits(), short);
    assert_eq!(record.limits().max_work_units(), needed - 1);
    assert_eq!(record.work_units_used(), needed - 1);
    assert!(!record.may_exhaustive());
    // The analysis budget is not a runtime quota: LOCAL still runs.
    assert_eq!(local_entry(&bytes, "main", cfg()), Ok(()));
    // At exactly the needed bound the entry is admitted: no record.
    let tight = HostMathAnalysisLimits::new(needed).expect("valid");
    assert!(strict_entry_with(&bytes, "main", tight, cfg())
        .admission_refusal()
        .is_none());
}

#[test]
fn r08_may_and_indet_coexist_in_one_record() {
    // b1v-07 (S15-B1 probe PR1), promoted to a test.
    let bytes = closure_param_and_sqrt();
    for args in [Vec::new(), vec![Value::F64(1.0)]] {
        let record = assert_record_contract(&strict_fn(&bytes, "main", args), &bytes, "main");
        assert_eq!(
            tokens(&record),
            (vec!["sqrt"], vec!["callable_entry_parameter"])
        );
        assert_eq!(
            record.indeterminate(),
            [CallableEntryParameter { index: 0 }]
        );
        assert_eq!(record.hazards(), "unknown");
        assert!(!record.may_exhaustive(), "May is a lower bound under Indet");
    }
}

#[test]
fn r09_may_under_an_analysis_limit_is_an_incomplete_lower_bound() {
    // b1v-12 (S15-B1 probe PR2), promoted to a test.
    let bytes = o0(SQRT_THEN_COS);
    let full = standalone(&bytes, "main", limits());
    assert_eq!(full.may(), [Cos, Sqrt]);
    let short = HostMathAnalysisLimits::new(full.work_units_used() - 1).expect("valid");
    let record = assert_record_contract(
        &strict_entry_with(&bytes, "main", short, cfg()),
        &bytes,
        "main",
    );
    assert_eq!(record.indeterminate(), [AnalysisLimit]);
    assert!(!record.may_exhaustive());
    assert!(record.may().iter().all(|b| full.may().contains(b)));
    assert!(
        record.may().len() < full.may().len(),
        "the recorded May is strictly smaller than the full May: {:?}",
        record.may()
    );
}

#[test]
fn r10_reasons_are_sorted_duplicate_free_and_complete() {
    // b1v-23: several Indet kinds and repeated May sites.
    let bytes = mixed_reasons();
    let record = assert_record_contract(&strict_fn(&bytes, "main", Vec::new()), &bytes, "main");
    assert_eq!(record.may(), [Abs, Sqrt]);
    assert_eq!(
        record.indeterminate(),
        [
            CallableEntryParameter { index: 0 },
            CallableEntryParameter { index: 2 },
            CallableEntryParameter { index: 3 },
            ClosureOriginUnknown,
        ]
    );
    assert_eq!(
        tokens(&record),
        (
            vec!["abs", "sqrt"],
            vec![
                "callable_entry_parameter",
                "callable_entry_parameter",
                "callable_entry_parameter",
                "closure_origin_unknown",
            ]
        )
    );
    let mut sorted = record.indeterminate().to_vec();
    sorted.sort();
    sorted.dedup();
    assert_eq!(record.indeterminate(), sorted.as_slice());
}

#[test]
fn r11_records_are_value_equal_across_runs_and_siblings() {
    // b1v-21 / INV-B2-06.
    for (bytes, entry) in [
        (o0(DIRECT_SQRT), "main"),
        (mixed_reasons(), "main"),
        (o0(HAZARD_FN), "root"),
    ] {
        let first = refusal(&strict_fn(&bytes, entry, Vec::new()));
        for _ in 0..3 {
            assert_eq!(refusal(&strict_fn(&bytes, entry, Vec::new())), first);
            assert_eq!(refusal(&strict_entry(&bytes, entry)), first);
        }
        // The accessor does not consume or alter the outcome.
        let outcome = strict_entry(&bytes, entry);
        assert_eq!(outcome.admission_refusal(), outcome.admission_refusal());
        assert!(matches!(
            outcome.into_result(),
            Err(StrictRunError::AdmissionRefused)
        ));
    }
}

#[test]
fn r12_debug_metadata_changes_only_the_artifact_digest() {
    // b1v-22.
    let plain = o0(DIRECT_SQRT);
    let debug = compile_program_to_semcode_with_options_debug(
        DIRECT_SQRT,
        CompileProfile::RustLike,
        OptLevel::O0,
        true,
    )
    .expect("compiles");
    let a = assert_record_contract(&strict_entry(&plain, "main"), &plain, "main");
    let b = assert_record_contract(&strict_entry(&debug, "main"), &debug, "main");
    assert_eq!(tokens(&a), tokens(&b));
    if plain != debug {
        assert_ne!(a.artifact_sha256(), b.artifact_sha256());
    }
}

// ------------------------------------------------- N: no record, no relabelling

#[test]
fn n01_strict_success_has_no_record() {
    // b1v-03 / b1v-24.
    let bytes = o0(CLEAN);
    let outcome = strict_entry(&bytes, "main");
    assert_eq!(outcome.result(), &Ok(()));
    assert!(outcome.admission_refusal().is_none());
    let outcome = strict_fn(&o0(RECURSIVE), "count", vec![Value::I32(3)]);
    assert_eq!(outcome.result(), &Ok(Value::I32(3)));
    assert!(outcome.admission_refusal().is_none());
}

#[test]
fn n02_runtime_failures_have_no_record_and_keep_their_r_mapping() {
    // b1v-13, b1v-15, b1v-29, b1v-35 / INV-B2-04.
    let trap = strict_entry(&o0(TRAP_SAFE), "main");
    assert_eq!(
        trap.result(),
        &Err(StrictRunError::Runtime(RuntimeError::Trap(
            RuntimeTrap::DivisionByZero
        )))
    );
    assert!(trap.admission_refusal().is_none());
    assert!(trap.execution_attempted());
    match trap.result() {
        Err(StrictRunError::Runtime(e)) => assert_eq!(e.admitted_code(), Some("R0003")),
        other => panic!("{other:?}"),
    }

    let mut tight = cfg();
    tight.quotas.max_steps = 1;
    let quota = strict_entry_with(&o0(CLEAN), "main", limits(), tight);
    match quota.result() {
        Err(StrictRunError::Runtime(e @ RuntimeError::QuotaExceeded(_))) => {
            assert_eq!(e.admitted_code(), Some("R0010"))
        }
        other => panic!("{other:?}"),
    }
    assert!(quota.admission_refusal().is_none());

    let wrong_args = strict_fn(
        &o0("fn twice(v: f64) -> f64 {\n    return v + v;\n}\nfn main() {\n    return;\n}\n"),
        "twice",
        vec![Value::I32(4)],
    );
    match wrong_args.result() {
        Err(StrictRunError::Runtime(e @ RuntimeError::TypeMismatchRuntime(_))) => {
            assert_eq!(e.admitted_code(), None)
        }
        other => panic!("{other:?}"),
    }
    assert!(wrong_args.admission_refusal().is_none());

    let mut shallow = cfg();
    shallow.quotas.max_stack_depth = 1;
    let deep = strict_fn_with(
        &o0(RECURSIVE),
        "count",
        vec![Value::I32(30)],
        limits(),
        shallow,
    );
    match deep.result() {
        Err(StrictRunError::Runtime(e)) => {
            assert_ne!(e.admitted_code(), Some("A0001"));
            assert!(!e.admitted_code().unwrap_or("").starts_with('A'));
        }
        other => panic!("a runtime failure is expected: {other:?}"),
    }
    assert!(deep.admission_refusal().is_none());
}

#[test]
fn n03_standalone_analysis_is_not_an_admission_refusal_event() {
    // b1v-25 / INV-B2-01: a Refuse analysis exists, yet no API turns it into a record.
    let analysis = standalone(&o0(DIRECT_SQRT), "main", limits());
    assert_eq!(analysis.verdict(), HostMathVerdict::Refuse);
    // Positive controls: the probes do detect an implementation when one exists.
    assert!(has_default!(u8));
    assert!(has_from_analysis!(HostMathAnalysis));
    assert!(!has_default!(HostMathAdmissionRefusal));
    assert!(!has_from_analysis!(HostMathAdmissionRefusal));
}

#[test]
fn n04_refusal_precedes_guest_effects_and_argument_validation_without_local_fallback() {
    // b1v-14, b1v-30 / INV-B2-02, INV-B2-03.
    let bytes = o0(TRAP_BEFORE);
    assert_eq!(
        local_entry(&bytes, "main", cfg()),
        Err(RuntimeError::Trap(RuntimeTrap::DivisionByZero)),
        "LOCAL executes the guest instruction before the hazard"
    );
    let record = assert_record_contract(&strict_entry(&bytes, "main"), &bytes, "main");
    assert_eq!(tokens(&record).0, ["sqrt"]);

    // Wrong-family, missing and extra arguments are never validated on refusal.
    let bytes = o0(HAZARD_FN);
    let token = verify_semcode_token(&bytes).expect("verified");
    let entry = token.require_entry("root").expect("entry");
    for args in [
        vec![Value::I32(4)],
        Vec::new(),
        vec![Value::F64(4.0), Value::F64(1.0)],
    ] {
        let local = run_verified_function_semcode_with_args_and_config(&entry, args.clone(), cfg());
        assert!(matches!(local, Err(RuntimeError::TypeMismatchRuntime(_))));
        let record = assert_record_contract(&strict_fn(&bytes, "root", args), &bytes, "root");
        assert_eq!(tokens(&record).0, ["sqrt"]);
    }

    // No LOCAL fallback: LOCAL succeeds, strict refuses and yields no value.
    let bytes = o0(DIRECT_SQRT);
    assert_eq!(local_entry(&bytes, "main", cfg()), Ok(()));
    let outcome = strict_entry(&bytes, "main");
    assert_ne!(outcome.result(), &Ok(()));
    assert!(outcome.admission_refusal().is_some());
}

#[test]
fn n05_verifier_rejection_stays_a_verifier_failure() {
    // b1v-01 / INV-B2-05.
    let bytes = o0(DIRECT_SQRT);
    let truncated = &bytes[..bytes.len() - 3];
    let report = verify_semcode_token(truncated).expect_err("truncated artifact never verifies");
    for d in &report.diagnostics {
        assert!(d.code.code_token().starts_with('V'), "{d:?}");
    }
    match run_verified_semcode(truncated) {
        Err(e @ RuntimeError::VerifierRejected(_)) => {
            assert_eq!(e.admitted_code(), Some("R0030"))
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn n06_admission_codes_are_disjoint_from_runtime_codes() {
    // INV-B2-04 / INV-B2-10.
    assert_eq!(ADMISSION_DIAGNOSTIC_CODES, ["A0001"]);
    assert_eq!(
        RUNTIME_DIAGNOSTIC_CODES,
        ["R0001", "R0002", "R0003", "R0004", "R0010", "R0020", "R0030"]
    );
    for code in ADMISSION_DIAGNOSTIC_CODES {
        assert!(!RUNTIME_DIAGNOSTIC_CODES.contains(&code));
        assert!(code.starts_with('A'));
    }
}

// ------------------------------------------------- C: catalog, mirror, bounds

#[test]
fn c01_a0001_is_catalogued_once_explainable_and_mirrored() {
    // b1v-32 / INV-B2-10.
    let entries: Vec<_> = diagnostic_catalog()
        .iter()
        .filter(|(code, _)| code.starts_with('A'))
        .collect();
    assert_eq!(entries.len(), 1, "exactly one admission code is allocated");
    let (code, text) = entries[0];
    assert_eq!(*code, "A0001");
    for fragment in [
        "admission refusal",
        "HOST_MATH_FREE@draft1",
        "S3b",
        "no VM execution",
        "not a verifier rejection or a runtime failure",
        "typed refusal record",
    ] {
        assert!(
            text.contains(fragment),
            "catalog text lacks {fragment:?}: {text}"
        );
    }
    for query in ["A0001", "a0001"] {
        assert_eq!(smc_cli::CliPipeline::explain(query), Some(*text));
    }
    let mirror =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/ERROR_CODES.md"))
            .unwrap()
            .replace("\r\n", "\n");
    assert!(
        mirror.contains(&format!("- `A0001`: {text}\n")),
        "mirror differs"
    );
}

#[test]
fn c02_smc_explain_cli_recognises_a0001() {
    let smc = env!("CARGO_BIN_EXE_smc");
    let one = Command::new(smc)
        .args(["explain", "A0001"])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&one.stdout);
    assert!(one.status.success(), "{text}");
    assert!(text.contains("A0001:") && text.contains("HOST_MATH_FREE@draft1"));
    let list = Command::new(smc)
        .args(["explain", "--list"])
        .output()
        .unwrap();
    let list = String::from_utf8_lossy(&list.stdout);
    let first = list.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
    assert!(list.contains("A0001"), "{list}");
    assert!(first.contains("A0001"), "A0001 sorts first: {first}");
}

#[test]
fn c03_entry_names_are_never_interpolated_into_the_message() {
    // b1v-27 / INV-B2-09: hostile and maximal-length entry names.
    let long = "z".repeat(MAX_STRING_LEN);
    for name in ["evil\u{1b}[31m\n%s{}", long.as_str()] {
        let bytes = emit(&[ir_fn(
            name,
            Vec::new(),
            vec![IrInstr::LoadF64 { dst: 0, val: 2.0 }, call("sqrt"), ret()],
        )]);
        let record = assert_record_contract(&strict_entry(&bytes, name), &bytes, name);
        assert_eq!(record.entry().len(), name.len());
        assert_eq!(record.message(), FIXED_MESSAGE);
        assert!(!record.message().contains(name));
    }
}

#[test]
fn c04_maximal_parameter_index_list_is_complete() {
    // b1v-28: no truncation at the format's own bound.
    let n = MAX_SIGNATURE_PARAMETERS_PER_FUNCTION;
    let bytes = emit(&[ir_fn(
        "main",
        vec![CallableValueFamily::Closure; n],
        vec![ret()],
    )]);
    let record = assert_record_contract(&strict_fn(&bytes, "main", Vec::new()), &bytes, "main");
    let expected: Vec<_> = (0..n)
        .map(|index| CallableEntryParameter { index })
        .collect();
    assert_eq!(record.indeterminate(), expected.as_slice());
}

#[test]
fn j01_smc_check_emits_no_admission_code() {
    // b1v-31 / INV-B2-12, narrow claim: `smc check` runs no strict stage, so
    // neither a clean hazardous source nor a diagnostic-bearing one yields an
    // admission code, and both stay schema v1. Byte identity of v1 output is
    // proven by the unchanged `ssf09_editor_baseline` JSON goldens, not here.
    let broken = "fn main() {\n    let y: i32 = sqrt(2.0);\n    return;\n}\n";
    for (name, src, ok) in [("clean", DIRECT_SQRT, true), ("broken", broken, false)] {
        let dir = std::env::temp_dir().join(format!("s15b2_j01_{name}_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("main.sm"), src).unwrap();
        let out = Command::new(env!("CARGO_BIN_EXE_smc"))
            .args(["check", "main.sm", "--format", "json"])
            .current_dir(&dir)
            .output()
            .unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert_eq!(out.status.success(), ok, "{name}: {stdout}");
        assert!(
            stdout.contains("\"schema\":\"semantic.diagnostics\"")
                || stdout.contains("\"schema\": \"semantic.diagnostics\""),
            "{name}: {stdout}"
        );
        assert!(
            stdout.contains("\"schema_version\":1") || stdout.contains("\"schema_version\": 1"),
            "{name}: {stdout}"
        );
        if !ok {
            assert!(
                stdout.contains("\"code\""),
                "{name} carries a diagnostic: {stdout}"
            );
        }
        assert!(!stdout.contains("A0001"), "{name}: {stdout}");
        assert!(!stdout.contains("admission"), "{name}: {stdout}");
    }
}
