# R12 UI Projection Builder Public API Lock Closeout

## 1. Purpose

The public API lock line is closed as a regression guard for the R12 projection substrate API surface.

It locks callable projection API signatures and boundaries without adding new authority, behavior, renderer/runtime/verifier/capability integration, or public unchecked projection.

## 2. Closed chain

- PR #935: R12 UI Projection Builder full-line ledger audit clean
- PR #936: Internal Projection Helper source PR
- PR #937: Internal Projection Helper closeout
- PR #939: Public API Lock Source/Test

## 3. Locked API surface

- `project_ir_to_projection`
- `ValidatedUiIr` constructors and validation accessors
- `new_with_config` / `validate_ui_ir_for_projection_with_config`
- `project_validated_ir_to_projection`
- `UiProjectionErrorCode` / `validation_diagnostics`
- `source_ir_root` / `source_ir_node_id` / traces
- Inert property / action / effect classifications

## 4. Test strategy

Public API lock was achieved through module-local signature locking and integration-style smoke testing directly in `projection.rs`. Explicit function pointer casts or closure signature boundaries were used to freeze the compile-time shape of the public projection contract.

## 5. What changed

- `crates/prom-ui/src/projection.rs` (Added API lock and compile-signature tests)
- `docs/roadmap/post_ui/r12_ui_projection_builder_public_api_lock_closeout.md` (This file)

## 6. What did not change

- No public unchecked paths were introduced
- The internal projection helper remains private
- No production behavior changes
- No cargo dependencies or Cargo.lock files altered

## 7. Evidence Matrix

| Area | Status |
|---|---|
| public API lock tests | IMPLEMENTED |
| project_ir_to_projection signature | LOCKED |
| ValidatedUiIr constructors | LOCKED |
| config-aware validation helper | LOCKED |
| project_validated_ir_to_projection | LOCKED |
| private helper remains private | PRESERVED |
| public unchecked projection | ABSENT / FORBIDDEN |
| source behavior changes | ABSENT |
| Cargo changes | ABSENT |

## 8. Consolidation audit result

PASS. The source test PR merged cleanly and post-merge validation confirmed the test suite locked the requested API surfaces without expanding capabilities or breaching security boundaries.

## 9. Admission Guard table

| Area | Observed state | Classification | Status |
|---|---|---|---|
| API lock tests | Implemented | ADMITTED | PASS |
| production behavior changes | Absent | FORBIDDEN | PASS |
| public unchecked path | Absent | FORBIDDEN | PASS |
| private helper exposure | Absent | FORBIDDEN | PASS |
| renderer/runtime/capability | Absent | FORBIDDEN | PASS |
| dependency additions | Absent | FORBIDDEN | PASS |

## 10. Remaining future gates

- R12-UI-PROJECTION-BUILDER-R12-FINAL-CLOSEOUT-LINE-FULL-PACKAGE

## 11. Final decision

PASS — R12 UI PROJECTION BUILDER PUBLIC API LOCK LINE CLOSED CLEANLY
