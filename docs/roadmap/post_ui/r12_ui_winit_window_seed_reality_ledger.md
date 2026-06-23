# R12 UI Winit Window Seed Reality Ledger

## 1. Purpose
This ledger records the actual repository state for the planned
`R12-UI-WINIT-WINDOW-SEED-PR`.

The planned seed gate is superseded by repository reality:
`crates/prom-ui-backend-native` already contains a crate-local,
feature-gated `winit-backend` surface with window lifecycle scaffolding.

This PR does not add source code, tests, Cargo changes, new `winit` usage,
`wgpu`, drawing, frame presentation, event loop expansion, hit testing,
interaction, runtime handoff, action execution, effect execution, or
capability admission.

## 2. Closed Basis
| PR | Role | Status |
|----|------|--------|
| #1120 | First Visible Surface Boundary | MERGED |
| #1121 | Backend Native Reality Audit | MERGED |
| #1122 | Backend Native Baseline Ledger | MERGED |
| #1123 | Backend Frame Sink Trait | MERGED |
| #1124 | Offscreen Static Frame Test | MERGED |
| #1125 | Windowing Boundary | MERGED |
| #1131 | Admission Guard CI Parity Profile | MERGED |

## 3. Why This Ledger Exists
The roadmap expected a future source gate named
`R12-UI-WINIT-WINDOW-SEED-PR`.

Repository reality is already ahead of that assumption:

- `crates/prom-ui-backend-native` exists.
- `crates/prom-ui-backend-native` is workspace-registered.
- `winit-backend` already exists as an optional feature.
- `winit` already exists as an optional dependency.
- crate-local event-loop creation scaffolding already exists.
- crate-local window configuration translation already exists.
- crate-local native window creation scaffolding already exists.
- manual/ignored native window smoke paths already exist.
- `NativeBackend::run_event_loop(...)` remains staged and does not own the
  `winit` app loop.

Therefore, a normal "seed winit window support" PR would be false. The correct
action is to record the existing baseline and keep future expansion gated.

## 4. Existing Winit Baseline
Current baseline facts:

- `winit-backend` feature exists: YES.
- optional `winit` dependency exists: YES.
- `EventLoop` creation helper exists: YES.
- `WindowConfig` to `WindowAttributes` translation exists: YES.
- `ActiveEventLoop::create_window(...)` scaffold exists: YES.
- `ApplicationHandler` scaffolds exist: YES.
- manual/ignored `run_app(...)` smoke paths exist: YES.
- separate `NativeBackendWinitApp` facade exists: YES.
- `NativeBackendWinitAppState` exists: YES.
- `NativeBackend::run_event_loop(...)` owns normal runtime integration: NO.
- draw backend selected: NO.
- `wgpu` approved: NO.
- frame presentation exists: NO.

## 5. Seed Gate Disposition
`R12-UI-WINIT-WINDOW-SEED-PR` is BLOCKED / SUPERSEDED.

Reason:
the repository already contains the crate-local `winit-backend` seed surface.

No source seed PR is created by this ledger.
No dependency seed PR is created by this ledger.
No new window behavior is introduced by this ledger.

## 6. SEMANTIC_UI_DNA Compliance
PASS - Semantic UI remains projection/cache, not semantic authority.
PASS - native backend remains a boundary crate.
PASS - `prom-ui` core remains backend-free and windowing-free.
PASS - existing `winit` usage remains crate-local to `prom-ui-backend-native`.
PASS - `NativeBackend::run_event_loop(...)` is not converted into `winit`
authority by this ledger.
PASS - semantic model mutation is not introduced.
PASS - direct action execution is not introduced.
PASS - direct effect execution is not introduced.
PASS - runtime/capability bypass is not introduced.
PASS - Unknown/Conflict semantics are not flattened.

docs/dna inspected: YES

DNA files inspected:

- [SEMANTIC_UI_DNA.md](../../dna/SEMANTIC_UI_DNA.md)

DNA alignment:

- UI may display evidence but does not become semantic truth.
- Native backend code may adapt platform behavior but does not own Semantic UI
  model authority.
- Window lifecycle remains a host boundary and does not become verifier,
  runtime, action, effect, or capability authority.

DNA conflicts detected: none.

## 7. Boundary Constraints
Future backend-native changes require explicit gates.

The accepted baseline must not be silently expanded.

Mandatory constraints:

- no `winit` expansion without a named windowing/source gate;
- no `wgpu` dependency without draw backend selection gate;
- no draw backend without draw backend source gate;
- no frame presentation without frame presentation gate;
- no hit testing without hit-testing boundary/source gates;
- no interaction mapping without interaction boundary/source gates;
- no action execution from native events;
- no effect execution from native events;
- no runtime/capability bypass from native events;
- no semantic model mutation from native backend state.

## 8. Explicit Non-Capabilities
This ledger does not mean:

- visible UI is complete;
- draw backend is selected;
- `wgpu` is approved;
- GPU drawing exists;
- software rasterization exists;
- frame presentation exists;
- normal `NativeBackend::run_event_loop(...)` is backed by `winit`;
- hit testing exists;
- interaction exists;
- event-to-action admission exists;
- runtime handoff exists;
- effect handoff exists;
- capability admission exists.

## 9. Future-Gated Work
Recommended future gates:

- `R12-UI-WINIT-RUN-LOOP-INTEGRATION-BOUNDARY-PR`
  - defines whether the existing manual `winit` app paths may become normal
    backend lifecycle integration.
- `R12-UI-DRAW-BACKEND-SELECTION-PR`
  - selects a draw backend through an explicit dependency decision.
- `R12-UI-DRAW-BACKEND-MINIMAL-SOURCE-PR`
  - implements minimal draw behavior in the backend-native layer only.
- `R12-UI-FRAME-PRESENTATION-BOUNDARY-PR`
  - defines when frame presentation becomes admitted behavior.
- `R12-UI-STATIC-VISIBLE-DEMO-PR`
  - runs a static visible demo without interaction.

## 10. Repository Scope
- source files changed: NO
- test files changed: NO
- docs changed: YES
- `Cargo.toml` changed: NO
- `Cargo.lock` changed: NO
- `docs/dna` changed: NO
- Admission Guard changed: NO
- GitHub CI used: NO

## 11. Validation
- `pwsh -File scripts/local_ci.ps1`: PASS
- `cargo test -p prom-ui-backend-native --features winit-backend`: PASS
- `git diff --check`: PASS
- tracked `pr_body` files: NO

## 12. Remaining Warnings
The existing backend-native source contains manual/ignored native window smoke
paths. That is baseline reality, not new behavior introduced by this ledger.

This ledger does not approve expanding those paths without a future explicit
gate.

## 13. Final Decision
PASS WITH WARNINGS - R12 UI Winit Window Seed reality ledger completed.

The existing `crates/prom-ui-backend-native` crate-local `winit-backend`
surface is accepted as the already-present DoD-3 winit window seed baseline.

`R12-UI-WINIT-WINDOW-SEED-PR` is blocked / superseded because the seed already
exists in repository reality.

No source, test, Cargo, DNA, or Admission Guard files were modified.

The next recommended lane is `R12-UI-WINIT-RUN-LOOP-INTEGRATION-BOUNDARY-PR`.

## 14. Recommended Next Lane
`R12-UI-WINIT-RUN-LOOP-INTEGRATION-BOUNDARY-PR`

Do not start it in this PR.
