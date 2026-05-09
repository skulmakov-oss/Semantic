# Semantic UI Interaction-Action Trace Ladder

Status: Draft
Track: POST-UI / I-series
Purpose: document the inert interaction-to-action trace ladder introduced by PR-UI-I5 through PR-UI-I12

## 1. Goal

This document records the current interaction-action trace ladder in `prom-ui`.

The ladder exists to make UI interaction semantics observable before any dispatcher,
admission engine, Workbench bridge, renderer affordance map, or effect bridge is introduced.

The current implementation is intentionally inert.

It does not execute actions.
It does not admit actions.
It does not request effects.
It does not call the VM.
It does not call Host ABI.
It does not mutate runtime state.

## 2. Current ladder

```text
RawUiEvent
  -> InteractionIntentDescriptor
  -> InteractionIntentTraceReport
  -> InteractionIntentStreamReport
  -> InteractionActionBindingDescriptor
  -> InteractionActionBindingTraceReport
  -> InteractionActionBindingTraceStreamReport
  -> InteractionActionCandidateSummary
```

## 3. Layer ownership

| Layer                | Type / Function                             | Meaning                                             | Not allowed                |
| -------------------- | ------------------------------------------- | --------------------------------------------------- | -------------------------- |
| Raw event            | `RawUiEvent`                                | Platform-neutral raw UI input model                 | event loop, native backend |
| Intent               | `InteractionIntentDescriptor`               | Interpreted UI interaction intent                   | action execution           |
| Intent trace         | `InteractionIntentTraceReport`              | Explanation of raw event -> intent                  | dispatcher                 |
| Intent stream        | `InteractionIntentStreamReport`             | Stable ordered batch of intent traces               | async stream, queue        |
| Binding descriptor   | `InteractionActionBindingDescriptor`        | Candidate mapping from intent to future action name | admission, effect          |
| Binding trace        | `InteractionActionBindingTraceReport`       | Explanation of whether a binding candidate exists   | admitted action            |
| Binding trace stream | `InteractionActionBindingTraceStreamReport` | Stable ordered batch of binding traces              | runtime queue              |
| Candidate summary    | `InteractionActionCandidateSummary`         | Compact diagnostic summary                          | dispatcher, admission      |

## 4. Invariants

The ladder preserves these invariants:

```text
raw event is not intent
intent is not action
binding descriptor is not admission
binding trace is not admitted action
candidate summary is not dispatcher
semantic UI action is not effect
effect requires a separate boundary
```

No layer may skip directly from raw input or intent to effect.

## 5. Relationship to semantic action boundary

The semantic action boundary defines the conceptual chain:

```text
native event
  -> InputEvent
  -> interaction intent
  -> admission
  -> semantic UI action
  -> trace
  -> optional effect request
```

The current I-series code does not implement the admission/action/effect segment.

It only prepares observable candidate metadata before admission exists.

## 6. What I5-I12 introduced

| PR        | Layer                         | Result                                      |
| --------- | ----------------------------- | ------------------------------------------- |
| PR-UI-I5  | interaction intent descriptor | `InteractionIntentDescriptor`               |
| PR-UI-I6  | raw event mapping             | `RawUiEvent -> InteractionIntentDescriptor` |
| PR-UI-I7  | intent trace                  | `InteractionIntentTraceReport`              |
| PR-UI-I8  | intent trace stream           | `InteractionIntentStreamReport`             |
| PR-UI-I9  | action binding descriptor     | `InteractionActionBindingDescriptor`        |
| PR-UI-I10 | action binding trace          | `InteractionActionBindingTraceReport`       |
| PR-UI-I11 | action binding trace stream   | `InteractionActionBindingTraceStreamReport` |
| PR-UI-I12 | candidate summary             | `InteractionActionCandidateSummary`         |

## 7. Explicit non-authority rules

The following types are diagnostic or contract data only:

```text
InteractionIntentTraceReport
InteractionIntentStreamReport
InteractionActionBindingDescriptor
InteractionActionBindingTraceReport
InteractionActionBindingTraceStreamReport
InteractionActionCandidateSummary
```

They must not be treated as:

* admitted semantic actions;
* permission grants;
* effect requests;
* audit authority;
* runtime commands;
* Workbench commands;
* renderer commands;
* component callbacks.

## 8. Future allowed next steps

Allowed future PR families:

| Family          | Allowed                              | Still forbidden       |
| --------------- | ------------------------------------ | --------------------- |
| docs            | refine ladder, boundary, invariants  | code behavior changes |
| trace model     | add compact diagnostic views         | dispatcher            |
| admission draft | define admission descriptor scaffold | execution             |
| denial draft    | define denial trace scaffold         | hidden no-op          |
| Workbench docs  | define consumption rules             | command bridge        |
| renderer docs   | define affordance display boundary   | renderer authority    |

## 9. First future code boundary

Before adding any dispatcher or action execution, the project must introduce:

```text
action admission descriptor scaffold
  -> denial trace scaffold
  -> admitted action object
  -> dispatcher
```

in separate PRs.

No PR should combine these steps.

## 10. Forbidden shortcuts

Future PRs must not:

* turn binding trace into dispatcher;
* turn candidate summary into action queue;
* use Workbench button clicks as semantic actions;
* allow renderer affordances to define action meaning;
* treat bound candidate as admitted action;
* perform effect request from binding;
* bypass admission because a binding exists;
* hide unbound candidates as no-op;
* add VM/Host ABI bridge from this ladder directly.

## 11. Validation expectation

Docs-only PR validation:

```text
git diff --check
```

No Rust tests are required for this PR.
