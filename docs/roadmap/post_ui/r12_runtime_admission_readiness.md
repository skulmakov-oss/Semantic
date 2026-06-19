# R12 Runtime Admission Readiness Audit

## Overview
This document outlines how `crates/prom-ui-runtime` will integrate with the abstract boundaries established in the `prom-ui` core crate, specifically `ActionAdmissionGuard` and `ActionRuntimeDispatcher`.

Currently, `prom-ui` successfully defines an intent-to-dispatch pipeline:
`RawUiEvent -> UiHitTest -> InteractionIntent -> ActionRequest -> ActionAdmissionGuard -> ActionRuntimeDispatcher`.

Up until now, we have used dummy implementations in `prom-ui-demo` to verify this pipeline. The next phase requires formalizing these components within the actual runtime crate (`prom-ui-runtime`).

## Admission Guard Readiness

The `ActionAdmissionGuard` is responsible for receiving an `ActionRequestDescriptor` and determining if it should be allowed to proceed. It must return an `ActionAdmissionResult` (`Admitted` or `Denied`).

### Implementation Strategy in `prom-ui-runtime`
- **Component**: We will introduce a struct `RuntimeAdmissionController` in `prom-ui-runtime` that implements `ActionAdmissionGuard`.
- **Policy Checking**: The controller will evaluate the `ActionKind` against active application state and security policies.
- **Traceability**: If denied, it must generate a meaningful `ActionDenialTrace` capturing *why* the action was rejected (e.g., "Insufficient permissions", "Element disabled", "Target missing").
- **Handoff**: If admitted, it wraps the request into an `AdmittedAction`.

## Dispatcher Readiness

The `ActionRuntimeDispatcher` boundary takes an `AdmittedAction` and executes it.

### Implementation Strategy in `prom-ui-runtime`
- **Component**: We will introduce a `HostActionDispatcher` in `prom-ui-runtime` that implements `ActionRuntimeDispatcher`.
- **Execution Loop**: The dispatcher will place the `AdmittedAction` into an execution queue or pass it directly to the semantic host handlers (which could trigger side-effects like opening a window, making a network call, or altering local layout state).
- **Decoupling**: The runtime dispatcher remains entirely agnostic to *how* the UI rendered or captured the event. It only knows that the admission guard already approved it.

## Conclusion & Next Steps
The core boundaries are stable and robust. The integration path is clear.
Our next immediate step is to implement `RuntimeAdmissionController` and `HostActionDispatcher` within `crates/prom-ui-runtime` in PR **R12-UI-RUNTIME-ADMISSION-INTEGRATION-PR**.
