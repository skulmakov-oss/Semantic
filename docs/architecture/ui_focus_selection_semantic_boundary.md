# Semantic UI Focus and Selection Boundary

Status: Draft
Track: POST-UI / H-series
Purpose: define focus and selection semantics before implementation

## 1. Goal

This document defines the Semantic UI boundary for focus and selection.

Focus and selection are semantic UI states.

They are not raw input states.

The project must distinguish:

```text
hover != focus
focus != selection
selection != active action target
native pointer target != semantic target
component-local focus != system focus
```

Focus and selection must be admitted, traceable, and inspectable where they affect behavior.

## 2. Relationship to interaction/input boundary

Focus and selection depend on the interaction/input semantic boundary:

```text
docs/architecture/ui_interaction_input_semantic_boundary.md
```

Current interaction chain:

```text
native host event
  -> InputEvent
  -> interaction intent
  -> admission check
  -> semantic UI action
  -> trace/effect
```

Focus/selection-specific chain:

```text
input signal
  -> interaction intent
  -> focus/selection request
  -> admission
  -> semantic focus/selection state
  -> trace
```

H6 does not implement this chain.
It defines the boundary.

## 3. Layer separation

| Layer | Meaning | Owner |
| --- | --- | --- |
| native pointer/key target | host/platform location or key event | native backend / renderer plumbing |
| `InputEvent` | normalized input signal | `prom-ui-runtime` |
| interaction intent | interpreted request | future interaction layer |
| focus request | request to make something the active inspection/keyboard target | future focus layer |
| selection request | request to mark an object/range/entity as selected | future selection layer |
| active action target | admitted target for operation | future action/admission layer |
| trace | observable causality | audit/runtime boundary |

This preserves:

```text
Pointer target is not semantic target.
Focus is not selection.
Selection is not action.
Action requires admission.
```

## 4. Focus definition

Focus is the current semantic attention target for UI operation routing.

Focus may answer:

- where keyboard-like intent routes;
- which inspector context is active;
- which trace lane is active;
- which component receives semantic interaction first;
- which object is currently being inspected.

Focus must not imply:

- object selected;
- action admitted;
- capability available;
- effect prepared;
- state mutation.

## 5. Selection definition

Selection is the current semantic chosen object, range, trace item, module, or capability target.

Selection may answer:

- what object is selected?
- what trace event is selected?
- what module is selected?
- what capability is selected?
- what range or set is selected?

Selection must not imply:

- keyboard focus;
- active action target;
- capability admission;
- effect commitment;
- mutation.

## 6. Active action target definition

Active action target is the admitted target for a semantic action.

It is downstream from focus/selection and admission.

Example:

```text
selected trace event
  -> request.rollback_effect
  -> admission check
  -> active action target
  -> rollback action
```

Selection alone is not permission to act.

## 7. Hover and pointer target boundary

Hover and pointer target are low-level input/plumbing facts.

They may support future interaction intent derivation.

They must not be treated as semantic focus or selection without admission.

Out of scope for H6:

```text
mouse hover model
pointer capture
drag selection
touch focus
hit-test implementation
coordinate routing
```

These require separate implementation PRs.

## 8. Component relationship

Components may expose focusable or selectable semantic surfaces.

Examples:

```text
TraceEventRow
  -> selectable trace event

CapabilityDecisionView
  -> focusable capability decision

SemanticObjectInspector
  -> focusable inspector context

ModuleStatusCard
  -> selectable module
```

Components must not directly mutate global focus or selection from raw input.

They may produce interaction intents:

```text
request.focus_component
request.select_object
request.select_trace_event
request.focus_inspector
```

Admission decides whether focus/selection state changes.

## 9. Layout relationship

Layout primitives provide target regions.

Examples:

```text
TraceLane
  -> trace event target context

InspectorPane
  -> inspector focus context

ModuleRegion
  -> module selection context

SystemMap
  -> graph/node selection context
```

Layout primitives do not own focus/selection meaning.

They provide spatial context and identity boundaries.

## 10. Renderer relationship

Renderer may support hit-testing or visual focus indication after admission.

Renderer must not decide:

- semantic focus;
- semantic selection;
- active action target;
- admission result;
- trace result.

Renderer may report low-level target facts only after a renderer/input model is admitted.

Renderer output must not be treated as semantic focus automatically.

## 11. Native backend relationship

Native backend may capture host events.

Native backend may translate:

```text
host event -> InputEvent
```

Native backend must not own:

```text
semantic focus
semantic selection
active action target
component focus rules
selection policy
```

Native backend is not the focus model.

## 12. Focus admission rule

Changing focus may require admission if it affects operation routing, inspection, or action target context.

Admission may check:

- lifecycle state;
- component existence;
- target ownership;
- target visibility;
- quarantine/conflict state;
- capability requirement;
- current interaction mode;
- traceability requirement.

A denied focus request must not silently mutate focus.

## 13. Selection admission rule

Changing selection may require admission if it affects action target, operation scope, or trace context.

Admission may check:

- target type;
- target ownership;
- target lifecycle state;
- target visibility;
- multi-selection policy;
- quarantine/conflict state;
- operation mode;
- traceability requirement.

A denied selection request must remain visible or inspectable if it affects user action.

## 14. Focus vs selection examples

| Scenario | Focus | Selection |
| --- | --- | --- |
| keyboard routing to trace lane | `TraceLane` | none |
| object selected in system map | maybe `SystemMap` | selected object |
| inspector active | `InspectorPane` | inspected object |
| capability gate opened | `CapabilityGate` | capability item |
| denial overlay open | `DenialReasonOverlay` | original denied target |

Focus and selection may coincide, but they are not the same state.

## 15. Multi-selection boundary

Multi-selection is not admitted in H6.

Future multi-selection must define:

- allowed target types;
- ordering;
- range semantics;
- conflict/quarantine behavior;
- admission checks;
- trace behavior;
- action target derivation.

H6 only reserves the boundary.

## 16. Focus history and trace

Focus/selection changes may be trace-relevant when they affect:

- admitted action target;
- capability decision context;
- rollback target;
- committed effect target;
- inspection result.

Future implementation must define when focus/selection changes produce trace records.

Decorative focus movement should not flood trace.

Semantic focus/selection changes that affect behavior must be explainable.

## 17. Visual representation

Visual focus and visual selection must be distinct.

Visual representation must not rely on color alone.

Candidate visual distinctions:

```text
focus
  -> active routing boundary
  -> keyboard/inspection target
  -> subtle but clear outline/region emphasis

selection
  -> chosen object/entity/range
  -> stronger object identity mark
  -> traceable target indicator

active action target
  -> admitted operation target
  -> explicit action context marker
```

Concrete visual tokens are not implemented in H6.

## 18. Forbidden shortcuts

The system must not:

- treat hover as focus;
- treat focus as selection;
- treat selection as action permission;
- treat renderer hit-test as semantic target;
- mutate focus/selection directly from raw input;
- hide denied focus/selection requests;
- let native backend own focus semantics;
- let renderer own selection semantics;
- collapse focus, selection, and active action target into one field;
- add Workbench-specific focus semantics to core UI contracts.

## 19. Required focus/selection admission rule

A future focus/selection implementation PR must define:

1. focus/selection state type;
2. source interaction intent;
3. target identity model;
4. target ownership;
5. admitted/denied behavior;
6. lifecycle/capability constraints;
7. visual representation boundary;
8. trace behavior;
9. tests/snapshots where applicable.

No focus/selection rule should be added only because it is convenient for UI wiring.

## 20. Future implementation shape

H6 does not mandate implementation.

Possible future shapes:

```text
docs/spec/ui_focus_selection.md
crates/prom-ui-interaction/
crates/prom-ui-focus/
crates/prom-ui-selection/
prom-ui-runtime focus module
Workbench focus/selection map
renderer hit-test adapter
```

Any implementation must preserve:

```text
hover/pointer target
  -> input signal
  -> interaction intent
  -> focus/selection request
  -> admission
  -> semantic focus/selection state
  -> trace if behavior-relevant
```

## 21. Current decision

Focus and selection are not implemented in H6.

H6 only defines the boundary.

Current admitted visual/interaction architecture:

```text
visual doctrine
  -> visual token boundary
  -> layout primitive boundary
  -> component admission boundary
  -> interaction/input semantic boundary
  -> focus/selection semantic boundary
```

Not yet admitted:

```text
focus structs
selection structs
multi-selection
hover model
hit testing
pointer capture
drag selection
focus traversal
selection actions
Workbench focus/selection implementation
renderer-owned focus/selection semantics
```
