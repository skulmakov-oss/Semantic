# Semantic UI Action Admission Descriptor Boundary

Status: Draft
Track: POST-UI / I-series
Purpose: define the boundary for future action admission descriptor scaffolding before any admission engine or action execution exists

## 1. Goal

This document defines the boundary for future Semantic UI action admission descriptors.

An action admission descriptor is not an admission result.

It is a static or inert description of what must be checked before an action candidate may become an admitted Semantic UI action.

It does not execute actions.
It does not admit actions.
It does not deny actions.
It does not request effects.
It does not call the VM.
It does not call Host ABI.
It does not mutate runtime state.

## 2. Position in the ladder

The current interaction-action trace ladder ends at candidate summary:

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

The next boundary is:

```text
InteractionActionCandidateSummary / InteractionActionBindingTraceReport
  -> ActionAdmissionDescriptor
  -> future ActionAdmissionResult
  -> future AdmittedSemanticUiAction
```

Only the descriptor boundary is defined here.

## 3. Definitions

| Term                 | Meaning                                         | Not allowed      |
| -------------------- | ----------------------------------------------- | ---------------- |
| action candidate     | future action name found through binding trace  | permission       |
| admission descriptor | inert description of required checks            | decision         |
| admission result     | future decision: admitted or denied             | execution        |
| admitted action      | future action object after successful admission | effect by itself |
| denial trace         | future visible denial explanation               | hidden no-op     |

## 4. Core invariant

```text
candidate is not permission
descriptor is not decision
decision is not execution
action is not effect
effect requires separate admission
```

No implementation may treat a bound candidate as admitted simply because a binding exists.

## 5. Required descriptor fields

A future action admission descriptor must identify:

1. action name;
2. source intent kind;
3. binding identity;
4. target requirement;
5. target ownership requirement;
6. lifecycle requirement;
7. capability requirement;
8. trace requirement;
9. effect relationship;
10. denial visibility requirement;
11. policy gate namespace;
12. future admission result shape.

The descriptor must be explicit about missing target behavior.

## 6. Target admission rules

A target may be:

```text
required
optional / may be resolved later
ignored
```

If target is required and missing:

```text
candidate
  -> admission descriptor
  -> future denial result
  -> denial trace
  -> no admitted action
```

The system must not convert missing target into silent no-op.

## 7. Lifecycle admission rules

A descriptor may require lifecycle state such as:

```text
session active
window alive
surface ready
frame open
frame not submitted
runtime not quarantined
```

Lifecycle requirement is a descriptor field only.

I14 does not implement lifecycle checking.

## 8. Capability admission rules

A descriptor may require UI capability state such as:

```text
DesktopSession
InputPoll
FrameEmit
future action-specific capability
future effect capability
```

Capability requirement is declarative only.

I14 does not implement capability checks.

## 9. Trace requirement

Admission decisions must be traceable when they affect semantic UI behavior.

Future admission results must distinguish:

```text
admitted
denied_missing_target
denied_lifecycle
denied_capability
denied_policy
denied_effect_boundary
denied_unknown
```

I14 does not implement result types.

It only defines that future result types must preserve denial reason visibility.

## 10. Effect relationship

An admitted semantic UI action may later request an effect.

The action is still not the effect.

Required separation:

```text
action admission
  -> admitted semantic UI action
  -> optional effect request
  -> effect/capability admission
  -> effect execution
```

No descriptor may authorize external effects directly.

## 11. Relationship to existing I-series types

| Existing type                               | Relationship to admission                 |
| ------------------------------------------- | ----------------------------------------- |
| `InteractionActionBindingDescriptor`        | candidate vocabulary only                 |
| `InteractionActionBindingTraceReport`       | explains candidate presence/absence       |
| `InteractionActionBindingTraceStreamReport` | batch diagnostic data                     |
| `InteractionActionCandidateSummary`         | aggregate diagnostic summary              |
| future `ActionAdmissionDescriptor`          | describes required checks                 |
| future `ActionAdmissionResult`              | performs decision reporting               |
| future `AdmittedSemanticUiAction`           | represents admitted action after decision |

## 12. Forbidden shortcuts

Future PRs must not:

* treat `InteractionActionBindingDescriptor` as admission;
* treat `InteractionActionBindingTraceReport` as admitted action;
* treat `InteractionActionCandidateSummary` as action queue;
* skip target ownership checks;
* skip lifecycle checks;
* skip capability checks;
* hide denied candidates as no-op;
* merge admission result and dispatcher in one PR;
* merge action execution and effect request in one PR;
* let Workbench command define core action admission;
* let renderer affordance define permission;
* call VM/Host ABI from admission descriptor layer.

## 13. Required implementation order

Future implementation must proceed in separate PRs:

```text
docs admission boundary
  -> admission descriptor scaffold
  -> admission result / denial trace scaffold
  -> admitted action object scaffold
  -> dispatcher scaffold
  -> effect request bridge
```

No PR should combine descriptor, decision, action object, dispatcher, and effect bridge.

## 14. Validation expectation

Docs-only PR validation:

```text
git diff --check
```

No Rust tests are required for this PR.
