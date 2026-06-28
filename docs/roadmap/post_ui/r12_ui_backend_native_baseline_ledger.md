# R12 UI Backend Native Baseline Ledger

## 1. Purpose
This ledger records the accepted baseline status of the existing `crates/prom-ui-backend-native` crate after the backend-native reality audit.

It does not claim new implementation work.

## 2. Closed Basis
| PR | Role | Status |
|----|------|--------|
| #1120 | First Visible Surface Boundary | MERGED |
| #1121 | Backend Native Reality Audit | MERGED |

## 3. Baseline Decision
Decision: ACCEPT EXISTING CRATE AS BASELINE.

Based on #1121, record:

- `crates/prom-ui-backend-native` exists: YES
- workspace registered: YES
- backend-native depends on `prom-ui`: YES
- backend-native depends on `prom-ui-runtime`: YES
- `prom-ui` depends on backend-native: NO
- `winit` present: YES, crate-local
- `wgpu` / `vello` / `tiny-skia` / `softbuffer`: NO
- semantic model mutation found: NO
- `cargo test -p prom-ui-backend-native`: PASS

## 4. Seed Gate Disposition
`R12-UI-BACKEND-NATIVE-CRATE-SEED-PR` is BLOCKED / SUPERSEDED.

Reason: `crates/prom-ui-backend-native` already exists and is not a blank scaffold.

No seed-only PR was created.

## 5. Accepted Backend-Native Baseline
The existing backend-native crate is accepted as the DoD-3 backend-native baseline.

This acceptance does not approve new backend behavior.
This acceptance does not approve new windowing behavior.
This acceptance does not approve new drawing behavior.
This acceptance does not approve event-loop expansion.
This acceptance does not approve runtime/action/effect/capability behavior.

## 6. Dependency Direction
PASS — `prom-ui` core does not depend on `prom-ui-backend-native`.
PASS — backend-native depends outward on `prom-ui` / `prom-ui-runtime`.
PASS WITH WARNINGS — `winit` exists inside backend-native boundary only.

Unsafe condition:

- `prom-ui` depends on `prom-ui-backend-native`

If that condition appears in a future audit, it is a stop condition.

## 7. SEMANTIC_UI_DNA Compliance
PASS — `prom-ui` core remains backend-free.
PASS — backend-native remains a boundary crate.
PASS — backend-native does not become semantic authority.
PASS — semantic model mutation was not found.
PASS — no direct action execution path was found.
PASS — no direct effect execution path was found.
PASS — no runtime/capability bypass was found.
PASS — Unknown/Conflict semantics were not flattened.

## 8. Baseline Constraints
Future backend-native changes require explicit gates.
Existing baseline must not be expanded silently.
No `wgpu` dependency may be added without draw backend selection gate.
No new `winit` / event-loop behavior may be added without windowing gate.
No frame sink trait may be added without a frame sink trait gate.
No drawing behavior may be added without a draw backend source gate.
No runtime/action/effect/capability behavior may be added without runtime/admission gates.

## 9. Future-Gated Work
- `R12-UI-BACKEND-FRAME-SINK-TRAIT-PR`
- `R12-UI-OFFSCREEN-STATIC-FRAME-TEST-PR`
- `R12-UI-WINDOWING-BOUNDARY-PR`
- `R12-UI-WINIT-WINDOW-SEED-PR`
- `R12-UI-DRAW-BACKEND-SELECTION-PR`
- `R12-UI-DRAW-BACKEND-MINIMAL-SOURCE-PR`
- `R12-UI-STATIC-VISIBLE-DEMO-PR`

## 10. Explicit Non-Capabilities
Baseline acceptance does not mean:

- visible UI is complete;
- frame sink trait exists;
- offscreen frame tests exist;
- drawing backend is selected;
- `wgpu` is approved;
- GPU drawing exists;
- software rasterization exists;
- hit testing exists;
- interaction exists;
- event loop expansion is approved;
- action request admission exists;
- runtime handoff exists;
- effect handoff exists;
- capability admission exists.

## 11. Repository Scope
- source files changed: NO
- test files changed: NO
- docs changed: YES
- `Cargo.toml` changed: NO
- `Cargo.lock` changed: NO
- `docs/dna` changed: NO
- `Admission Guard` changed: NO
- GitHub CI used: NO

## 12. Validation
- `cargo fmt --check`: PASS
- `cargo test -p prom-ui --lib`: PASS
- `cargo test -p prom-ui`: PASS
- `cargo test -p prom-ui-backend-native`: PASS
- `git diff --check`: PASS
- tracked `pr_body` files: NO
- Admission Guard executed: YES
- Admission Guard result: FAIL - ENVIRONMENT PATHING
- Admission Guard changed: NO

## 13. Remaining Warnings
Admission Guard still fails locally due environment pathing.
This is unchanged and does not reflect a repository regression.

`cargo test` also emits unrelated existing warnings from other test files, but no failures.

## 14. Final Decision
PASS WITH WARNINGS — R12 UI Backend Native baseline ledger completed.

The existing `crates/prom-ui-backend-native` crate is accepted as the DoD-3 backend-native baseline following #1121.

`R12-UI-BACKEND-NATIVE-CRATE-SEED-PR` is blocked / superseded because the crate already exists and is not a blank scaffold.

No source, test, Cargo, DNA, or Admission Guard files were modified.

The next recommended lane is `R12-UI-BACKEND-FRAME-SINK-TRAIT-PR`.

## 15. Recommended Next Lane
`R12-UI-BACKEND-FRAME-SINK-TRAIT-PR`

Do not start it in this PR.
