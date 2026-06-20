# R12 UI Backend Native Reality Audit

## 1. Purpose
This audit records the actual state of `crates/prom-ui-backend-native` because the planned seed-only crate PR is blocked by repository reality.

## 2. Why Seed Gate Is Blocked
`R12-UI-BACKEND-NATIVE-CRATE-SEED-PR` is blocked because `crates/prom-ui-backend-native` already exists.

A seed-only PR would be false because the crate is not a blank scaffold.

No seed-only PR was created.

## 3. Closed Basis
Closed basis for this audit:

| PR | Title / Gate | Status |
|----|--------------|--------|
| #1119 | R12 UI Minimal Layout Capability Closeout | MERGED |
| #1120 | R12 UI First Visible Surface Boundary | MERGED |

Current state assumed by this audit:

- `origin/main` contains #1120.
- DoD-1 Foundation UI Ready is complete.
- DoD-2 Minimal Layout Ready is complete.
- DoD-3 Visible Static UI Ready is open.
- First visible surface boundary is documented.
- `cargo fmt --check` passed in the closed basis.
- `cargo test -p prom-ui` passed in the closed basis.

## 4. Existing Backend-Native Crate Inventory
- crate exists: YES
- workspace member: YES
- `Cargo.toml` present: YES
- `src` file count: 1
- `test` file count: 18
- backend/windowing terms found: YES

Inventory notes:
- The crate is already populated and cannot be treated as a fresh seed scaffold.
- The test surface is large and split between non-winit staging and winit-gated contracts.

## 5. Workspace / Cargo State
Root workspace registration is present in `Cargo.toml`.

`crates/prom-ui-backend-native` is listed as a workspace member.

Dependency direction observed:
- `prom-ui-backend-native` depends on `prom-ui`
- `prom-ui-backend-native` depends on `prom-ui-runtime`
- `prom-ui` does not depend on `prom-ui-backend-native`

This direction is acceptable for a backend/native layer boundary.

## 6. Dependency Surface
Observed dependency surface:

- `winit`: YES, optional via `winit-backend`
- `wgpu`: NO
- `vello`: NO
- `tiny-skia`: NO
- `softbuffer`: NO
- `prom-ui`: YES
- `prom-ui-runtime`: YES
- other backend-related dependencies: NO additional external backend crate dependencies found

Classification:
- `winit` presence in backend-native only: warning / baseline fact
- `winit` leakage into `prom-ui` core: not observed

## 7. Source Surface
Observed source surface in `crates/prom-ui-backend-native/src/lib.rs`:

- windowing code: YES
- drawing code: YES, as staged draw accounting / adapter plumbing
- frame sink code: partial / staging surface present, no dedicated frame sink trait
- visible surface frame code: NO
- event loop code: YES, feature-gated winit scaffolding
- hit testing: NO explicit hit-test surface found
- interaction: YES, via input-event translation and event staging
- runtime/action/effect/capability: NO direct execution path found
- semantic model mutation: NO mutation of `UiTree`, `UiAst`, `UiIr`, `UiProjectionArtifact`, `UiRenderModel`, or `UiLayoutRectModel` found

Classification:
- This crate is not a blank scaffold.
- This crate is a backend/native boundary implementation with explicit staging and smoke scaffolds.

## 8. Test Surface
Observed test surface:

- total test files: 18
- non-winit staged-state tests: present
- winit-gated tests: present
- `cargo test -p prom-ui-backend-native`: PASS
- `cargo test -p prom-ui`: PASS

Test classification:
- Tests are not seed scaffolding only.
- Tests exercise backend/native staging, event translation, run-loop planning, window config, and winit-gated scaffolds.

## 9. SEMANTIC_UI_DNA Risk Audit
| Rule | Status | Notes |
|------|--------|-------|
| UI remains projection/cache | PASS | No evidence that backend-native redefines semantic truth. |
| prom-ui core backend-free | PASS | No dependency from `prom-ui` to backend-native observed. |
| backend does not become semantic authority | PASS WITH WARNINGS | The crate is operationally rich, but remains a backend/native boundary rather than a semantic owner. |
| source evidence preserved | PASS | The crate carries `prom-ui` / `prom-ui-runtime` evidence through its boundary types. |
| renderer boundary non-authoritative | PASS | No renderer authority was introduced here. |
| no direct action execution | PASS | No direct action execution path found. |
| no direct effect execution | PASS | No direct effect execution path found. |
| no runtime/capability bypass | PASS | No bypass path found in this audit. |
| Unknown/Conflict not flattened | PASS | Audit records the crate as existing reality rather than pretending it is blank. |

## 10. Authority Boundary Audit
Findings:

- `winit` exists only behind the backend-native crate boundary and feature gate.
- `prom-ui` does not import backend-native.
- backend-native owns staged/native behavior in its own crate.
- no evidence was found that backend-native mutates Semantic core models.
- no evidence was found that backend-native becomes verifier/runtime/capability authority.

Risk classification:
- baseline backend/native behavior present: YES
- authority leak into `prom-ui` core: NO
- cleanup requirement from this audit alone: not established

## 11. prom-ui Core Dependency Direction
Answers:

- Does `prom-ui` depend on `prom-ui-backend-native`? NO
- Does `prom-ui-backend-native` depend on `prom-ui`? YES
- Is dependency direction acceptable? YES, for a backend/native boundary crate
- Does backend-native leak into `prom-ui` core? NO evidence found

## 12. Validation Results
Validation run for audit:

- `cargo fmt --check`: PASS
- `cargo test -p prom-ui --lib`: PASS
- `cargo test -p prom-ui`: PASS
- `cargo test -p prom-ui-backend-native`: PASS
- `git diff --check`: PASS expected after doc-only change
- `tracked pr_body files`: NO
- Admission Guard executed: YES
- Admission Guard result: FAIL - ENVIRONMENT PATHING
- Admission Guard changed: NO
- GitHub CI used: NO

## 13. Findings
Primary findings:

1. `crates/prom-ui-backend-native` already exists and is populated.
2. The crate is already workspace-registered.
3. The crate depends on `prom-ui` and `prom-ui-runtime`.
4. The crate contains optional `winit` backend scaffolding and related tests.
5. `prom-ui` does not depend on `prom-ui-backend-native`.
6. `cargo test -p prom-ui-backend-native` passes in the current state.
7. A seed-only PR would be dishonest under this repository reality.

## 14. Decision Options
Option A — ACCEPT EXISTING CRATE AS BASELINE
- Use if the existing backend/windowing code is isolated, tests pass, and it does not leak into `prom-ui` core or semantic authority.

Option B — QUARANTINE / CLEANUP REQUIRED
- Use if the existing crate introduces premature backend behavior, dependency-direction risk, failing tests, or authority boundary problems.

Option C — SEED GATE SUPERSEDED
- Use if the crate is effectively a harmless scaffold and the seed gate is redundant.

Selected option: Option A — ACCEPT EXISTING CRATE AS BASELINE

Reason:
- The crate is already a real backend/native boundary layer rather than a blank scaffold.
- It is isolated from `prom-ui` core.
- No reverse dependency from `prom-ui` was found.
- The current code and tests pass.

## 15. Recommended Next Lane
Recommended next lane:

`R12-UI-BACKEND-NATIVE-BASELINE-LEDGER-PR`

If the roadmap wants to jump directly to the next boundary contract instead of a ledger PR, the next candidate is:

`R12-UI-BACKEND-FRAME-SINK-TRAIT-PR`

## 16. Repository Scope
- source files changed: NO
- test files changed: NO
- docs changed: YES
- `Cargo.toml` changed: NO
- `Cargo.lock` changed: NO
- `docs/dna` changed: NO
- `Admission Guard` changed: NO
- GitHub CI used: NO

## 17. Final Verdict
PASS WITH WARNINGS — R12 UI Backend Native reality audit completed; seed gate is blocked/superseded by existing crate reality.

`R12-UI-BACKEND-NATIVE-CRATE-SEED-PR` is BLOCKED / SUPERSEDED BY REALITY AUDIT.

No source files were modified.
No tests were modified.
No Cargo files were modified.
No DNA files were modified.
No Admission Guard files were modified.
