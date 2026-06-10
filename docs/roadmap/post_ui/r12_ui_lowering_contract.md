# R12 UI Lowering Contract

Status: Draft
Track: R12 / POST-UI / Semantic UI Model
Scope type: lowering contract specification
Implementation status: not authorized by this document

## Purpose

This document defines the future contract for Semantic UI AST→IR lowering.
It applies after the current inert `UiAst` / `UiIr` model and AST/IR boundary work.
It does not authorize lowering implementation.
It does not authorize parser integration.
It does not authorize verifier/VM/runtime integration.
It does not authorize renderer/backend integration.
It does not authorize Workbench or Semantic Studio implementation.
It does not claim stable or final API status.

## Current Factual State

- `UiAst` and `UiIr` exist.
- AST and IR are separate.
- AST/IR separation is reinforced by tests.
- No AST→IR lowering exists.
- No lowering function exists.
- No conversion traits exist.
- No parser hook exists.
- No verifier hook exists.
- No runtime hook exists.
- No renderer hook exists.
- No WGPU/winit backend exists.

## Definition of Lowering

Future lowering is a deterministic transformation from an admitted UI AST input into a normalized UI IR output.

Lowering is not parsing.
Lowering is not verification.
Lowering is not VM execution.
Lowering is not rendering.
Lowering is not layout.
Lowering is not event handling.
Lowering is not capability enforcement by itself.
Lowering does not create semantic truth.

## Input Contract

Future lowering input is expected to be a `UiAst` value or a future bounded AST input wrapper.
AST must be structurally valid according to a future AST validation contract.
AST must not be assumed parser-produced unless parser integration is separately gated.
AST must not imply source validity by existence alone.

Current `UiAst` is inert and not yet admitted input.
Future admitted input requires a separate gate.

## Output Contract

Future lowering output is expected to be a `UiIr` value or a future bounded IR output wrapper.
IR must be normalized relative to the future lowering rules.
IR output must not imply verifier admission.
IR output must not imply runtime readiness.
IR output must not imply renderer readiness.

Current `UiIr` is inert and not executable.

## Determinism Contract

Future lowering must be deterministic:

- same AST input + same lowering configuration = same IR output
- no wall-clock time
- no randomness
- no file I/O
- no environment reads
- no network access
- no host effects
- no global mutable state
- no dependency on renderer/backend/runtime availability

## Diagnostics / Error Contract

Future lowering errors must be structured diagnostics, not panics.

- malformed AST must return diagnostics
- unsupported AST vocabulary must return diagnostics
- unresolved future bindings/actions must return diagnostics
- diagnostics must preserve source and structural context where available
- diagnostics must not execute effects
- diagnostics must not call renderer/runtime/verifier/VM

## Authority Boundary

UI may display truth. UI does not become truth.

- lowering does not own semantic truth
- lowering does not decide source validity
- lowering does not decide verifier admission
- lowering does not decide release readiness
- lowering does not own Local Admission Guard
- lowering output is not proof of correctness

## State Boundary

UI state is projection/cache, not semantic state.

- lowering input state is not Semantic state
- lowering output state is not Semantic state
- lowering state is not runtime state
- lowering state is not renderer state
- lowering state is not Workbench/Studio state
- lowering may produce future projection artifacts only through explicit contracts

## Quad-State Preservation Contract

Future lowering must preserve N/F/T/S semantics where applicable.
Unknown must not be silently dropped.
Conflict must not be flattened into ordinary failure.
Denied must not be treated as false.
Not admitted must not be treated as invalid source.
Any future Quad-state mapping must be explicit and tested.

## Forbidden Lowering Behaviors

- no parser invocation
- no verifier invocation
- no VM/runtime invocation
- no renderer/backend invocation
- no WGPU/winit usage
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

## Future Implementation Gate

Future lowering implementation requires:

- this contract merged
- explicit owner approval
- bounded PR
- input/output test vectors or local unit tests
- deterministic behavior tests
- diagnostics tests
- no dependency additions unless separately admitted
- no parser/verifier/VM/runtime/renderer integration unless separately gated
- no Workbench/Studio changes

## Allowed Future Implementation Shape

The likely future first implementation shape is a new local module under `crates/prom-ui`, for example `lowering.rs`.
The first implementation should be a pure function shape only if approved later.
It must use no external dependencies, no host effects, no renderer/runtime, and local unit tests only.

This section describes a possible future shape. It does not authorize implementation.

## Test Contract

Future tests may cover:

- deterministic AST→IR output for simple inert AST
- diagnostics for unsupported structures
- no conversion trait reliance
- no renderer/runtime/verifier invocation
- no Workbench/Studio invocation
- no release artifacts
- no npm/external UI toolkit

## Relationship to Existing Documents

This contract references and stays subordinate to:

- `docs/roadmap/post_ui/r12_ui_model_invariants.md`
- `docs/roadmap/post_ui/r12_ui_ast_ir_boundary.md`
- `docs/dna/SEMANTIC_UI_DNA.md`

Model invariants remain stronger than implementation convenience.
AST/IR boundary remains active.
Semantic UI DNA remains the authority doctrine.

## Next Recommended Step

Recommended next step after this spec:

R12-UI-LOWERING-CONTRACT-AUDIT

Then, only if audit passes and owner explicitly approves:

R12-UI-LOWERING-MINIMAL-PLAN

No lowering implementation before separate owner approval.
No code PR is authorized by this document.

## Final Decision

Final decision:
READY — USE THIS CONTRACT BEFORE ANY UI AST TO IR LOWERING IMPLEMENTATION
