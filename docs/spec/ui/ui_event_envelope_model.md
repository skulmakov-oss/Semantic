# Deterministic UI Event Envelope Model

Status: Draft
Track: POST-UI
Depends on:
- `host_runtime_effect_path_boundary.md`
- `ui_effect_envelope_v0.md`
- `ui_capability_taxonomy.md`
Scope: UI event envelope contract only
Implementation: out of scope

Related:

- `README.md`
- `../../architecture/ui_interaction_input_semantic_boundary.md`
- `../../architecture/ui_full_effect_trace_ladder.md`

## 1. Purpose

This document defines how external UI events become VM-visible deterministic
event data.

The host event stream is external and nondeterministic.
The VM-visible `UiEventEnvelope` must be explicit, bounded, normalized, ordered,
and replayable.

## 2. Relationship to I50, I51, and I52

- `I50` defines the host runtime effect path.
- `I51` defines `UiEffectEnvelope v0`.
- `I52` defines the UI capability taxonomy.
- This document defines the deterministic event model consumed by
  `PollEvents` and the `UiEventRead` capability.

## 3. Core rule

```text
Host event stream is external and nondeterministic.
VM-visible UiEventEnvelope must be explicit, bounded, normalized, ordered, and
replayable.
```

Short form:

```text
UiEventEnvelope is data, not host input latency.
```

## 4. Relationship to PollEvents

`PollEvents` is the UI effect that requests a bounded event batch.

This document defines the shape of the returned event data and the constraints
on how the runtime produces it.

`PollEvents` must not expose raw host events directly to the VM.

## 5. Envelope family

The event model has two layers:

```text
UiEventEnvelope
  -> UiEventBatch
```

`UiEventEnvelope` represents one normalized event.
`UiEventBatch` represents a bounded ordered collection of envelopes.

## 6. Event envelope v0

The v0 event envelope shape is:

```text
UiEventEnvelopeV0 {
  envelope_version: 0,
  envelope_id: UiEnvelopeId,
  event_id: UiEventId,

  source: UiEventSource,
  target: UiEventTarget,
  kind: UiEventKind,
  payload: UiEventPayload,

  ordering: UiEventOrderingMetadata,
  determinism: UiEventDeterminismMetadata,
  audit: UiEventAuditMetadata
}
```

## 7. Envelope lifecycle

The event lifecycle is:

```text
1. Host emits raw event
2. prom-ui-runtime captures or polls it
3. runtime normalizes it into UiEventEnvelope
4. runtime validates shape and bounds
5. runtime orders it inside a batch
6. runtime records audit metadata when required
7. VM consumes the explicit event envelope
8. replay tooling can re-feed the same normalized event sequence
```

`UiEventEnvelope` must not be emitted as a hidden side channel.

## 8. Envelope fields

### 8.1 `envelope_version`

```text
envelope_version: 0
```

Rules:

- fixes the event envelope format;
- allows future major versions without silent reinterpretation;
- unknown major versions are invalid.

### 8.2 `envelope_id`

```text
envelope_id: UiEnvelopeId
```

Rules:

- correlation id within the runtime session;
- used for trace and replay linkage;
- unique within a batch or session scope as defined by runtime policy.

### 8.3 `event_id`

```text
event_id: UiEventId
```

Rules:

- identifies the normalized event kind;
- selects the payload schema;
- drives replay and audit handling.

### 8.4 `source`

```text
UiEventSource {
  runtime_session_id,
  window_id?,
  device_id?,
  host_sequence?
}
```

Rules:

- source must be normalized;
- source must not carry raw backend structs;
- source must not expose unbounded host internals.

### 8.5 `target`

```text
UiEventTarget {
  window_id?,
  frame_id?,
  surface_id?
}
```

Rules:

- target is optional for pure session events;
- target must remain consistent with runtime ownership;
- target must not substitute for capability.

### 8.6 `kind`

```text
UiEventKind
```

The kind selects payload shape and ordering constraints.

### 8.7 `payload`

Payload is typed by `event_id`.

Forbidden payload content:

- raw host event objects;
- OS handles;
- raw pointers;
- callbacks;
- frontend AST nodes;
- unbounded blobs;
- backend-specific device structures.

### 8.8 `ordering`

```text
UiEventOrderingMetadata {
  batch_index,
  stream_index,
  host_sequence?
}
```

Rules:

- events inside a batch must be ordered;
- ordering must be explicit;
- host ordering metadata may exist, but the VM-visible order is the normalized
  order after runtime processing.

### 8.9 `determinism`

```text
UiEventDeterminismMetadata {
  class,
  replay_policy,
  source_stability
}
```

Classes:

- `DeterministicData`
- `ExternalInputRead`
- `PlatformDependentOrdering`
- `PlatformDependentFailure`

The VM-visible data is deterministic after normalization; the host source is not.

### 8.10 `audit`

```text
UiEventAuditMetadata {
  audit_class,
  correlation_id?,
  sampled?,
  sensitive?
}
```

Event audit classes:

- `Trace`
- `InputBoundary`
- `Sensitive`

## 9. Allowed v0 event kinds

The v0 set is:

- `Quit`
- `KeyDown`
- `KeyUp`
- `MouseMove`
- `MouseDown`
- `MouseUp`
- `Resize`
- `Tick`

## 10. Event batch model

```text
UiEventBatchV0 {
  envelope_version: 0,
  batch_id: UiBatchId,
  source_effect_envelope_id: UiEnvelopeId?,
  max_events: usize,
  events: UiEventEnvelope[]
}
```

Rules:

- batch must be bounded;
- batch must preserve event order;
- batch must be stable under replay with the same input stream and runtime
  policy;
- batch must not expose raw host events.

## 11. Bounded polling result

`PollEvents` returns a bounded `UiEventBatch`.

The runtime must honor:

- `max_events` upper bound;
- session/window scope;
- event normalization;
- timeout policy;
- replay policy.

Unbounded polling is forbidden.

## 12. Ordering rules

Rules:

- events are ordered by runtime after normalization;
- batch order is explicit and reproducible;
- if two host events are observed, runtime must assign a deterministic order
  according to its policy;
- reordering must be documented by policy, not hidden.

## 13. Replay policy

Replay policy must identify whether the batch is:

- exact replayable;
- host-input dependent but explicit;
- replayable only within a bounded approximation;
- non-replayable due to platform failure.

Default v0 rule:

- normalized event batches are replayable as explicit data;
- host latency and host scheduling are not part of VM semantics.

## 14. Event timestamp policy

Timestamps, if present, are policy-driven metadata only.

Rules:

- timestamps must be bounded and explicit;
- timestamps must not expose raw platform time without policy;
- timestamps must not become a hidden determinism source;
- monotonic ordering is preferred over wall-clock dependence.

## 15. Invalid event batch conditions

An event batch is invalid when any of these hold:

- unknown `envelope_version`;
- unknown `event_id`;
- event kind does not match payload;
- event payload exceeds bounds;
- batch exceeds `max_events`;
- batch order is ambiguous or missing;
- raw host event object leaks through;
- raw OS handle leaks through;
- timestamp policy is absent where required;
- event targets violate runtime ownership;
- batch contains unsupported sensitive data.

## 16. Relationship to capability taxonomy

`PollEvents` requires `UiEventRead`.

This document does not redefine the capability taxonomy; it consumes it.

## 17. Forbidden event payload content

The following must not appear in event payloads:

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
- unbounded strings;
- unbounded arrays;
- host timestamps without policy.

## 18. Extension policy

New event kinds require:

- explicit `event_id`;
- payload schema;
- ordering policy;
- replay policy;
- audit class;
- boundedness rule;
- capability mapping if consumed through a UI effect.

No new UI event kind without policy metadata.

## 19. Out of scope

This document does not add:

- code changes;
- ABI widening;
- VM changes;
- verifier changes;
- `prom-ui-runtime` implementation;
- platform adapter implementation;
- actual event loop;
- renderer/backend/widget framework;
- tests beyond docs/link checks.

## 20. Acceptance checklist

- `docs/spec/ui/ui_event_envelope_model.md` exists;
- `docs/spec/ui/README.md` links it;
- `docs/spec/index.md` links it;
- `UiEventEnvelope` and `UiEventBatch` models are defined;
- normalization rules are defined;
- ordering rules are defined;
- bounded polling result is defined;
- replay policy is defined;
- external input boundary is defined;
- allowed v0 event kinds are listed;
- forbidden payload content is listed;
- timestamp policy is defined;
- invalid batch conditions are listed;
- relationship to `PollEvents` and `UiEventRead` is explicit;
- no code changes;
- no ABI widening;
- no VM/runtime behavior changes.
