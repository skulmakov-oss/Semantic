# First Native Semantic Application

**Date:** 2026-07-30  
**Status:** Executable local proof; implementation not yet promoted to the published stable or qualified release contour

Semantic has reached a major development milestone: a complete native interactive application now runs through the platform's language, verified execution, application-state, projection, and UI contours.

The proof application is a dual-mode **Quad Logic Calculator** with:

- standard arithmetic;
- native four-valued logic over `N`, `F`, `T`, and `S`;
- keyboard and pointer interaction;
- explicit evaluation and recovery state;
- a native rendered window.

## Observed application modes

### Arithmetic mode

![Quad Logic Calculator — arithmetic mode](../images/quad-logic-calculator-arithmetic.jpg)

### Quad Logic mode

![Quad Logic Calculator — Quad Logic mode](../images/quad-logic-calculator-quad-mode.jpg)

## End-to-end contour demonstrated

```text
Semantic source
  -> compile to SemCode
  -> verifier admission
  -> typed VM function invocation
  -> CalculatorState + CalculatorAction
  -> returned CalculatorState
  -> admitted projection update
  -> native UI rendering
```

The calculator state transitions are implemented in Semantic code. Rust provides the host transport, verified VM invocation, structured value conversion, and native presentation boundary.

## Why this milestone matters

This proof demonstrates that Semantic can support a complete interactive application with:

- verified typed function invocation;
- structured arguments and return values;
- persistent Semantic-owned application state;
- native Quad Logic operations;
- deterministic action processing;
- projection-driven native rendering;
- keyboard and pointer input;
- explicit error and recovery behavior.

The calculator is intentionally a small application. Its importance is architectural: it closes the first visible end-to-end loop from Semantic source to a working native program.

## Status boundary

This document records an observed local executable milestone. It does **not** by itself promote the calculator, the typed application invocation boundary, or the UI contour into Semantic's published stable or qualified limited-release promise.

Promotion requires the normal repository process: code review, qualification evidence, CI, status-matrix updates, and an explicit release decision.
