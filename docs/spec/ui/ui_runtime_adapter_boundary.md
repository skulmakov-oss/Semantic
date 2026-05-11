# UI Runtime Adapter Boundary

Status: Draft
Track: POST-UI
Depends on:
- `host_runtime_effect_path_boundary.md`
- `ui_effect_envelope_v0.md`
- `ui_capability_taxonomy.md`
- `ui_event_envelope_model.md`
- `ui_frame_lifecycle_contract.md`
- `ui_draw_command_batch_contract.md`
Scope: UI runtime-to-platform adapter boundary only
Implementation: out of scope

Related:

- `README.md`
- `../../architecture/ui_native_backend_boundary.md`
- `../../architecture/ui_renderer_admission_boundary.md`

## 1. Purpose

This document defines the boundary between `prom-ui-runtime` and platform
adapters.

```text
prom-ui-runtime owns normalized UI runtime semantics.
Platform adapters own OS-specific execution.
Neither side may bypass runtime admission, capability, budget, or audit.
```

`I50-I55` describe what passes across the UI boundary.
`I56` describes who is allowed to execute it on the host/platform side.

## 2. Relationship to I50-I55

- `I50` defines the host runtime effect path.
- `I51` defines `UiEffectEnvelope`.
- `I52` defines UI capability policy.
- `I53` defines UI event input model.
- `I54` defines frame lifecycle.
- `I55` defines draw batch contract.
- `I56` defines runtime-to-platform adapter boundary.

The boundary sits below `prom-runtime` admission and above OS/platform-specific
implementation:

```text
prom-runtime
  -> admitted UiEffectEnvelope
  -> prom-ui-runtime
  -> normalized adapter request
  -> platform adapter
  -> normalized adapter result
  -> prom-ui-runtime
  -> audit/result boundary
```

## 3. Core rule

```text
Platform adapters are dumb executors of admitted, normalized UI runtime requests.
```

Rules:

- platform adapters do not decide capability;
- platform adapters do not decide audit;
- platform adapters do not inspect VM state;
- platform adapters do not interpret SemCode.

## 4. Layer ownership

### 4.1 `prom-runtime`

Owns:

- runtime session;
- effect admission orchestration;
- capability check;
- budget check;
- audit intent/outcome coordination;
- VM-visible result boundary.

Must not own:

- OS window handles;
- platform event loop;
- native renderer;
- adapter internals.

### 4.2 `prom-ui-runtime`

Owns:

- normalized UI runtime state;
- logical window/session registry;
- mapping admitted `UiEffectEnvelope` to adapter request;
- normalized event envelope production;
- frame lifecycle tracking;
- draw batch validation handoff;
- adapter result normalization;
- platform error classification.

Must not own:

- VM instruction semantics;
- SemCode parsing;
- capability source of truth;
- audit storage implementation;
- OS-specific handles as public semantic values;
- renderer backend internals.

### 4.3 Platform adapter

Examples:

- `prom-ui-adapter-recording`
- `prom-ui-adapter-windows`
- `prom-ui-adapter-macos`
- `prom-ui-adapter-linux`

Owns:

- OS window handle;
- platform event polling;
- native surface access;
- platform draw submission;
- platform error translation;
- backend-specific resource lifetime.

Must not own:

- capability decision;
- audit policy decision;
- VM state;
- SemCode verification;
- frontend/compiler structures;
- Semantic source interpretation.

## 5. Adapter request model

The pseudo-shape is:

```text
UiAdapterRequestV0 {
  request_version: 0,
  request_id: AdapterRequestId,

  session_id,
  effect_id,
  target,

  normalized_payload,
  budget_snapshot,
  audit_correlation,
  replay_metadata?
}
```

Rules:

- adapter request is created only after runtime admission;
- adapter request is not a capability;
- adapter request is not a VM instruction;
- adapter request must not contain raw VM or frame internals;
- adapter request must not contain frontend or compiler objects.

## 6. Adapter result model

```text
UiAdapterResultV0 =
  Performed(value)
  Rejected(AdapterRejected)
  Failed(PlatformFailure)
```

### 6.1 `Performed`

```text
Performed {
  request_id,
  value,
  platform_summary?
}
```

Allowed normalized values:

- `Unit`
- `WindowCreated(logical_window_id)`
- `FramePresented(logical_frame_result)`
- `EventBatch(normalized_events)`

No OS handle is returned to the VM.

### 6.2 `Rejected`

```text
Rejected {
  request_id,
  reason
}
```

Use when adapter refuses a normalized admitted request because it cannot support
it.

Examples:

- `UnsupportedWindowMode`
- `UnsupportedDrawCommand`
- `UnsupportedTextFeature`
- `UnsupportedSurfaceMode`

### 6.3 `Failed`

```text
Failed {
  request_id,
  platform_failure_class,
  reason
}
```

Examples:

- `WindowUnavailable`
- `SurfaceLost`
- `BackendUnavailable`
- `PlatformPermissionDenied`
- `NativeEventLoopFailure`

Adapter failure is not capability denial.
Capability denial happens before adapter dispatch.

## 7. Window handle boundary

Core rule:

```text
OS handles stay inside the platform adapter.
Semantic-visible WindowId is logical and session-scoped.
```

Forbidden:

- returning `HWND` / `NSWindow*` / X11 / Wayland handles to the VM;
- storing raw OS handles in `UiEffectEnvelope`;
- treating `WindowId` as an OS handle;
- treating `WindowId` as a capability;
- sharing OS handles through audit logs.

Allowed:

```text
Platform adapter may internally map:
WindowId -> native OS handle
```

But this mapping is:

- private;
- adapter-owned;
- session-bound;
- not serializable as Semantic program state.

## 8. Event ingestion boundary

`I53` defined normalized event envelopes. `I56` defines who creates them.

```text
Platform adapter reads native events.
prom-ui-runtime normalizes them into UiEventEnvelope / UiEventBatch.
VM consumes only normalized event batch.
```

Forbidden:

- VM reads native event queue;
- platform adapter pushes events directly into VM;
- native platform event structs become VM-visible values;
- event callback mutates VM state;
- unbounded event stream enters runtime.

Allowed:

```text
Platform adapter returns bounded raw/adapter event batch to prom-ui-runtime.
prom-ui-runtime normalizes, orders, bounds, and records it.
```

## 9. Frame and draw dispatch boundary

`I54` and `I55` define legal frame and draw batch contracts.

```text
prom-ui-runtime:
  validates admitted frame/draw protocol shape
  prepares normalized adapter draw request
  tracks logical frame state

platform adapter:
  translates normalized draw commands to backend-specific drawing
  owns native surface and renderer internals
```

Forbidden:

- adapter accepts draw commands outside active frame;
- adapter presents without `EndFrame`;
- adapter creates its own frame lifecycle;
- adapter bypasses `DrawSubmission` budget;
- adapter mutates frame state without reporting normalized result.

## 10. Platform error mapping

Platform adapter errors are normalized into:

- `Rejected`
- `Failed`

The adapter must classify errors rather than exposing native backend types.

## 11. Audit handoff

Audit should see:

- effect envelope id;
- adapter request id;
- logical `window_id` / `frame_id`;
- `effect_id`;
- already admitted capability;
- budget class consumed;
- adapter result class;
- platform failure class if any;
- redacted summaries for draw, text, and event payload.

Audit should not see by default:

- raw OS handles;
- raw platform event structs;
- raw GPU handles;
- full text payload unless policy allows;
- native renderer state.

## 12. Determinism and replay boundary

```text
prom-ui-runtime must expose deterministic normalized data to the VM.
Platform adapter execution may be platform-dependent.
```

Replay target:

```text
Replay reproduces:
- admitted effect sequence;
- normalized event batches;
- frame lifecycle sequence;
- draw command batches;
- normalized adapter results.

Replay does not guarantee:
- pixel-identical output across OS/renderers;
- same native timing;
- same native window-manager behavior.
```

## 13. No-op / recording adapter

A recording adapter is a future deterministic test adapter.

```text
Recording adapter =
  deterministic test adapter
  records admitted adapter requests
  returns normalized synthetic results
  owns no real OS window
  performs no real drawing
```

Purpose:

- future smoke tests;
- replay validation;
- CI without desktop session;
- negative path validation.

No implementation in this PR.

## 14. Forbidden dependencies

`prom-ui-runtime` must not depend on:

- `sm-front`;
- `sm-ir`;
- `sm-emit`;
- parser internals;
- compiler AST;
- platform-specific adapter crates directly, if using a trait/dynamic boundary
  later;
- renderer implementation internals.

Platform adapter must not depend on:

- `sm-front`;
- `sm-ir`;
- `sm-emit`;
- `sm-vm` internals;
- `prom-cap` policy internals as decision owner;
- source files / compiler diagnostics.

This keeps UI runtime from turning into a compiler/runtime hybrid.

## 15. Forbidden bypasses

- platform adapter called before capability admission;
- platform adapter called before budget admission;
- platform adapter called before audit intent when required;
- adapter returns raw OS handles to VM;
- adapter pushes event callback directly into VM;
- adapter performs policy decisions;
- adapter interprets SemCode;
- adapter reads Semantic source;
- adapter bypasses frame lifecycle;
- adapter bypasses draw batch bounds;
- debug/dev mode bypasses runtime admission.

## 16. Reserved future adapter classes

Not part of v0 implementation:

- `RecordingAdapter`
- `HeadlessAdapter`
- `WindowsAdapter`
- `MacOSAdapter`
- `LinuxWaylandAdapter`
- `LinuxX11Adapter`
- `WebCanvasAdapter`
- `MobileAdapter`
- `GpuAdapter`
- `RemoteAdapter`

These may be named as future categories, but must not be defined as
implementations here.

## 17. Extension policy

No new adapter boundary extension without:

- ownership statement;
- request shape;
- result shape;
- error mapping;
- audit handoff;
- determinism and replay impact;
- forbidden bypass review;
- no raw handle exposure rule.

## 18. Out of scope

This document does not add:

- Rust traits;
- crate creation;
- `prom-ui-runtime` implementation;
- platform adapter implementation;
- OS window backend;
- renderer;
- GPU or shader pipeline;
- draw command binary encoding;
- actual event loop;
- widget/layout framework;
- scene graph;
- font engine;
- image decoder;
- texture upload;
- ABI widening;
- VM changes;
- verifier changes;
- tests beyond docs/link checks;
- `.claude/`.

## 19. Acceptance checklist

- `docs/spec/ui/ui_runtime_adapter_boundary.md` exists;
- `docs/spec/ui/README.md` links it;
- `docs/spec/index.md` links it;
- `host_runtime_effect_path_boundary.md` cross-links it;
- `ui_draw_command_batch_contract.md` cross-links it;
- the document references `I50-I55`;
- layer ownership is defined;
- `prom-ui-runtime` responsibilities are defined;
- platform adapter responsibilities are defined;
- adapter request/result model is defined;
- OS handle boundary is defined;
- event ingestion boundary is defined;
- frame/draw dispatch boundary is defined;
- audit handoff is defined;
- determinism and replay boundary is defined;
- recording/no-op adapter role is reserved;
- forbidden dependencies are listed;
- forbidden bypasses are listed;
- no code changes;
- no ABI widening;
- no adapter/backend implementation.
