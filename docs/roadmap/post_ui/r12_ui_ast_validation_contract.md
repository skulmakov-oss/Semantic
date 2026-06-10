# R12 UI AST Validation Contract

Status: Draft
Track: R12 / POST-UI / Semantic UI Model
Scope type: validation contract specification
Implementation status: not authorized by this document

## 1. Purpose

This document defines the future structural validation contract for `UiAst`.
It applies after the minimal AST→IR lowering seed.
It does not authorize validation implementation.
It does not authorize parser integration.
It does not authorize verifier/VM/runtime integration.
It does not authorize renderer/backend integration.
It does not authorize Workbench or Semantic Studio implementation.

## 2. Current Factual State

- `UiAst` exists as a structural container.
- Minimal `lower_ast_to_ir` exists.
- No AST validation implementation exists.
- No parser exists.
- No verifier admission exists.
- No runtime execution exists.
- No renderer adapter exists.
- Current lowering does not validate graph consistency.

## 3. Definition of AST Validation

A pure, deterministic structural check over `UiAst` before admitted lowering.

Validation is not:

- parsing
- semantic verification
- type checking
- name resolution
- effect admission
- VM/runtime execution
- rendering
- layout
- event handling
- capability enforcement

## 4. Validation Input

Future input is:

- borrowed `&UiAst`
- no source text required
- no parser-produced assumption
- no host state
- no runtime state
- no renderer state

## 5. Validation Output

Future output shape is conceptually:

- success if structural constraints pass
- diagnostics if structural constraints fail
- no panics for malformed AST
- no partial mutation
- no lowering output produced by validation itself

This document does not define final Rust types.
Future implementation may introduce validation diagnostics only after separate approval.

## 6. Minimal Structural Rules

Candidate future rules:

- AST node IDs should be unique within one `UiAst`.
- At most one Root should exist unless future rules allow fragments explicitly.
- Parent handles should refer to existing AST node IDs.
- Children handles should refer to existing AST node IDs.
- Parent/children relationships should be internally consistent if both sides are present.
- A node should not be its own parent.
- Direct self-child should be invalid.
- Cycles should be diagnosed if traversal validation is introduced.
- Empty AST policy must be explicit before validation code.

These are future validation rules, not current implementation.

## 7. Minimal Lowering Relationship

- Lowering and validation are separate layers.
- Current lowering seed can operate without validation.
- Future admitted lowering may require prior validation.
- Validation success must not imply verifier admission.
- Validation success must not imply runtime readiness.
- Validation success must not imply renderer readiness.

## 8. Diagnostics Contract

Future diagnostics should cover:

- duplicate node ID
- missing parent target
- missing child target
- inconsistent parent/child relationship
- self-parent
- self-child
- cycle if cycle validation is added
- invalid root structure
- unsupported structural shape

Diagnostics:

- must be structured
- must be deterministic
- must not panic
- must not execute effects
- must not call parser/verifier/VM/runtime/renderer

## 9. Authority Boundary

UI may display truth. UI does not become truth.

AST validation does not own semantic truth.
AST validation does not decide source validity.
AST validation does not decide verifier admission.
AST validation does not decide release readiness.
AST validation does not own Local Admission Guard.

## 10. State Boundary

UI state is projection/cache, not semantic state.

AST validation state is not Semantic state.
AST validation state is not runtime state.
AST validation state is not renderer state.
AST validation state is not Workbench/Studio state.
Validation must not mutate repository truth.

## 11. Quad-State Boundary

The validation contract does not implement Quad-state UI semantics yet.
Future validation must not flatten unknown/conflict into ordinary booleans if Quad-state UI markers are introduced.
Unknown is not absent.
Conflict is not merely failure.
Denied is not false.
Not admitted is not invalid source.

## 12. Forbidden Behavior

- no parser invocation
- no verifier invocation
- no VM/runtime invocation
- no renderer/backend invocation
- no WGPU/winit
- no layout calculation
- no draw command generation
- no event loop behavior
- no widget framework behavior
- no host effects
- no file I/O
- no network access
- no command execution
- no dependency additions
- no Workbench/Studio coupling
- no new lowering behavior

## 13. Future Implementation Gate

Future AST validation implementation requires:

- this contract merged
- explicit owner approval
- bounded PR
- expected changed files declared
- no dependency additions unless separately admitted
- local pure tests
- no parser/verifier/VM/runtime/renderer integration
- no Workbench/Studio changes

## 14. Relationship to Existing Documents

- `docs/roadmap/post_ui/r12_ui_model_invariants.md`
- `docs/roadmap/post_ui/r12_ui_ast_ir_boundary.md`
- `docs/roadmap/post_ui/r12_ui_lowering_contract.md`
- `docs/roadmap/post_ui/r12_ui_lowering_seed_posture.md`
- `docs/dna/SEMANTIC_UI_DNA.md`

Model invariants remain stronger than implementation convenience.
AST/IR boundary remains active.
lowering contract remains active.
lowering posture remains active.
Semantic UI DNA remains authority doctrine.

## 15. Next Recommended Step

Recommended next step after this contract:

R12-UI-AST-VALIDATION-CONTRACT-AUDIT

Then, only if audit passes and owner explicitly approves:

R12-UI-AST-VALIDATION-MINIMAL-SEED

No validation implementation before separate owner approval.
No code PR is authorized by this document.

## 16. Final Decision

Final decision:
READY — USE THIS CONTRACT BEFORE ANY UI AST VALIDATION IMPLEMENTATION
