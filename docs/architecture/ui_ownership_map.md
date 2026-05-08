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
