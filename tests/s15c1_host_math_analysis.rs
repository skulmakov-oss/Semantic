//! S15-C1: executable qualification of the verifier-owned host-math analysis
//! (`sm_verify::analyze_host_math_free_draft1`) against
//! `docs/spec/portable_math_profile_draft_v0.md` §4-§7.
//!
//! Every verdict here is computed on a real verifier-admitted artifact. Test
//! names carry the S15-C1 matrix id. The analysis is mathematical evidence
//! only: nothing here executes a guest under a strict profile.

use std::collections::{BTreeMap, BTreeSet};

use sm_format::semcode_decode::decode_semcode;
use sm_format::semcode_format::{CallableValueFamily, Opcode, CAP_F64_MATH, MAGIC1};
use sm_ir::{
    compile_program_to_semcode_with_options_debug, emit_ir_to_semcode, CompileProfile, IrFunction,
    IrInstr, OptLevel,
};
use sm_verify::{
    analyze_host_math_free_draft1, verify_semcode_token, EntryResolutionError, HostMathAnalysis,
    HostMathAnalysisLimits, HostMathBuiltin, HostMathIndeterminate, HostMathVerdict,
    VerificationCode, HOST_MATH_ANALYSER_IDENTITY, HOST_MATH_FREE_DRAFT1_PROFILE,
};
use sm_vm::Value;

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

fn ir_call(name: &str, args: Vec<u16>) -> IrInstr {
    IrInstr::Call {
        dst: None,
        name: name.to_string(),
        args,
    }
}

fn ret() -> IrInstr {
    IrInstr::Ret { src: None }
}

fn emit(functions: &[IrFunction], debug: bool) -> Vec<u8> {
    emit_ir_to_semcode(functions, debug).expect("IR emits")
}

fn analyze_with(bytes: &[u8], entry: &str, limits: HostMathAnalysisLimits) -> HostMathAnalysis {
    let token = verify_semcode_token(bytes).expect("artifact is verifier-admitted");
    let entry = token.require_entry(entry).expect("entry exists");
    analyze_host_math_free_draft1(&entry, &limits)
}

fn analyze(bytes: &[u8], entry: &str) -> HostMathAnalysis {
    analyze_with(bytes, entry, HostMathAnalysisLimits::draft1_default())
}

fn assert_admit(a: &HostMathAnalysis) {
    assert_eq!(a.verdict(), HostMathVerdict::Admit, "{a:?}");
    assert!(a.may().is_empty() && a.indeterminate().is_empty(), "{a:?}");
    assert!(!a.hazards_unknown());
}

fn assert_refuse_may(a: &HostMathAnalysis, may: &[HostMathBuiltin]) {
    assert_eq!(a.verdict(), HostMathVerdict::Refuse, "{a:?}");
    assert_eq!(a.may(), may, "{a:?}");
    assert!(a.indeterminate().is_empty(), "{a:?}");
}

fn assert_refuse_indet(a: &HostMathAnalysis, indet: &[HostMathIndeterminate]) {
    assert_eq!(a.verdict(), HostMathVerdict::Refuse, "{a:?}");
    assert_eq!(a.indeterminate(), indet, "{a:?}");
    assert!(a.hazards_unknown());
}

/// The mathematical part of a result: what Q9a requires to be identical.
fn math(
    a: &HostMathAnalysis,
) -> (
    HostMathVerdict,
    Vec<HostMathBuiltin>,
    Vec<HostMathIndeterminate>,
    u64,
) {
    (
        a.verdict(),
        a.may().to_vec(),
        a.indeterminate().to_vec(),
        a.work_units_used(),
    )
}

const CLEAN: &str = "fn main() {\n    let a: i32 = 2;\n    let b: i32 = a * 3;\n    return;\n}\n";
const DIRECT_SQRT: &str = "fn main() {\n    let y: f64 = sqrt(16.0);\n    return;\n}\n";
const TAN_POW: &str =
    "fn main() {\n    let a: f64 = tan(1.0);\n    let b: f64 = pow(a, 2.0);\n    return;\n}\n";
const VIA_HELPER: &str = "fn helper(x: f64) -> f64 {\n    return cos(x);\n}\nfn main() {\n    let y: f64 = helper(1.0);\n    return;\n}\n";
const SHADOW_SAFE: &str = "fn sqrt(x: f64) -> f64 {\n    return x + 100.0;\n}\nfn main() {\n    let y: f64 = sqrt(16.0);\n    return;\n}\n";
const SHADOW_HAZARD: &str = "fn sqrt(x: f64) -> f64 {\n    return tan(x);\n}\nfn main() {\n    let y: f64 = sqrt(16.0);\n    return;\n}\n";
const UNREACHABLE_FN: &str =
    "fn unused() {\n    let y: f64 = sqrt(2.0);\n    return;\n}\nfn main() {\n    return;\n}\n";
const RECURSIVE: &str = "fn down(n: i32) -> i32 {\n    if n == 0 {\n        return 0;\n    }\n    return down(n - 1);\n}\nfn main() {\n    let r: i32 = down(3);\n    return;\n}\n";
const BRANCH_THEN: &str = "fn main() {\n    let a: i32 = 2;\n    if a > 1 {\n        let s: f64 = sin(1.0);\n    }\n    return;\n}\n";
const BRANCH_ELSE: &str = "fn main() {\n    let a: i32 = 2;\n    if a > 1 {\n        let s: f64 = 1.0;\n    } else {\n        let c: f64 = cos(1.0);\n    }\n    return;\n}\n";
const CLOSURE_HAZARD: &str = "fn main() {\n    let f: Closure(f64 -> f64) = (x => sin(x));\n    let y: f64 = f(1.0);\n    return;\n}\n";
const CLOSURE_UNCALLED: &str =
    "fn main() {\n    let f: Closure(f64 -> f64) = (x => sin(x));\n    return;\n}\n";
const ENTRIES: &str = "fn clean() {\n    return;\n}\nfn hot() {\n    let y: f64 = sqrt(2.0);\n    return;\n}\nfn main() {\n    return;\n}\n";
/// A closure returned by a guest function and invoked in the guest: the
/// `MakeClosure` is discovered (in `make`) after `main`'s `ClosureCall`.
const CLOSURE_RETURNED: &str = "fn make() -> Closure(f64 -> f64) {\n    let f: Closure(f64 -> f64) = (x => abs(x));\n    return f;\n}\nfn main() {\n    let g: Closure(f64 -> f64) = make();\n    let y: f64 = g(1.0);\n    return;\n}\n";
const CLOSURE_CAPTURED: &str = "fn main() {\n    let k: f64 = 2.0;\n    let f: Closure(f64 -> f64) = (x => pow(x, k));\n    let y: f64 = f(1.0);\n    return;\n}\n";
const CLOSURE_IN_TUPLE: &str = "fn main() {\n    let f: Closure(f64 -> f64) = (x => sqrt(x));\n    let t: (Closure(f64 -> f64), i32) = (f, 1);\n    let (g, n) = t;\n    let y: f64 = g(4.0);\n    return;\n}\n";
/// A closure target that itself creates and calls another closure.
const CLOSURE_CHAIN: &str = "fn inner(y: f64) -> f64 {\n    let g: Closure(f64 -> f64) = (z => cos(z));\n    return g(y);\n}\nfn main() {\n    let f: Closure(f64 -> f64) = (x => inner(x));\n    let y: f64 = f(1.0);\n    return;\n}\n";
const CLOSURE_PASSED: &str = "fn apply(f: Closure(f64 -> f64), v: f64) -> f64 {\n    return f(v);\n}\nfn main() {\n    let y: f64 = apply((x => tan(x)), 1.0);\n    return;\n}\n";
const DIRECT_ABS: &str = "fn main() {\n    let y: f64 = abs(-2.5);\n    return;\n}\n";

/// Every source the O0/O1, Q9a and VM-differential controls range over.
const CORPUS: &[(&str, &str)] = &[
    ("clean", CLEAN),
    ("direct_sqrt", DIRECT_SQRT),
    ("tan_pow", TAN_POW),
    ("via_helper", VIA_HELPER),
    ("shadow_safe", SHADOW_SAFE),
    ("shadow_hazard", SHADOW_HAZARD),
    ("unreachable_fn", UNREACHABLE_FN),
    ("recursive", RECURSIVE),
    ("branch_then", BRANCH_THEN),
    ("branch_else", BRANCH_ELSE),
    ("closure_hazard", CLOSURE_HAZARD),
    ("closure_uncalled", CLOSURE_UNCALLED),
    ("closure_returned", CLOSURE_RETURNED),
    ("closure_captured", CLOSURE_CAPTURED),
    ("closure_in_tuple", CLOSURE_IN_TUPLE),
    ("closure_chain", CLOSURE_CHAIN),
    ("closure_passed", CLOSURE_PASSED),
    ("direct_abs", DIRECT_ABS),
];

/// `main` creates and calls a clean closure; `MakeClosure(hazard)` appears
/// only after `main`'s `Ret` and in a function nothing calls. With
/// `hazard_reachable`, the same `MakeClosure` is moved before the `Ret`.
fn unreachable_closure_origins(hazard_reachable: bool) -> Vec<u8> {
    use CallableValueFamily::F64;
    let make_hazard = IrInstr::MakeClosure {
        dst: 2,
        name: "hazard".to_string(),
        captures: Vec::new(),
    };
    let mut main = vec![
        IrInstr::MakeClosure {
            dst: 0,
            name: "clean".to_string(),
            captures: Vec::new(),
        },
        IrInstr::LoadF64 { dst: 1, val: 1.0 },
        IrInstr::ClosureCall {
            dst: None,
            closure: 0,
            arg: 1,
        },
    ];
    if hazard_reachable {
        main.extend([make_hazard, ret()]);
    } else {
        main.extend([ret(), make_hazard, ret()]);
    }
    emit(
        &[
            ir_fn("clean", vec![F64], vec![IrInstr::Ret { src: Some(0) }]),
            ir_fn("hazard", vec![F64], vec![ir_call("sin", vec![0]), ret()]),
            ir_fn(
                "unused",
                Vec::new(),
                vec![
                    IrInstr::MakeClosure {
                        dst: 0,
                        name: "hazard".to_string(),
                        captures: Vec::new(),
                    },
                    ret(),
                ],
            ),
            ir_fn("main", Vec::new(), main),
        ],
        false,
    )
}

/// `main: Ret; Call sqrt; Ret` - a prohibited call after an unconditional return.
fn after_return() -> Vec<u8> {
    emit(
        &[ir_fn(
            "main",
            Vec::new(),
            vec![
                IrInstr::LoadF64 { dst: 0, val: 2.0 },
                ret(),
                ir_call("sqrt", vec![0]),
                ret(),
            ],
        )],
        false,
    )
}

/// `main: ClosureCall` with no `MakeClosure` anywhere and no parameters.
fn closure_call_without_origin() -> Vec<u8> {
    emit(
        &[ir_fn(
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
        )],
        false,
    )
}

fn entry_with_params(params: Vec<CallableValueFamily>) -> Vec<u8> {
    emit(&[ir_fn("main", params, vec![ret()])], false)
}

/// A rev-2 artifact (canonical SIG0 signatures start at rev 20), hand-built like
/// the verifier's own pre-#1773 fixtures: empty string table, `RET`.
fn legacy_without_signature() -> Vec<u8> {
    let code = [0u8, 0, Opcode::Ret as u8, 0];
    let mut bytes = MAGIC1.to_vec();
    bytes.extend_from_slice(&4u16.to_le_bytes());
    bytes.extend_from_slice(b"main");
    bytes.extend_from_slice(&(code.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&code);
    bytes
}

/// Q9a: appends a string no instruction references to `function`'s string
/// table. Existing string ids and every instruction byte are unchanged.
fn append_unreferenced_string(bytes: &[u8], function: &str, extra: &str) -> Vec<u8> {
    let decoded = decode_semcode(bytes).expect("decodes");
    let env = decoded
        .functions
        .iter()
        .find(|env| env.name == function)
        .expect("function exists");
    let (code_at, code_len) = (env.code_offset, env.code_len);
    let code = &bytes[code_at..code_at + code_len];
    let count = u16::from_le_bytes([code[0], code[1]]) as usize;
    let mut table_end = 2;
    for _ in 0..count {
        let len = u16::from_le_bytes([code[table_end], code[table_end + 1]]) as usize;
        table_end += 2 + len;
    }
    let mut new_code = ((count + 1) as u16).to_le_bytes().to_vec();
    new_code.extend_from_slice(&code[2..table_end]);
    new_code.extend_from_slice(&(extra.len() as u16).to_le_bytes());
    new_code.extend_from_slice(extra.as_bytes());
    new_code.extend_from_slice(&code[table_end..]);
    let mut out = bytes[..code_at - 4].to_vec();
    out.extend_from_slice(&(new_code.len() as u32).to_le_bytes());
    out.extend_from_slice(&new_code);
    out.extend_from_slice(&bytes[code_at + code_len..]);
    out
}

// ------------------------------------------------------- positive controls

#[test]
fn p01_clean_zero_parameter_entry_admits() {
    let a = analyze(&o0(CLEAN), "main");
    assert_admit(&a);
    assert_eq!(a.profile(), HOST_MATH_FREE_DRAFT1_PROFILE);
    assert_eq!(a.analyser(), HOST_MATH_ANALYSER_IDENTITY);
    assert_eq!(a.entry(), "main");
    assert_eq!(a.limits(), HostMathAnalysisLimits::draft1_default());
}

#[test]
fn p02_user_shadowed_sqrt_with_safe_body_admits() {
    assert_admit(&analyze(&o0(SHADOW_SAFE), "main"));
}

#[test]
fn p03_prohibited_builtin_in_unreachable_function_admits() {
    assert_admit(&analyze(&o0(UNREACHABLE_FN), "main"));
}

#[test]
fn p04_prohibited_builtin_after_unconditional_return_admits() {
    assert_admit(&analyze(&after_return(), "main"));
}

#[test]
fn p05_clean_recursive_call_graph_admits() {
    assert_admit(&analyze(&o0(RECURSIVE), "main"));
}

#[test]
fn p06_same_graph_with_and_without_debug_metadata_gives_the_same_verdict() {
    for (name, src) in [("clean", CLEAN), ("direct_sqrt", DIRECT_SQRT)] {
        let plain = compile(src, OptLevel::O0, false);
        let debug = compile(src, OptLevel::O0, true);
        assert_ne!(plain, debug, "{name}: debug variant must differ in bytes");
        let (a, b) = (analyze(&plain, "main"), analyze(&debug, "main"));
        assert_ne!(a.artifact_hash(), b.artifact_hash());
        assert_eq!(math(&a), math(&b), "{name}");
    }
}

#[test]
fn p07_distinct_entries_are_analysed_independently() {
    let bytes = o0(ENTRIES);
    assert_admit(&analyze(&bytes, "clean"));
    assert_admit(&analyze(&bytes, "main"));
    assert_refuse_may(&analyze(&bytes, "hot"), &[Sqrt]);
}

#[test]
fn p08_closure_created_but_never_called_admits_under_c0() {
    assert_admit(&analyze(&o0(CLOSURE_UNCALLED), "main"));
}

// ------------------------------------------------------- negative controls

#[test]
fn n01_direct_reachable_sqrt_refuses() {
    assert_refuse_may(&analyze(&o0(DIRECT_SQRT), "main"), &[Sqrt]);
}

#[test]
fn n02_reachable_tan_and_pow_refuse_sorted() {
    assert_refuse_may(&analyze(&o0(TAN_POW), "main"), &[Pow, Tan]);
}

#[test]
fn n03_prohibited_call_through_reachable_program_function_refuses() {
    assert_refuse_may(&analyze(&o0(VIA_HELPER), "main"), &[Cos]);
}

#[test]
fn n04_user_shadowed_function_reaching_a_prohibited_builtin_refuses() {
    // The user `sqrt` is not builtin:sqrt; its body's `tan` is the hazard.
    assert_refuse_may(&analyze(&o0(SHADOW_HAZARD), "main"), &[Tan]);
}

#[test]
fn n05_prohibited_builtin_in_either_jmpif_branch_refuses() {
    assert_refuse_may(&analyze(&o0(BRANCH_THEN), "main"), &[Sin]);
    assert_refuse_may(&analyze(&o0(BRANCH_ELSE), "main"), &[Cos]);
}

#[test]
fn n06_reachable_closure_targeting_a_hazardous_function_refuses() {
    assert_refuse_may(&analyze(&o0(CLOSURE_HAZARD), "main"), &[Sin]);
}

#[test]
fn n07_callable_carrying_entry_parameter_families_are_indeterminate() {
    use CallableValueFamily::*;
    for family in [Closure, Tuple, Record, Adt, Sequence, Map] {
        let a = analyze(&entry_with_params(vec![I32, family]), "main");
        assert_refuse_indet(
            &a,
            &[HostMathIndeterminate::CallableEntryParameter { index: 1 }],
        );
        assert!(a.may().is_empty());
    }
    for family in [Quad, Bool, Text, I32, U32, Fx, F64, Unit, Bytes] {
        assert_admit(&analyze(&entry_with_params(vec![family]), "main"));
    }
}

#[test]
fn n08_missing_canonical_entry_signature_is_indeterminate() {
    let bytes = legacy_without_signature();
    let decoded = decode_semcode(&bytes).expect("legacy artifact decodes");
    assert!(decoded.functions[0].signature.is_none());
    assert_refuse_indet(
        &analyze(&bytes, "main"),
        &[HostMathIndeterminate::EntrySignatureUnavailable],
    );
}

#[test]
fn n09_closure_call_with_unknown_callable_origin_is_indeterminate() {
    assert_refuse_indet(
        &analyze(&closure_call_without_origin(), "main"),
        &[HostMathIndeterminate::ClosureOriginUnknown],
    );
}

#[test]
fn n11_resource_exhaustion_is_a_deterministic_refusal() {
    let bytes = o0(VIA_HELPER);
    let full = analyze(&bytes, "main");
    let needed = full.work_units_used();
    let tight = HostMathAnalysisLimits::new(needed).expect("valid");
    assert_eq!(math(&analyze_with(&bytes, "main", tight)), math(&full));
    let short = HostMathAnalysisLimits::new(needed - 1).expect("valid");
    let first = analyze_with(&bytes, "main", short);
    assert!(first
        .indeterminate()
        .contains(&HostMathIndeterminate::AnalysisLimit));
    assert_eq!(first.verdict(), HostMathVerdict::Refuse);
    for _ in 0..3 {
        assert_eq!(math(&analyze_with(&bytes, "main", short)), math(&first));
    }
    let tiny = HostMathAnalysisLimits::new(1).expect("valid");
    assert_refuse_indet(
        &analyze_with(&o0(CLEAN), "main", tiny),
        &[HostMathIndeterminate::AnalysisLimit],
    );
}

#[test]
fn n13_closure_returned_by_a_guest_function_and_invoked_refuses_abs() {
    assert_eq!(Abs.name(), "abs");
    assert_refuse_may(&analyze(&o0(DIRECT_ABS), "main"), &[Abs]);
    assert_refuse_may(&analyze(&o0(CLOSURE_RETURNED), "main"), &[Abs]);
}

#[test]
fn n14_captured_stored_chained_and_passed_closures_refuse() {
    assert_refuse_may(&analyze(&o0(CLOSURE_CAPTURED), "main"), &[Pow]);
    assert_refuse_may(&analyze(&o0(CLOSURE_IN_TUPLE), "main"), &[Sqrt]);
    assert_refuse_may(&analyze(&o0(CLOSURE_CHAIN), "main"), &[Cos]);
    assert_refuse_may(&analyze(&o0(CLOSURE_PASSED), "main"), &[Tan]);
}

#[test]
fn p09_unreachable_closure_origins_do_not_pollute_a_reachable_closure_call() {
    assert_admit(&analyze(&unreachable_closure_origins(false), "main"));
    // Control: the same MakeClosure at a reachable offset is a hazard.
    assert_refuse_may(&analyze(&unreachable_closure_origins(true), "main"), &[Sin]);
}

#[test]
fn n15_work_accounting_is_exact_on_a_real_artifact() {
    // VIA_HELPER is straight-line: main -> helper -> cos. Expected work is
    // computed independently: instruction-stream bytes of both reached
    // functions (format decoder) plus their call-like sites (VM listing).
    let bytes = o0(VIA_HELPER);
    let decoded = decode_semcode(&bytes).expect("decodes");
    let stream: usize = decoded
        .functions
        .iter()
        .map(|env| env.code_slice.len() - env.instr_start_offset)
        .sum();
    let listing = sm_vm::disasm_semcode(&bytes).expect("VM disassembles");
    let sites = listing
        .lines()
        .filter(|l| {
            l.split(": ").nth(1).is_some_and(|text| {
                ["CALL ", "MAKE_CLOSURE ", "CLOSURE_CALL "]
                    .iter()
                    .any(|m| text.starts_with(m))
            })
        })
        .count();
    assert_eq!(decoded.functions.len(), 2);
    assert_eq!(sites, 2, "main calls helper, helper calls cos");
    let analysis = analyze(&bytes, "main");
    assert_refuse_may(&analysis, &[Cos]);
    assert_eq!(analysis.work_units_used(), (stream + sites) as u64);
}

#[test]
fn n12_work_is_measured_over_reachable_functions_only() {
    // Unreachable function bodies cost nothing (spec §4.1 rule 6).
    let small = analyze(&o0(CLEAN), "main");
    let big_dead = format!(
        "fn dead() {{\n{}    return;\n}}\n{CLEAN}",
        "    let y: f64 = sqrt(2.0);\n".repeat(200)
    );
    let with_dead = analyze(&o0(&big_dead), "main");
    assert_eq!(math(&small), math(&with_dead));
}

// ----------------------------------------------------- structural controls

#[test]
fn s01_malformed_semcode_is_a_verifier_rejection_before_analysis() {
    let bytes = o0(DIRECT_SQRT);
    let report = verify_semcode_token(&bytes[..bytes.len() - 3])
        .expect_err("truncated artifact never yields a token");
    assert_eq!(
        report.diagnostics[0].code,
        VerificationCode::TruncatedFunction
    );
    assert!(verify_semcode_token(&bytes[..4]).is_err());
}

#[test]
fn s02_missing_entry_remains_an_entry_resolution_failure() {
    let bytes = o0(CLEAN);
    let token = verify_semcode_token(&bytes).expect("verified");
    assert_eq!(
        token.require_entry("absent").map(|_| ()),
        Err(EntryResolutionError::MissingEntry {
            entry: "absent".to_string()
        })
    );
}

#[test]
fn s03_rev27_artifacts_remain_rejected_by_existing_verifier_rules() {
    let fixture: &[u8] = include_bytes!("vectors/q3_10_evr0_structural_only.smc");
    let report = verify_semcode_token(fixture).expect_err("rev27 never mints a token");
    assert_eq!(
        report.diagnostics[0].code,
        VerificationCode::EvidenceIssuerUnavailable
    );
}

#[test]
fn s04_cap_f64_math_alone_never_proves_host_math_safety() {
    let float_only =
        o0("fn main() {\n    let a: f64 = 1.5;\n    let b: f64 = a * 2.0;\n    return;\n}\n");
    let with_sqrt = o0(DIRECT_SQRT);
    for bytes in [&float_only, &with_sqrt] {
        let header = decode_semcode(bytes).expect("decodes").header;
        assert_ne!(header.capabilities & CAP_F64_MATH, 0, "both carry the bit");
    }
    assert_admit(&analyze(&float_only, "main"));
    assert_refuse_may(&analyze(&with_sqrt, "main"), &[Sqrt]);
}

#[test]
fn s05_local_execution_is_unchanged_by_analysis() {
    let src = "fn probe() -> f64 {\n    return sqrt(16.0);\n}\nfn main() {\n    return;\n}\n";
    let bytes = o0(src);
    let token = verify_semcode_token(&bytes).expect("verified");
    let entry = token.require_entry("probe").expect("entry");
    let before = sm_vm::run_verified_function_semcode_with_args(&entry, Vec::new());
    let analysis = analyze_host_math_free_draft1(&entry, &HostMathAnalysisLimits::draft1_default());
    assert_refuse_may(&analysis, &[Sqrt]);
    let after = sm_vm::run_verified_function_semcode_with_args(&entry, Vec::new());
    assert_eq!(before, Ok(Value::F64(4.0)), "LOCAL still runs host sqrt");
    assert_eq!(before, after);
}

// ---------------------------------------------- Q9a metadata invariance

#[test]
fn q01_debug_records_do_not_change_the_math_verdict() {
    for (name, src) in CORPUS {
        for opt in [OptLevel::O0, OptLevel::O1] {
            let plain = compile(src, opt, false);
            let debug = compile(src, opt, true);
            assert_ne!(plain, debug);
            assert_eq!(
                math(&analyze(&plain, "main")),
                math(&analyze(&debug, "main")),
                "{name} {opt:?}"
            );
        }
    }
    let plain = after_return();
    let debug = emit(
        &[ir_fn(
            "main",
            Vec::new(),
            vec![
                IrInstr::LoadF64 { dst: 0, val: 2.0 },
                ret(),
                ir_call("sqrt", vec![0]),
                ret(),
            ],
        )],
        true,
    );
    assert_ne!(plain, debug);
    assert_eq!(
        math(&analyze(&plain, "main")),
        math(&analyze(&debug, "main"))
    );
}

#[test]
fn q02_unreferenced_string_table_entries_do_not_change_the_math_verdict() {
    for (name, src) in CORPUS {
        let bytes = o0(src);
        let changed = append_unreferenced_string(&bytes, "main", "q9a-unrelated");
        assert_ne!(bytes, changed);
        assert_eq!(
            math(&analyze(&bytes, "main")),
            math(&analyze(&changed, "main")),
            "{name}"
        );
    }
}

#[test]
fn q03_string_literal_data_does_not_change_the_math_verdict() {
    let with_text = |text: &str| {
        emit(
            &[ir_fn(
                "main",
                Vec::new(),
                vec![
                    IrInstr::LoadText {
                        dst: 0,
                        val: text.to_string(),
                    },
                    IrInstr::LoadF64 { dst: 1, val: 2.0 },
                    ir_call("sqrt", vec![1]),
                    ret(),
                ],
            )],
            false,
        )
    };
    let (a, b) = (with_text("alpha"), with_text("sqrt tan pow"));
    assert_ne!(a, b);
    let (ra, rb) = (analyze(&a, "main"), analyze(&b, "main"));
    assert_eq!(math(&ra), math(&rb));
    assert_refuse_may(&ra, &[Sqrt]);
}

#[test]
fn q04_resource_limit_outcome_is_identical_across_debug_variants() {
    for (name, src) in CORPUS {
        let plain = compile(src, OptLevel::O0, false);
        let debug = compile(src, OptLevel::O0, true);
        let needed = analyze(&plain, "main").work_units_used();
        for budget in [1, needed.saturating_sub(1).max(1), needed] {
            let limits = HostMathAnalysisLimits::new(budget).expect("valid");
            assert_eq!(
                math(&analyze_with(&plain, "main", limits)),
                math(&analyze_with(&debug, "main", limits)),
                "{name} budget {budget}"
            );
        }
    }
}

// ------------------------------------------------- O0/O1 per artifact

/// `X ⊑ Y` of spec §7 on `Hazards`; `None` stands for `UNKNOWN`.
fn hazards(a: &HostMathAnalysis) -> Option<BTreeSet<HostMathBuiltin>> {
    (!a.hazards_unknown()).then(|| a.may().iter().copied().collect())
}

fn below(x: &Option<BTreeSet<HostMathBuiltin>>, y: &Option<BTreeSet<HostMathBuiltin>>) -> bool {
    match (x, y) {
        (_, None) => true,
        (None, Some(_)) => false,
        (Some(x), Some(y)) => x.is_subset(y),
    }
}

#[test]
fn o01_each_optimization_level_is_decided_on_its_own_artifact() {
    let mut om_holds = 0;
    for (name, src) in CORPUS {
        let (a0, a1) = (
            compile(src, OptLevel::O0, false),
            compile(src, OptLevel::O1, false),
        );
        let (r0, r1) = (analyze(&a0, "main"), analyze(&a1, "main"));
        // Each result is a property of its own bytes: re-analysis is stable.
        assert_eq!(math(&r0), math(&analyze(&a0, "main")));
        assert_eq!(math(&r1), math(&analyze(&a1, "main")));
        let holds = below(&hazards(&r1), &hazards(&r0));
        om_holds += usize::from(holds);
        println!(
            "S15C1_OM_PAIR {name} same_bytes={} O0={:?}/{:?} O1={:?}/{:?} om_holds={holds}",
            a0 == a1,
            r0.verdict(),
            r0.may(),
            r1.verdict(),
            r1.may()
        );
    }
    println!(
        "S15C1_OM_SUMMARY pairs={} om_holds={om_holds}",
        CORPUS.len()
    );
}

// ------------------------------------- verifier vs VM decoding agreement

/// An independent reference model built only from the VM's disassembler
/// (its own operand decoder): per-function CFG over `JMP`/`JMP_IF`/`RET`,
/// `CALL`/`MAKE_CLOSURE`/`CLOSURE_CALL` sites, then the C0 fixed point.
/// Call names come from each function's string table (`fn#n`).
fn vm_reference(bytes: &[u8], entry: &str) -> (BTreeSet<String>, bool) {
    let listing = sm_vm::disasm_semcode(bytes).expect("VM disassembles");
    let token = verify_semcode_token(bytes).expect("verified");
    let strings: BTreeMap<String, Vec<String>> = token.with_decoded_envelopes(|_, envs| {
        envs.iter()
            .map(|env| (env.name.clone(), env.strings.clone()))
            .collect()
    });
    enum Kind {
        Call(String),
        Make(String),
        ClosureCall,
        Jmp(usize),
        JmpIf(usize),
        Ret,
        Other,
    }
    let mut functions: BTreeMap<String, Vec<(usize, Kind)>> = BTreeMap::new();
    let mut current = String::new();
    for line in listing.lines() {
        if let Some(rest) = line.strip_prefix("fn ") {
            current = rest.split(':').next().unwrap().to_string();
            functions.insert(current.clone(), Vec::new());
            continue;
        }
        let Some((pc, text)) = line.trim_start().split_once(": ") else {
            continue;
        };
        let Ok(pc) = usize::from_str_radix(pc, 16) else {
            continue;
        };
        let words: Vec<&str> = text.split_whitespace().collect();
        let kind = match words[0] {
            "CALL" => {
                let sid: usize = words[3].trim_start_matches("fn#").parse().unwrap();
                Kind::Call(strings[&current][sid].clone())
            }
            "MAKE_CLOSURE" => Kind::Make(words[2].trim_end_matches(',').to_string()),
            "CLOSURE_CALL" => Kind::ClosureCall,
            "JMP" => Kind::Jmp(words[1].parse().unwrap()),
            "JMP_IF" => Kind::JmpIf(words[2].parse().unwrap()),
            "RET" => Kind::Ret,
            _ => Kind::Other,
        };
        functions.get_mut(&current).unwrap().push((pc, kind));
    }
    let reach = |instrs: &Vec<(usize, Kind)>| -> BTreeSet<usize> {
        let index: BTreeMap<usize, usize> = instrs
            .iter()
            .enumerate()
            .map(|(i, (pc, _))| (*pc, i))
            .collect();
        let mut seen = BTreeSet::new();
        let mut stack = vec![0usize];
        while let Some(pc) = stack.pop() {
            let Some(&i) = index.get(&pc) else { continue };
            if !seen.insert(pc) {
                continue;
            }
            let next = instrs.get(i + 1).map(|(pc, _)| *pc);
            match &instrs[i].1 {
                Kind::Ret => {}
                Kind::Jmp(target) => stack.push(*target),
                Kind::JmpIf(target) => stack.extend([Some(*target), next].into_iter().flatten()),
                _ => stack.extend(next),
            }
        }
        seen
    };
    let prohibited = ["abs", "cos", "pow", "sin", "sqrt", "tan"];
    let (mut reached, mut queue) = (BTreeSet::from([entry.to_string()]), vec![entry.to_string()]);
    let (mut may, mut targets, mut closure_called) = (BTreeSet::new(), BTreeSet::new(), false);
    while let Some(f) = queue.pop() {
        let instrs = &functions[&f];
        let live = reach(instrs);
        for (pc, kind) in instrs {
            if !live.contains(pc) {
                continue;
            }
            let mut add = |name: &String, reached: &mut BTreeSet<String>| {
                if reached.insert(name.clone()) {
                    queue.push(name.clone());
                }
            };
            match kind {
                Kind::Call(name) if functions.contains_key(name) => add(name, &mut reached),
                Kind::Call(name) if prohibited.contains(&name.as_str()) => {
                    may.insert(name.clone());
                }
                Kind::Make(name) => {
                    targets.insert(name.clone());
                    if closure_called {
                        add(name, &mut reached);
                    }
                }
                Kind::ClosureCall if !closure_called => {
                    closure_called = true;
                    for t in targets.clone() {
                        add(&t, &mut reached);
                    }
                }
                _ => {}
            }
        }
    }
    (may, closure_called && targets.is_empty())
}

#[test]
fn v01_verifier_and_vm_decoding_agree_on_hazards_across_the_corpus() {
    let artifacts = vm_corpus();
    let mut refusals = 0;
    for (name, bytes) in &artifacts {
        let analysis = analyze(bytes, "main");
        let (may, origin_unknown) = vm_reference(bytes, "main");
        let ours: BTreeSet<String> = analysis
            .may()
            .iter()
            .map(|b| b.name().to_string())
            .collect();
        assert_eq!(
            ours, may,
            "{name}: May differs from the VM-decoded reference"
        );
        assert_eq!(
            analysis
                .indeterminate()
                .contains(&HostMathIndeterminate::ClosureOriginUnknown),
            origin_unknown,
            "{name}"
        );
        refusals += usize::from(analysis.verdict() == HostMathVerdict::Refuse);
    }
    assert!(
        artifacts.len() >= 40 && refusals >= 28,
        "corpus too weak: {} artifacts, {refusals} refusals",
        artifacts.len()
    );
}

#[test]
fn v02_vm_execution_agrees_with_user_first_resolution() {
    let run = |src: &str| {
        let bytes = o0(src);
        let token = verify_semcode_token(&bytes).expect("verified");
        let entry = token.require_entry("probe").expect("entry");
        let analysis =
            analyze_host_math_free_draft1(&entry, &HostMathAnalysisLimits::draft1_default());
        let value =
            sm_vm::run_verified_function_semcode_with_args(&entry, Vec::new()).expect("LOCAL run");
        (analysis, value)
    };
    let main = "fn main() {\n    return;\n}\n";
    // builtin:sqrt is what the VM runs when no program function shadows it.
    let (a, v) = run(&format!(
        "fn probe() -> f64 {{\n    return sqrt(16.0);\n}}\n{main}"
    ));
    assert_refuse_may(&a, &[Sqrt]);
    assert_eq!(v, Value::F64(4.0));
    // A program `sqrt` is entered instead, exactly as the analysis resolves it.
    let (a, v) = run(&format!(
        "fn sqrt(x: f64) -> f64 {{\n    return x + 100.0;\n}}\nfn probe() -> f64 {{\n    return sqrt(16.0);\n}}\n{main}"
    ));
    assert_admit(&a);
    assert_eq!(v, Value::F64(116.0));
    // The closure target the analysis marks reachable is the one the VM runs.
    let (a, v) = run(&format!(
        "fn probe() -> f64 {{\n    let f: Closure(f64 -> f64) = (x => tan(x));\n    return f(1.0);\n}}\n{main}"
    ));
    assert_refuse_may(&a, &[Tan]);
    assert_eq!(v, Value::F64(1.0f64.tan()));
    // Several string-table entries: the VM runs exactly the builtins in May.
    let (a, v) = run(&format!(
        "fn probe() -> f64 {{\n    let t: f64 = 3.0;\n    let u: f64 = pow(t, 2.0);\n    return abs(-2.5) + u;\n}}\n{main}"
    ));
    assert_refuse_may(&a, &[Abs, Pow]);
    assert_eq!(v, Value::F64(11.5));
    // A closure returned by a guest function runs its target (abs) in the VM.
    let (a, v) = run(&format!(
        "fn make() -> Closure(f64 -> f64) {{\n    let f: Closure(f64 -> f64) = (x => abs(x));\n    return f;\n}}\nfn probe() -> f64 {{\n    let g: Closure(f64 -> f64) = make();\n    return g(-3.0);\n}}\n{main}"
    ));
    assert_refuse_may(&a, &[Abs]);
    assert_eq!(v, Value::F64(3.0));
}

/// Every artifact the VM-decoding comparisons range over.
fn vm_corpus() -> Vec<(String, Vec<u8>)> {
    let mut artifacts: Vec<(String, Vec<u8>)> = Vec::new();
    for (name, src) in CORPUS {
        for opt in [OptLevel::O0, OptLevel::O1] {
            artifacts.push((format!("{name}/{opt:?}"), compile(src, opt, false)));
        }
    }
    artifacts.push(("after_return".into(), after_return()));
    artifacts.push((
        "closure_without_origin".into(),
        closure_call_without_origin(),
    ));
    artifacts.push((
        "unreachable_closure_origins".into(),
        unreachable_closure_origins(false),
    ));
    artifacts.push((
        "reachable_closure_origin".into(),
        unreachable_closure_origins(true),
    ));
    artifacts
}

const VM_FIXTURE: &str = "crates/sm-verify/tests/fixtures/s15c1_vm_disasm_corpus.txt";

/// Renders, for every corpus artifact, its bytes and the VM disassembler's
/// listing. `sm-verify` cannot depend on `sm-vm` (no Cargo change), so its
/// crate-internal test compares its own per-offset facts against this
/// VM-produced fixture (`s15c1_vm_decoding_matches_analysis_per_offset`).
fn render_vm_fixture() -> String {
    let mut out = String::from(
        "# S15-C1 VM-decoding fixture: generated by tests/s15c1_host_math_analysis.rs\n\
         # (v03); regenerate with S15C1_UPDATE_VM_FIXTURE=1. Do not edit by hand.\n",
    );
    for (name, bytes) in vm_corpus() {
        let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
        let listing = sm_vm::disasm_semcode(&bytes).expect("VM disassembles");
        out.push_str(&format!("artifact {name}\nbytes {hex}\n{listing}end\n"));
    }
    out
}

#[test]
fn v03_vm_disassembly_fixture_is_current() {
    let actual = render_vm_fixture();
    if std::env::var("S15C1_UPDATE_VM_FIXTURE").as_deref() == Ok("1") {
        std::fs::write(VM_FIXTURE, &actual).expect("write fixture");
    }
    let expected = std::fs::read_to_string(VM_FIXTURE)
        .expect("fixture exists")
        .replace("\r\n", "\n");
    assert_eq!(
        actual, expected,
        "the VM's decoding of the corpus changed; regenerate the fixture and re-run sm-verify's equivalence test"
    );
}
