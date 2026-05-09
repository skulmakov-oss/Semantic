# Native UI Backend Boundary

Status: Draft
Track: POST-UI
Purpose: define the implementation boundary before adding a native backend

## 1. Goal

This document defines the boundary for the first native UI backend implementation.

The native backend must plug into the existing `UiBackendAdapter` seam and must not move
platform-specific behavior into `prom-ui-runtime`.

Current reference path:

```text
DesktopSession<InMemoryBackend>
  -> lifecycle gate
  -> injected events
  -> tick_in_memory_frame(...)
  -> DrawFrame
  -> captured frame transcript
```

Future native path:

```text
DesktopSession<NativeBackend>
  -> lifecycle gate
  -> native event loop
  -> DrawFrame
  -> draw staging/accounting
```

## 2. Current runtime ownership

`prom-ui-runtime` owns:

* `DesktopSession`
* `UiLifecycleGate`
* `SessionState`
* `EventBuffer`
* `InputEvent`
* `InputEventKind`
* `DrawFrame`
* `DrawCommand`
* `Color`
* `Rect`
* `UiBackendAdapter`
* `InMemoryBackend`

`prom-ui-runtime` must remain platform-neutral.

## 3. Native backend ownership

The native backend owns:

* native window creation;
* native event loop integration;
* platform event translation into `InputEvent`;
* platform draw submission from `DrawFrame`;
* native close/window lifecycle interaction.

The native backend must not own:

* Semantic lifecycle rules;
* UI capability admission;
* verifier admission;
* SemCode format;
* VM dispatch;
* Workbench policy.

## 4. First native backend target

The first native backend should be introduced as a separate crate:

```text
crates/prom-ui-backend-native/
```

Preferred public type:

```rust
NativeBackend
```

The crate must implement:

```rust
UiBackendAdapter for NativeBackend
```

The native backend crate may depend on platform/window libraries.
`prom-ui-runtime` must not.

## 5. Dependency boundary

Allowed:

```text
prom-ui-backend-native -> prom-ui-runtime
prom-ui-backend-native -> native window/event crates
```

Forbidden:

```text
prom-ui-runtime -> winit
prom-ui-runtime -> tao
prom-ui-runtime -> wgpu
prom-ui-runtime -> platform-specific APIs
sm-vm -> native UI backend
sm-verify -> native UI backend
prom-ui -> native UI backend
prom-cap -> native UI backend
```

## 6. Adapter contract

The native backend must implement the existing adapter:

```rust
create_window(&WindowConfig) -> Result<(), UiRuntimeError>
run_event_loop(on_event) -> Result<(), UiRuntimeError>
draw_frame(&DrawFrame) -> Result<(), UiRuntimeError>
close_window()
```

No trait changes are allowed in the first native backend PR unless a separate contract PR justifies them.

## 7. Event translation

The first native backend may translate only the current first-wave event set:

```text
KeyDown { key_code }
KeyUp { key_code }
CloseRequested
```

Out of scope for the first native backend:

```text
mouse
touch
gamepad
IME
clipboard
drag/drop
multi-window
browser target
mobile target
```

## 8. Draw translation

The first native backend may translate only the current first-wave draw set:

```text
Clear
FillRect
DrawText
```

Out of scope for the first native backend:

```text
images
paths
fonts API
layout engine
widgets
GPU pipeline abstraction
animation system
```

## 9. Lifecycle rule

The native backend must not bypass `DesktopSession`.

All native UI behavior must still pass through:

```text
DesktopSession
  -> UiLifecycleGate
  -> UiBackendAdapter
```

Invalid lifecycle operations must fail before backend calls.

## 10. Determinism rule

The native backend itself is host-bound and not deterministic by default.

The deterministic reference remains:

```text
InMemoryBackend
```

Native backend tests must compare against reference behavior where possible, but native timing and OS event ordering are outside the deterministic core.

## 11. Native facade transcript boundary

`prom-ui-backend-native` exposes a separate native facade path for winit-backed execution:

```text
NativeBackend
  -> NativeBackendWinitApp
  -> NativeBackendWinitAppState
  -> winit EventLoop::run_app(...)
  -> NativeBackendWinitAppFacadeTranscript
```

This path is intentionally separate from `UiBackendAdapter`.

`NativeBackend::run_event_loop(...)` remains a staged deterministic seam and is not the winit app runner.

### Ownership split

| Layer | Ownership | Status |
|---|---|---|
| `prom-ui-runtime` | platform-neutral session/lifecycle contracts | stable boundary |
| `UiBackendAdapter` | staged backend adapter seam | unchanged |
| `NativeBackend` | staged native backend state and accounting | no persistent native window ownership |
| `NativeBackendWinitApp` | native facade ownership | owns the winit app path |
| `NativeBackendWinitAppState` | `ApplicationHandler` state | owns native window during run |
| renderer | not implemented | out of scope |

### Transcript hierarchy

The native facade path exposes transcript objects for different levels of observation:

| Transcript | Meaning | Renderer implication |
|---|---|---|
| `NativeBackendWinitAppRunTranscript` | event-loop/window lifecycle facts | none |
| `NativeBackendWinitAppEventTranscript` | derived event facts from summary counters | none |
| `NativeBackendWinitAppDrawTranscript` | staged draw-frame accounting facts | none |
| `NativeBackendWinitAppFacadeTranscript` | combined run/event/draw facts | none |

Draw transcript facts are staging/accounting only. They do not mean that a frame was rendered or presented.

### Current admitted path

The currently admitted native path is:

```text
staged WindowConfig
  -> NativeBackendWinitApp::new(...)
  -> NativeBackendWinitAppState
  -> EventLoop::run_app(...)
  -> summary/transcript
```

The currently admitted draw path is:

```text
DrawFrame
  -> NativeBackendWinitApp::stage_draw_frame(...)
  -> NativeBackend submitted_frames accounting
  -> NativeBackendWinitAppDrawTranscript
```

The draw path is draw staging/accounting only.

## Renderer admission boundary

Renderer implementation is intentionally not part of the native facade transcript boundary.

The renderer boundary is defined separately in:

```text
docs/architecture/ui_renderer_admission_boundary.md
```

Until that boundary is admitted, draw facts remain staging/accounting only:

```text
DrawFrame
  -> stage_draw_frame(...)
  -> submitted_frames accounting
  -> draw transcript
```

No renderer, surface, GPU, native drawing, or frame presentation is admitted by the native facade transcript track.

## 12. First implementation PR boundary

The first implementation PR after this document should only add:

```text
crates/prom-ui-backend-native/
Cargo.toml
src/lib.rs
basic NativeBackend skeleton
compile tests
```

It should not add:

```text
Workbench integration
VM integration
SemCode integration
verifier changes
capability changes
new UI operations
new draw commands
new input event kinds
```

## 13. Explicit non-goals

The native facade transcript boundary does not introduce:

* renderer ownership;
* surface/pixels/wgpu integration;
* native drawing calls;
* frame presentation;
* changes to `UiBackendAdapter`;
* changes to `prom-ui-runtime`;
* integration of winit into `NativeBackend::run_event_loop(...)`;
* Workbench integration.

### Renderer ownership is not admitted yet

The renderer is not owned by `prom-ui-runtime`, `UiBackendAdapter`, or `NativeBackend::run_event_loop(...)`.

Renderer ownership must be introduced through a separate admitted layer.

Current status:

| Component | Renderer ownership |
|---|---|
| `prom-ui-runtime` | none |
| `UiBackendAdapter` | none |
| `NativeBackend` | none |
| `NativeBackendWinitApp` | none |
| `NativeBackendWinitAppState` | none |
| future renderer type/crate | not admitted yet |

Draw staging is not renderer ownership.

## Interaction boundary

Native backend may translate host events into `InputEvent`.

Native backend must not interpret those events as admitted semantic actions.

Interaction semantics are defined separately in:

```text
docs/architecture/ui_interaction_input_semantic_boundary.md
```

## Focus and selection boundary

Native backend must not own semantic focus or semantic selection.

Native backend may translate host events into `InputEvent`.

Focus and selection semantics are defined separately in:

```text
docs/architecture/ui_focus_selection_semantic_boundary.md
```

## Semantic action boundary

Native backend must not own Semantic UI actions.

Native backend may translate host events into `InputEvent`.

Semantic UI actions are defined separately in:

```text
docs/architecture/ui_semantic_action_boundary.md
```

## Effect request boundary

Native backend must not perform Semantic UI effects directly from host events.

Effect requests and UI capabilities are defined separately in:

```text
docs/architecture/ui_effect_request_capability_boundary.md
```

Native backend may perform platform operations only after an admitted effect/lifecycle boundary exists.

## Trace and audit visual boundary

Native backend may expose transcript facts.

Native backend must not define audit meaning.

Trace/audit visual boundaries are defined separately in:

```text
docs/architecture/ui_trace_audit_visual_boundary.md
```

## Error, denial, and quarantine boundary

Native backend may expose native failure facts.

Native backend must not define semantic denial, conflict, or quarantine meaning.

Error, denial, and quarantine visual boundaries are defined separately in:

```text
docs/architecture/ui_error_denial_quarantine_visual_boundary.md
```

## Recovery and rollback boundary

Native backend may expose native failure facts.

Native backend must not define recovery or rollback semantics.

Recovery and rollback visual boundaries are defined separately in:

```text
docs/architecture/ui_recovery_rollback_visual_boundary.md
```

## Renderer transcript and presentation status boundary

Native backend must not treat submitted frames as presented frames.

Renderer transcript and presentation status boundaries are defined separately in:

```text
docs/architecture/ui_renderer_transcript_presentation_boundary.md
```

Native backend may participate in presentation only after an admitted renderer/native presentation boundary exists.

## 14. Stop rules

Stop the native backend implementation if it requires:

* changing `UiBackendAdapter` immediately;
* adding platform dependencies to `prom-ui-runtime`;
* storing native handles in `DesktopSession`;
* changing lifecycle semantics;
* adding VM/verifier/SemCode coupling;
* making Workbench the owner of runtime UI semantics.
