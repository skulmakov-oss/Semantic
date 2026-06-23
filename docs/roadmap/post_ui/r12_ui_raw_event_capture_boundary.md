# R12 UI Raw Event Capture Boundary

## 1. Purpose
This boundary document defines the future architectural expectations for capturing raw input events (keyboard, mouse, window lifecycle) from the physical host backend into the `prom-ui-backend-native` layer.

It does not implement event translation, event routing, interaction models, hit-testing, or semantic action mapping.
It introduces no source code, tests, Cargo changes, dependencies, or runtime mutations.

## 2. Closed Basis
| PR | Role | Status |
|----|------|--------|
| #1120 | First Visible Surface Boundary | MERGED |
| #1122 | Backend Native Baseline Ledger | MERGED |
| #1125 | Windowing Boundary | MERGED |
| #1133 | Winit Run Loop Integration Boundary | MERGED |
| #1136 | Frame Presentation Boundary | MERGED |
| #1137 | Frame Presentation Source | MERGED |
| #1138 | Static Visible Demo | MERGED |

## 3. Boundary Summary
With the static visible demo completed, the event loop exists and can render physical frames. The next capability is observing user input.

This boundary mandates that raw events originating from the host (e.g., `winit::event::WindowEvent`) must be captured as inert physical evidence (`RawBackendEvent`). This evidence remains within the backend boundary and must not instantly mutate semantic state or bypass capability gating.

## 4. SEMANTIC_UI_DNA Compliance
PASS - Raw inputs are treated strictly as inert evidence.
PASS - Backend input capture does not assert semantic truth.
PASS - Handlers are forbidden from bypassing runtime intent or executing actions directly.
PASS - Unknown/Conflict semantics remain visible and unflattened.

docs/dna inspected: YES
DNA files inspected:
- [SEMANTIC_UI_DNA.md](../../dna/SEMANTIC_UI_DNA.md)

DNA alignment:
- Input handling at this boundary translates host physical events into semantic-agnostic generic backend events.
- Direct runtime state mutation is physically disconnected from the raw event capture phase.

## 5. Proposed Future Source Shape
A future source gate will define representations for backend-level events:

```rust
pub enum RawBackendEvent {
    WindowResized { width: u32, height: u32 },
    KeyboardInput { key: RawKeyCode, state: RawButtonState },
    PointerMoved { x: f64, y: f64 },
    CloseRequested,
}

pub enum RawKeyCode { ... }
pub enum RawButtonState { Pressed, Released }
```

The future PR will implement translation from `winit::event::WindowEvent` into `RawBackendEvent` inside `prom-ui-backend-native`.

## 6. Allowed Semantics
Allowed future semantics, if admitted by a later source PR:
- Translating `winit` keyboard, pointer, and window events into an internal enum format.
- Passing the inert event enum down the pipeline.
- Logging raw input diagnostics.

## 7. Forbidden Semantics
Forbidden in this boundary and immediate future source gates:
- No `RawBackendEvent` source code is written in this PR.
- No semantic mapping (e.g., translating `Enter` to `SubmitForm`).
- No hit-testing geometry inside the raw capture boundary.
- No direct calls into `prom-ui` action executors.
- No `winit` types leaking across backend interfaces into `prom-ui` core.

## 8. Event Lifecycle Rules
Raw events are observational evidence. Capturing an event does not guarantee a reaction.
The translation layer must not infer semantic intent (e.g., if spacebar is pressed, it must be captured as a `Spacebar` key event, not a generic "Select" action).
Platform-specific event quirks should be sanitized into a neutral structural format at the edge.

## 9. Dependency Boundary Rules
- `winit` remains isolated inside `prom-ui-backend-native`.
- `prom-ui` core event interfaces (if any are introduced later) must not expose `winit` structs.

## 10. Future-Gated Work
- `R12-UI-RAW-EVENT-CAPTURE-SOURCE-PR`
  - Defines the translation from `winit` to `RawBackendEvent`.
- `R12-UI-INTERACTION-ROUTING-BOUNDARY-PR`
  - Defines how inert events cross the backend boundary to find semantic targets via hit-testing.
- `R12-UI-ACTION-MAPPING-BOUNDARY-PR`
  - Defines how raw events on semantic targets map to structured intent/actions.

## 11. Repository Scope
- source files changed: NO
- test files changed: NO
- docs changed: YES
- `Cargo.toml` changed: NO
- `Cargo.lock` changed: NO
- `docs/dna` changed: NO
- Admission Guard changed: NO
- GitHub CI used: NO

## 12. Final Decision
PASS — R12 UI Raw Event Capture Boundary defined.

This PR defines the future boundary for physical event capture.
It introduces no source code, tests, interaction mappings, hit testing, or semantic execution authority.

## 13. Recommended Next Lane
`R12-UI-RAW-EVENT-CAPTURE-SOURCE-PR`

Do not start it in this PR.
