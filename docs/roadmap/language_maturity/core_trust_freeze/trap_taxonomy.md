# CTF-2 — Trap Taxonomy

Status: frozen for PCC core readiness
Parent lane: `core_trust_freeze/index.md`

## 1. Purpose

This file freezes the current execution-failure surface that PCC practical core
readiness depends on.

The goal is not to forecast every future failure mode. The goal is to make the
current failure surface explicit, stable, and reviewable before PCC-1 starts.

## 2. Freeze status

- Frozen trap classes are the classes that current `main` already implements
  and/or tests as stable failure outcomes.
- Freeze-candidate classes are classes that exist in code but are not part of
  the PCC blocker set in this audit pass.
- Planned / non-admitted classes are not part of the current PCC freeze set.
- Out-of-scope classes are boundary-adjacent failure families owned by other
  CTF docs or later PCC phases.

Rule:

```text
Any new trap class must enter as freeze-candidate first, then move to frozen
only with code or test evidence and an explicit PCC review note.
```

## 3. Frozen trap classes

| Trap class | Meaning | Source / owner | Evidence | Allowed phase | Stability status | Verifier / VM / CLI / diagnostics impact |
|---|---|---|---|---|---|---|
| Malformed SemCode / unsupported header | Loader or verifier rejects malformed or unsupported SemCode before public execution continues. | `sm-verify`, `sm-vm` | `E2-test: tests/g1_execution_integrity.rs::g1_execution_integrity_malformed_semcode_rejects_before_execution; E2-test: tests/bytecode_compat.rs::compat_unsupported_version_has_migration_hint; E1-code: crates/sm-vm/src/semcode_vm.rs::run_verified_semcode_with_entry_and_config` | verifier admission | frozen | verifier, VM, CLI, diagnostics |
| Unknown opcode | VM load rejects an opcode the current decoder does not admit. | `sm-vm` | `E2-test: crates/sm-vm/src/semcode_vm.rs::vm_rejects_unknown_opcode_on_load; E1-code: crates/sm-vm/src/semcode_vm.rs::map_format_err` | verifier admission / VM load | frozen | verifier, VM, CLI, diagnostics |
| Invalid jump target | VM rejects a jump target that cannot be executed safely. | `sm-vm` | `E1-code: crates/sm-vm/src/semcode_vm.rs::RuntimeError::InvalidJumpAddress` | VM execution | frozen | VM, CLI, diagnostics |
| Missing call target / unknown function | VM rejects a call target that does not resolve to a function. | `sm-vm` | `E1-code: crates/sm-vm/src/semcode_vm.rs::RuntimeError::UnknownFunction` | VM execution | frozen | VM, CLI, diagnostics |
| Type mismatch runtime | Runtime rejects a value shape that does not match the expected type at execution time. | `sm-vm` | `E1-code: crates/sm-vm/src/semcode_vm.rs::RuntimeError::TypeMismatchRuntime` | VM execution | frozen | VM, CLI, diagnostics |
| Stack underflow | VM rejects a call-stack pop or equivalent operation with no available frame. | `sm-vm` | `E1-code: crates/sm-vm/src/semcode_vm.rs::RuntimeError::StackUnderflow` | VM execution | frozen | VM, CLI, diagnostics |
| Stack overflow | VM or configured stack budget rejects excessive nesting / frames. | `sm-vm`, `sm-runtime-core` | `E2-test: crates/sm-vm/src/semcode_vm.rs::vm_enforces_configured_stack_depth; E1-code: crates/sm-runtime-core/src/lib.rs::RuntimeTrap::StackOverflow` | VM execution | frozen | VM, CLI, diagnostics |
| Quota exceeded | Runtime budget is exceeded for a configured quota family. | `sm-runtime-core`, `sm-vm` | `E2-test: crates/sm-vm/src/semcode_vm.rs::vm_enforces_configured_register_budget; E1-code: crates/sm-runtime-core/src/lib.rs::QuotaExceeded; E1-code: crates/sm-runtime-core/src/lib.rs::RuntimeQuotas` | VM execution | frozen | VM, CLI, diagnostics |
| Division by zero | Numeric division rejects a zero denominator. | `sm-vm` | `E2-test: crates/sm-vm/src/semcode_vm.rs::vm_traps_on_fx_division_by_zero; E2-test: tests/snake_benchmark_gap_matrix.rs::snake_benchmark_runtime_negative_reports_contract_violations` | VM execution | frozen | VM, CLI, diagnostics |
| Arithmetic overflow | Numeric operation exceeds the representable runtime range. | `sm-vm` | `E1-code: crates/sm-vm/src/semcode_vm.rs::RuntimeTrap::ArithmeticOverflow` | VM execution | frozen | VM, CLI, diagnostics |
| Assertion failure | Runtime assert fails on a false predicate. | `sm-vm` | `E2-test: crates/sm-vm/src/semcode_vm.rs::vm_traps_on_failed_assert; E2-test: tests/snake_benchmark_gap_matrix.rs::snake_benchmark_runtime_negative_reports_contract_violations` | VM execution | frozen | VM, CLI, diagnostics |
| Borrow-write conflict | Runtime detects an overlapping write against an active borrow. | `sm-vm` | `E2-test: crates/sm-vm/src/semcode_vm.rs::vm_rejects_write_after_borrow_same_path; E2-test: tests/runtime_ownership_e2e.rs::runtime_ownership_rejects_same_path_write_deterministically` | VM execution | frozen | VM, CLI, diagnostics |
| Verifier rejected | Verified execution rejects invalid SemCode before VM run. | `sm-verify`, `sm-vm` | `E2-test: crates/sm-vm/src/semcode_vm.rs::verified_run_rejects_invalid_bytecode_before_execution; E2-test: tests/g1_execution_integrity.rs::g1_execution_integrity_malformed_semcode_rejects_before_execution` | verifier admission | frozen | verifier, VM, CLI, diagnostics |

## 4. Freeze-candidate classes

None remain in the PCC-0F freeze set after this pass.

If a future PCC PR introduces a new runtime-failure class, it starts here and
must not be claimed as frozen until evidence exists.

## 5. Planned / non-admitted classes

None remain in the current PCC practical-core trap set.

Later feature-specific failure families must be admitted through a separate PCC
or CTF review before they are treated as frozen public failure classes.

## 6. Out-of-scope classes

Boundary-adjacent denial surfaces are tracked in their own CTF docs and are not
the blocker set for FM-035 in this pass.

That includes capability / UI capability / host-ABI denial surfaces that are
owned by the capability and boundary documentation lanes.

## 7. Admission rule for new trap classes

1. Name the class explicitly.
2. Add code or test evidence.
3. Declare the owning layer.
4. Declare whether it is verifier-reject, VM-trap, or boundary denial.
5. Keep it freeze-candidate until a PCC review promotes it.
6. Do not add a new trap class silently in a feature PR.

## 8. Evidence map

| Evidence id | What it proves |
|---|---|
| `E2-test: tests/g1_execution_integrity.rs::g1_execution_integrity_malformed_semcode_rejects_before_execution` | Malformed SemCode is rejected before execution. |
| `E2-test: tests/bytecode_compat.rs::compat_unsupported_version_has_migration_hint` | Unsupported bytecode versions have a stable rejection path and hint. |
| `E2-test: crates/sm-vm/src/semcode_vm.rs::vm_rejects_unknown_opcode_on_load` | Unknown opcode is rejected at VM load. |
| `E2-test: crates/sm-vm/src/semcode_vm.rs::vm_enforces_configured_stack_depth` | Stack budget overflow is stable. |
| `E2-test: crates/sm-vm/src/semcode_vm.rs::vm_enforces_configured_register_budget` | Quota-exceeded behavior is stable for configured register budgets. |
| `E2-test: crates/sm-vm/src/semcode_vm.rs::vm_traps_on_fx_division_by_zero` | Division by zero traps deterministically. |
| `E2-test: crates/sm-vm/src/semcode_vm.rs::vm_traps_on_failed_assert` | Assertion failure traps deterministically. |
| `E2-test: crates/sm-vm/src/semcode_vm.rs::vm_rejects_write_after_borrow_same_path` | Borrow-write conflict is stable. |
| `E2-test: crates/sm-vm/src/semcode_vm.rs::verified_run_rejects_invalid_bytecode_before_execution` | Verifier rejects invalid bytecode before VM execution. |
| `E2-test: tests/runtime_ownership_e2e.rs::runtime_ownership_rejects_same_path_write_deterministically` | Borrow-write conflict remains stable across runs. |
| `E2-test: tests/snake_benchmark_gap_matrix.rs::snake_benchmark_runtime_negative_reports_contract_violations` | Runtime negative surface remains deterministic under CLI evidence. |
| `E2-test: tests/ui_capability_admission_contract.rs::default_manifest_denies_all_ui_operations` | UI capability denial remains explicit and stable. |

## 9. PCC impact

FM-035 is now frozen enough for PCC practical-core readiness.

The remaining non-frozen families are explicitly deferred or owned by other
boundary docs, so they do not block the practical-core trap taxonomy freeze in
this pass.

FM-036 remains a separate blocker and is not changed by this PR.

## 10. Acceptance checklist

```text
[x] frozen trap classes are listed
[x] frozen classes have code and/or test evidence
[x] freeze-candidate bucket is explicit
[x] planned / non-admitted bucket is explicit
[x] out-of-scope bucket is explicit
[x] admission rule for new trap classes is explicit
[x] PCC impact is explicit
[x] FM-035 can be treated as frozen for PCC readiness
[x] FM-036 is left untouched
```
