# R12 UI AST / IR Boundary

Status: Draft
Track: R12 / POST-UI / Semantic UI Model
Scope type: boundary specification
Implementation status: not authorized by this document

## 1. Purpose

This document defines the boundary between current inert UI AST and UI IR.
It applies to the model introduced in `crates/prom-ui/src/model.rs`.
It does not authorize AST→IR lowering.
It does not authorize parser integration.
It does not authorize verifier/VM/runtime integration.
It does not authorize renderer/backend integration.
It does not authorize Workbench or Semantic Studio implementation.

## 2. Current Factual State

- `UiAst` and `UiIr` exist as inert containers.
- `UiAstNodeKind` and `UiIrNodeKind` exist as structural vocabularies.
- There is no transform between AST and IR.
- There is no Semantic UI parser.
- There is no UI verifier admission.
- There is no UI runtime execution.
- There is no renderer adapter.
- There is no WGPU/winit backend.

## 3. UI AST Boundary

UI AST is the structural representation of future Semantic UI source-level concepts.

At this stage it is:

- not parser-bound yet;
- not syntax-final;
- not type-checked;
- not name-resolved;
- not executable;
- not admitted;
- not lowered.

Current AST vocabulary:

- Root
- Element
- Text
- Fragment
- Attribute
- Binding
- Action

`Attribute` / `Binding` / `Action` are placeholders for future source-level structure, not executable semantics.

## 4. UI IR Boundary

UI IR is the future normalized representation after admitted lowering.

At this stage it is:

- inert;
- not VM-bound yet;
- not runtime-bound yet;
- not renderer-bound yet;
- not draw commands;
- not GPU commands;
- not layout commands;
- not effect execution.

Current IR vocabulary:

- Root
- Element
- Text
- Fragment
- Property
- Action
- EffectBoundary

`EffectBoundary` is structural vocabulary, not capability enforcement or admission logic.

## 5. AST Is Not IR

- AST and IR are separate layers.
- AST is closer to future source intent.
- IR is closer to future normalized execution/admission form.
- AST nodes must not be treated as executable IR.
- IR nodes must not be treated as source syntax.
- The existence of both containers does not imply lowering exists.

## 6. No Lowering Yet

- no AST→IR transform is authorized;
- no lowering function is authorized;
- no implicit conversion is authorized;
- no parser hook is authorized;
- no verifier hook is authorized;
- no runtime hook is authorized;
- no renderer hook is authorized.

## 7. Future AST→IR Lowering Gate

Before any lowering implementation can be authorized, the following evidence is required:

- AST/IR boundary spec merged;
- model invariants preserved;
- explicit owner approval;
- lowering input/output contract;
- deterministic behavior requirement;
- error/diagnostic policy;
- no host effects;
- no renderer dependency;
- no runtime execution;
- no verifier/VM integration unless separately gated.

## 8. Authority Boundary

UI may display truth. UI does not become truth.

- AST does not own truth.
- IR does not own truth.
- lowering, when future-approved, must not create truth by transformation alone.
- verifier/admission remains separate.
- Local Admission Guard remains separate.

## 9. State Boundary

UI state is projection/cache, not semantic state.

- AST state is not Semantic state.
- IR state is not Semantic state.
- AST/IR state is not runtime state.
- AST/IR state is not repository truth.
- AST/IR state is not Workbench/Studio state.

## 10. Quad-State Boundary

- future AST/IR layers must preserve `N` / `F` / `T` / `S` semantics where applicable;
- current AST/IR model does not implement Quad-state logic;
- future lowering must not flatten unknown/conflict into booleans;
- unknown is not absent;
- conflict is not merely failure;
- denied is not false;
- not admitted is not equivalent to invalid source.

## 11. Dependency Boundary

AST/IR boundary must remain dependency-free at this stage.

- no WGPU
- no winit
- no Tauri
- no React
- no Slint/Floem/Makepad/Zed runtime adoption
- no renderer/backend dependency
- no manifest change unless separately approved

## 12. Test Boundary

Future tests may cover:

- AST/IR separation;
- inert construction;
- handle-only parent/child behavior;
- deterministic future lowering once authorized.

Tests must not invoke:

- renderer;
- runtime;
- VM;
- verifier;
- Workbench;
- Studio;
- npm/external UI toolkits;
- release artifacts.

## 13. Forbidden Expansion Without New Gate

- AST→IR lowering implementation
- parser integration
- verifier integration
- VM/runtime integration
- renderer adapter
- draw commands
- layout engine
- widget framework
- event loop
- WGPU/winit backend
- Workbench product implementation
- Semantic Studio implementation
- dependency additions
- stable/public API claims

## 14. Next Recommended Step

Recommended next step after this spec:

R12-UI-AST-IR-BOUNDARY-AUDIT

Then, only if audit passes:

R12-UI-AST-IR-SEPARATION-HARDENING

No lowering implementation before separate owner approval.

## 15. Final Decision

Final decision:
READY — USE THIS BOUNDARY BEFORE ANY UI AST TO IR LOWERING WORK
