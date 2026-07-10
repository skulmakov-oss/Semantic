# Harness Report: CORE-QUAD-EXACT-PLANE-DELTA-SPLIT

## Status
* **Starting Commit:** `a46024ccf3898786ce89ae7995811aef9ef36589`
* **Changed Files:**
  * `.harness/current.task.yaml`
  * `.harness/reports/CORE-QUAD-EXACT-PLANE-DELTA-SPLIT.md`
  * `docs/roadmap/core_quad/exact_plane_delta_split.md`
  * `crates/semantic-core-quad/src/lib.rs`

## Inventory and Design
* **Existing `StateDelta32`:** The public data shape and from_regs computation of StateDelta32 remain unchanged. Rust documentation was clarified.
* **Compatibility Posture:** The four legacy truth/falsity plane fields of StateDelta32 are semantically equivalent to the corresponding fields of PlaneDelta32. StateDelta32 remains a broader mixed compatibility structure and also contains exact-super/conflict and aggregate changed/known events.
  * Plane subset:
    * entered_true
    * left_true
    * entered_false
    * left_false
  * Exact S/conflict subset:
    * entered_super
    * left_super
    * became_conflicted
    * resolved_conflict
  * Aggregate subset:
    * changed
    * became_known
    * became_unknown
* **New Public API:**
  * `PlaneDelta32` struct
  * `ExactStateDelta32` struct
  * `QuadroReg32::plane_delta` method
  * `QuadroReg32::exact_state_delta` method

## Plane Formulas
* `entered_truth = current_truth AND NOT previous_truth`
* `left_truth = previous_truth AND NOT current_truth`
* `entered_falsity = current_false AND NOT previous_false`
* `left_falsity = previous_false AND NOT current_false`

## Exact-State Formulas
* N/F/T/S coverage: All 4 quad states are fully covered by the exact-state API.
* `entered_Q = current_exact_Q AND NOT previous_exact_Q`
* `left_Q = previous_exact_Q AND NOT current_exact_Q`

## Tests Added
* **10 Explicit Transitions:** T->S, S->T, F->S, S->F, N->S, S->N, N->T, T->N, N->F, F->N.
* **4x4 Exhaustive Matrix:** Tests all 16 `previous x current` transitions for both exact state and plane events using scalar logic.
* **Identity Transitions:** N->N, F->F, T->T, S->S tested; all delta fields are empty.
* **Mixed-Lane Register Test:** Lane 0 (T->S), Lane 1 (S->T), Lane 2 (F->N), Lane 3 (N->F) transitioning concurrently within one delta operation.
* **Compatibility Test:** Verifies `delta32_plane_field_compatibility_matrix` behavior is equivalent to `PlaneDelta32`. `legacy_delta32_full_field_matrix_remains_stable` verifies all legacy fields.

## Verification
* `cargo fmt -p semantic-core-quad -- --check`: Passed
* `cargo test -p semantic-core-quad --quiet`: Passed (117 tests ok)
* `cargo check -p semantic-core-quad --no-default-features`: Passed
* `cargo test -p semantic-core-quad --all-features --quiet`: Passed (117 tests ok)
* `cargo test -p semantic-core-capsule --quiet`: Passed (8 tests ok)
* `git diff --check`: Passed
* `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/harness-check.ps1`: Passed (`[harness] ok`)
* `git status --short`: Verified untracked files are untouched.

## Explicit Non-Changes
* `StateDelta32` logic remains exactly the same.
* Tile APIs (`StateDelta128`, `QuadTile128`) are explicitly not modified, deferred to tile-level slice.
* No changes to physical mask or parsing/execution logic.

