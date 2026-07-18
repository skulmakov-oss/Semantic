# Shell Player Evidence Inventory

Status: UI-DNA2-9A1 EVIDENCE INVENTORY
Scope: tracked experimental and calculator reference evidence
Canonical implementation authority: NONE

## Classification

| Classification | Meaning |
| --- | --- |
| Direct evidence | Existing behavior or test that can be inspected |
| Reusable concept | Idea that may inform the canonical contract |
| Test-pattern evidence | Deterministic testing technique that may be reused |
| Experimental-only behavior | Must remain in the experiment without promotion |
| Stale documentary claim | Documentation not matching live tracked files |
| Missing evidence | Required canonical behavior with no current proof |
| Forbidden authority transfer | Experimental behavior that must never become authority |

## Live tracked surface

`git ls-files experiments/ui-shell-kit` identifies 15 tracked files:

- `experiments/ui-shell-kit/Cargo.toml`;
- `experiments/ui-shell-kit/src/action.rs`;
- `experiments/ui-shell-kit/src/calculator_controller.rs`;
- `experiments/ui-shell-kit/src/calculator_scene.rs`;
- `experiments/ui-shell-kit/src/event.rs`;
- `experiments/ui-shell-kit/src/focus.rs`;
- `experiments/ui-shell-kit/src/lib.rs`;
- `experiments/ui-shell-kit/src/paint.rs`;
- `experiments/ui-shell-kit/src/snapshot.rs`;
- `experiments/ui-shell-kit/src/theme.rs`;
- `experiments/ui-shell-kit/src/ui_shell.rs`;
- `experiments/ui-shell-kit/tests/calculator_focus_action_trace.rs`;
- `experiments/ui-shell-kit/tests/calculator_hit_test_stability.rs`;
- `experiments/ui-shell-kit/tests/calculator_motion_phase_evidence.rs`;
- `experiments/ui-shell-kit/tests/calculator_reference_scenario.rs`.

## Concept inventory

| Area | Classification and live path | Observed evidence | Canonical relevance | Authority limitation | Promotion status |
| --- | --- | --- | --- | --- | --- |
| Events | Direct evidence — `src/event.rs` | Pointer move/down/up and close-request carriers | Backend-neutral input-carrier reference | Event carrier is not Semantic intent or admission | Experimental-only |
| Actions | Direct evidence — `src/action.rs` | Button-press and focus-change action queue | Local action-candidate and trace pattern | Experimental action is not `ActionIntent`, admission, or dispatch | Experimental-only |
| Focus | Direct and test-pattern evidence — `src/focus.rs`, `tests/calculator_focus_action_trace.rs` | Local `FocusRing`, deterministic focus-change assertions | Local focus-state concept and executable evidence pattern | Focus is not selection, permission, or Semantic truth | Experimental-only |
| Hover/pressed state | Direct evidence — `src/calculator_controller.rs`, `src/calculator_scene.rs`, `src/ui_shell.rs` | Pointer-driven hover/selected state and visible button-state mapping | Local ephemeral interaction-state concept | Display state is not runtime or Semantic state | Experimental-only |
| Hit testing | Direct and test-pattern evidence — `src/calculator_scene.rs`, `tests/calculator_hit_test_stability.rs` | Deterministic button lookup, repeat-hit and outside-hit assertions | Hit-test realization and stability-test pattern | A hit-test match is not action authorization | Experimental-only |
| Layout | Direct evidence — `src/calculator_scene.rs`, `src/ui_shell.rs` | Fixed calculator rectangles, centering, grid-cell helpers | Evidence that layout can produce stable hit-test geometry | Fixed calculator geometry does not freeze canonical layout ownership or algorithm | Experimental-only |
| Painting | Direct evidence — `src/paint.rs`, `src/ui_shell.rs` | Backend-neutral `DrawFrame`/`DrawCommand` construction | Draw-seam and command-production reference | Draw commands are not pixels or renderer authority | Experimental-only |
| Snapshots | Direct and test-pattern evidence — `src/snapshot.rs`, `tests/calculator_motion_phase_evidence.rs`, `tests/calculator_reference_scenario.rs` | Deterministic textual command snapshots and before/after comparisons | Reusable deterministic evidence technique | Snapshot text is not a canonical draw encoding or production proof | Experimental-only |
| Accessibility | Missing evidence — no tracked `ui-shell-kit` source or test | No accessibility realization is implemented or qualified | Canonical Shell Player requires a separately specified accessibility boundary | Absence must not be filled by assumption | Not promotable |
| Calculator controller | Direct evidence — `src/calculator_controller.rs` | UI-local state, event handling, checked arithmetic, focus and render projection | Reference for local transition organization | Calculator state is not task, connectivity, admission, or Semantic truth | Experimental-only |
| Calculator scene | Direct evidence — `src/calculator_scene.rs` | Button identities, layout, hit testing and draw projection | Reference scene for deterministic local realization | Calculator-specific structure is not canonical Static UI IR or Shell Player structure | Experimental-only |
| Deterministic draw output | Direct and test-pattern evidence — `src/snapshot.rs`, `tests/calculator_motion_phase_evidence.rs` | Equal snapshots for equal state and distinct snapshots after `7 + 3 = 10` | Reusable determinism qualification pattern | Does not freeze draw-command encoding or renderer behavior | Experimental-only |
| Non-panicking error behavior | Direct source evidence — `src/calculator_controller.rs` | Checked arithmetic and explicit `ERR` state, including divide-by-zero handling | Reference for explicit local failure without panic | No focused tracked test qualifies the full canonical error policy | Experimental-only |

## Calculator documentary evidence

`docs/spec/ui/calculator_shell_contract.md` and
`docs/spec/ui/calculator_reference_scenario.md` are reference evidence only.
Their `7 + 3 = 10` scenario aligns with the tracked executable tests.

The following claims are stale at this repository baseline because the named
paths are not tracked:

- `experiments/ui-shell-kit/src/calculator_state.rs`;
- `experiments/ui-shell-kit/src/components.rs`;
- `experiments/ui-shell-kit/src/layout.rs`;
- `experiments/ui-shell-kit/src/hit_test.rs`;
- the `calculator_interaction_dump` example path;
- the `calculator_motion_dump` example path.

The relevant live behavior is consolidated in `calculator_controller.rs`,
`calculator_scene.rs`, `ui_shell.rs`, and the four tracked integration tests.
This inventory records the drift; UI-DNA2-9A1 does not repair the older
documents.

## Forbidden authority transfer

- calculator state must not become Semantic truth;
- `UiAction` must not become admitted action by implicit conversion;
- experimental hit testing must not grant authority;
- snapshot output must not become renderer, audit, or production authority;
- `DrawCommand` evidence must not assign pixel ownership to Shell Player;
- `ui-shell-kit` must not become canonical by reuse alone.

## Status

```text
ui-shell-kit promotion = NOT AUTHORIZED
Shell Player implementation = NOT AUTHORIZED
production use = NOT AUTHORIZED
Gate D = CLOSED
NEXT AUTHORIZED IMPLEMENTATION SLICE = NONE
```
