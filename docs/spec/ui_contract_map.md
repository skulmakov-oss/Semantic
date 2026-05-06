# Semantic UI Contract Map

Status: Draft
Track: POST-UI
Scope: public contract sketch only
Implementation: out of scope

Related:
- `docs/spec/ui_abi_capability_admission.md`
- `docs/architecture/ui_ownership_map.md`

## 1. Purpose

This document defines the minimal UI contracts that future implementation PRs must follow.

It does not define final syntax.
It defines contract ownership and first-slice semantics.

## 2. Minimal UI contract stack

```text
UI Program
  ↓
UI Host Calls
  ↓
UI Capabilities
  ↓
UI Event Stream
  ↓
Frame Lifecycle
  ↓
Draw Command Buffer
```

## 3. UI host call families

First slice host calls:

```text
UiWindowCreate
UiWindowClose
UiPollEvent
UiBeginFrame
UiEndFrame
UiDrawClear
UiDrawRect
UiDrawText
```

Out of first slice:

```text
UiImage
UiFontLoad
UiLayout
UiWidget
UiClipboard
UiDragDrop
UiGpuPipeline
UiNetworkResource
```

## 4. Capability contract

| Capability | Allows | Does not allow |
| --- | --- | --- |
| `CAP_UI_WINDOW` | create/close window | drawing or input access |
| `CAP_UI_EVENTS` | poll/read UI events | drawing, window creation |
| `CAP_UI_DRAW` | submit draw commands | event polling or window creation |

Admission rule:

```text
If SemCode contains UI host calls,
the declared capability manifest must include matching UI capabilities.
```

## 5. Event model v0

Minimal event set:

```text
Quit
KeyDown
KeyUp
MouseMove
MouseDown
MouseUp
Resize
Tick
```

Event stream rule:

```text
Given the same event stream, Semantic UI program execution must be replayable.
```

## 6. Frame lifecycle v0

Valid order:

```text
UiBeginFrame(window)
  UiDraw*
UiEndFrame(window)
```

Invalid:

- draw before begin frame;
- nested begin frame;
- end frame without begin;
- draw after close window.

## 7. Draw command model v0

Minimal commands:

```text
Clear
Rect
Text
Line
```

Rules:

- commands are submitted to a frame buffer / command buffer;
- Semantic VM does not draw pixels directly;
- backend-specific rendering is owned by `prom-ui-runtime`.

## 8. Contract map

| Contract | Owner | Verified by | Executed by |
| --- | --- | --- | --- |
| UI call IDs | `prom-abi` | `sm-verify` | `prom-runtime` / `sm-vm` host bridge |
| UI capability requirements | `prom-cap` | `sm-verify` / `prom-runtime` | `prom-runtime` |
| Event representation | `prom-ui` | tests / spec | `prom-ui-runtime` |
| Frame state machine | `prom-ui` contract, `prom-ui-runtime` enforcement | tests | `prom-ui-runtime` |
| Draw command schema | `prom-ui` | tests | `prom-ui-runtime` |
| Platform rendering | backend crate | backend tests | backend crate |

## 9. Non-goals

This PR does not introduce:

- UI syntax;
- new opcodes;
- SemCode format changes;
- new capabilities in code;
- runtime implementation;
- Workbench integration;
- demo app;
- platform backend.

## 10. Acceptance

The contract is acceptable when future implementation PRs can answer:

1. Where is this UI concept owned?
2. Does this cross VM/runtime/compiler boundaries correctly?
3. Which capability gates it?
4. Which ABI call represents it?
5. Is it deterministic under replayed event stream?
6. Is it inside or outside first UI slice?
