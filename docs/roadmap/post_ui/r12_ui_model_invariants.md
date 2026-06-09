# R12 UI Model Invariants

Status: Draft
Track: R12 / POST-UI / Semantic UI Model
Scope type: invariant contract
Implementation status: not authorized by this document

## 1. Purpose

This document defines invariants for the inert Semantic UI model seed.
It applies to `crates/prom-ui/src/model.rs`.
It does not authorize new code by itself.
It does not authorize renderer/backend/runtime/parser/verifier/VM integration.
It does not authorize Workbench or Semantic Studio implementation.
It does not claim stable or final API status.

## 2. Current Model Scope

Current model surfaces:

- UI Tree
  - `UiNodeId`
  - `UiTreeId`
  - `UiNodeKind`
  - `UiNode`
  - `UiTree`
- UI AST
  - `UiAstNodeId`
  - `UiAstNodeKind`
  - `UiAstNode`
  - `UiAst`
- UI IR
  - `UiIrNodeId`
  - `UiIrNodeKind`
  - `UiIrNode`
  - `UiIr`

These types are foundation handles and inert containers.
They are not UI execution.
They are not rendering.
They are not admission.
They are not parser or VM integration.

## 3. Inertness Invariant

The model must remain inert until a separate approved PR explicitly introduces a new layer.

Inert means:

- no rendering
- no execution
- no admission decision logic
- no capability enforcement
- no parser integration
- no lowering integration
- no verifier integration
- no VM integration
- no runtime integration
- no Workbench coupling
- no Semantic Studio coupling
- no file I/O
- no host effects
- no global mutable state
- no command execution

## 4. Identifier Invariants

- IDs are local handles.
- IDs are not global semantic identity.
- IDs are not repository identity.
- IDs are not runtime handles.
- IDs are not renderer handles.
- IDs must not imply allocation ownership.
- IDs must not imply admission or authority.
- Raw ID access is allowed only as local representation.
- Future ID policy changes require explicit review.

## 5. Tree Invariants

- UI Tree is structural.
- `UiNode` parent/children are handles only.
- Parent/child handles do not imply validated graph consistency yet.
- Tree insertion does not imply traversal semantics.
- Tree insertion does not imply layout.
- Tree insertion does not imply rendering.
- Tree insertion does not imply event behavior.
- Tree insertion does not imply admission.
- `Root` / `Element` / `Text` / `Fragment` / `Slot` are structural vocabulary, not widget framework.

## 6. AST Invariants

- UI AST is not parser-bound yet.
- UI AST nodes are structural candidates.
- AST does not define final Semantic UI syntax.
- AST does not lower to IR in this seed.
- AST does not perform name resolution.
- AST does not perform type checking.
- AST does not perform effect admission.
- `Attribute` / `Binding` / `Action` variants are vocabulary placeholders, not executable semantics.

## 7. IR Invariants

- UI IR is not VM-bound yet.
- UI IR is not runtime-bound yet.
- UI IR is not renderer-bound yet.
- IR does not contain draw commands.
- IR does not contain GPU commands.
- IR does not contain layout commands.
- IR does not execute effects.
- `EffectBoundary` is structural vocabulary, not capability admission logic.

## 8. State Separation Invariants

UI model state is not Semantic state.
UI model state is not runtime state.
UI model state is not admission state.
UI model state is not repository truth.
UI model state is not Workbench state.
UI model state is not Studio state.
UI model state may become an input to later layers only through explicit contracts.

UI state is projection/cache, not semantic state.

## 9. Authority Invariants

UI may display truth. UI does not become truth.

- model types do not own semantic truth.
- model types do not own release truth.
- model types do not own verifier admission.
- model types do not own Local Admission Guard.
- model types do not decide readiness.
- model types do not decide whether source is valid.

## 10. Quad-State Preservation Invariant

Future UI layers must preserve `N` / `F` / `T` / `S` meaning.
Current model seed does not implement Quad-state UI logic yet.
Future UI state must not flatten unknown/conflict into ordinary boolean status.

- unknown is not absent.
- conflict is not merely failure.
- denied is not false.
- not admitted is not equivalent to invalid source.

## 11. Public API Boundary

Current public exports are accepted as bounded foundation model exports.
They are not a final API freeze.
Future changes require bounded PRs and compatibility review.
No release/stable claim is made.
Public model names are foundation handles, not product UI API.

## 12. Dependency Invariant

Model layer must remain dependency-free beyond existing crate dependencies.

- no WGPU
- no winit
- no Tauri
- no React
- no Slint/Floem/Makepad/Zed runtime adoption
- no renderer/backend dependencies
- no manifest changes unless separately approved

## 13. Test Invariants

Tests should remain local and pure.
Tests may check ID round-trips.
Tests may check empty containers.
Tests may check inert insertion.
Tests may check parent/children as handles only.
Tests must not invoke renderer/runtime/VM/verifier/Workbench/Studio.
Tests must not create release artifacts.
Tests must not depend on npm or external UI toolkits.

## 14. Allowed Future Hardening

Possible future code hardening, not authorized here:

- constructor consistency
- private-field migration if desired
- additional accessors
- additional unit tests
- debug/display stability tests
- documentation comments
- explicit no-runtime/no-renderer compile-level checks
- AST/IR separation tests

Each future hardening step requires its own bounded PR.

## 15. Forbidden Future Expansion Without New Gate

- renderer backend
- WGPU/winit
- draw commands
- layout engine
- widget system
- event loop
- runtime integration
- parser/lowering integration
- verifier/VM integration
- Workbench product implementation
- Semantic Studio implementation
- browser/WebView ownership
- dependency additions
- release/stable/public-ready claims

## 16. Next Recommended Step

Recommended next step after this invariant contract:

R12-UI-MODEL-INVARIANTS-HARDENING

Allowed only after this spec is merged.

Scope should remain:

- `crates/prom-ui` only
- model-level tests and minor hardening only
- no renderer/runtime/Workbench/Studio

## 17. Final Decision

Final decision:
READY — USE THESE INVARIANTS AS THE GATE FOR FUTURE SEMANTIC UI MODEL HARDENING
