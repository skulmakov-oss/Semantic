# Semantic UI Full Effect Trace Ladder

Status: Draft
Track: POST-UI / I-series
Purpose: document the complete inert UI-side trace ladder from raw input to committed effect record

## 1. Goal

This document records the full UI-side effect trace ladder introduced across the I-series.

The ladder exists to make every transition from user interaction to committed
effect record explicit, observable, deterministic, and bounded.

It does not define new runtime behavior.

It does not execute effects.
It does not call Host ABI.
It does not call VM.
It does not mutate runtime state.
It does not implement an audit backend.
It does not enter a host runtime effect path.

## 2. Full ladder

```text
RawUiEvent
  -> InteractionIntentDescriptor
  -> InteractionIntentTraceReport
  -> InteractionIntentStreamReport
  -> InteractionActionBindingDescriptor
  -> InteractionActionBindingTraceReport
  -> InteractionActionBindingTraceStreamReport
  -> InteractionActionCandidateSummary
  -> InteractionActionAdmissionDescriptor
  -> InteractionActionAdmissionResult
  -> InteractionActionDenialTrace
  -> InteractionAdmittedSemanticAction
  -> InteractionSemanticActionDispatchRoute
  -> InteractionSemanticActionDispatchRecord
  -> InteractionSemanticActionDispatchTraceReport
  -> InteractionSemanticActionDispatchSummary
  -> InteractionEffectRequestDescriptor
  -> InteractionEffectRequestTraceReport
  -> InteractionEffectRequestSummary
  -> InteractionUiCapabilityAdmissionDescriptor
  -> InteractionUiCapabilityAdmissionResult
  -> InteractionUiCapabilityDenialTrace
  -> InteractionRuntimeCapabilityMappingDescriptor
  -> InteractionRuntimeCapabilityMappingResult
  -> InteractionPreparedEffectDescriptor
  -> InteractionPreparedEffectResult
  -> InteractionCommitBoundaryDescriptor
  -> InteractionCommitBoundaryResult
  -> InteractionCommittedEffectDescriptor
  -> InteractionCommittedEffectRecord
```

## 3. Reduced conceptual ladder

```text
raw input
  -> intent
  -> binding candidate
  -> admission
  -> admitted action
  -> dispatch
  -> effect request
  -> UI capability admission
  -> runtime capability mapping
  -> prepared effect
  -> commit boundary
  -> committed effect record
```

## 4. Layer table

| Stage | Main type | Meaning | Explicitly not |
| --- | --- | --- | --- |
| Raw input | `RawUiEvent` | Platform-neutral input sample | event loop, native backend |
| Intent | `InteractionIntentDescriptor` | Classified UI intent | action |
| Intent trace | `InteractionIntentTraceReport` | Raw event -> intent explanation | dispatcher |
| Intent stream | `InteractionIntentStreamReport` | Ordered batch trace | async queue |
| Binding | `InteractionActionBindingDescriptor` | Candidate intent -> action name binding | admission |
| Binding trace | `InteractionActionBindingTraceReport` | Binding visibility | admitted action |
| Binding trace stream | `InteractionActionBindingTraceStreamReport` | Ordered binding trace batch | runtime queue |
| Candidate summary | `InteractionActionCandidateSummary` | Compact diagnostic summary | dispatcher, admission |
| Action admission descriptor | `InteractionActionAdmissionDescriptor` | Future admission request | admission decision |
| Action admission result | `InteractionActionAdmissionResult` | Admit/deny metadata | execution |
| Action denial trace | `InteractionActionDenialTrace` | Denial visibility | retry execution |
| Admitted action | `InteractionAdmittedSemanticAction` | Inert admitted semantic action object | effect |
| Dispatch route | `InteractionSemanticActionDispatchRoute` | Candidate route | dispatch execution |
| Dispatch record | `InteractionSemanticActionDispatchRecord` | Dispatch record metadata | effect |
| Dispatch trace | `InteractionSemanticActionDispatchTraceReport` | Dispatch visibility | Host ABI |
| Dispatch summary | `InteractionSemanticActionDispatchSummary` | Dispatch diagnostics | scheduler |
| Effect request descriptor | `InteractionEffectRequestDescriptor` | Future effect request metadata | effect execution |
| Effect request trace | `InteractionEffectRequestTraceReport` | Effect request visibility | prepared effect |
| Effect request summary | `InteractionEffectRequestSummary` | Effect request diagnostics | capability grant |
| UI capability admission descriptor | `InteractionUiCapabilityAdmissionDescriptor` | UI capability admission request | grant |
| UI capability admission result | `InteractionUiCapabilityAdmissionResult` | Admit/deny UI capability metadata | runtime grant |
| UI capability denial trace | `InteractionUiCapabilityDenialTrace` | UI capability denial visibility | runtime mapping |
| Runtime mapping descriptor | `InteractionRuntimeCapabilityMappingDescriptor` | Runtime capability mapping request | runtime grant |
| Runtime mapping result | `InteractionRuntimeCapabilityMappingResult` | Mapped/denied metadata | Host ABI |
| Prepared effect descriptor | `InteractionPreparedEffectDescriptor` | Future prepared effect descriptor | execution |
| Prepared effect result | `InteractionPreparedEffectResult` | Prepared/denied metadata | commit |
| Commit boundary descriptor | `InteractionCommitBoundaryDescriptor` | Future commit boundary request | commit decision |
| Commit boundary result | `InteractionCommitBoundaryResult` | Committed/denied metadata | committed effect |
| Committed effect descriptor | `InteractionCommittedEffectDescriptor` | UI-side committed effect descriptor | Host ABI path |
| Committed effect record | `InteractionCommittedEffectRecord` | Final inert UI-side committed effect record | runtime mutation |

## 5. Global invariants

```text
raw event is not intent
intent is not action
binding is not admission
admission descriptor is not decision
admission result is not execution
admitted action is not effect
dispatch is not Host ABI
effect request is not effect execution
UI capability admission is not runtime capability grant
runtime capability mapping is not Host ABI
prepared effect is not committed effect
commit boundary result is not Host ABI authority
committed effect record is not runtime mutation
```

## 6. Authority separation

No current UI ladder layer is allowed to act as:

* Host ABI authority;
* VM authority;
* effect execution authority;
* runtime mutation authority;
* audit backend;
* renderer authority;
* Workbench command authority;
* host runtime effect path.

## 7. Determinism rule

Each layer must preserve deterministic identity propagation.

The current scaffold pattern is:

```text
next_id = source_id
```

unless a future boundary explicitly defines another deterministic mapping.

No layer may introduce nondeterministic ordering, hidden filtering, hidden deduplication, or time-dependent identifiers.

## 8. Denial visibility rule

Denied states must be visible.

The ladder must not hide denial as no-op.

Denied states currently appear or are expected around:

* action admission;
* UI capability admission;
* runtime capability mapping;
* prepared effect;
* commit boundary.

Future denial trace documents may refine presentation, but denial must remain explicit.

## 9. Effect execution separation

The ladder stops at:

```text
InteractionCommittedEffectRecord
```

This is still not:

* Host ABI call;
* VM call;
* effect execution;
* runtime mutation;
* host runtime effect path;
* audit backend write.

Future Host runtime effect work requires a separate boundary document.

## 10. Relationship to early interaction-action ladder

The early interaction-action ladder is defined in:

```text
docs/architecture/ui_interaction_action_trace_ladder.md
```

That document covers the early segment:

```text
RawUiEvent
  -> InteractionIntentDescriptor
  -> InteractionActionCandidateSummary
```

This document extends the view across the full effect-side chain.

## 11. Relationship to capability and effect boundaries

Capability/effect boundaries are defined in:

```text
docs/architecture/ui_effect_request_capability_boundary.md
docs/architecture/ui_capability_admission_boundary.md
docs/architecture/ui_runtime_capability_mapping_boundary.md
docs/architecture/ui_prepared_effect_boundary.md
docs/architecture/ui_committed_effect_boundary.md
```

This document does not replace them.

It is an index/trace ladder overview.

## 12. Forbidden shortcuts

Future PRs must not:

* jump from raw input to action;
* jump from intent to effect request;
* treat binding as admission;
* treat admitted action as effect execution;
* treat dispatch as Host ABI;
* treat effect request as capability grant;
* treat UI capability admission as runtime grant;
* treat runtime mapping as Host ABI;
* treat prepared effect as committed effect;
* treat commit boundary result as runtime mutation;
* treat committed effect record as host runtime path;
* hide denial as no-op;
* let renderer or Workbench define semantic authority.

## 13. Future allowed next steps

Allowed next PR families after this document:

| Family | Allowed | Still forbidden |
| --- | --- | --- |
| docs | refine full ladder / add diagrams | behavior changes |
| committed effect trace | denial/record trace docs | Host ABI |
| host runtime boundary | boundary docs only | runtime mutation |
| audit visibility | docs or inert descriptors | audit backend writes |
| renderer consumption | presentation contract | renderer authority |
| Workbench consumption | read-only consumption docs | command execution |

## 14. Validation expectation

Docs-only PR validation:

```text
git diff --check
```

No Rust tests are required for this PR.

## 15. Host runtime effect boundary dependency

The host runtime effect boundary is defined separately in:

```text
docs/architecture/ui_host_runtime_effect_boundary.md
```

The current full UI-side ladder stops at:

```text
InteractionCommittedEffectRecord
```

The next future boundary is:

```text
InteractionCommittedEffectRecord
  -> future HostRuntimeEffectBoundary
  -> future HostRuntimeEffectPath
```

The host runtime effect boundary is still not Host ABI execution, VM authority,
runtime mutation, or audit backend implementation.
