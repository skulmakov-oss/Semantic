# UI Verifier Admission Metadata

Status: Draft
Track: POST-UI
Owner: future `sm-verify` admission contract
Scope: metadata plan only
Implementation: out of scope

Related:

- `docs/spec/verifier.md`
- `docs/spec/ui_abi_capability_admission.md`
- `docs/roadmap/post_ui/ui_admission_checklist.md`

## 1. Purpose

This document defines the planned verifier-visible metadata needed to admit
future Semantic UI operations.

It does not introduce executable UI support.
It does not add opcodes, SemCode header bits, verifier code, VM code, or runtime
lifecycle behavior.

The goal is to keep POST-UI admission explicit before implementation starts.

## 2. Current facts

Current UI operation identity is owned by `prom-ui`:

```text
WindowCreate
WindowRun
WindowClose
EventPoll
FrameSubmit
```

Current UI capability identity is owned by `prom-ui`:

```text
DesktopSession
InputPoll
FrameEmit
```

Current operation-to-capability mapping:

| UI operation | Required UI capability |
| --- | --- |
| `WindowCreate` | `DesktopSession` |
| `WindowRun` | `DesktopSession` |
| `WindowClose` | `DesktopSession` |
| `EventPoll` | `InputPoll` |
| `FrameSubmit` | `FrameEmit` |

This mapping is locked by `tests/ui_capability_admission_contract.rs`.

## 3. Admission principle

A future UI operation is verifier-admissible only when the SemCode artifact makes
the following information visible to `sm-verify`:

```text
operation identity
required UI capability
POST-UI surface/profile admission
metadata version
static payload shape, if any
```

A UI operation must not become executable merely because it reaches VM dispatch.

## 4. Candidate metadata model

Future implementation may encode UI admission metadata using any SemCode-compatible
mechanism approved in its own PR. The required logical shape is:

```text
UiAdmissionRecord {
  version,
  operation_id,
  required_capability,
  surface_class,
  payload_shape,
}
```

Logical fields:

| Field | Meaning |
| --- | --- |
| `version` | UI admission metadata schema version |
| `operation_id` | canonical `UiOperationId` value |
| `required_capability` | canonical `UiCapabilityKind` value |
| `surface_class` | POST-UI / post-stable admission class |
| `payload_shape` | static payload contract, if relevant |

This document intentionally does not choose a binary layout.

## 5. Verifier rejection rules

Future verifier support must reject SemCode when:

- UI metadata is malformed or truncated;
- UI metadata version is unsupported;
- operation id is unknown;
- capability id is unknown;
- operation-to-capability mapping does not match the canonical mapping;
- UI metadata appears without POST-UI/profile admission;
- UI operation payload shape is invalid where statically checkable;
- UI operation claims stable-v1 admission without explicit promotion.

## 6. Verifier non-goals

Future UI verifier admission must not:

- parse `.sm` source;
- own UI runtime state;
- execute UI operations;
- create windows;
- validate native platform handles;
- perform layout or widget checks;
- decide event ordering;
- replace runtime lifecycle enforcement.

The verifier admits structure and declared capability consistency only.

## 7. Runtime handoff rule

Verifier admission is necessary but not sufficient.

After admission, runtime/VM bridge must still check:

```text
manifest validity
require_ui_op(operation)
session lifecycle
frame lifecycle
event polling boundary
frame submission boundary
```

The verifier must not make runtime checks optional.

## 8. Stable-line rule

UI metadata remains POST-UI / post-stable unless explicitly promoted by a future
release decision.

This document does not change the published stable line.

## 9. Future implementation gates

Before executable UI support, a future PR must add or update:

1. SemCode metadata encoding spec;
2. verifier structural tests for valid UI metadata;
3. verifier rejection tests for malformed/unknown/mismatched UI metadata;
4. capability admission tests proving fail-closed behavior remains intact;
5. runtime lifecycle tests before any platform backend is introduced.

## 10. Non-goals

This document does not add:

- new opcodes;
- new SemCode header bits;
- new `HostCallId` variants;
- new `UiOperationId` values;
- new `UiCapabilityKind` values;
- verifier implementation;
- VM implementation;
- runtime implementation;
- Workbench integration;
- platform backend.
