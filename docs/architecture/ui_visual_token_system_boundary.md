# Semantic UI Visual Token System Boundary

Status: Draft
Track: POST-UI / H-series
Purpose: define the visual token system boundary before implementation

## 1. Goal

This document defines the boundary for Semantic UI visual tokens.

Visual tokens are not decoration.

Visual tokens are named, controlled visual primitives that encode Semantic UI meaning.

They must support:

- semantic state;
- capability status;
- admission result;
- lifecycle state;
- trace category;
- conflict/quarantine state;
- effect status;
- focus and inspection state;
- renderer transcript distinction.

Visual tokens must not be introduced as arbitrary theme variables.

## 2. Relationship to visual doctrine

The visual token system implements the Semantic UI visual doctrine:

```text
docs/architecture/ui_visual_design_doctrine.md
```

The doctrine owns visual meaning.

The token system owns reusable visual vocabulary.

The renderer consumes resolved tokens.

```text
visual doctrine
  -> visual token system
  -> layout primitives
  -> renderer
```

The renderer must not invent token meaning.

## 3. Token ownership

Visual tokens are owned by the UI architecture layer, not by the renderer.

| Layer | Token ownership |
| --- | --- |
| `prom-ui-runtime` | none |
| `prom-ui-backend-native` | none |
| renderer | consumes resolved tokens |
| visual token system | owns token vocabulary |
| visual doctrine | owns meaning and rules |

This preserves:

```text
Renderer executes visual grammar.
Renderer does not define Semantic UI meaning.
```

## 4. Token categories

The first admitted token system may define categories, but not concrete values yet.

Allowed token categories:

| Category | Purpose |
| --- | --- |
| color role tokens | semantic color roles, not raw palette |
| surface tokens | panel/surface hierarchy |
| border tokens | boundaries, gates, conflict lines |
| typography role tokens | labels, state names, trace text, code text |
| spacing tokens | layout rhythm and density |
| radius tokens | component geometry discipline |
| elevation/depth tokens | hierarchy without visual noise |
| opacity tokens | disabled/unknown/quarantine state |
| motion tokens | state transition timing and intent |
| iconography tokens | status/capability/trace symbols |
| trace lane tokens | causal path and audit visualization |
| capability tokens | admitted/denied/missing capability states |
| lifecycle tokens | created/running/closed/failure states |

Concrete values are out of scope for H2.

## 5. Color role tokens

Color tokens must describe meaning, not appearance.

Allowed examples:

```text
color.state.ready
color.state.running
color.state.closed
color.admission.granted
color.admission.denied
color.capability.available
color.capability.missing
color.trace.committed
color.trace.pending
color.conflict.active
color.quarantine.isolated
color.unknown.subdued
```

Forbidden examples:

```text
color.blue500
color.coolAccent
color.fancyGlow
color.neonPrimary
color.marketingGradient
```

Raw palette names may exist later as implementation details, but Semantic UI code should prefer semantic role tokens.

## 6. Surface tokens

Surface tokens describe hierarchy:

```text
surface.root
surface.panel.primary
surface.panel.secondary
surface.inspector
surface.trace
surface.overlay
surface.quarantine
```

Surface tokens must support visual hierarchy without decorative clutter.

## 7. Border and boundary tokens

Border tokens express ownership and boundaries:

```text
border.lifecycle.active
border.lifecycle.closed
border.admission.denied
border.capability.missing
border.conflict.active
border.quarantine.isolated
border.trace.focused
```

Borders are not ornamental.
They represent semantic or structural boundaries.

## 8. Typography role tokens

Typography tokens must describe information role:

```text
type.system.title
type.module.name
type.state.label
type.capability.label
type.trace.body
type.error.message
type.code.inline
type.inspector.metadata
```

Typography must support inspection, not marketing visual weight.

## 9. Spacing and density tokens

Spacing tokens must support technical density.

Allowed direction:

```text
space.stack.tight
space.stack.normal
space.panel.padding
space.trace.row
space.inspector.section
space.control.gap
```

The goal is readable density, not empty decorative space.

## 10. Motion tokens

Motion tokens must describe state transition meaning:

```text
motion.admission.granted
motion.admission.denied
motion.lifecycle.transition
motion.trace.commit
motion.rollback
motion.conflict.isolate
motion.focus.shift
```

Forbidden:

```text
motion.idle.random
motion.decorative.loop
motion.loading.fake_progress
```

Motion tokens must never imply progress or success where the system did not produce it.

## 11. Trace and capability tokens

Trace and capability must be first-class visual token domains.

Trace tokens:

```text
trace.status.pending
trace.status.committed
trace.status.failed
trace.lane.causal
trace.lane.effect
trace.lane.rollback
```

Capability tokens:

```text
capability.status.available
capability.status.missing
capability.status.denied
capability.status.admitted
capability.status.quarantined
```

Generic disabled styling is insufficient for Semantic UI.

## 12. Renderer transcript distinction

Visual tokens must preserve the distinction between:

```text
draw staging
render attempted
render succeeded
frame presented
```

These must not collapse into one visual state.

Candidate token domains:

```text
render.status.not_admitted
render.status.staged_only
render.status.attempted
render.status.succeeded
render.status.presented
render.status.failed
```

Renderer-specific tokens are not implemented in H2.
This section only reserves the semantic space.

## 13. Forbidden token behavior

The token system must not:

- encode raw brand styling as architecture;
- introduce arbitrary colors without semantic role;
- use color as the only state signal;
- hide capability failures behind generic disabled states;
- treat draw staging as rendering;
- treat submitted frames as presented frames;
- allow renderer-specific values to define Semantic UI meaning;
- bypass visual doctrine.

## 14. Required token admission rule

A future token implementation PR must define:

1. token category;
2. semantic purpose;
3. owner;
4. allowed consumers;
5. forbidden consumers;
6. relation to doctrine;
7. tests or snapshots where applicable.

No token should be added only because it “looks nice”.

## 15. Future implementation shape

H2 does not mandate file format.

Possible future shapes:

```text
docs/spec/ui_visual_tokens.md
crates/prom-ui-style/
crates/prom-ui-tokens/
apps/workbench_ts_tauri_legacy visual token map
renderer-local token resolver
```

Any implementation must preserve:

```text
meaning first
tokens second
renderer third
```

## 16. Current decision

Visual tokens are not implemented in H2.

H2 only defines the boundary.

Current admitted visual layer:

```text
visual doctrine
  -> token boundary
```

Not yet admitted:

```text
concrete palette
CSS variables
Rust token structs
renderer token resolver
theme switching
visual components
```

## Layout primitive dependency

Layout primitive admission depends on the visual token system boundary.

```text
visual doctrine
  -> visual token system
  -> layout primitives
```

Layout primitives must consume semantic tokens rather than raw colors, spacing, typography, or motion values.

## Component dependency

Component admission depends on the visual token system boundary.

Components must consume semantic tokens rather than raw colors, spacing, typography, or motion values.

```text
visual doctrine
  -> visual token system
  -> layout primitives
  -> semantic components
```
