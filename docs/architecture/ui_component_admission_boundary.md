# Semantic UI Component Admission Boundary

Status: Draft
Track: POST-UI / H-series
Purpose: define the component admission boundary before implementation

## 1. Goal

This document defines the boundary for Semantic UI components.

Components are not arbitrary widgets.

Components are reusable semantic UI units that combine:

- semantic purpose;
- layout primitives;
- visual tokens;
- state interpretation;
- capability/admission meaning;
- trace relationship;
- failure/denial visibility.

A component must exist because it represents a meaningful Semantic UI concept.

A component must not be added only because it is visually convenient.

## 2. Relationship to doctrine, tokens, and layout

The component layer follows:

```text
docs/architecture/ui_visual_design_doctrine.md
docs/architecture/ui_visual_token_system_boundary.md
docs/architecture/ui_layout_primitive_boundary.md
```

Ownership chain:

```text
visual doctrine
  -> visual token system
  -> layout primitives
  -> semantic components
  -> renderer
```

The doctrine owns meaning.
Tokens own reusable visual vocabulary.
Layout primitives own spatial grammar.
Components own reusable semantic UI behavior and composition.
Renderer executes admitted component/layout output.

The renderer must not invent component meaning.

## 3. Component ownership

Components are owned by the UI architecture layer.

| Layer | Component role |
| --- | --- |
| `prom-ui-runtime` | exposes state/lifecycle data, does not own components |
| `prom-ui-backend-native` | exposes native facade/transcripts, does not own components |
| visual doctrine | owns meaning and rules |
| visual token system | supplies visual vocabulary |
| layout primitive system | supplies spatial grammar |
| component system | owns reusable semantic UI units |
| renderer | consumes resolved component/layout output |

This preserves:

```text
Meaning first.
Tokens second.
Layout third.
Components fourth.
Renderer fifth.
```

## 4. Component vs layout primitive

Layout primitives define spatial grammar.

Components define reusable semantic UI units.

Example distinction:

| Concept | Layout primitive | Component |
| --- | --- | --- |
| visual region | `ModuleRegion` | `ModuleStatusCard` |
| ordered trace space | `TraceLane` | `TraceEventRow` |
| admission area | `CapabilityGate` | `CapabilityDecisionView` |
| state surface | `StatePanel` | `LifecycleStateCard` |
| inspection area | `InspectorPane` | `SemanticObjectInspector` |

A component may use layout primitives.
A layout primitive must not own component semantics.

## 5. Component vs widget

A widget is a generic interactive UI element.

A Semantic UI component is not generic.

Examples:

| Generic widget | Semantic UI component |
| --- | --- |
| button | `AdmitActionButton` |
| badge | `CapabilityStatusBadge` |
| card | `LifecycleStateCard` |
| list row | `TraceEventRow` |
| modal | `DenialReasonOverlay` |
| sidebar | `TraceInspectorPane` |

Widgets may appear later as implementation details.
They are not admitted in H4.

## 6. Allowed component domains

Allowed future component domains:

| Domain | Purpose |
| --- | --- |
| lifecycle components | created/running/closed/failed visual units |
| capability components | admitted/denied/missing/quarantined capability units |
| trace components | trace rows, trace summaries, causal path units |
| admission components | operation request/result/denial units |
| effect components | prepared/committed/rolled-back effect units |
| error components | structured failure and recovery units |
| conflict components | conflict/quarantine/merge/inspection units |
| renderer transcript components | staging/render/presentation distinction units |
| module components | module identity/state/ownership units |
| inspector components | structured semantic object inspection units |

Concrete component implementation is out of scope for H4.

## 7. Candidate component names

H4 reserves candidate component names without implementing them:

```text
LifecycleStateCard
CapabilityStatusBadge
CapabilityDecisionView
TraceEventRow
TraceSummaryPanel
AdmissionGateView
EffectCommitView
RollbackTraceView
SemanticObjectInspector
ModuleStatusCard
ConflictBoundaryView
QuarantineNotice
RendererTranscriptView
DrawStagingStatusView
DenialReasonOverlay
```

These names are not API commitments yet.
They define conceptual vocabulary for future implementation.

## 8. Lifecycle components

Lifecycle components must show:

- current lifecycle state;
- allowed transitions;
- denied transitions;
- trace link for transitions;
- closed/final state clearly;
- failure state explicitly.

Lifecycle components must not hide invalid lifecycle operations behind generic disabled controls.

## 9. Capability components

Capability components must show:

- required capability;
- current capability status;
- admission result;
- denial reason;
- quarantine/conflict state if applicable;
- trace link if available.

Generic badges are insufficient unless they carry Semantic capability meaning.

## 10. Trace components

Trace components must preserve causality.

A trace component should be able to expose:

1. requested action;
2. admission context;
3. state before;
4. state after;
5. effect or refusal;
6. error if present;
7. transcript link.

Trace components must not become decorative timelines.

## 11. Admission components

Admission components visualize decision boundaries.

They must show:

- requested operation;
- required preconditions;
- granted/denied result;
- reason;
- capability/lifecycle relation;
- trace/audit relation.

Admission UI must be explicit and inspectable.

## 12. Renderer transcript components

Renderer transcript components must preserve distinction between:

```text
draw staging
render attempted
render succeeded
frame presented
```

They must not collapse submitted frame accounting into presentation success.

Candidate components:

```text
DrawStagingStatusView
RendererAttemptView
FramePresentationStatusView
RendererTranscriptView
```

These are not implemented in H4.

## 13. Component-token relationship

Components consume visual tokens.

Example:

```text
CapabilityStatusBadge
  -> capability.status.admitted
  -> color.admission.granted
  -> border.capability.available
  -> type.capability.label

LifecycleStateCard
  -> color.state.running
  -> border.lifecycle.active
  -> type.state.label
  -> motion.lifecycle.transition
```

Components must not define raw colors, spacing, typography, motion, or renderer behavior.

## 14. Component-layout relationship

Components consume layout primitives.

Example:

```text
SemanticObjectInspector
  -> InspectorPane
  -> panel.inspector
  -> trace/detail rows

CapabilityDecisionView
  -> CapabilityGate
  -> admission/result layout
```

Components may compose layout primitives.
Components must not redefine layout ownership rules.

## 15. Renderer relationship

Renderer consumes resolved component/layout output.

Renderer must not decide:

- which components exist;
- what components mean;
- how admission/capability meaning is interpreted;
- whether draw staging equals rendering;
- whether hidden state may be omitted.

Renderer executes admitted UI grammar.
Renderer does not own component meaning.

## 16. Forbidden component behavior

The component system must not:

- introduce arbitrary widgets without semantic purpose;
- copy generic dashboard component libraries;
- hide denial/failure behind generic disabled states;
- make visual appeal the reason for admission;
- bypass layout primitive boundary;
- bypass token boundary;
- let renderer define component semantics;
- introduce Workbench-specific components as core UI semantics;
- treat transcripts as decorative logs;
- collapse lifecycle/capability/admission distinctions.

## 17. Required component admission rule

A future component implementation PR must define:

1. component name;
2. semantic purpose;
3. consumed layout primitives;
4. consumed token categories;
5. accepted state inputs;
6. denied/failed/quarantined behavior;
7. trace/transcript relationship;
8. allowed consumers;
9. forbidden consumers;
10. tests or snapshots where applicable.

No component should be added only because it looks useful.

## 18. Future implementation shape

H4 does not mandate implementation.

Possible future shapes:

```text
docs/spec/ui_components.md
crates/prom-ui-components/
crates/prom-ui-layout/
historical TS/Tauri component map (retired, see docs/history/workbench_ts_tauri_legacy.md)
renderer-local component resolver
```

Any implementation must preserve:

```text
meaning first
tokens second
layout third
components fourth
renderer fifth
```

## 19. Current decision

Components are not implemented in H4.

H4 only defines the admission boundary.

Current admitted visual architecture:

```text
visual doctrine
  -> visual token boundary
  -> layout primitive boundary
  -> component admission boundary
```

Not yet admitted:

```text
component structs
component rendering
widget system
CSS component classes
Workbench visual components
renderer component resolver
interactive component behavior
```

## Interaction dependency

Interaction semantic admission depends on the component admission boundary.

```text
visual doctrine
  -> visual token system
  -> layout primitives
  -> semantic components
  -> interaction semantics
```

Components may expose interaction surfaces, but must not directly mutate semantic state from raw input.

Components produce interaction intent candidates.
Admission decides whether an intent becomes an action.

## Focus and selection dependency

Components may expose focusable and selectable surfaces.

Focus and selection semantics are defined separately in:

```text
docs/architecture/ui_focus_selection_semantic_boundary.md
```

Components must not directly mutate global focus or selection from raw input.

## Semantic action dependency

Components may expose action affordances.

Semantic UI actions are defined separately in:

```text
docs/architecture/ui_semantic_action_boundary.md
```

Components must not execute semantic actions directly from raw input or visual callbacks.

## Effect request dependency

Components may expose effect request affordances.

Effect requests and UI capabilities are defined separately in:

```text
docs/architecture/ui_effect_request_capability_boundary.md
```

Components must not perform effects directly.

## Trace and audit visual dependency

Components may expose trace/audit projection surfaces.

Trace/audit visual boundaries are defined separately in:

```text
docs/architecture/ui_trace_audit_visual_boundary.md
```

Components must not treat trace display as trace authority.

## Error, denial, and quarantine visual dependency

Components may expose denial, error, conflict, and quarantine surfaces.

Error, denial, and quarantine visual boundaries are defined separately in:

```text
docs/architecture/ui_error_denial_quarantine_visual_boundary.md
```

Components must not invent denial/error/quarantine meaning.

## Recovery and rollback visual dependency

Components may expose recovery, retry, cancel, and rollback surfaces.

Recovery and rollback visual boundaries are defined separately in:

```text
docs/architecture/ui_recovery_rollback_visual_boundary.md
```

Components must not invent recovery or rollback meaning.

## Workbench consumption dependency

Workbench may consume admitted components or define Workbench-local components.

Workbench UI consumption is defined separately in:

```text
docs/architecture/ui_workbench_consumption_boundary.md
```

Workbench-local components must not become core components without admission.

## Renderer transcript and presentation status dependency

Components may expose renderer transcript and presentation status surfaces.

Renderer transcript and presentation status boundaries are defined separately in:

```text
docs/architecture/ui_renderer_transcript_presentation_boundary.md
```

Components must not treat frame presentation as semantic success.
