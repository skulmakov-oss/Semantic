# Semantic UI Action Dispatcher Boundary

Status: Draft
Track: POST-UI / I-series
Purpose: define the boundary for a future Semantic UI action dispatcher before execution or effect bridges exist

## 1. Goal

This document defines the boundary for a future Semantic UI action dispatcher.

A Semantic UI action dispatcher is a future layer that may consume admitted Semantic UI action objects and route them to future handling paths.

The dispatcher is not the admitted action object.
The dispatcher is not effect execution.
The dispatcher is not a VM operation.
The dispatcher is not a Host ABI operation.
The dispatcher must not silently mutate runtime state.
The dispatcher must not bypass effect/capability admission.

## 2. Position in the ladder

The current implemented scaffold reaches:

```text
InteractionActionAdmissionResult::Admitted
  -> InteractionAdmittedSemanticAction
```

The next future boundary is:

```text
InteractionAdmittedSemanticAction
  -> future SemanticActionDispatcher
  -> future dispatch record
  -> future effect request descriptor if needed
```

Only the dispatcher boundary is defined here.

## 3. Core separation

```text
admitted action object is not dispatcher
dispatcher is not effect bridge
dispatcher is not VM bridge
dispatcher is not Host ABI bridge
dispatch record is not effect execution
effect requires separate admission
```

## 4. Dispatcher input rule

A future dispatcher may only consume:

```text
InteractionAdmittedSemanticAction
```

It must not consume directly:

* raw input;
* interaction intent;
* binding descriptor;
* binding trace;
* candidate summary;
* admission descriptor;
* denied admission result;
* denial trace;
* renderer affordance;
* Workbench command;
* component callback.

## 5. Dispatcher output rule

A future dispatcher may produce only explicit dispatch metadata, such as:

1. dispatch id;
2. admitted action id;
3. action name;
4. dispatch route;
5. trace requirement;
6. effect relationship;
7. policy gate namespace;
8. dispatch status;
9. future effect request eligibility.

It must not directly produce:

* external effects;
* VM calls;
* Host ABI calls;
* runtime mutation;
* renderer commands;
* Workbench commands.

## 6. Dispatch route classes

Future dispatch routes may include:

| Route                     | Meaning                                    |
| ------------------------- | ------------------------------------------ |
| `NoopSemanticRecord`      | record-only semantic action, no effect     |
| `LocalUiStateCandidate`   | future local UI state transition candidate |
| `EffectRequestCandidate`  | future explicit effect request path        |
| `WorkbenchLocalCandidate` | future Workbench-local handling path       |
| `DiagnosticOnlyCandidate` | diagnostic route only                      |
| `Unknown`                 | unresolved route                           |

These routes are not execution authority.

## 7. Dispatch result classes

Future dispatch result/status must distinguish:

```text
routed
blocked_missing_route
blocked_policy
blocked_effect_boundary
blocked_unknown
```

A blocked dispatch must be visible if it affects user expectation or semantic UI state.

## 8. Effect relationship

A dispatcher may identify that an admitted action requires an effect path.

It must not execute that effect.

Required future order:

```text
InteractionAdmittedSemanticAction
  -> SemanticActionDispatcher
  -> EffectRequestDescriptor
  -> effect/capability admission
  -> effect execution
```

No dispatcher PR may silently produce external effects.

## 9. Runtime mutation relationship

Dispatcher construction and dispatch metadata are not runtime mutation.

Any future runtime mutation must pass through a separate boundary.

Required future order:

```text
dispatch metadata
  -> future state transition descriptor
  -> future state transition admission
  -> future runtime mutation
```

No dispatcher should mutate runtime state directly.

## 10. Trace relationship

A future dispatcher must preserve traceability when dispatch affects:

* semantic UI state;
* lifecycle state;
* capability/effect path;
* denial/recovery path;
* user-visible semantic result.

Trace display is not audit authority by itself.

## 11. Workbench and renderer relationship

Workbench may consume future dispatch metadata only through a Workbench consumption boundary.

Renderer may display dispatch state or affordances only as presentation.

Neither Workbench nor renderer may define:

* dispatch permission;
* action meaning;
* effect permission;
* runtime mutation;
* audit finality.

## 12. Forbidden shortcuts

Future PRs must not:

* dispatch directly from interaction intent;
* dispatch directly from binding trace;
* dispatch directly from candidate summary;
* dispatch denied results;
* treat denial trace as fallback action;
* perform effects inside dispatcher;
* call VM/Host ABI from dispatcher;
* mutate runtime state from dispatcher;
* treat renderer affordance as dispatch permission;
* treat Workbench command as dispatch permission;
* combine dispatcher and effect bridge in one PR;
* combine dispatcher and runtime mutation in one PR.

## 13. Required implementation order

Future implementation must proceed in separate PRs:

```text
docs dispatcher boundary
  -> dispatcher route enum scaffold
  -> dispatch record scaffold
  -> dispatch trace/summary scaffold if needed
  -> effect request boundary docs
  -> effect request descriptor scaffold
  -> effect/capability admission boundary
```

No PR should combine dispatcher, effect request, capability admission, and execution.

## 14. Effect request descriptor dependency

Effect request descriptor boundary is defined separately in:

```text
docs/architecture/ui_effect_request_descriptor_boundary.md
```

The dispatch summary layer stops before effect request descriptor construction.

```text
InteractionSemanticActionDispatchSummary
  -> future EffectRequestDescriptor
```

Dispatch metadata is not effect request.
Effect request descriptor is not capability admission.
Effect requires separate admission.

## 15. Validation expectation

Docs-only PR validation:

```text
git diff --check
```

No Rust tests are required for this PR.
