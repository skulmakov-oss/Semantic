# UI Effect Envelope v0

Status: Draft
Track: POST-UI
Depends on: `host_runtime_effect_path_boundary.md`
Scope: UI effect envelope contract only
Implementation: out of scope

Related:

- `README.md`
- `ui_capability_taxonomy.md`
- `../../architecture/ui_host_runtime_effect_boundary.md`
- `../../architecture/ui_full_effect_trace_ladder.md`
- `../abi.md`
- `../capabilities.md`
- `../audit.md`
- `../runtime.md`
- `../vm.md`

## 1. Purpose

`UiEffectEnvelope v0` defines the minimal policy-carrying request object that
travels across the host runtime effect path defined in
`host_runtime_effect_path_boundary.md`.

The envelope is the thing the Semantic VM emits for a UI-visible host effect.
It is intent, not execution.

The runtime admits the envelope before `prom-ui-runtime` dispatches it to a
platform adapter.

## 2. Relationship to I50

`I50` defines where the UI effect travels.

This document defines what travels on that path.

In short:

```text
I50 = where the UI effect goes
I51 = what the UI effect envelope contains
```

## 3. Core rule

```text
UiEffectEnvelope v0 =
  explicit, validated, policy-carrying request
  for a UI-visible host effect
  emitted by VM execution
  and admitted by prom-runtime before prom-ui-runtime dispatch.
```

Short form:

```text
UiEffectEnvelope is intent, not execution.
```

## 4. Envelope lifecycle

The required lifecycle is:

```text
1. VM reaches host-effect instruction / host-call surface
2. VM creates UiEffectEnvelope
3. prom-runtime validates envelope shape
4. prom-runtime checks capability
5. prom-runtime checks budget
6. prom-runtime records audit intent when required
7. prom-ui-runtime dispatches admitted effect
8. platform adapter performs host-visible operation
9. audit outcome is recorded
10. UiEffectResult returns to VM/runtime boundary
```

An envelope must not be executable by itself.

## 5. Envelope schema v0

The v0 shape is:

```text
UiEffectEnvelopeV0 {
  envelope_version: 0,
  envelope_id: UiEnvelopeId,
  effect_id: UiEffectId,

  source: UiEffectSource,
  target: UiEffectTarget,

  policy: UiEffectPolicy,
  payload: UiEffectPayload,

  determinism: UiDeterminismMetadata,
  audit: UiAuditMetadata
}
```

## 6. Field definitions

### 6.1 `envelope_version`

```text
envelope_version: 0
```

Rules:

- fixes the envelope format;
- allows future major versions without silent reinterpretation;
- runtime must reject unknown major versions.

Unknown `envelope_version` is `InvalidEnvelope`.

### 6.2 `envelope_id`

```text
envelope_id: UiEnvelopeId
```

Rules:

- correlation id inside a runtime session;
- links request, audit intent, dispatch, and outcome;
- not a global persistent identifier;
- unique within one runtime session.

### 6.3 `effect_id`

```text
effect_id: UiEffectId
```

Rules:

- identifies the host-visible UI action;
- selects the payload schema;
- drives capability, budget, and audit mapping.

### 6.4 `source`

Recommended logical form:

```text
UiEffectSource {
  program_digest,
  function_symbol?,
  instruction_offset?,
  call_index
}
```

Rules:

- used for traceability, audit, debugging, replay, and denial diagnostics;
- must not contain frontend ASTs, parser nodes, or source text.

Forbidden in `source`:

- `Expr`
- `Stmt`
- span-heavy AST objects
- frontend symbol table
- raw source fragments

### 6.5 `target`

```text
UiEffectTarget {
  window_id?,
  frame_id?,
  surface_id?
}
```

Target requirements:

| Effect | Required target |
| --- | --- |
| `WindowCreate` | none |
| `WindowClose` | `window_id` |
| `PollEvents` | optional `window_id` |
| `BeginFrame` | `window_id` |
| `SubmitDrawCommands` | `window_id + frame_id` |
| `EndFrame` | `window_id + frame_id` |

### 6.6 `policy`

```text
UiEffectPolicy {
  declared_capability,
  budget_class,
  audit_class
}
```

Rules:

- runtime can check the effect before dispatch;
- `prom-ui-runtime` must not accept an effect without admitted policy;
- platform adapter does not decide policy.

### 6.7 `payload`

Payload is typed by `effect_id`.

Forbidden payload forms:

- arbitrary JSON blob;
- platform-native pointer;
- OS handle;
- callback function;
- raw renderer object.

### 6.8 `determinism`

```text
UiDeterminismMetadata {
  class,
  replay_policy,
  external_input
}
```

Classes:

| Class | Meaning |
| --- | --- |
| `DeterministicRequest` | effect request itself is deterministic |
| `ExternalInputRead` | result depends on host event stream |
| `HostVisibleOutput` | effect changes visible host UI state |
| `PlatformDependentFailure` | failure may depend on host backend |

The envelope is deterministic data.
The host event source is not deterministic.
A VM-visible event result must be explicit, bounded, normalized, and replayable.

### 6.9 `audit`

```text
UiAuditMetadata {
  audit_class,
  intent_required,
  outcome_required,
  correlation_id?
}
```

Audit classes:

| Class | Meaning |
| --- | --- |
| `None` | no audit required |
| `Trace` | low-cost trace-level audit |
| `IntentOutcome` | audit before and after effect |
| `Sensitive` | reserved for future clipboard / file picker / input capture |

Recommended v0 mapping:

| Effect | Audit |
| --- | --- |
| `WindowCreate` | `IntentOutcome` |
| `WindowClose` | `IntentOutcome` |
| `PollEvents` | `Trace` |
| `BeginFrame` | `Trace` |
| `SubmitDrawCommands` | `Trace` / budgeted |
| `EndFrame` | `Trace` |

## 7. Allowed `UiEffectId` v0

The admitted v0 set is:

```text
WindowCreate
WindowClose
PollEvents
BeginFrame
SubmitDrawCommands
EndFrame
```

### 7.1 `WindowCreate`

Purpose:

```text
Request creation of a host-visible application window.
```

Payload:

```text
WindowCreatePayload {
  title,
  width,
  height,
  resizable,
  initial_visibility
}
```

Returns:

```text
WindowId
```

No platform handle is returned to the VM.

### 7.2 `WindowClose`

Purpose:

```text
Request closing an existing window.
```

Payload:

```text
WindowClosePayload {
  window_id
}
```

Returns:

```text
Unit
```

Invalid examples:

- unknown `window_id`;
- `window_id` owned by another runtime session;
- already closed window.

### 7.3 `PollEvents`

Purpose:

```text
Request a bounded batch of normalized UI input events.
```

Payload:

```text
PollEventsPayload {
  window_id?,
  max_events,
  timeout_policy
}
```

Returns:

```text
UiEventBatch
```

Rules:

- `max_events` must be budgeted;
- timeout must not create nondeterministic VM blocking semantics;
- events must be normalized before VM-visible delivery.

### 7.4 `BeginFrame`

Purpose:

```text
Open a frame submission scope for a window.
```

Payload:

```text
BeginFramePayload {
  window_id
}
```

Returns:

```text
FrameId
```

Rules:

- cannot begin a second active frame for the same window unless explicitly allowed later;
- `frame_id` must be runtime-scoped;
- no platform draw occurs at `BeginFrame`.

### 7.5 `SubmitDrawCommands`

Purpose:

```text
Submit a bounded list of draw commands for the current frame.
```

Payload:

```text
SubmitDrawCommandsPayload {
  window_id,
  frame_id,
  command_count,
  command_buffer_ref
}
```

This document does not define the draw command binary format.

Rules:

- `command_count` must be budgeted;
- command buffer must be runtime-owned or validated;
- no raw GPU handles;
- no shader pipeline;
- no platform-native renderer object.

### 7.6 `EndFrame`

Purpose:

```text
Close frame submission and allow platform adapter to present / finalize.
```

Payload:

```text
EndFramePayload {
  window_id,
  frame_id
}
```

Returns:

```text
FrameResult
```

Rules:

- `frame_id` must be active;
- `EndFrame` without `BeginFrame` is `InvalidEnvelope`;
- double `EndFrame` is `InvalidEnvelope`.

## 8. Reserved effects

The following are reserved but not admitted in v0:

```text
ClipboardRead
ClipboardWrite
FilePickerOpen
DragDropRead
TextInputIME
RawPointerCapture
GamepadInput
AudioOutput
GPUDeviceCreate
ShaderCompile
TextureUpload
```

These effects are sensitive, platform-heavy, or require separate capability and
audit policy.

## 9. Result model

```text
UiEffectResultV0 =
  Accepted(UiEffectValue)
  Denied(UiEffectDenial)
  Failed(UiEffectFailure)
```

### 9.1 `Accepted`

```text
Accepted {
  envelope_id,
  value,
  audit_outcome_id?
}
```

Allowed accepted values:

- `Unit`
- `WindowId`
- `FrameId`
- `UiEventBatch`
- `FrameResult`

### 9.2 `Denied`

```text
Denied {
  envelope_id,
  code,
  reason,
  audit_event_id?
}
```

Denial codes:

- `CapabilityDenied`
- `BudgetExceeded`
- `AuditRequired`
- `InvalidEnvelope`
- `UnsupportedEffect`

Denied is not platform failure.
Denied means runtime admission refused the effect.

### 9.3 `Failed`

```text
Failed {
  envelope_id,
  code,
  reason,
  platform_failure_class?
}
```

Failure codes:

- `PlatformFailure`
- `WindowUnavailable`
- `SurfaceLost`
- `BackendUnavailable`
- `AdapterRejected`

Failed means the effect was admitted but could not be performed by the UI
runtime or platform adapter.

## 10. Invalid envelope conditions

`InvalidEnvelope` applies when any of these are true:

- unknown `envelope_version`;
- unknown `effect_id`;
- missing required target;
- target field present where forbidden;
- payload does not match `effect_id`;
- payload exceeds budget-declared bounds;
- declared capability does not match `effect_id`;
- audit class is weaker than required minimum;
- envelope contains raw OS handle;
- envelope contains function pointer or callback;
- envelope references frontend or compiler internal object;
- `BeginFrame` / `EndFrame` sequence is invalid;
- `SubmitDrawCommands` references inactive frame;
- `PollEvents` has unbounded `max_events`.

## 11. Capability, budget, and audit placeholders

This document does not finalize the full UI capability taxonomy.

Required policy slots exist:

| Effect | Required capability placeholder | Budget class | Audit class |
| --- | --- | --- | --- |
| `WindowCreate` | `UiWindowCreate` | `WindowLifecycle` | `IntentOutcome` |
| `WindowClose` | `UiWindowClose` | `WindowLifecycle` | `IntentOutcome` |
| `PollEvents` | `UiEventRead` | `EventRead` | `Trace` |
| `BeginFrame` | `UiFrameWrite` | `FrameLifecycle` | `Trace` |
| `SubmitDrawCommands` | `UiDrawSubmit` | `DrawSubmission` | `Trace` |
| `EndFrame` | `UiFrameWrite` | `FrameLifecycle` | `Trace` |

Exact capability taxonomy is defined by `PR-UI-I52`.

Unbounded event polling and unbounded draw submission are forbidden.

## 12. Forbidden envelope content

An envelope must not contain:

- raw OS handles;
- raw GPU handles;
- file descriptors;
- sockets;
- function pointers;
- callbacks;
- closures;
- raw pointers;
- frontend AST nodes;
- IR objects;
- SemCode mutable references;
- platform-native event structs;
- unbounded strings;
- unbounded arrays;
- host timestamps without explicit policy.

## 13. Extension policy

New `UiEffectId` values are allowed only if they also define:

- capability slot;
- budget class;
- audit class;
- payload schema;
- result schema;
- invalid envelope rules;
- determinism classification;
- out-of-scope check;
- negative path test plan, if implementation PR follows.

No new UI effect without policy metadata.

## 14. Out of scope

This document does not add:

- Rust structs;
- ABI calls;
- VM changes;
- verifier changes;
- `prom-ui-runtime` implementation;
- window backend;
- renderer;
- widget/layout framework;
- draw command binary format;
- actual event loop;
- platform-specific adapter;
- tests beyond docs/link checks.

## 15. Acceptance checklist

- `docs/spec/ui/ui_effect_envelope_v0.md` exists;
- `docs/spec/ui/README.md` links it;
- `docs/spec/index.md` links it;
- the document depends on `I50` boundary;
- envelope lifecycle is defined;
- envelope fields are listed;
- allowed `UiEffectId` v0 set is defined;
- payload model is typed by `effect_id`;
- result model separates `Accepted` / `Denied` / `Failed`;
- invalid envelope conditions are listed;
- forbidden envelope content is listed;
- capability/budget/audit slots exist without overdefining taxonomy;
- determinism boundary is explained;
- out-of-scope forbids implementation creep;
- no code changes;
- no ABI widening;
- no VM/runtime behavior changes.
