# CTF-3 — Determinism Matrix

Status: frozen for PCC core readiness
Parent lane: `core_trust_freeze/index.md`

## 1. Purpose

This file freezes the determinism surfaces that PCC practical core readiness
depends on.

The goal is not to predict every future repeatability problem. The goal is to
make the current determinism surface explicit, stable, and reviewable before
PCC-1 starts.

## 2. Freeze status

- Frozen determinism surfaces are the surfaces that current `main` already
  implements and/or tests as stable repeated behavior.
- Freeze-candidate surfaces are behavior families that exist in the repo but
  are not yet part of the PCC freeze set.
- Planned / non-admitted surfaces are later-phase determinism-sensitive
  behaviors that are intentionally not frozen for PCC core readiness.
- Out-of-scope surfaces are external or boundary-adjacent timing / backend
  surfaces that this PCC freeze pass does not claim.

Rule:

```text
Any new determinism-sensitive behavior must enter the matrix explicitly,
collect code or test evidence, and stay freeze-candidate until the PCC review
promotes it.
```

## 3. Frozen determinism surfaces

| Surface | Meaning | Source / owner | Evidence | Allowed phase | Stability status | Affected layer |
|---|---|---|---|---|---|---|
| SemCode emission byte stability | Same source/options produce the same emitted SemCode bytes. | `sm-emit`, `sm-ir` | `E2-test: tests/g1_execution_integrity.rs::g1_execution_integrity_repeated_compiles_are_byte_stable` | PCC-0.5 | frozen | emit |
| Verifier admission determinism | Same SemCode/config produces the same accept/reject decision. | `sm-verify`, `sm-vm` | `E2-test: tests/g1_execution_integrity.rs::g1_execution_integrity_malformed_semcode_rejects_before_execution; E2-test: crates/sm-vm/src/semcode_vm.rs::verified_run_rejects_invalid_bytecode_before_execution` | CTF | frozen | verifier, diagnostics |
| Verified VM execution stability | Same verified program/config produces the same result or trap. | `sm-vm` | `E2-test: tests/g1_execution_integrity.rs::g1_execution_integrity_stage_summaries_match_current_baseline; E2-test: tests/canonical_examples.rs::canonical_positive_examples_check_run_compile_and_verify` | CTF | frozen | VM, trace |
| Canonical pipeline stability | Canonical examples stay stable through `check -> compile -> verify -> run-smc`. | `smc-cli`, `sm-verify`, `sm-vm` | `E2-test: tests/canonical_examples.rs::canonical_positive_examples_check_run_compile_and_verify` | PCC-0.5 | frozen | CLI, verifier, VM |
| Direct `smc run-smc` stability | The direct `smc run-smc` CLI path runs an emitted SemCode artifact successfully and repeatably. | `smc-cli` | `E2-test: tests/smc_run_smc_cli.rs::smc_run_smc_executes_emitted_semcode_artifact` | PCC-0.5 | frozen | CLI |
| Trap outcome stability | Known runtime traps keep their class and meaning for the same input. | `sm-vm` | `E2-test: crates/sm-vm/src/semcode_vm.rs::vm_rejects_unknown_opcode_on_load; E2-test: crates/sm-vm/src/semcode_vm.rs::vm_traps_on_failed_assert; E2-test: crates/sm-vm/src/semcode_vm.rs::vm_traps_on_fx_division_by_zero; E2-test: crates/sm-vm/src/semcode_vm.rs::vm_rejects_write_after_borrow_same_path` | CTF | frozen | VM, diagnostics |
| Budget / stack determinism | Configured stack and register budgets fail the same way for the same program/config. | `sm-runtime-core`, `sm-vm` | `E2-test: crates/sm-vm/src/semcode_vm.rs::vm_enforces_configured_stack_depth; E2-test: crates/sm-vm/src/semcode_vm.rs::vm_enforces_configured_register_budget` | CTF | frozen | VM, diagnostics |
| Runtime ownership rejection determinism | Borrow / write overlap rejection stays stable across repeated runs. | `sm-vm`, runtime ownership tests | `E2-test: tests/runtime_ownership_e2e.rs::runtime_ownership_rejects_same_path_write_deterministically` | CTF | frozen | VM, diagnostics |

## 4. Freeze-candidate surfaces

None remain in the PCC core determinism freeze set after this pass.

If a future PCC PR introduces a new determinism-sensitive behavior, it starts
here and must not be called frozen until evidence exists.

## CTF-WP3 PCC-4..PCC-9 Determinism Sync

PCC-4..PCC-9 closeouts do not freeze broad feature families.

They do provide bounded determinism evidence for current admitted fixture-backed surfaces.

`freeze-candidate` means protected from silent change, not release freeze.

Future widening still requires new CTF review.

## 5. Planned / non-admitted surfaces

These determinism-sensitive families are intentionally not frozen for the
current PCC core-readiness gate:

| Surface family | Meaning | Owner | Evidence | Allowed phase | Stability status | Affected layer |
|---|---|---|---|---|---|---|
| Numeric behavior | Arithmetic and numeric result/trap rules that belong to later language phases. | PCC-2 | docs / roadmap only | PCC-2 | planned | typecheck, emit, VM |
| Text behavior | Text operations and text-result stability. | PCC-3 | docs / roadmap only | PCC-3 | planned | typecheck, emit, VM |
| Records | Current record literal/access/update fixture surface only. | PCC-4 | PCC-4 closeout | PCC-4 | freeze-candidate | typecheck, emit, VM |
| ADT + basic match | Current constructors + basic match fixtures only. | PCC-5 | PCC-5 closeout | PCC-5 | freeze-candidate | typecheck, emit, VM |
| Option / Result | Standard forms only. | PCC-6 | PCC-6 closeout | PCC-6 | freeze-candidate | typecheck, emit, VM |
| Sequence | Current Sequence operations covered by PCC-7B/D. | PCC-7 | PCC-7 closeout | PCC-7 | freeze-candidate | typecheck, emit, VM |
| Map | Admitted baseline only; missing-key, iteration policy, and quota remain open. | PCC-7 | PCC-7 closeout | PCC-7 | audit-needed / freeze-candidate | typecheck, emit, VM |
| Stdlib helpers | `assert` / `print` / `to_text` admitted helper surface only. | PCC-8 | PCC-8 closeout | PCC-8 | freeze-candidate | typecheck, emit, VM |
| Project model manifest baseline | Current `Semantic.package` helper baseline only. | PCC-9 | PCC-9 closeout | PCC-9 | freeze-candidate | CLI |

Open determinism notes:

- map missing-key behavior remains unresolved;
- map iteration policy remains unresolved;
- collection memory/quota determinism remains open;
- project-root discovery remains open;
- semantic.toml parse/load determinism remains open;
- src/main.sm discovery remains open;
- smc new output determinism remains open;
- project-level 7hell determinism remains open.

## 6. Out-of-scope surfaces

These surfaces are not part of the current PCC determinism freeze claim:

| Surface family | Meaning | Owner | Evidence | Allowed phase | Stability status | Affected layer |
|---|---|---|---|---|---|---|
| Wall-clock / system time | Time-based outputs and timestamps. | future runtime / tooling | out of scope | none | out-of-scope | capability / runtime boundary |
| Concurrency / async scheduling | Scheduling-dependent ordering or race behavior. | future runtime / tooling | out of scope | none | out-of-scope | runtime, CLI, UI |
| UI / Workbench event timing | Frontend event ordering and repaint timing. | UI / Workbench | out of scope | none | out-of-scope | UI boundary |
| GPU / backend / hardware timing | Platform backend timing and GPU execution behavior. | backend / platform | out of scope | none | out-of-scope | backend |
| Network / host ABI effects | External side effects and host-bound nondeterminism. | PROMETHEUS boundary | out of scope | none | out-of-scope | capability / runtime boundary |

## 7. Rule for adding determinism-sensitive behavior

1. Name the determinism surface explicitly.
2. Add code or test evidence.
3. Declare the owning layer.
4. Declare whether the surface is emit, verifier, VM, CLI, diagnostics, trace,
   or capability / runtime boundary.
5. Keep the surface freeze-candidate until a PCC review promotes it.
6. Do not introduce a determinism-sensitive behavior silently in a feature PR.

## 8. Evidence map

| Evidence id | What it proves |
|---|---|
| `E2-test: tests/g1_execution_integrity.rs::g1_execution_integrity_repeated_compiles_are_byte_stable` | Repeated compiles stay byte-stable. |
| `E2-test: tests/g1_execution_integrity.rs::g1_execution_integrity_stage_summaries_match_current_baseline` | Stage summaries stay stable for the same baseline. |
| `E2-test: tests/canonical_examples.rs::canonical_positive_examples_check_run_compile_and_verify` | Canonical examples stay stable through `check -> compile -> verify -> run-smc`. |
| `E2-test: tests/smc_run_smc_cli.rs::smc_run_smc_executes_emitted_semcode_artifact` | Direct `smc run-smc` on emitted SemCode succeeds through the public CLI path. |
| `E2-test: crates/sm-vm/src/semcode_vm.rs::vm_rejects_unknown_opcode_on_load` | Unknown opcode rejection is stable. |
| `E2-test: crates/sm-vm/src/semcode_vm.rs::vm_traps_on_failed_assert` | Assertion failure trap is stable. |
| `E2-test: crates/sm-vm/src/semcode_vm.rs::vm_traps_on_fx_division_by_zero` | Division-by-zero trap is stable. |
| `E2-test: crates/sm-vm/src/semcode_vm.rs::vm_rejects_write_after_borrow_same_path` | Borrow-write conflict rejection is stable. |
| `E2-test: crates/sm-vm/src/semcode_vm.rs::vm_enforces_configured_stack_depth` | Stack-budget failure is stable. |
| `E2-test: crates/sm-vm/src/semcode_vm.rs::vm_enforces_configured_register_budget` | Register-budget failure is stable. |
| `E2-test: crates/sm-vm/src/semcode_vm.rs::verified_run_rejects_invalid_bytecode_before_execution` | Verifier rejection remains before VM execution. |
| `E2-test: tests/runtime_ownership_e2e.rs::runtime_ownership_rejects_same_path_write_deterministically` | Runtime ownership rejection stays deterministic across runs. |

## 9. PCC impact

FM-036 is now frozen enough for PCC practical-core readiness.

The remaining non-frozen families are intentionally later-phase or out of scope
for the current PCC determinism freeze pass, so they do not block FM-036.

FM-035 is unchanged by this PR.

## 10. Acceptance checklist

```text
[x] frozen determinism surfaces are listed
[x] frozen surfaces have code and/or test evidence
[x] freeze-candidate bucket is explicit
[x] planned / non-admitted bucket is explicit
[x] out-of-scope bucket is explicit
[x] rule for new determinism-sensitive behavior is explicit
[x] PCC impact is explicit
[x] FM-036 can be treated as frozen for PCC readiness
[x] FM-035 is left untouched
```
