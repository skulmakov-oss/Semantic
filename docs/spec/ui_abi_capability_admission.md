# UI ABI and Capability Admission

Status: Draft
Track: POST-UI
Scope: admission checklist and ABI/capability bridge only
Implementation: out of scope

## 1. Purpose

This document defines how future Semantic UI operations must pass through:

- `prom-ui` operation identity;
- `prom-cap` UI capability checking;
- `prom-abi` host boundary rules;
- future `sm-verify` admission checks.

It does not introduce executable UI support.

## 2. Current boundary facts

Current code-level facts:

- `prom-ui` owns UI operation identity and UI capability taxonomy.
- `prom-cap` owns UI capability admission and denial reporting.
- `prom-abi` owns general host-call descriptors and effect/determinism classes.
- `sm-verify` owns SemCode admission before VM execution.

## 3. Existing UI operation surface

Current `prom-ui` UI operation identities:

```text
WindowCreate
WindowRun
WindowClose
EventPoll
FrameSubmit
```

Current required capability mapping:

| UI operation | Required UI capability |
| --- | --- |
| `WindowCreate` | `DesktopSession` |
| `WindowRun` | `DesktopSession` |
| `WindowClose` | `DesktopSession` |
| `EventPoll` | `InputPoll` |
| `FrameSubmit` | `FrameEmit` |

## 4. Admission rule

A UI operation is admissible only if all of these are true:

```text
operation is known
AND operation is inside the admitted UI surface
AND matching UI capability is declared
AND manifest schema/version is valid
AND verifier can see the required UI admission metadata
AND runtime checks the same capability before dispatch
```

No UI operation may execute only because it reached the VM.

## 5. ABI bridge rule

UI execution must not create a private side-effect path.

Allowed future paths:

```text
SemCode UI operation
  ↓
sm-verify admission
  ↓
VM host bridge
  ↓
prom-cap UI capability check
  ↓
prom-ui-runtime
  ↓
platform backend
```

If future work maps UI operations into `prom-abi`, that mapping must preserve:

- effect class;
- determinism class;
- stability class;
- return-value contract;
- capability requirement.

## 6. Effect classification

Recommended first-slice classification:

| UI operation | Effect class | Determinism class | Returns value |
| --- | --- | --- | --- |
| `WindowCreate` | HostWrite | HostBound | implementation-defined / handle-like token only |
| `WindowRun` | HostWrite / EventEmit boundary | HostBound | false |
| `WindowClose` | HostWrite | HostBound | false |
| `EventPoll` | HostQuery | HostBound | true |
| `FrameSubmit` | HostWrite | HostBound | false |

Notes:

- UI is host-bound.
- Replay determinism requires the same admitted program, same runtime config, same capability context, and same event stream.
- Platform timing is not part of Semantic deterministic core.

## 7. Verifier admission checklist

Future verifier support must reject SemCode when:

- UI operation id is unknown;
- UI operation is encoded without required metadata;
- required UI capability is missing from declared manifest/capability bits;
- UI operation appears in a stable-v1 profile where POST-UI is not admitted;
- frame/event/window operation violates static profile constraints, if any are expressible statically.

Future verifier support must not:

- parse UI source syntax;
- own UI runtime state;
- execute UI operations;
- validate platform handles;
- perform layout/widget checks.

## 8. Runtime admission checklist

Future runtime/VM bridge must check:

- manifest validity;
- `require_ui_op(operation)`;
- session/window lifecycle state;
- frame lifecycle state;
- event polling boundary;
- draw/frame submission boundary.

Runtime must fail closed.

## 9. Capability denial behavior

Missing UI capability must produce structured denial equivalent to:

```text
UiCapabilityDenied {
  capability,
  operation,
  code: MissingCapability,
  manifest,
  message
}
```

Denial must be observable through local diagnostics/audit path once audit integration exists.

## 10. Stable-line rule

UI remains POST-UI / post-stable unless explicitly promoted.

No B2 document may claim:

- UI is part of published stable v1;
- UI operations are executable today;
- UI verifier admission is already implemented;
- UI runtime is complete.

## 11. Non-goals

This document does not add:

- new opcodes;
- new SemCode header bits;
- new `HostCallId` variants;
- implementation in `prom-abi`;
- implementation in `prom-cap`;
- implementation in `sm-verify`;
- implementation in `sm-vm`;
- demo applications;
- Workbench integration.
