# Semantic UI Action Boundary

Status: Draft
Track: POST-UI / H-series
Purpose: define semantic UI action boundaries before implementation

## 1. Goal

This document defines the boundary for Semantic UI actions.

A Semantic UI action is an admitted UI-level operation derived from interaction intent.

It is not raw input.
It is not a component callback.
It is not an effect by itself.
It is not a VM or SemCode operation.

The project must preserve this chain:

```text
native event
  -> InputEvent
  -> interaction intent
  -> admission
  -> semantic UI action
  -> trace
  -> optional effect request
```

No layer may skip admission and directly perform effects.

## 2. Relationship to interaction and focus/selection

Semantic actions depend on:

```text
docs/architecture/ui_interaction_input_semantic_boundary.md
docs/architecture/ui_focus_selection_semantic_boundary.md
```

Interaction intent proposes what the user wants.
Focus/selection may provide target context.
Admission decides whether intent becomes action.
The action is the admitted UI operation.

```text
interaction intent
  -> target context
  -> admission
  -> semantic action
```

## 3. Layer separation

| Layer | Meaning | Owner |
| --- | --- | --- |
| native event | host/platform input | native backend |
| `InputEvent` | normalized input signal | `prom-ui-runtime` |
| interaction intent | interpreted user request | future interaction layer |
| focus/selection | semantic target context | future focus/selection layer |
| admission | gate deciding if action may exist | UI/runtime policy |
| semantic UI action | admitted UI operation | future action layer |
| trace | observable causal record | audit/runtime boundary |
| effect request | optional controlled external/runtime effect | effect/capability boundary |

This preserves:

```text
Input is not action.
Intent is not action.
Selection is not permission.
Action is not effect.
Effect requires its own boundary.
```

## 4. Semantic action definition

A Semantic UI action is:

```text
an admitted, named, traceable UI operation
```

It must have:

- name;
- source interaction intent;
- target context;
- admission result;
- lifecycle/capability preconditions;
- trace behavior;
- denial/failure behavior;
- effect relationship if any.

A Semantic UI action must not be anonymous callback behavior.

## 5. Action examples

Candidate action names:

```text
ui.action.close_window
ui.action.open_inspector
ui.action.select_trace_event
ui.action.focus_module
ui.action.expand_capability_gate
ui.action.prepare_effect
ui.action.commit_effect
ui.action.rollback_effect
ui.action.acknowledge_error
ui.action.open_denial_reason
ui.action.pin_trace
ui.action.quarantine_target
```

These are not implemented in H7.

They define conceptual vocabulary for future action admission.

## 6. Action vs interaction intent

Interaction intent is a request.

Semantic action is an admitted operation.

Example:

```text
request.select_trace_event
  -> admission
  -> ui.action.select_trace_event
```

Denied example:

```text
request.rollback_effect
  -> admission denied
  -> no semantic action
  -> denial trace/visibility
```

A denied intent must not be treated as an action.

## 7. Action vs focus/selection

Focus and selection may provide context.

They do not grant permission.

Example:

```text
selected effect trace
  -> request.rollback_effect
  -> admission check
  -> ui.action.rollback_effect
```

Selection alone does not mean rollback is allowed.
Focus alone does not mean action target is admitted.

## 8. Action vs effect

A semantic UI action may request an effect.

It is not the effect itself.

Example:

```text
ui.action.prepare_effect
  -> effect request
  -> capability/effect admission
  -> prepared effect
```

Another example:

```text
ui.action.close_window
  -> UI lifecycle request
  -> lifecycle admission
  -> close operation
```

The effect boundary must remain explicit.

No UI action may silently produce external effects.

## 9. Action vs VM/SemCode operation

Semantic UI actions are UI-level operations.

They are not automatically:

- VM instructions;
- SemCode operations;
- verifier admissions;
- capability effects;
- Workbench commands;
- Semantic language operations.

A future bridge from UI action to VM/SemCode must be separately admitted.

H7 does not define that bridge.

## 10. Admission requirement

Every semantic UI action must be admitted.

Admission may check:

- lifecycle state;
- capability state;
- component state;
- focus/selection target;
- target ownership;
- traceability requirement;
- renderer/native readiness;
- effect boundary;
- policy gates.

No action should exist before admission.

## 11. Denial behavior

Denied action requests must be visible.

They may produce:

```text
denial trace
denial reason
denial component state
capability missing indicator
lifecycle invalid indicator
policy refused indicator
```

Denied requests must not silently disappear if they affect user expectation or system state.

## 12. Trace requirement

Semantic UI actions must be traceable when they affect:

- lifecycle state;
- focus/selection state;
- capability/admission state;
- effect preparation;
- effect commitment;
- rollback;
- quarantine/conflict state;
- renderer/presentation state;
- visible UI state with semantic meaning.

Trace may be suppressed only for purely decorative or non-semantic movements.

## 13. Component relationship

Components may expose interaction surfaces that request semantic actions.

Example:

```text
CapabilityDecisionView
  -> request.open_capability_detail
  -> admission
  -> ui.action.open_capability_detail

DenialReasonOverlay
  -> request.acknowledge_error
  -> admission
  -> ui.action.acknowledge_error
```

Components must not execute actions directly from raw input.

## 14. Layout relationship

Layout primitives may provide action target context.

Example:

```text
TraceLane
  -> selected trace event
  -> rollback target context

CapabilityGate
  -> capability target context
```

Layout primitives do not own action meaning.

## 15. Renderer relationship

Renderer must not own Semantic UI actions.

Renderer may display action affordances after UI contracts admit them.

Renderer must not decide:

- action meaning;
- action admission;
- capability result;
- effect result;
- trace result;
- whether action should mutate state.

Renderer output is not action authority.

## 16. Native backend relationship

Native backend must not own Semantic UI actions.

Native backend may capture host events and translate them into normalized input signals.

Native backend must not decide:

```text
action meaning
action admission
semantic target
effect boundary
trace semantics
component policy
```

Native backend is not the action model.

## 17. Workbench relationship

Workbench may become a consumer of admitted UI actions.

Workbench must not define core action semantics.

Workbench-specific actions require one of:

- Workbench-local action namespace;
- explicit admission into core UI action contract;
- separate boundary document.

No Workbench convenience action should leak into core UI semantics.

## 18. Forbidden shortcuts

The system must not:

- treat `InputEvent` as action;
- treat interaction intent as action;
- treat selection as permission;
- perform effects from component callbacks without admission;
- let renderer define action meaning;
- let native backend define action policy;
- let Workbench define core action semantics;
- treat UI action as VM/SemCode operation by default;
- hide denied actions;
- collapse action and effect into one callback.

## 19. Required action admission rule

A future semantic action implementation PR must define:

1. action name;
2. source interaction intent;
3. target identity model;
4. target ownership;
5. admission preconditions;
6. lifecycle constraints;
7. capability constraints;
8. trace behavior;
9. denial behavior;
10. effect relationship if any;
11. tests/snapshots where applicable.

No action should be added only because it is convenient for UI wiring.

## 20. Future implementation shape

H7 does not mandate implementation.

Possible future shapes:

```text
docs/spec/ui_semantic_actions.md
crates/prom-ui-actions/
crates/prom-ui-interaction/
prom-ui-runtime action admission module
Workbench action map
renderer action affordance map
```

Any implementation must preserve:

```text
input
  -> intent
  -> admission
  -> semantic action
  -> trace
  -> optional effect request
```

## 21. Current decision

Semantic UI actions are not implemented in H7.

H7 only defines the boundary.

Current admitted visual/interaction architecture:

```text
visual doctrine
  -> visual token boundary
  -> layout primitive boundary
  -> component admission boundary
  -> interaction/input semantic boundary
  -> focus/selection semantic boundary
  -> semantic action boundary
```

Not yet admitted:

```text
action structs
action registry
action dispatcher
UI effect bridge
VM/SemCode bridge
Workbench action implementation
renderer action affordances
action-to-capability bridge
```

## Effect request and UI capability dependency

Effect requests and UI capabilities are defined separately in:

```text
docs/architecture/ui_effect_request_capability_boundary.md
```

Semantic UI action is not effect.

```text
semantic UI action
  -> optional effect request
  -> UI capability admission
  -> runtime capability mapping if admitted
```

No semantic UI action may silently produce external effects.

## Trace and audit visual dependency

Trace/audit visual boundaries are defined separately in:

```text
docs/architecture/ui_trace_audit_visual_boundary.md
```

Semantic actions may produce trace records.

A visual trace view may display those records, but must not define action success or audit authority.
