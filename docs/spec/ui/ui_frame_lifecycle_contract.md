# UI Frame Lifecycle Contract

Status: Draft
Track: POST-UI
Depends on:
- `host_runtime_effect_path_boundary.md`
- `ui_effect_envelope_v0.md`
- `ui_capability_taxonomy.md`
- `ui_event_envelope_model.md`
Scope: UI frame lifecycle protocol only
Implementation: out of scope

Related:

- `README.md`
- `../../architecture/ui_full_effect_trace_ladder.md`
- `../../architecture/ui_committed_effect_boundary.md`
- `ui_draw_command_batch_contract.md`

## 1. Purpose

This document defines the legal frame lifecycle for Semantic UI effects.

```text
WindowCreate
  -> PollEvents
  -> BeginFrame
  -> SubmitDrawCommands*
  -> EndFrame
  -> repeat
  -> WindowClose
```

The frame lifecycle is a runtime-scoped protocol, not a renderer implementation.

## 2. Relationship to I50-I53

- `I50` defines the host runtime effect path.
- `I51` defines `UiEffectEnvelope`.
- `I52` defines UI capabilities for frame effects.
- `I53` defines deterministic external UI event delivery.
- `I54` defines legal ordering and state transitions for frame effects.

Frame-related effects from `I51`:

- `BeginFrame`
- `SubmitDrawCommands`
- `EndFrame`

The minimal batch contract for `SubmitDrawCommands` is defined in
`ui_draw_command_batch_contract.md`.

Frame-related capabilities from `I52`:

- `UiFrameBegin`
- `UiDrawSubmit`
- `UiFrameEnd`

## 3. Core rule

```text
A frame is a bounded runtime scope for UI output submission.
```

Rules:

- no draw submission outside an active frame;
- no frame presentation without `EndFrame`;
- no platform rendering from the VM.

## 4. Frame lifecycle state machine

```text
WindowFrameState =
  NoWindow
  WindowReady
  FrameActive(frame_id)
  FrameEnded(frame_id)
  WindowClosing
  WindowClosed
```

### 4.1 Legal transitions

```text
NoWindow
  -> WindowReady              via WindowCreate

WindowReady
  -> WindowReady              via PollEvents
  -> FrameActive(frame_id)    via BeginFrame
  -> WindowClosing            via WindowClose

FrameActive(frame_id)
  -> FrameActive(frame_id)    via SubmitDrawCommands
  -> FrameEnded(frame_id)     via EndFrame

FrameEnded(frame_id)
  -> WindowReady              after frame outcome recorded

WindowClosing
  -> WindowClosed             after WindowClose outcome

WindowClosed
  -> terminal for that window_id
```

### 4.2 Forbidden transitions

```text
NoWindow -> BeginFrame
NoWindow -> SubmitDrawCommands
NoWindow -> EndFrame

WindowReady -> SubmitDrawCommands
WindowReady -> EndFrame

FrameActive -> BeginFrame for same window
FrameEnded -> SubmitDrawCommands
FrameEnded -> EndFrame

WindowClosed -> PollEvents
WindowClosed -> BeginFrame
WindowClosed -> SubmitDrawCommands
WindowClosed -> EndFrame
```

## 5. Window lifecycle relation

Frame lifecycle is nested inside window lifecycle.

Rules:

- `WindowCreate` opens the path into `WindowReady`;
- `WindowClose` terminates the window lifecycle;
- active frame work must belong to a live window;
- closed windows cannot receive new frame effects.

## 6. BeginFrame contract

### Purpose

```text
BeginFrame opens a bounded frame scope for one window.
```

### Required preconditions

- target `window_id` exists;
- window belongs to current runtime session;
- window state is `WindowReady`;
- there is no active frame for that window;
- envelope has `UiFrameBegin` capability;
- budget class `FrameLifecycle` is available;
- audit class `Trace` is available.

### Result

```text
Accepted(FrameId)
```

### Denial / invalid cases

- missing `UiFrameBegin` capability;
- invalid `window_id`;
- window belongs to another session;
- frame already active;
- frame lifecycle budget exceeded;
- envelope target missing `window_id`.

### Important rule

```text
BeginFrame does not draw and does not present.
```

## 7. SubmitDrawCommands contract

### Purpose

```text
SubmitDrawCommands submits a bounded draw batch into an active frame scope.
```

### Required preconditions

- target `window_id` exists;
- target `frame_id` exists;
- frame_id is active;
- frame_id belongs to target `window_id`;
- envelope has `UiDrawSubmit` capability;
- budget class `DrawSubmission` is available;
- `command_count` is bounded;
- command buffer is runtime-owned or validated;
- no raw GPU or platform handles are included.

### Result

```text
Accepted(Unit)
```

### Denial / invalid cases

- missing `UiDrawSubmit` capability;
- no active frame;
- `frame_id` mismatch;
- `command_count` exceeds budget;
- command buffer invalid;
- command payload includes raw OS or GPU handles;
- `SubmitDrawCommands` after `EndFrame`.

### Important rule

```text
I54 does not define draw command binary format.
```

## 8. EndFrame contract

### Purpose

```text
EndFrame closes a frame scope and allows the platform adapter to finalize/present.
```

### Required preconditions

- target `window_id` exists;
- target `frame_id` exists;
- frame_id is active;
- frame_id belongs to target `window_id`;
- envelope has `UiFrameEnd` capability;
- budget class `FrameLifecycle` is available;
- audit class `Trace` is available.

### Result

```text
Accepted(FrameResult)
```

FrameResult v0 may be documented as:

```text
FrameResult {
  frame_id,
  status,
  submitted_draw_batches,
  audit_outcome_id?
}
```

### Denial / invalid cases

- missing `UiFrameEnd` capability;
- `EndFrame` without `BeginFrame`;
- double `EndFrame`;
- inactive `frame_id`;
- `frame_id` / `window_id` mismatch;
- frame lifecycle budget exceeded.

## 9. PollEvents relation

`PollEvents` is part of the UI loop, but not part of frame submission.

Allowed:

```text
WindowReady -> PollEvents -> WindowReady
```

Strict v0 recommendation:

```text
PollEvents during FrameActive is not admitted.
```

Reason:

```text
It avoids interleaving host input reads with output frame submission.
```

So the v0 loop should be:

```text
PollEvents
  -> update app state
  -> BeginFrame
  -> SubmitDrawCommands*
  -> EndFrame
```

Future extensions may allow event polling during active frames only with an
explicit interleaving policy.

## 10. FrameId ownership and scope

Rules:

- `FrameId` is runtime-scoped;
- `FrameId` is session-local;
- `FrameId` is window-bound;
- `FrameId` is not a capability;
- `FrameId` is not a platform handle;
- `FrameId` is not reusable after `EndFrame`.

Forbidden:

- `FrameId` reused across windows;
- `FrameId` reused across sessions;
- `FrameId` treated as OS handle;
- `FrameId` used after `WindowClose`;
- `FrameId` accepted without active frame state.

## 11. Ordering model

Within one window:

```text
BeginFrame(frame_id)
  -> SubmitDrawCommands(frame_id)*
  -> EndFrame(frame_id)
```

For v0, the recommended constraint is:

```text
One active frame per window.
```

A runtime profile may restrict this further to one active frame per session.

## 12. Budget hooks

Frame lifecycle consumes budget in two places:

```text
BeginFrame / EndFrame -> FrameLifecycle
SubmitDrawCommands    -> DrawSubmission
```

Budget constraints may include:

- max active frames;
- max frames per tick or cycle;
- max draw submissions per frame;
- max draw commands per frame;
- max command buffer bytes;
- max frame lifecycle churn.

This document defines slots, not numeric limits.

## 13. Audit hooks

Recommended audit:

| Event | Audit |
| --- | --- |
| `BeginFrame` | `Trace` |
| `SubmitDrawCommands` | `Trace` or budgeted summary |
| `EndFrame` | `Trace` |
| invalid frame transition | denial audit |
| platform failure after `EndFrame` | outcome audit |

Frame lifecycle should be replay-explainable.

Audit should be able to reconstruct:

- `window_id`;
- `frame_id`;
- begin order;
- number of draw submissions;
- total `command_count`;
- end status;
- denial or failure if any.

## 14. Determinism and replay policy

```text
Frame lifecycle requests are deterministic data.
Platform presentation timing is host-dependent.
VM-visible frame result must be explicit and replayable.
```

Forbidden:

- VM asks host whether frame is currently visible;
- VM reads platform refresh rate directly;
- VM uses wall-clock frame timestamp without explicit policy;
- platform callback mutates VM frame state directly.

Allowed:

```text
FrameResult is delivered as explicit runtime result.
```

## 15. Invalid lifecycle conditions

`InvalidFrameLifecycle` applies when any of these hold:

- `BeginFrame` without valid window;
- `BeginFrame` while frame already active for same window;
- `SubmitDrawCommands` without active frame;
- `SubmitDrawCommands` with mismatched `window_id` / `frame_id`;
- `SubmitDrawCommands` after `EndFrame`;
- `EndFrame` without active frame;
- `EndFrame` twice for same `frame_id`;
- `FrameId` reused after `EndFrame`;
- `WindowClose` during active frame without explicit abort policy;
- `PollEvents` during `FrameActive` in v0;
- frame payload contains raw platform or GPU handles.

For `WindowClose` during active frame, the v0 rule is:

```text
WindowClose during active frame is invalid unless the runtime first aborts the
frame through a future explicit FrameAbort policy.
```

`FrameAbort` is reserved and not defined in v0.

## 16. Reserved future frame effects

Not admitted in v0:

- `FrameAbort`
- `FrameResize`
- `FrameSuspend`
- `FrameResume`
- `SwapchainRecreate`
- `SurfaceLostRecover`
- `VSyncWait`
- `GpuFenceWait`
- `TextureUpload`
- `ShaderBind`

These are renderer or backend specific, or require deeper platform policy.

## 17. Forbidden bypasses

Forbidden bypasses include:

- `SubmitDrawCommands` without `BeginFrame`;
- platform adapter presents without `EndFrame`;
- VM receives raw platform frame callback;
- renderer mutates VM state;
- `FrameId` used as capability;
- `WindowId` used as capability;
- raw GPU handle included in frame payload;
- debug mode skips frame lifecycle checks;
- `prom-ui-runtime` dispatches frame effect before runtime admission.

## 18. Extension policy

New frame effects require:

- state transition;
- capability mapping;
- budget class;
- audit class;
- result shape;
- invalid transition rules.

No new frame effect without policy metadata.

## 19. Out of scope

This document does not add:

- Rust structs;
- ABI calls;
- VM changes;
- verifier changes;
- `prom-ui-runtime` implementation;
- platform adapter implementation;
- renderer;
- GPU or shader pipeline;
- draw command binary format;
- widget/layout framework;
- actual event loop;
- tests beyond docs/link checks;
- `.claude/`.

## 20. Acceptance checklist

- `docs/spec/ui/ui_frame_lifecycle_contract.md` exists;
- `docs/spec/ui/README.md` links it;
- `docs/spec/index.md` links it;
- `ui_effect_envelope_v0.md` cross-links it;
- the document references `I50-I53`;
- frame state machine is defined;
- `BeginFrame` contract is defined;
- `SubmitDrawCommands` contract is defined;
- `EndFrame` contract is defined;
- `PollEvents` relation is defined;
- `FrameId` ownership is defined;
- invalid lifecycle transitions are listed;
- budget hooks are defined;
- audit hooks are defined;
- determinism and replay policy is defined;
- reserved future frame effects are listed;
- forbidden bypasses are listed;
- no code changes;
- no ABI widening;
- no renderer or backend implementation.
