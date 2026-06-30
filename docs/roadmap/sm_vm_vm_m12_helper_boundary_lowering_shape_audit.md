# sm-vm VM-M12 Helper-Boundary Lowering Shape Audit

## Status

VM-M12 audits helper-boundary lowering shape after VM-M11 identified helper argument/result staging as the strongest stable scalar movement signal.

This document does not approve VM optimization, lowering changes, SemCode changes, fixture changes, or runtime changes.

## Context

- VM-M9 first-generation helper pair strengthened the helper-boundary hypothesis.
- VM-M11 G2 helper single-call kept the signal stable.
- VM-M11 G2 call-chain amplified the signal.
- VM-M12 inspects source, lowering, and VM execution shape.

## Method

- Re-read VM-M9 and VM-M11 evidence.
- Inspected helper-boundary fixtures.
- Inspected call, lowering, opcode, frame, and local-slot paths.
- Re-ran `vm-profile` workload tests.
- Did not modify code, tests, fixtures, VM, verifier, SemCode, parser, or lowering.

## Commands

- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads -- --nocapture`
- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads -- --ignored --nocapture`
- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads`
- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads -- --ignored`
- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads --no-run`
- `cargo clippy -p sm-vm --all-targets --all-features -- -D warnings`
- `cargo check -p sm-vm --no-default-features`
- `git diff --check`
- `cargo fmt --check`
- `git status --short`

## Evidence Boundary

Pair equivalence is enforced by matching fixture-local assertions, not by a shared returned-value comparison at the Rust harness level.

VM-M12 does not introduce result-inspection APIs.

## Helper Evidence Summary

| Fixture Pair | Helper Scalar % | Inline Scalar % | Delta Ratio | Delta Count | Signal |
|---|---:|---:|---:|---:|---|
| VM-M9 helper boundary (`scalar_helper_boundary_helper.sm` vs `scalar_helper_boundary_inline.sm`) | 40.09% | 36.72% | 3.37% | 128 | Helper-heavy higher |
| VM-M11 G2 helper single-call (`scalar_helper_boundary_single_call_helper.sm` vs `scalar_helper_boundary_single_call_inline.sm`) | 37.16% | 34.56% | 2.60% | 32 | Helper-heavy higher |
| VM-M11 G2 helper call-chain (`scalar_helper_boundary_call_chain_helper.sm` vs `scalar_helper_boundary_call_chain_inline.sm`) | 42.78% | 36.72% | 6.06% | 256 | Helper-heavy higher |

## Source Shape Comparison

| Pair | Helper calls per iteration | Helper depth | Return type | Caller local staging | Final assertions | Notes |
|---|---:|---:|---|---|---|---|
| VM-M9 helper boundary | 2 | 1 | `quad` | `score`, `merged_count`, `checksum`, `state`, `next` | `score > 0`, `merged_count > 0`, `checksum == 120` | Two helper calls per loop iteration; inline variant keeps the same semantics without helper calls. |
| G2 single-call helper boundary | 1 | 1 | `quad` | `score`, `hit_count`, `checksum`, `state` | `score == 20`, `hit_count == 2`, `checksum == 28` | Cleaner one-call pair; helper pressure remains visible even with smaller call density. |
| G2 call-chain helper boundary | 2 | 2 | `i32` then `quad` | `score`, `chain_hits`, `checksum`, `class`, `state`, `next_class`, `next_state` | `score == 40`, `chain_hits > 0`, `checksum == 120` | Small helper chain amplifies the helper-boundary effect. |

## Lowering / VM Path Inventory

| Area | File / Symbol | What it controls | Relevance to scalar movement |
|---|---|---|---|
| Frontend call normalization | `crates/sm-front/src/lib.rs` / `reorder_call_args` | Normalizes call argument order before later compilation stages | Affects the shape of argument evaluation before lowering, which can change staging pressure around helper calls. |
| Call typing / resolution | `crates/sm-front/src/typecheck.rs` / `Expr::Call` branch | Resolves and typechecks function-call expressions | Establishes the call-shaped source form that later lowering sees. |
| SemCode compilation entrypoint | `crates/sm-ir/src/lib.rs` / `compile_program_to_semcode` | Front door from frontend IR into SemCode emission | Provides the compilation path used by the profiling fixtures. |
| Lowering of calls and returns | `crates/sm-ir/src/legacy_lowering.rs` / `lower_expr`, `lower_expr_with_expected`, `IrInstr::Call`, `IrInstr::Ret` | Lowers source expressions into IR instructions, including function calls and returns | This is the clearest source-side location where helper boundaries can become staged loads/stores before and after calls. |
| Lowering of locals | `crates/sm-ir/src/legacy_lowering.rs` / `IrInstr::LoadVar`, `IrInstr::StoreVar` | Emits explicit local load/store IR | Direct source-side staging point for helper arguments, helper results, and caller locals. |
| VM frame setup and call execution | `crates/sm-vm/src/semcode_vm.rs` / `Frame`, `push_frame`, `Opcode::Call`, `get_reg` | Creates callee frames and copies call arguments into callee registers | Confirms that helper boundaries move values across a frame boundary in the runtime path. |
| VM return handling | `crates/sm-vm/src/semcode_vm.rs` / `Opcode::Ret`, `write_reg` | Returns values to the caller and writes them into the caller register file | Confirms that return-value staging is a real runtime path, not just a source-shape artifact. |
| VM local load/store execution | `crates/sm-vm/src/semcode_vm.rs` / `Opcode::LoadVar`, `Opcode::StoreVar` | Reads and writes local slots | Direct runtime mechanism for the scalar movement seen in the profiling output. |

## Helper-Boundary Mechanism Analysis

### Argument local load

The helper fixtures show that arguments are evaluated in the caller before the callee frame is entered. The lowering path and the VM call path both support this as a real source of scalar movement.

Confidence: Supported by lowering inspection, supported by VM execution inspection.

### Parameter/local initialization

`Opcode::Call` pushes a new frame and copies call arguments into callee registers. That means helper-boundary pressure is not only caller-side; the callee context also receives staged values.

Confidence: Supported by VM execution inspection.

### Return-value staging

The helper fixtures consistently show `Ret` among the top opcodes, and `Opcode::Ret` writes back into the caller context when a return destination exists. This is a plausible contributor to the helper-boundary delta.

Confidence: Supported by profile, supported by VM execution inspection.

### Caller result binding

The caller-side locals in the fixtures (`checksum`, `score`, `hit_count`, `chain_hits`, `state`, `next_state`) provide the binding points where returned helper values are recorded and later reused.

Confidence: Confirmed by source, supported by lowering inspection.

### Call-chain repeated staging

The call-chain helper pair has the largest delta in the evidence set. That is consistent with repeated staging of an intermediate helper result before the final value is produced.

Confidence: Confirmed by profile, supported by source.

### Helper-internal branch classification

The helper bodies are not purely arithmetic; they include state shaping and classification-like behavior. That means some helper-boundary pressure may be coming from work inside the helper body, not only from the boundary itself.

Confidence: Hypothesis.

### Frame/local slot movement

The VM call/return path explicitly moves values through frame registers and local slots, and the scalar-movement opcodes operate on the same local state. This makes frame/local movement a credible mechanism for the measured signal.

Confidence: Supported by VM execution inspection.

## Candidate Mechanisms

| Mechanism | Evidence | Confidence | Follow-up needed |
|---|---|---|---|
| Argument local load | Helper variants stay above inline in VM-M9 and both VM-M11 G2 pairs. | Strong | Compare a future helper-light pair that preserves the same callee shape but minimizes caller-side staging. |
| Parameter/local initialization | `Opcode::Call` pushes a frame and copies arguments into callee registers. | Medium | Inspect whether callee-side initialization can be separated from caller-side loads in a future audit. |
| Return-value staging | `Opcode::Ret` writes back to the caller; helper fixtures consistently show `Ret` among top opcodes. | Strong | Compare a return-free helper shape only if the language already allows a safe equivalent. |
| Caller result binding | Helper variants maintain caller locals for score, counts, and checksum. | Strong | Trace the exact local-slot flow in a future lowering-level audit if implementation work ever becomes relevant. |
| Call-chain repeated staging | G2 call-chain amplifies the helper-boundary delta relative to the single-call pair. | Strong | Use a future audit to separate chain depth from helper-body work. |
| Helper-internal branch classification | Helper bodies likely contribute some of the pressure, but the call-chain amplification suggests boundary cost is still material. | Medium | Compare a helper that is structurally simpler but keeps the same boundary pattern. |
| Frame/local slot movement | VM call/return logic moves values between caller and callee contexts, matching the observed scalar movement pattern. | Strong | If this ever becomes an implementation candidate, inspect a smaller call/return path before touching semantics. |

## Interpretation

The helper-boundary signal is stable across VM-M9 and VM-M11.

The source shape suggests argument/result staging contributes to scalar movement.

The lowering and VM execution paths both show value movement across local slots and call frames.

This is a candidate for future implementation audit, not a code change.

## Recommended Next Slice

### Option 1

`VM-M13: docs(sm-vm): select scalar movement implementation audit candidate`

Use this only if the project wants to pick the next implementation-focused audit from the helper-boundary evidence.

### Option 2

`VM-M13: docs(sm-vm): audit temporary-staging lowering shape`

Use this if helper-boundary remains broad and the project wants to compare it with the other stable signal before narrowing further.

### Option 3

`VM-M13: docs(sm-vm): specify helper-boundary result-equivalence check`

Use this if the fixture-local assertion boundary is still the main evidence weakness.

### Option 4

`VM-M13: test(sm-vm): add helper-boundary third-generation fixtures`

Use this only if helper-boundary mechanism evidence is still too broad after VM-M12.

## Non-claims

This document does not claim:

- VM performance improved;
- runtime behavior changed;
- verifier behavior changed;
- SemCode format changed;
- parser/typechecker/lowering changed;
- fixtures changed;
- helper calls are bad;
- inlining is required;
- Pulsar P5-A is reopened;
- scalar optimization is approved;
- any code change is safe without follow-up proof.

## Validation

- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads -- --nocapture` passed
- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads -- --ignored --nocapture` passed
- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads` passed
- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads -- --ignored` passed
- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads --no-run` passed
- `cargo clippy -p sm-vm --all-targets --all-features -- -D warnings` passed
- `cargo check -p sm-vm --no-default-features` passed
- `git diff --check` passed
- `cargo fmt --check` failed due to pre-existing unrelated formatting drift in `crates/prom-ui-backend-native/src/lib.rs`
- `git status --short` still shows unrelated pre-existing worktree changes outside this VM-M12 slice
