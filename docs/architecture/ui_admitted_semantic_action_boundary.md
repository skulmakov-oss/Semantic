# Semantic UI Admitted Semantic Action Object Boundary

Status: Draft
Track: POST-UI / I-series
Purpose: define the boundary for future admitted Semantic UI action objects before dispatchers or effect bridges exist

## 1. Goal

This document defines the boundary for future admitted Semantic UI action objects.

An admitted Semantic UI action object is the future inert representation created only after an admitted action admission result.

It is not a dispatcher.
It is not execution.
It is not an effect request.
It is not a VM operation.
It is not a Host ABI operation.
It does not mutate runtime state.

## 2. Position in the ladder

The current implemented scaffold reaches:

```text
InteractionActionAdmissionDescriptor
  -> InteractionActionAdmissionResult
  -> InteractionActionDenialTrace for denied results
```

The next future admitted path is:

```text
InteractionActionAdmissionResult::Admitted
  -> future AdmittedSemanticUiAction
  -> future Dispatcher
```

Only the admitted action object boundary is defined here.

## 3. Core separation

```text
admission result is not action object
denial trace is not action object
action object is not dispatcher
dispatcher is not effect bridge
action is not effect
effect requires separate admission
```

## 4. Creation rule

A future admitted action object may only be constructed from:

```text
InteractionActionAdmissionResult::Admitted
```

It must not be constructed from:

* raw input;
* interaction intent;
* binding descriptor;
* binding trace;
* candidate summary;
* admission descriptor alone;
* denied admission result;
* denial trace;
* Workbench command;
* renderer affordance;
* component callback.

## 5. Required action object fields

A future admitted semantic action object must preserve:

1. action identity;
2. source intent kind;
3. binding identity;
4. admission result identity;
5. descriptor identity;
6. trace requirement;
7. effect relationship;
8. policy gate namespace;
9. target relationship if available;
10. lifecycle relationship if available.

It must not include executable callback handles.

## 6. Non-authority rules

A future admitted action object is not authority for:

* effect execution;
* VM execution;
* Host ABI calls;
* audit finality;
* renderer authority;
* Workbench command semantics;
* capability grant;
* runtime mutation.

It may only be a typed, traceable representation of an admitted UI-level operation.

## 7. Dispatcher separation

A dispatcher may consume an admitted action object in a future PR.

The action object itself must not dispatch.

Required separation:

```text
AdmittedSemanticUiAction
  -> future dispatcher
  -> future effect request if needed
  -> future effect/capability admission
```

No PR should combine admitted action object, dispatcher, and effect bridge.

## 8. Denial path separation

Denied results must not produce admitted action objects.

Denied path remains:

```text
InteractionActionAdmissionResult::Denied
  -> InteractionActionDenialTrace
  -> future denial presentation / diagnostic consumer
```

A denial trace is not a fallback action.

## 9. Effect relationship

An admitted action object may declare that an effect relationship exists.

It still must not request the effect directly.

Required future order:

```text
AdmittedSemanticUiAction
  -> future effect request descriptor
  -> future effect/capability admission
  -> future effect execution
```

## 10. Workbench and renderer relationship

Workbench may display or request admitted actions only through explicit future consumption boundaries.

Renderer may display affordances for admitted actions.

Neither Workbench nor renderer may define:

* action meaning;
* admission result;
* dispatch permission;
* effect permission;
* runtime mutation.

## 11. Forbidden shortcuts

Future PRs must not:

* create admitted action object from candidate summary;
* create admitted action object from binding trace;
* create admitted action object from admission descriptor alone;
* create admitted action object from denied result;
* create dispatcher inside admitted action object PR;
* create effect request inside admitted action object PR;
* store executable callbacks in action object;
* call VM/Host ABI from action object;
* mutate runtime state from action object construction;
* treat renderer affordance as admitted action;
* treat Workbench command as admitted action.

## 12. Required implementation order

Future implementation must proceed in separate PRs:

```text
docs admitted action boundary
  -> admitted action object scaffold
  -> admitted action trace/summary scaffold if needed
  -> dispatcher boundary docs
  -> dispatcher scaffold
  -> effect request boundary docs
  -> effect request scaffold
```

## 14. Semantic action dispatcher dependency

Semantic action dispatcher boundary is defined separately in:

```text
docs/architecture/ui_semantic_action_dispatcher_boundary.md
```

The admitted action object layer stops before dispatch.

```text
InteractionAdmittedSemanticAction
  -> future SemanticActionDispatcher
```

The action object is not dispatcher.
The dispatcher is not effect bridge.
Effect requires separate admission.

## 15. Validation expectation

Docs-only PR validation:

```text
git diff --check
```

No Rust tests are required for this PR.
