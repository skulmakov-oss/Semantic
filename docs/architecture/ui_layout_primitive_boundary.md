# Semantic UI Layout Primitive Boundary

Status: Draft
Track: POST-UI / H-series
Purpose: define the layout primitive boundary before implementation

## 1. Goal

This document defines the boundary for Semantic UI layout primitives.

Layout primitives are not widgets.

Layout primitives are controlled spatial structures that express Semantic UI architecture.

They must support:

- ownership boundaries;
- semantic state hierarchy;
- trace inspection;
- capability admission;
- lifecycle visibility;
- module grouping;
- effect/result separation;
- conflict/quarantine isolation;
- renderer-independent structure.

Layout primitives must not be introduced as arbitrary UI components.

## 2. Relationship to doctrine and tokens

The layout primitive system follows:

```text
docs/architecture/ui_visual_design_doctrine.md
docs/architecture/ui_visual_token_system_boundary.md
```

Ownership chain:

```text
visual doctrine
  -> visual token system
  -> layout primitives
  -> renderer
```

The doctrine owns meaning.
Tokens own reusable visual vocabulary.
Layout primitives own spatial grammar.
Renderer executes admitted layout instructions.

The renderer must not invent layout meaning.

## 3. Layout primitive ownership

Layout primitives are owned by the UI architecture layer.

| Layer | Layout primitive role |
| --- | --- |
| `prom-ui-runtime` | none |
| `prom-ui-backend-native` | none |
| renderer | consumes resolved layout output |
| visual token system | supplies visual roles |
| layout primitive system | owns spatial grammar |
| visual doctrine | owns meaning and rules |

This preserves:

```text
Meaning first.
Tokens second.
Layout third.
Renderer fourth.
```

## 4. Primitive categories

Allowed primitive categories:

| Category | Purpose |
| --- | --- |
| root surfaces | whole UI frame / app shell |
| module regions | module ownership grouping |
| panels | stable inspection/control surfaces |
| lanes | ordered trace/event/result flows |
| inspectors | structured object introspection |
| state cards | compact semantic state surfaces |
| gates | admission/capability decision surfaces |
| boundaries | ownership/conflict/quarantine separators |
| timelines | ordered lifecycle/trace visualization |
| maps | architecture/system/module maps |
| overlays | temporary inspection or refusal details |
| split regions | stable multi-pane working layout |

Concrete layout implementation is out of scope for H3.

## 5. Core primitive candidates

H3 reserves the following candidate names without implementing them:

```text
LayoutRoot
ModuleRegion
StatePanel
TraceLane
CapabilityGate
LifecycleStrip
InspectorPane
EffectLane
ConflictBoundary
QuarantineRegion
SystemMap
TimelineRail
SplitRegion
OverlaySurface
```

These names are not API commitments yet.
They define the conceptual vocabulary for future implementation.

## 6. Root and shell primitives

Root primitives define the highest visible boundary.

They must answer:

- what system is being viewed?
- what mode is active?
- what lifecycle state is active?
- what capabilities are globally available?
- what trace context is selected?

Forbidden root behavior:

- decorative full-screen shells;
- hidden global state;
- random dashboard grids;
- renderer-owned layout policy.

## 7. Module region primitives

Module regions represent ownership.

A module region should make clear:

- module identity;
- module state;
- module boundary;
- active/inactive status;
- failure/quarantine state;
- trace relation.

Module regions must not become generic cards without ownership meaning.

## 8. Panel primitives

Panels are stable inspection/control surfaces.

Allowed panel types:

```text
panel.state
panel.trace
panel.capability
panel.effect
panel.error
panel.inspector
panel.runtime
```

Panels must have a reason to exist.
Decorative panels are forbidden.

## 9. Lane primitives

Lanes represent ordered flows.

Allowed lanes:

```text
lane.trace
lane.effect
lane.event
lane.rollback
lane.admission
lane.lifecycle
```

A lane must preserve ordering and causal direction.

A lane must not hide denied, failed, or quarantined states.

## 10. Gate primitives

Gate primitives visualize admission and capability decisions.

Examples:

```text
gate.capability
gate.lifecycle
gate.effect
gate.verifier
gate.renderer_admission
```

A gate must show:

1. requested operation;
2. required condition;
3. admission result;
4. denial reason if denied;
5. trace link if available.

Generic disabled controls are insufficient.

## 11. Inspector primitives

Inspectors expose structured state.

Inspectors may display:

- semantic object identity;
- type/category;
- lifecycle state;
- capability requirements;
- trace references;
- errors/denials;
- transcript facts;
- renderer/staging distinction.

Inspectors must not mutate hidden state without explicit action and trace.

## 12. Boundary primitives

Boundaries express separation.

Allowed boundaries:

```text
boundary.ownership
boundary.lifecycle
boundary.capability
boundary.conflict
boundary.quarantine
boundary.runtime
boundary.renderer
```

Boundary primitives are not ornamental.
They indicate where meaning, authority, or lifecycle changes.

## 13. Map primitives

Map primitives visualize architecture.

Allowed maps:

```text
map.system
map.module
map.trace
map.capability
map.effect
map.runtime
```

Map primitives must preserve semantic relationships.
They must not become decorative node graphs.

## 14. Overlay primitives

Overlays are temporary inspection or refusal surfaces.

Allowed overlays:

```text
overlay.error_detail
overlay.denial_reason
overlay.trace_detail
overlay.capability_detail
overlay.conflict_detail
overlay.renderer_detail
```

Overlays must not hide base state.
They must be dismissible and traceable.

## 15. Layout-token relationship

Layout primitives consume visual tokens.

Examples:

```text
ModuleRegion
  -> surface.panel.primary
  -> border.lifecycle.active
  -> type.module.name
  -> space.panel.padding

CapabilityGate
  -> color.admission.denied
  -> border.capability.missing
  -> type.capability.label
  -> motion.admission.denied
```

Layout primitives must not define their own raw colors, spacing, typography, or motion.

## 16. Renderer relationship

Renderer implementation must consume resolved layout output.

Renderer must not decide:

- which primitives exist;
- what they mean;
- which state is important;
- how capability/admission meaning is encoded;
- whether draw staging equals rendering.

Renderer executes admitted spatial grammar.
Renderer does not own layout meaning.

## 17. Forbidden layout behavior

The layout system must not:

- introduce arbitrary widgets without semantic role;
- copy generic dashboard layouts;
- hide ownership boundaries;
- hide trace path;
- collapse denied/failed states into generic disabled controls;
- let renderer determine layout semantics;
- treat visual grouping as decoration;
- introduce layout engine before primitive boundary is admitted;
- bind layout meaning to platform/native backend.

## 18. Required primitive admission rule

A future layout primitive implementation PR must define:

1. primitive name;
2. semantic purpose;
3. owner;
4. allowed states;
5. required tokens;
6. allowed consumers;
7. forbidden consumers;
8. transcript/trace relationship if applicable;
9. tests or snapshots where applicable.

No primitive should be added only because it is visually convenient.

## 19. Future implementation shape

H3 does not mandate implementation.

Possible future shapes:

```text
docs/spec/ui_layout_primitives.md
crates/prom-ui-layout/
crates/prom-ui-style/
apps/workbench layout maps
renderer-local layout resolver
```

Any implementation must preserve:

```text
meaning first
tokens second
layout third
renderer fourth
```

## 20. Current decision

Layout primitives are not implemented in H3.

H3 only defines the boundary.

Current admitted visual architecture:

```text
visual doctrine
  -> visual token boundary
  -> layout primitive boundary
```

Not yet admitted:

```text
layout engine
layout structs
component system
CSS layout maps
renderer layout resolver
Workbench visual layout implementation
```

## Component dependency

Component admission depends on the layout primitive boundary.

```text
visual doctrine
  -> visual token system
  -> layout primitives
  -> semantic components
```

Components must compose admitted layout primitives rather than redefining spatial grammar.
