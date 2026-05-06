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
UI Operations
  ↓
UI Capabilities
  ↓
UI Event Stream
  ↓
Frame Lifecycle
  ↓
Draw Command Buffer
```

## 3. UI operation families

Current `prom-ui` UI operation identities:

```text
WindowCreate
WindowRun
WindowClose
EventPoll
FrameSubmit
```

These names intentionally match `UiOperationId`.

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
MultiWindow
BrowserTarget
MobileTarget
```

These are not current `UiOperationId` values.
They are future/non-goal categories only.

## 4. Capability contract

Current `prom-ui` capability identities:

| Capability | Allows | Does not allow |
| --- | --- | --- |
| `DesktopSession` | create/run/close one desktop UI session | event polling or frame submission by itself |
| `InputPoll` | poll/read UI events in an admitted session | window creation or frame submission |
| `FrameEmit` | submit a frame through the admitted UI surface | event polling or window creation |

Operation mapping:

| UI operation | Required capability |
| --- | --- |
| `WindowCreate` | `DesktopSession` |
| `WindowRun` | `DesktopSession` |
| `WindowClose` | `DesktopSession` |
| `EventPoll` | `InputPoll` |
| `FrameSubmit` | `FrameEmit` |

Admission rule:

```text
If SemCode contains admitted UI operations,
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

Current `prom-ui` exposes frame submission as a coarse operation:

```text
FrameSubmit
```

The detailed internal frame lifecycle is still a future runtime contract.

Valid future runtime order remains conceptually:

```text
begin frame
  collect draw commands
submit frame
```

Invalid future runtime cases:

- frame submission before a desktop session exists;
- frame submission after window close;
- nested frame submission if the runtime model later forbids it;
- drawing/frame emission without `FrameEmit`.

## 7. Draw command model v0

Current `prom-ui` does not expose individual draw commands as operation identities.

Current public operation:

```text
FrameSubmit
```

Conceptual future draw-command payload may include:

```text
Clear
Rect
Text
Line
```

Rules:

- draw commands are payload concepts for future frame submission;
- `Clear`, `Rect`, `Text`, and `Line` are not current `UiOperationId` values;
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
