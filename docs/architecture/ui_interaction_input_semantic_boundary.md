# Semantic UI Interaction and Input Semantic Boundary

Status: Draft
Track: POST-UI / H-series
Purpose: define the interaction/input semantic boundary before implementation

## 1. Goal

This document defines the boundary between native input, runtime input events, UI interactions, and admitted semantic actions.

Interaction is not raw input.

Interaction is an interpreted user intent that may request a Semantic UI action.

Input events are signals.
Interactions are semantic intents.
Actions require admission.

The project must preserve this chain:

```text
native host event
  -> InputEvent
  -> interaction intent
  -> admission check
  -> semantic UI action
  -> trace/effect
```

No layer may skip admission and directly mutate Semantic UI state.

## 2. Relationship to UI architecture

The interaction layer follows:

```text
docs/architecture/ui_visual_design_doctrine.md
docs/architecture/ui_visual_token_system_boundary.md
docs/architecture/ui_layout_primitive_boundary.md
docs/architecture/ui_component_admission_boundary.md
```

Ownership chain:

```text
visual doctrine
  -> visual token system
  -> layout primitives
  -> semantic components
  -> interaction semantics
  -> renderer/native backend
```

Components expose possible interaction surfaces.
Interaction semantics interpret input into meaningful requests.
Admission decides whether a request may become an action.

## 3. Layer separation

| Layer | Meaning | Owns |
| --- | --- | --- |
| native event | host/platform event | native backend |
| `InputEvent` | normalized runtime input signal | `prom-ui-runtime` |
| interaction intent | interpreted UI request | future interaction layer |
| admission | whether intent may execute | UI/runtime policy |
| semantic action | accepted UI operation | future action layer |
| trace/effect | observable result | audit/runtime boundary |

This preserves:

```text
Input is not interaction.
Interaction is not action.
Action is not effect.
Effect is not hidden.
```

## 4. Native event boundary

Native events are host-bound.

Examples:

```text
winit WindowEvent
keyboard input
close request
pointer event
platform focus event
```

Native events must be translated before entering platform-neutral runtime contracts.

Native backend may translate:

```text
native event -> InputEvent
```

Native backend must not decide:

- semantic action meaning;
- admission result;
- capability result;
- trace meaning;
- component state mutation;
- Workbench policy.

## 5. InputEvent boundary

`InputEvent` is a normalized signal.

Current admitted examples:

```text
KeyDown
KeyUp
CloseRequested
```

`InputEvent` does not mean:

```text
button clicked
action admitted
state changed
effect committed
```

It only means the runtime received a normalized input signal.

Adding new `InputEventKind` values requires a separate contract PR.

## 6. Interaction intent boundary

Interaction intent is the first semantic interpretation of input.

Examples:

```text
request.close_window
request.focus_trace
request.inspect_object
request.open_capability_detail
request.retry_denied_action
request.expand_module_region
request.select_trace_event
request.prepare_effect
request.rollback_effect
```

These are not implemented in H5.

They define the conceptual layer between raw input and admitted actions.

## 7. Admission boundary

An interaction intent must pass admission before becoming an action.

Admission may check:

- lifecycle state;
- capability state;
- selected component state;
- focus state;
- target ownership;
- renderer/native readiness;
- policy gates;
- traceability requirement.

Denied interaction must be visible and traceable.

A denied interaction must not silently disappear.

## 8. Semantic UI action boundary

A semantic UI action is an admitted interaction intent.

Examples:

```text
close_window
open_inspector
select_trace_event
focus_module
expand_capability_gate
prepare_effect
commit_effect
rollback_effect
acknowledge_error
```

Actions are not implemented in H5.

A future action implementation must define:

1. action name;
2. source interaction;
3. target component/layout primitive;
4. required state;
5. required capability/admission;
6. trace behavior;
7. failure behavior.

## 9. Trace and audit requirement

Every important interaction must be explainable.

For admitted actions, the UI should be able to answer:

1. What input occurred?
2. What interaction intent was derived?
3. Which target was selected?
4. Which admission checks ran?
5. Was action admitted or denied?
6. What state changed?
7. What trace was produced?
8. What effect was prepared/committed, if any?

The UI must not hide causality.

## 10. Component relationship

Components expose interaction surfaces.

A component may expose:

```text
select
focus
expand
collapse
inspect
admit
deny detail
retry
acknowledge
```

But a component must not directly mutate semantic state from raw input.

A component may produce an interaction intent candidate.
Admission decides whether the intent becomes an action.

## 11. Layout relationship

Layout primitives provide spatial target context.

Example:

```text
TraceLane
  -> target trace event
  -> select_trace_event intent

CapabilityGate
  -> target capability
  -> open_capability_detail intent
```

Layout primitives must not own interaction meaning.
They provide structure and target regions.

## 12. Renderer relationship

Renderer may deliver low-level input to the runtime/native layer.

Renderer must not decide:

- interaction meaning;
- action admission;
- capability result;
- lifecycle result;
- semantic state mutation.

Renderer may help hit-test or map coordinates after admission of a renderer/input model, but it must not own Semantic UI intent.

Renderer executes admitted visual/input plumbing.
Renderer does not own interaction semantics.

## 13. Native backend relationship

Native backend translates host events.

Native backend may own:

```text
host event capture
platform event translation
window close event source
keyboard/pointer source after admission
```

Native backend must not own:

```text
semantic interaction intent
action admission
component semantics
trace semantics
capability semantics
```

## 14. Close request rule

`CloseRequested` is a normalized input signal, not an automatic semantic shutdown.

The system may map it to:

```text
request.close_window
```

Then admission must decide whether close is allowed.

Examples of future refusal:

```text
unsaved trace state
effect in commit window
critical operation in progress
policy denied close
```

H5 does not implement this rule.
It documents the semantic boundary.

## 15. Focus and selection rule

Focus and selection are semantic UI states.

They must not be implicit side effects of raw input.

Future focus/selection implementation must define:

- source input;
- target component/layout primitive;
- focus ownership;
- selection ownership;
- trace behavior if relevant;
- denial behavior if target is invalid.

## 16. Pointer/input expansion rule

Future pointer/touch/IME/clipboard/gamepad events require separate admission.

Out of scope for H5:

```text
mouse
touch
IME
clipboard
drag/drop
gamepad
multi-window pointer routing
text editing model
```

These require separate event/input contract PRs.

## 17. Forbidden shortcuts

The interaction system must not:

- treat native events as semantic actions;
- mutate UI state directly from raw input;
- bypass lifecycle gates;
- bypass capability/admission checks;
- hide denied interactions;
- let renderer define interaction semantics;
- let native backend define component behavior;
- collapse input, interaction, action, and effect into one callback;
- add Workbench-specific interaction semantics to core UI contracts.

## 18. Required interaction admission rule

A future interaction implementation PR must define:

1. input source;
2. interaction intent name;
3. target ownership;
4. component/layout relation;
5. required lifecycle state;
6. required capability/admission;
7. trace behavior;
8. denial behavior;
9. tests/snapshots where applicable.

No interaction should be added only because it is convenient for UI wiring.

## 19. Future implementation shape

H5 does not mandate implementation.

Possible future shapes:

```text
docs/spec/ui_interactions.md
crates/prom-ui-interaction/
crates/prom-ui-actions/
prom-ui-runtime interaction module
Workbench interaction map
renderer hit-test adapter
```

Any implementation must preserve:

```text
native event
  -> InputEvent
  -> interaction intent
  -> admission
  -> semantic action
  -> trace/effect
```

## 20. Current decision

Interaction semantics are not implemented in H5.

H5 only defines the boundary.

Current admitted visual architecture:

```text
visual doctrine
  -> visual token boundary
  -> layout primitive boundary
  -> component admission boundary
  -> interaction/input semantic boundary
```

Not yet admitted:

```text
interaction structs
semantic action structs
event expansion
pointer/touch/routing
hit testing
focus model
selection model
Workbench interaction implementation
renderer-owned input semantics
```

## Focus and selection dependency

Focus and selection semantics are defined separately in:

```text
docs/architecture/ui_focus_selection_semantic_boundary.md
```

Focus and selection are downstream from input and interaction intent.

```text
input signal
  -> interaction intent
  -> focus/selection request
  -> admission
  -> semantic focus/selection state
```

Raw input must not directly mutate focus or selection.

## Semantic action dependency

Semantic UI actions are defined separately in:

```text
docs/architecture/ui_semantic_action_boundary.md
```

Interaction intent is not action.

```text
interaction intent
  -> admission
  -> semantic UI action
```

No interaction intent may directly perform effects.
