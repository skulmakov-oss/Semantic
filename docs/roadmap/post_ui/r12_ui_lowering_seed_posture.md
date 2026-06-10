# R12 UI Lowering Seed Posture

Status: Draft
Track: R12 / POST-UI / Semantic UI Model
Scope type: factual posture note
Implementation status: no new implementation authorized by this document

## 1. Purpose

This document records the factual posture after the minimal Semantic UI AST→IR lowering seed.
It prevents interpreting the seed as a complete UI pipeline.
It does not authorize new code.
It does not authorize parser/verifier/VM/runtime integration.
It does not authorize renderer/backend integration.
It does not authorize Workbench or Semantic Studio implementation.

## 2. What Exists Now

- `crates/prom-ui/src/lowering.rs`
- `lower_ast_to_ir`
- `UiLoweringConfig`
- `UiLoweringDiagnosticKind`
- `UiLoweringDiagnostic`
- `UiLoweringDiagnostics`
- `UiLoweringResult`
- `crates/prom-ui/src/lib.rs` lowering exports
- local pure tests
- supported subset:
  - Root
  - Element
  - Text
  - Fragment

## 3. What The Seed Does

It performs a pure local AST→IR transformation for the minimal subset.
It maps supported AST node kinds to matching IR node kinds.
It maps parent/children handles by raw value into the IR handle domain.
It returns structured diagnostics for unsupported AST kinds.
It returns `Err` instead of partial IR if unsupported nodes exist.
It remains deterministic.

## 4. Unsupported By Design

- Attribute returns diagnostics.
- Binding returns diagnostics.
- Action returns diagnostics.
- EffectBoundary is not generated.
- IR Property is not generated.
- IR Action is not generated.
- No capability/effect semantics are introduced.

## 5. What Still Does Not Exist

- Semantic UI parser
- AST validation contract implementation
- verifier admission for UI
- Local Admission Guard integration
- VM/runtime execution
- renderer adapter
- WGPU/winit backend
- layout engine
- draw commands
- event loop
- widget framework
- Workbench product implementation
- Semantic Studio implementation

## 6. What This Seed Proves

The AST and IR containers can be connected by a deterministic local transform.
The transform can reject unsupported semantic vocabulary through diagnostics.
The transform can preserve handle-only parent/children relationships.
The transform can stay free of runtime, renderer, parser, verifier, and VM dependencies.

## 7. What This Seed Does Not Prove

It does not prove UI source syntax.
It does not prove parser correctness.
It does not prove verifier admission.
It does not prove runtime execution.
It does not prove renderer readiness.
It does not prove layout semantics.
It does not prove event handling.
It does not prove application shell capability.
It does not make Workbench or Semantic Studio authorized product applications.

## 8. Authority Boundary

UI may display truth. UI does not become truth.

Lowering output is not semantic truth.
Lowering output is not verifier admission.
Lowering output is not runtime readiness.
Lowering output is not renderer readiness.
Lowering output is not release readiness.

## 9. State Boundary

UI state is projection/cache, not semantic state.

AST state is not Semantic state.
IR state is not Semantic state.
Lowering state is not runtime state.
Lowering state is not renderer state.
Lowering state is not Workbench/Studio state.

## 10. Quad-State Boundary

The minimal seed does not implement Quad-state UI semantics yet.
Future UI lowering must preserve N/F/T/S semantics where applicable.
Unknown must not be dropped.
Conflict must not be flattened into ordinary failure.
Denied must not be treated as false.
Not admitted must not be treated as invalid source.

## 11. Future Gates Still Required

- AST validation contract
- lowering diagnostics hardening
- parser boundary spec
- UI verifier admission spec
- runtime boundary spec
- renderer adapter contract
- no Workbench/Studio product work until Semantic can author UI shells through its own UI model

## 12. Next Recommended Step

Recommended next step after this posture note:

R12-UI-LOWERING-SEED-POSTURE-AUDIT

Then choose one bounded path:

- R12-UI-LOWERING-DIAGNOSTICS-HARDENING
- or R12-UI-AST-VALIDATION-CONTRACT-SPEC

No parser/runtime/renderer work is authorized by this document.

## 13. Final Decision

Final decision:
READY — TREAT MINIMAL LOWERING AS A LOCAL SEED, NOT A COMPLETE UI PIPELINE
