//! S15-C1: verifier-owned Executable Host Math Analysis for
//! `HOST_MATH_FREE@draft1` (`docs/spec/portable_math_profile_draft_v0.md`
//! §4-§6). Crate-internal: the only public surface is
//! `analyze_host_math_free_draft1` and its result types in `lib.rs`.
//!
//! The control-flow graph is re-derived from the verifier's own pieces, not
//! from a second decoder: `decode_operands` (operand shapes and call/closure
//! string references), `instruction_successors_of` (spec F9),
//! `verify_reachable_control_flow` (`Reach(f)`) and `resolve_call_target`
//! (spec F4, user-first). Only verifier-admitted tokens reach this module.

use super::{
    decode_operands, instruction_successors_of, resolve_call_target, verify_reachable_control_flow,
    CallTargetResolution, HostMathAnalysis, HostMathAnalysisLimits, HostMathBuiltin,
    HostMathIndeterminate, HostMathVerdict, MetadataCollector, NoCollect, VerifiedEntrySemCode,
};
use sm_format::semcode_decode::DecodedFunctionEnvelope;
use sm_format::semcode_format::{read_u8, CallableSignature, CallableValueFamily, Opcode};
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

/// A call-like instruction of one function, by instruction offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Site<'a> {
    Call { offset: usize, callee: &'a str },
    MakeClosure { offset: usize, target: &'a str },
    ClosureCall { offset: usize },
}

impl Site<'_> {
    fn offset(&self) -> usize {
        match *self {
            Site::Call { offset, .. }
            | Site::MakeClosure { offset, .. }
            | Site::ClosureCall { offset } => offset,
        }
    }
}

/// What the analysis needs from one function: its verifier `Reach(f)` and
/// every decoded call-like site (reachable or not, in offset order).
#[derive(Debug, Clone)]
pub(super) struct FunctionFacts<'a> {
    pub(super) reachable: HashSet<usize>,
    pub(super) sites: Vec<Site<'a>>,
}

/// The program the fixed point runs over. `stream_len` is charged before
/// `facts` is derived, so the work bound also caps decoding.
pub(super) trait Program<'a> {
    fn contains(&self, name: &str) -> bool;
    fn stream_len(&self, name: &str) -> usize;
    /// `None`: the verifier's control flow could not be re-derived.
    fn facts(&self, name: &str) -> Option<FunctionFacts<'a>>;
}

enum CallLike {
    Call,
    MakeClosure,
    ClosureCall,
    Neutral,
}

/// Exhaustive on purpose: an opcode added to the format must be classified
/// here before this crate compiles again (spec F10: `Call` and `ClosureCall`
/// are today's only call-like opcodes; `MakeClosure` names a closure target).
fn call_like(opcode: Opcode) -> CallLike {
    use Opcode::*;
    match opcode {
        Call => CallLike::Call,
        MakeClosure => CallLike::MakeClosure,
        ClosureCall => CallLike::ClosureCall,
        LoadQ | LoadBool | LoadI32 | AddI32 | SubI32 | MulI32 | DivI32 | ModI32 | ConcatText
        | LoadU32 | LoadVar | StoreVar | QAnd | QOr | QNot | QImpl | BoolAnd | BoolOr | BoolNot
        | QTruthAnd | QTruthOr | QTruthNot | QTruthImpl | CmpEq | CmpNe | CmpI32Lt | CmpI32Le
        | CmpU32Lt | CmpU32Le | AddU32 | SubU32 | MulU32 | DivU32 | ModU32 | Jmp | JmpIf | Ret
        | Assert | MakeTuple | TupleGet | MakeRecord | RecordGet | MakeAdt | AdtTag | AdtGet
        | LoadF64 | AddF64 | SubF64 | MulF64 | DivF64 | LoadFx | AddFx | SubFx | MulFx | DivFx
        | LoadText | MakeSequence | SequenceGet | SequenceLen | SequenceIsEmpty
        | SequenceContains | SequencePush | SequencePrepend | SequencePop | MapEmpty
        | MapContains | MapGet | MapSet | RngSeed | RngNextI32 | GateRead | GateWrite
        | PulseEmit | StateQuery | StateUpdate | EventPost | ClockRead => CallLike::Neutral,
    }
}

/// Re-derives `FunctionFacts` for one verified function with the verifier's
/// decoder and CFG. Mirrors the first walk of `verify_function_code`.
pub(super) fn function_facts<'a>(
    env: &'a DecodedFunctionEnvelope<'_>,
) -> Option<FunctionFacts<'a>> {
    let name = env.name.as_str();
    let code = env.code_slice;
    let instr_start = env.instr_start_offset;
    let mut cursor = instr_start;
    let mut instr_starts = Vec::new();
    let mut successors = Vec::new();
    let mut sites = Vec::new();
    let mut jump_targets = Vec::new();
    let mut string_refs = Vec::new();
    let mut call_argcs = Vec::new();
    while cursor < code.len() {
        let offset = cursor - instr_start;
        instr_starts.push(offset);
        let opcode = Opcode::from_byte(read_u8(code, &mut cursor).ok()?).ok()?;
        let jumps_before = jump_targets.len();
        let refs_before = string_refs.len();
        let mut metadata = MetadataCollector {
            jump_targets: &mut jump_targets,
            string_refs: &mut string_refs,
            call_argcs: &mut call_argcs,
        };
        decode_operands(
            name,
            code,
            &mut cursor,
            offset,
            opcode,
            true,
            &mut NoCollect,
            &mut metadata,
        )
        .ok()?;
        let next_offset = cursor - instr_start;
        successors.push(instruction_successors_of(
            opcode,
            jump_targets.get(jumps_before).copied(),
            next_offset,
        )?);
        let named = |usage: &str| -> Option<&'a str> {
            let &(_, sid, used_as) = string_refs.get(refs_before)?;
            (used_as == usage).then(|| env.strings.get(sid).map(String::as_str))?
        };
        match call_like(opcode) {
            CallLike::Call => sites.push(Site::Call {
                offset,
                callee: named("call target")?,
            }),
            CallLike::MakeClosure => sites.push(Site::MakeClosure {
                offset,
                target: named("closure function name")?,
            }),
            CallLike::ClosureCall => sites.push(Site::ClosureCall { offset }),
            CallLike::Neutral => {}
        }
    }
    let instr_len = code.len().saturating_sub(instr_start);
    let reachable =
        verify_reachable_control_flow(name, instr_len, &instr_starts, &successors).ok()?;
    Some(FunctionFacts { reachable, sites })
}

struct VerifiedProgramView<'a, 'b> {
    functions: HashMap<&'a str, &'a DecodedFunctionEnvelope<'b>>,
}

impl<'a> Program<'a> for VerifiedProgramView<'a, '_> {
    fn contains(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }
    fn stream_len(&self, name: &str) -> usize {
        self.functions.get(name).map_or(0, |env| {
            env.code_slice.len().saturating_sub(env.instr_start_offset)
        })
    }
    fn facts(&self, name: &str) -> Option<FunctionFacts<'a>> {
        function_facts(*self.functions.get(name)?)
    }
}

pub(super) fn analyze(
    entry: &VerifiedEntrySemCode<'_, '_>,
    limits: &HostMathAnalysisLimits,
) -> HostMathAnalysis {
    let artifact = entry.artifact();
    let program = VerifiedProgramView {
        functions: artifact
            .decoded
            .iter()
            .map(|env| (env.name.as_str(), env))
            .collect(),
    };
    let signature = program
        .functions
        .get(entry.entry())
        .and_then(|env| env.signature.as_ref());
    let decision = decide(&program, entry.entry(), signature, limits.max_work_units());
    let verdict = if decision.may.is_empty() && decision.indeterminate.is_empty() {
        HostMathVerdict::Admit
    } else {
        HostMathVerdict::Refuse
    };
    HostMathAnalysis {
        artifact_hash: entry.artifact_hash(),
        entry: entry.entry().to_string(),
        limits: *limits,
        verdict,
        may: decision.may.into_iter().collect(),
        indeterminate: decision.indeterminate.into_iter().collect(),
        work_units_used: decision.work_units_used,
    }
}

pub(super) struct Decision {
    pub(super) may: BTreeSet<HostMathBuiltin>,
    pub(super) indeterminate: BTreeSet<HostMathIndeterminate>,
    pub(super) work_units_used: u64,
}

fn prohibited_builtin(name: &str) -> Option<HostMathBuiltin> {
    Some(match name {
        "abs" => HostMathBuiltin::Abs,
        "cos" => HostMathBuiltin::Cos,
        "pow" => HostMathBuiltin::Pow,
        "sin" => HostMathBuiltin::Sin,
        "sqrt" => HostMathBuiltin::Sqrt,
        "tan" => HostMathBuiltin::Tan,
        _ => return None,
    })
}

/// Spec §4.1 rule 6: SemCode records no element or field types, so no
/// container family can be proved callable-free.
fn family_can_carry_callable(family: CallableValueFamily) -> bool {
    use CallableValueFamily::*;
    match family {
        Closure | Tuple | Record | Adt | Sequence | Map => true,
        Quad | Bool | Text | I32 | U32 | Fx | F64 | Unit | Bytes => false,
    }
}

/// The C0 fixed point of spec §4.1 rules 3-7 and the hazard sets of §5.
/// Deterministic: FIFO order of discovery, sites in offset order, closure
/// targets in name order, no hash-map iteration.
pub(super) fn decide<'a>(
    program: &dyn Program<'a>,
    entry: &str,
    entry_signature: Option<&CallableSignature>,
    max_work_units: u64,
) -> Decision {
    let mut may = BTreeSet::new();
    let mut indeterminate = BTreeSet::new();
    match entry_signature {
        None => {
            indeterminate.insert(HostMathIndeterminate::EntrySignatureUnavailable);
        }
        Some(signature) => {
            for (index, family) in signature.families.iter().enumerate() {
                if family_can_carry_callable(*family) {
                    indeterminate.insert(HostMathIndeterminate::CallableEntryParameter { index });
                }
            }
        }
    }

    let mut work: u64 = 0;
    let mut charge = |units: u64| -> bool {
        work = work.saturating_add(units);
        work <= max_work_units
    };
    let mut reached: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<String> = VecDeque::new();
    let mut closure_targets: BTreeSet<&'a str> = BTreeSet::new();
    let mut closure_call_reached = false;
    fn enqueue(name: &str, reached: &mut HashSet<String>, queue: &mut VecDeque<String>) {
        if reached.insert(name.to_string()) {
            queue.push_back(name.to_string());
        }
    }
    enqueue(entry, &mut reached, &mut queue);

    let mut exhausted = false;
    'functions: while let Some(function) = queue.pop_front() {
        if !charge(program.stream_len(&function) as u64) {
            exhausted = true;
            break;
        }
        let Some(facts) = program.facts(&function) else {
            indeterminate.insert(HostMathIndeterminate::ResolutionMismatch);
            continue;
        };
        for site in &facts.sites {
            if !facts.reachable.contains(&site.offset()) {
                continue;
            }
            if !charge(1) {
                exhausted = true;
                break 'functions;
            }
            match *site {
                Site::Call { callee, .. } => {
                    match resolve_call_target(program.contains(callee), callee, true) {
                        CallTargetResolution::ProgramFunction => {
                            enqueue(callee, &mut reached, &mut queue)
                        }
                        CallTargetResolution::Builtin { .. } => {
                            if let Some(builtin) = prohibited_builtin(callee) {
                                may.insert(builtin);
                            }
                        }
                        CallTargetResolution::Unresolved => {
                            indeterminate.insert(HostMathIndeterminate::ResolutionMismatch);
                        }
                    }
                }
                Site::MakeClosure { target, .. } => {
                    if resolve_call_target(program.contains(target), target, false)
                        != CallTargetResolution::ProgramFunction
                    {
                        indeterminate.insert(HostMathIndeterminate::ResolutionMismatch);
                    } else if closure_targets.insert(target) && closure_call_reached {
                        enqueue(target, &mut reached, &mut queue);
                    }
                }
                Site::ClosureCall { .. } => {
                    if !closure_call_reached {
                        closure_call_reached = true;
                        for target in &closure_targets {
                            enqueue(target, &mut reached, &mut queue);
                        }
                    }
                }
            }
        }
    }

    if exhausted {
        indeterminate.insert(HostMathIndeterminate::AnalysisLimit);
    } else if closure_call_reached && closure_targets.is_empty() {
        indeterminate.insert(HostMathIndeterminate::ClosureOriginUnknown);
    }
    Decision {
        may,
        indeterminate,
        work_units_used: work.min(max_work_units),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        verify_function_code, verify_semcode_token, AnalysisWorkMeter, HostMathLimitsError,
        VerificationLimits,
    };
    use sm_ir::{
        compile_program_to_semcode_with_options_debug, emit_ir_to_semcode, CompileProfile,
        IrFunction, IrInstr, OptLevel,
    };
    use sm_runtime_core::RuntimeQuotas;

    const SOURCES: &[&str] = &[
        "fn main() {\n    let y: f64 = sqrt(16.0);\n    return;\n}\n",
        "fn sqrt(x: f64) -> f64 {\n    return x;\n}\nfn main() {\n    let a: i32 = 2;\n    let x: f64 = 2.0;\n    if a > 1 {\n        let b: f64 = sqrt(x);\n    } else {\n        let c: f64 = tan(x);\n    }\n    return;\n}\n",
        "fn main() {\n    let add: Closure(f64 -> f64) = (x => x + 1.0);\n    let y: f64 = add(2.0);\n    return;\n}\n",
        "fn down(n: i32) -> i32 {\n    if n == 0 {\n        return 0;\n    }\n    return down(n - 1);\n}\nfn main() {\n    let r: i32 = down(3);\n    return;\n}\n",
    ];

    fn ir(name: &str, params: Vec<CallableValueFamily>, instrs: Vec<IrInstr>) -> IrFunction {
        IrFunction {
            name: name.to_string(),
            instrs,
            ownership_events: Vec::new(),
            params,
        }
    }

    fn label(name: &str) -> IrInstr {
        IrInstr::Label {
            name: name.to_string(),
        }
    }

    fn call(name: &str, args: Vec<u16>) -> IrInstr {
        IrInstr::Call {
            dst: None,
            name: name.to_string(),
            args,
        }
    }

    fn corpus() -> Vec<Vec<u8>> {
        let mut out = Vec::new();
        for src in SOURCES {
            for opt in [OptLevel::O0, OptLevel::O1] {
                out.push(
                    compile_program_to_semcode_with_options_debug(
                        src,
                        CompileProfile::RustLike,
                        opt,
                        false,
                    )
                    .expect("corpus source compiles"),
                );
            }
        }
        // Dead code after `Ret`, a `Jmp` over a call, and a `JmpIf` diamond.
        out.push(
            emit_ir_to_semcode(
                &[ir(
                    "main",
                    Vec::new(),
                    vec![
                        IrInstr::LoadF64 { dst: 0, val: 4.0 },
                        IrInstr::LoadBool { dst: 1, val: true },
                        IrInstr::JmpIf {
                            cond: 1,
                            label: "skip".to_string(),
                        },
                        call("cos", vec![0]),
                        label("skip"),
                        IrInstr::Jmp {
                            label: "end".to_string(),
                        },
                        call("sin", vec![0]),
                        label("end"),
                        IrInstr::Ret { src: None },
                        call("sqrt", vec![0]),
                        IrInstr::Ret { src: None },
                    ],
                )],
                false,
            )
            .expect("crafted IR emits"),
        );
        out
    }

    /// Spec §4.1 rule 1 and §4.2: the analysis's `Reach(f)` and call/closure
    /// targets equal the verifier's own, function by function.
    #[test]
    fn s15c1_analysis_cfg_and_call_targets_equal_the_verifiers() {
        let (mut functions, mut calls, mut closures, mut closure_calls, mut unreachable) =
            (0, 0, 0, 0, 0);
        for bytes in corpus() {
            verify_semcode_token(&bytes).expect("corpus artifact is verifier-admitted");
            let decoded = sm_format::semcode_decode::decode_semcode(&bytes).expect("decodes");
            for env in &decoded.functions {
                let mut meter = AnalysisWorkMeter::new(usize::MAX);
                let pending = verify_function_code(
                    env,
                    &decoded.header,
                    decoded.adt_descriptors.as_ref(),
                    &RuntimeQuotas::verified_local(),
                    &VerificationLimits::default_profile(),
                    &mut meter,
                )
                .unwrap_or_else(|_| panic!("verifier accepts {}", env.name));
                let facts = function_facts(env).expect("analysis re-derives the CFG");
                assert_eq!(
                    facts.reachable, pending.reachable_offsets,
                    "Reach({}) differs from the verifier's",
                    env.name
                );
                let mut ours: Vec<(usize, String, bool)> = Vec::new();
                for site in &facts.sites {
                    match *site {
                        Site::Call { offset, callee } => ours.push((offset, callee.into(), true)),
                        Site::MakeClosure { offset, target } => {
                            ours.push((offset, target.into(), false))
                        }
                        Site::ClosureCall { .. } => closure_calls += 1,
                    }
                }
                let mut theirs: Vec<(usize, String, bool)> = pending
                    .call_targets
                    .iter()
                    .map(|(offset, name, builtin, _)| (*offset, name.clone(), *builtin))
                    .collect();
                ours.sort();
                theirs.sort();
                assert_eq!(ours, theirs, "call targets of {} differ", env.name);
                functions += 1;
                calls += ours.iter().filter(|site| site.2).count();
                closures += ours.iter().filter(|site| !site.2).count();
                unreachable += facts
                    .sites
                    .iter()
                    .filter(|site| !facts.reachable.contains(&site.offset()))
                    .count();
            }
        }
        // The corpus must actually exercise every compared shape.
        assert!(functions >= 12, "functions {functions}");
        assert!(calls >= 8, "calls {calls}");
        assert!(closures >= 2, "closures {closures}");
        assert!(closure_calls >= 2, "closure calls {closure_calls}");
        assert!(unreachable >= 2, "unreachable call sites {unreachable}");
    }

    #[derive(Clone)]
    struct Synthetic(Vec<(&'static str, usize, Option<FunctionFacts<'static>>)>);

    impl Program<'static> for Synthetic {
        fn contains(&self, name: &str) -> bool {
            self.0.iter().any(|f| f.0 == name)
        }
        fn stream_len(&self, name: &str) -> usize {
            self.0.iter().find(|f| f.0 == name).map_or(0, |f| f.1)
        }
        fn facts(&self, name: &str) -> Option<FunctionFacts<'static>> {
            self.0.iter().find(|f| f.0 == name)?.2.clone()
        }
    }

    fn all_reachable(sites: Vec<Site<'static>>) -> Option<FunctionFacts<'static>> {
        Some(FunctionFacts {
            reachable: sites.iter().map(Site::offset).collect(),
            sites,
        })
    }

    fn no_params() -> CallableSignature {
        CallableSignature {
            families: Vec::new(),
        }
    }

    /// Defensive paths a verifier-admitted artifact cannot reach today
    /// (evidence class INTERNAL_FACTS, not an artifact verdict).
    #[test]
    fn s15c1_unresolvable_targets_and_lost_cfg_are_indeterminate() {
        let unresolved = Synthetic(vec![(
            "main",
            1,
            all_reachable(vec![Site::Call {
                offset: 0,
                callee: "nope",
            }]),
        )]);
        let d = decide(&unresolved, "main", Some(&no_params()), 100);
        assert_eq!(
            d.indeterminate.into_iter().collect::<Vec<_>>(),
            vec![HostMathIndeterminate::ResolutionMismatch]
        );
        let lost = Synthetic(vec![("main", 1, None)]);
        let d = decide(&lost, "main", Some(&no_params()), 100);
        assert!(d
            .indeterminate
            .contains(&HostMathIndeterminate::ResolutionMismatch));
        let builtin_closure = Synthetic(vec![(
            "main",
            1,
            all_reachable(vec![Site::MakeClosure {
                offset: 0,
                target: "sqrt",
            }]),
        )]);
        let d = decide(&builtin_closure, "main", Some(&no_params()), 100);
        assert!(d
            .indeterminate
            .contains(&HostMathIndeterminate::ResolutionMismatch));
        assert!(d.may.is_empty(), "a closure never resolves to a builtin");
    }

    /// Rules 4 and 5: a `ClosureCall` seen before the `MakeClosure` that
    /// feeds it still makes the target reachable.
    #[test]
    fn s15c1_closure_fixed_point_is_order_independent() {
        let program = Synthetic(vec![
            (
                "main",
                1,
                all_reachable(vec![
                    Site::ClosureCall { offset: 0 },
                    Site::Call {
                        offset: 1,
                        callee: "maker",
                    },
                ]),
            ),
            (
                "maker",
                1,
                all_reachable(vec![Site::MakeClosure {
                    offset: 0,
                    target: "hazard",
                }]),
            ),
            (
                "hazard",
                1,
                all_reachable(vec![Site::Call {
                    offset: 0,
                    callee: "pow",
                }]),
            ),
        ]);
        let d = decide(&program, "main", Some(&no_params()), 100);
        assert_eq!(
            d.may.into_iter().collect::<Vec<_>>(),
            vec![HostMathBuiltin::Pow]
        );
        assert!(d.indeterminate.is_empty());
    }

    /// Rule 6: the work bound is exact and exhaustion is a refusal.
    #[test]
    fn s15c1_work_bound_is_exact_and_fails_closed() {
        let program = Synthetic(vec![
            (
                "main",
                10,
                all_reachable(vec![Site::Call {
                    offset: 0,
                    callee: "leaf",
                }]),
            ),
            ("leaf", 5, all_reachable(Vec::new())),
        ]);
        let enough = decide(&program, "main", Some(&no_params()), 16);
        assert!(enough.may.is_empty() && enough.indeterminate.is_empty());
        assert_eq!(enough.work_units_used, 16);
        let short = decide(&program, "main", Some(&no_params()), 15);
        assert_eq!(
            short.indeterminate.into_iter().collect::<Vec<_>>(),
            vec![HostMathIndeterminate::AnalysisLimit]
        );
        assert_eq!(short.work_units_used, 15);
    }

    #[test]
    fn s15c1_limits_have_a_fixed_ceiling() {
        assert_eq!(
            HostMathAnalysisLimits::new(0),
            Err(HostMathLimitsError::ZeroWorkUnits)
        );
        let ceiling = HostMathAnalysisLimits::MAX_WORK_UNITS_CEILING;
        assert_eq!(
            HostMathAnalysisLimits::new(ceiling).map(|l| l.max_work_units()),
            Ok(ceiling)
        );
        for requested in [ceiling + 1, u64::MAX] {
            assert_eq!(
                HostMathAnalysisLimits::new(requested),
                Err(HostMathLimitsError::AboveCeiling { requested, ceiling })
            );
        }
        assert!(HostMathAnalysisLimits::draft1_default().max_work_units() <= ceiling);
    }

    /// One function of a VM listing: its name and `(pc, mnemonic words)`.
    type VmFunction = (String, Vec<(usize, Vec<String>)>);

    /// Spec §4.1 rule 1, §4.2 and the GO's verifier-vs-VM decoding obligation,
    /// per function and per offset: the VM disassembler's own operand
    /// decoding (fixture written and kept current by the root test
    /// `v03_vm_disassembly_fixture_is_current`) must yield exactly the
    /// analysis's `Reach(f)` - recomputed here from the VM's JMP/JMP_IF/RET
    /// decoding, not from `instruction_successors_of` - and exactly its
    /// `Call`/`MakeClosure`/`ClosureCall` sites and targets.
    #[test]
    fn s15c1_vm_decoding_matches_analysis_per_offset() {
        let fixture =
            include_str!("../tests/fixtures/s15c1_vm_disasm_corpus.txt").replace("\r\n", "\n");
        let (mut artifacts, mut functions, mut sites_checked) = (0, 0, 0);
        for block in fixture.split("artifact ").skip(1) {
            let mut lines = block.lines();
            let name = lines.next().expect("artifact name");
            let hex = lines
                .next()
                .and_then(|l| l.strip_prefix("bytes "))
                .expect("bytes line");
            let bytes: Vec<u8> = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).expect("hex"))
                .collect();
            verify_semcode_token(&bytes).unwrap_or_else(|_| panic!("{name} verifies"));
            let decoded = sm_format::semcode_decode::decode_semcode(&bytes).expect("decodes");
            let mut vm: Vec<VmFunction> = Vec::new();
            for line in lines.take_while(|l| *l != "end") {
                if let Some(rest) = line.strip_prefix("fn ") {
                    vm.push((rest.split(':').next().unwrap().to_string(), Vec::new()));
                } else if let Some((pc, text)) = line.trim_start().split_once(": ") {
                    if let (Ok(pc), Some(f)) = (usize::from_str_radix(pc, 16), vm.last_mut()) {
                        f.1.push((pc, text.split_whitespace().map(str::to_string).collect()));
                    }
                }
            }
            // A bijection, not just equal counts: unique listing names whose
            // set is exactly the verifier-decoded function-name set.
            let vm_names: std::collections::BTreeSet<&str> =
                vm.iter().map(|(f, _)| f.as_str()).collect();
            let decoded_names: std::collections::BTreeSet<&str> = decoded
                .functions
                .iter()
                .map(|env| env.name.as_str())
                .collect();
            assert_eq!(vm_names.len(), vm.len(), "{name}: duplicate listing names");
            assert_eq!(vm_names, decoded_names, "{name}: function sets differ");
            for (fname, instrs) in &vm {
                let env = decoded
                    .functions
                    .iter()
                    .find(|env| &env.name == fname)
                    .unwrap_or_else(|| panic!("{name}: {fname} missing in verifier decode"));
                let facts = function_facts(env).expect("facts");
                // Reach from the VM's own control-flow decoding.
                let index: HashMap<usize, usize> = instrs
                    .iter()
                    .enumerate()
                    .map(|(i, (pc, _))| (*pc, i))
                    .collect();
                let mut vm_reach = HashSet::new();
                let mut stack = vec![0usize];
                while let Some(pc) = stack.pop() {
                    let Some(&i) = index.get(&pc) else { continue };
                    if !vm_reach.insert(pc) {
                        continue;
                    }
                    let next = instrs.get(i + 1).map(|(pc, _)| *pc);
                    let words = &instrs[i].1;
                    match words[0].as_str() {
                        "RET" => {}
                        "JMP" => stack.push(words[1].parse().unwrap()),
                        "JMP_IF" => {
                            stack.push(words[2].parse().unwrap());
                            stack.extend(next);
                        }
                        _ => stack.extend(next),
                    }
                }
                assert_eq!(
                    vm_reach, facts.reachable,
                    "{name}: Reach({fname}) VM vs analysis"
                );
                let mut vm_sites: Vec<(usize, &str, String)> = Vec::new();
                for (pc, words) in instrs {
                    match words[0].as_str() {
                        "CALL" => {
                            let sid: usize = words[3].trim_start_matches("fn#").parse().unwrap();
                            vm_sites.push((*pc, "call", env.strings[sid].clone()));
                        }
                        "MAKE_CLOSURE" => {
                            vm_sites.push((*pc, "make", words[2].trim_end_matches(',').to_string()))
                        }
                        "CLOSURE_CALL" => vm_sites.push((*pc, "closure_call", String::new())),
                        _ => {}
                    }
                }
                let ours: Vec<(usize, &str, String)> = facts
                    .sites
                    .iter()
                    .map(|site| match *site {
                        Site::Call { offset, callee } => (offset, "call", callee.to_string()),
                        Site::MakeClosure { offset, target } => {
                            (offset, "make", target.to_string())
                        }
                        Site::ClosureCall { offset } => (offset, "closure_call", String::new()),
                    })
                    .collect();
                assert_eq!(vm_sites, ours, "{name}: sites of {fname} VM vs analysis");
                functions += 1;
                sites_checked += ours.len();
            }
            artifacts += 1;
        }
        assert!(
            artifacts >= 40 && functions >= 70 && sites_checked >= 80,
            "fixture too weak: {artifacts} artifacts, {functions} functions, {sites_checked} sites"
        );
    }
}
