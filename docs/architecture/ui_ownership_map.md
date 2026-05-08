# Semantic UI Ownership Map

Status: Draft
Track: POST-UI
Scope: ownership and boundaries only
Implementation: out of scope

Related:
- `docs/spec/ui_contract_map.md`
- `docs/spec/ui_abi_capability_admission.md`
- `docs/roadmap/post_ui/ui_admission_checklist.md`
- `docs/architecture/ui_native_backend_boundary.md`
- `docs/architecture/ui_renderer_admission_boundary.md`
- `docs/architecture/ui_visual_design_doctrine.md`
- `docs/architecture/ui_visual_token_system_boundary.md`
- `docs/architecture/ui_layout_primitive_boundary.md`
- `docs/architecture/ui_component_admission_boundary.md`
- `docs/architecture/ui_interaction_input_semantic_boundary.md`
- `docs/architecture/ui_focus_selection_semantic_boundary.md`
- `docs/architecture/ui_semantic_action_boundary.md`
- `docs/architecture/ui_effect_request_capability_boundary.md`
- `docs/architecture/ui_trace_audit_visual_boundary.md`

## 1. Purpose

This document defines ownership boundaries for the Semantic UI/Application layer.

The UI layer must not become:

- a second compiler;
- a VM policy layer;
- a hidden host side-effect path;
- a Workbench-only feature;
- a widget/layout framework in the first slice.

## 2. Layer position

Semantic UI lives after verified execution and before platform-native rendering.

```text
Semantic source
  ↓
SemCode
  ↓
Verifier admission
  ↓
VM
  ↓
prom-abi HostCallEnvelope
  ↓
prom-cap capability check
  ↓
prom-ui contract types
  ↓
prom-ui-runtime
  ↓
platform backend
```

## 3. Ownership matrix

| Entity | Owner | May read | May mutate/execute | Must not own |
| --- | --- | --- | --- | --- |
| UI source surface | `sm-front` / `sm-sema` | `sm-ir`, `sm-emit` | nobody | `sm-vm`, `prom-ui-runtime` |
| UI call lowering | `sm-ir` / `sm-emit` | `sm-verify`, `sm-vm` | nobody | `prom-ui` |
| UI ABI call IDs | `prom-abi` | `sm-verify`, `sm-vm`, `prom-runtime`, `prom-ui-runtime` | `prom-runtime` dispatches | `sm-front`, `sm-sema` |
| UI capabilities | `prom-cap` | `sm-verify`, `prom-runtime`, `prom-ui-runtime` | `prom-cap` / `prom-runtime` | `sm-vm` policy logic |
| UI event model | `prom-ui` | `prom-ui-runtime`, apps, tests | `prom-ui-runtime` produces events | `sm-front`, `sm-vm` |
| Window lifecycle contract | `prom-ui` | `prom-ui-runtime`, `prom-runtime` | `prom-ui-runtime` | `sm-vm` |
| Draw command model | `prom-ui` | `prom-ui-runtime`, tests | `prom-ui-runtime` consumes | VM internals |
| Frame lifecycle | `prom-ui` | `prom-ui-runtime` | `prom-ui-runtime` | compiler layers |
| Platform backend | `prom-ui-runtime` or backend crate | nobody outside runtime boundary | backend implementation | `prom-ui` contract crate |
| Demo app | `prom-ui-demo` | tests / docs | demo only | core contracts |

## 4. Boundary rules

### Rule UI-1 - ABI-only host access

UI effects must go through `prom-abi`.

No UI crate may create an alternate side-effect path into the host.

### Rule UI-2 - Capability before UI effect

Every effectful UI operation must have an explicit capability path.

Examples:

- `DesktopSession`
- `InputPoll`
- `FrameEmit`

### Rule UI-3 - VM is not a UI runtime

The VM may dispatch admitted host calls, but must not:

- own windows;
- store platform handles as native UI objects;
- interpret widget semantics;
- perform layout;
- own UI capability policy.

### Rule UI-4 - UI runtime is not a compiler

`prom-ui-runtime` must not:

- parse `.sm`;
- typecheck Semantic source;
- lower AST/IR;
- verify SemCode structure.

### Rule UI-5 - Determinism boundary

UI execution is deterministic only under the same admitted program, same config, same capability context, and same external event stream.

```text
program determinism ≠ environment determinism
```

### Rule UI-6 - First slice is immediate-mode command boundary

The first UI slice is:

- window lifecycle;
- event polling;
- frame begin/end;
- minimal draw commands.

It is not:

- widget framework;
- layout engine;
- retained UI tree;
- browser target;
- mobile target;
- GPU/shader pipeline.

## 5. Workbench separation

Workbench may later visualize or drive UI app builds, but it does not own the Semantic UI application contract.

Workbench is tooling/operator surface.
Semantic UI is application/runtime boundary.

## 6. DoD

This document is complete when:

- every UI concept has one owner;
- VM/compiler/runtime boundaries are explicit;
- forbidden ownership leaks are listed;
- Workbench is separated from UI application boundary;
- capability and ABI ownership are explicit.

### Native backend facade ownership

`prom-ui-backend-native` now separates three roles:

| Role | Type | Responsibility |
|---|---|---|
| staged backend | `NativeBackend` | stores staged config/events/frame accounting |
| native app facade | `NativeBackendWinitApp` | owns the native app run path outside `UiBackendAdapter` |
| app state | `NativeBackendWinitAppState` | implements winit `ApplicationHandler` and owns native window state during run |

This preserves the core invariant:

```text
prom-ui-runtime remains platform-neutral.
UiBackendAdapter remains unchanged.
Native-specific ownership stays inside prom-ui-backend-native.
```

### Renderer ownership is not admitted yet

The renderer is not owned by `prom-ui-runtime`, `UiBackendAdapter`, or `NativeBackend::run_event_loop(...)`.

Renderer ownership must be introduced through a separate admitted layer.

Current status:

| Component | Renderer ownership |
|---|---|
| `prom-ui-runtime` | none |
| `UiBackendAdapter` | none |
| `NativeBackend` | none |
| `NativeBackendWinitApp` | none |
| `NativeBackendWinitAppState` | none |
| future renderer type/crate | not admitted yet |

Draw staging is not renderer ownership.

### Visual doctrine ownership

Semantic UI visual meaning is owned by the UI doctrine and architecture contracts, not by the renderer or native backend.

| Component | Visual meaning ownership |
|---|---|
| `prom-ui-runtime` | exposes state/lifecycle data, does not own visual style |
| `prom-ui-backend-native` | exposes native facade/transcripts, does not own visual doctrine |
| renderer | executes admitted visual grammar, does not define meaning |
| UI visual doctrine | owns visual principles and semantic visual grammar |

This preserves the rule:

```text
Renderer serves Semantic UI doctrine.
Renderer does not define Semantic UI doctrine.
```

### Visual token ownership

Visual tokens are owned by the UI architecture layer.

| Component | Token role |
|---|---|
| visual doctrine | owns meaning |
| visual token system | owns reusable visual vocabulary |
| renderer | consumes resolved tokens |
| native backend | does not own tokens |
| `prom-ui-runtime` | does not own tokens |

This preserves:

```text
Meaning first.
Tokens second.
Renderer third.
```

### Layout primitive ownership

Layout primitives are owned by the UI architecture layer.

| Component | Layout role |
|---|---|
| visual doctrine | owns meaning |
| visual token system | owns reusable visual vocabulary |
| layout primitive system | owns spatial grammar |
| renderer | consumes resolved layout output |
| native backend | does not own layout |
| `prom-ui-runtime` | does not own layout |

This preserves:

```text
Meaning first.
Tokens second.
Layout third.
Renderer fourth.
```

### Component ownership

Components are owned by the UI architecture layer.

| Component | Component role |
|---|---|
| visual doctrine | owns meaning |
| visual token system | supplies visual vocabulary |
| layout primitive system | supplies spatial grammar |
| component system | owns reusable semantic UI units |
| renderer | consumes resolved component/layout output |
| native backend | does not own components |
| `prom-ui-runtime` | does not own components |

This preserves:

```text
Meaning first.
Tokens second.
Layout third.
Components fourth.
Renderer fifth.
```

### Interaction/input semantic ownership

Interaction semantics are owned by the UI architecture layer.

| Component | Interaction role |
|---|---|
| native backend | captures/translates host events |
| `prom-ui-runtime` | owns normalized `InputEvent` contracts |
| component system | exposes possible interaction surfaces |
| layout primitive system | provides target context |
| interaction semantic layer | owns interpreted UI intent |
| admission/policy layer | decides whether intent becomes action |
| renderer | may provide input plumbing, does not own semantics |

This preserves:

```text
Native event first.
Input signal second.
Interaction intent third.
Admission fourth.
Semantic action fifth.
Trace/effect sixth.
```

### Focus and selection semantic ownership

Focus and selection are owned by the UI architecture layer.

| Component | Focus/selection role |
|---|---|
| native backend | captures/translates host events only |
| `prom-ui-runtime` | may own normalized `InputEvent` contracts later, not raw semantics |
| component system | exposes focusable/selectable surfaces |
| layout primitive system | provides target context |
| interaction semantic layer | produces focus/selection requests |
| focus/selection semantic layer | owns focus/selection meaning |
| admission/policy layer | decides whether focus/selection changes are allowed |
| renderer | may display admitted focus/selection state, does not own meaning |

This preserves:

```text
Hover is not focus.
Focus is not selection.
Selection is not action.
Action requires admission.
```

### Semantic action ownership

Semantic UI actions are owned by the UI architecture layer.

| Component | Action role |
|---|---|
| native backend | captures/translates host events only |
| `prom-ui-runtime` | may host future action admission contracts |
| component system | exposes action affordance surfaces |
| interaction semantic layer | produces action requests |
| focus/selection semantic layer | provides target context |
| semantic action layer | owns admitted UI actions |
| admission/policy layer | decides whether action may exist |
| renderer | may display action affordances, does not own action meaning |

This preserves:

```text
Intent is not action.
Selection is not permission.
Action is not effect.
Effect requires boundary.
```

### Effect request and UI capability ownership

Effect requests and UI capabilities are owned by the UI architecture/admission layer.

| Component | Effect/capability role |
|---|---|
| semantic action layer | may request effects |
| UI capability layer | owns UI-level capability admission |
| runtime capability layer | owns actual runtime capability gates |
| effect boundary | owns prepare/commit semantics |
| component system | exposes affordances only |
| renderer | displays admitted state, does not grant/perform effects |
| native backend | performs platform operation only after admitted boundary |
| Workbench | may consume admitted effects, does not define core semantics |

This preserves:

```text
Action is not effect.
Effect request is not committed effect.
UI capability is not runtime capability by default.
Prepared effect is not committed effect.
```

### Trace and audit visual ownership

Trace/audit visual projection is owned by the UI architecture layer.

| Component | Trace/audit role |
|---|---|
| semantic action layer | may produce action trace facts |
| effect boundary | may produce effect trace facts |
| capability/admission layer | may produce admission/denial trace facts |
| audit/runtime boundary | owns authoritative audit records |
| UI trace projection layer | displays trace/audit facts |
| component system | exposes trace surfaces |
| layout primitive system | provides trace lanes/panels |
| renderer | renders projections, does not own trace/audit meaning |
| native backend | may expose transcript facts, not audit authority |
| Workbench | may display projections, does not define core semantics |

This preserves:

```text
Trace is not decorative log.
Audit is not UI state.
Visual trace is not source of truth.
Renderer output is not audit authority.
```
