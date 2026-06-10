# R12 UI Lowering Minimal Plan

Status: Draft
Track: R12 / POST-UI / Semantic UI Model
Scope type: minimal implementation plan
Implementation status: not authorized by this document

## Purpose

This document describes the smallest possible future AST→IR lowering implementation shape.
It does not authorize implementation.
It does not authorize code changes.
It does not authorize parser/verifier/VM/runtime integration.
It does not authorize renderer/backend integration.
It does not authorize Workbench or Semantic Studio implementation.
It does not claim stable or final API status.

## Current Factual State

- `UiAst` and `UiIr` exist.
- AST and IR are separate.
- AST/IR separation is tested.
- Lowering contract exists.
- No lowering implementation exists.
- No lowering function exists.
- No conversion traits exist.
- No parser/verifier/VM/runtime hooks exist.
- No renderer/backend hooks exist.
- No WGPU/winit backend exists.

## Minimal Future Goal

A pure, local, deterministic AST→IR lowering function inside `crates/prom-ui`, only after explicit owner approval.

It should lower a tiny inert subset only.
It must not parse source.
It must not verify source.
It must not execute.
It must not render.
It must not calculate layout.
It must not handle events.
It must not enforce capabilities.
It must not call runtime, verifier, VM, renderer, or backend code.

## Candidate Future Module Shape

A possible future module could be `crates/prom-ui/src/lowering.rs`.

Possible future public items, not authorized yet:

- `UiLoweringConfig`
- `UiLoweringDiagnostic`
- `UiLoweringResult`
- `lower_ast_to_ir(ast: &UiAst, config: &UiLoweringConfig) -> UiLoweringResult`

This is only a candidate shape.
This document does not authorize creating the file.
This document does not authorize adding these types.
No conversion traits should be used.

## Minimal Future Input

Future lowering input is expected to be borrowed `&UiAst`.
An optional config object may refine behavior later.
Input must be structurally valid according to future validation.
Input must not be assumed parser-produced unless parser integration is separately gated.
Input must not be admitted by existence alone.

## Minimal Future Output

Future lowering output is expected to be `UiIr` inside a result type.
Failures should be represented as diagnostics.
The output must not imply verifier admission.
The output must not imply runtime readiness.
The output must not imply renderer readiness.
The output must not create semantic truth.

## Minimal Supported Subset

The first possible future subset is:

- `Root`
- `Element`
- `Text`
- `Fragment`

`Attribute`, `Binding`, and `Action` should initially return diagnostics unless explicitly admitted later.
`EffectBoundary` must not be generated unless a separate gate defines its meaning.
No capability/effect semantics belong in the first subset.

## Determinism Requirements

Same AST + same config = same IR or same diagnostics.
No wall-clock time.
No randomness.
No file I/O.
No environment reads.
No network access.
No host effects.
No global mutable state.
No dependency on renderer/backend/runtime availability.

## Diagnostics Requirements

Future diagnostics should cover:

- unsupported AST node kind
- malformed tree/parent-child relationship if future validation detects it
- unsupported `Attribute` / `Binding` / `Action`
- invalid structural root if defined later

Diagnostics must not panic.
Diagnostics must not execute effects.
Diagnostics must not call parser, verifier, VM, runtime, or renderer code.

## Test Plan

Future tests may cover:

- empty AST returns empty or diagnostic according to the chosen rule
- Root-only AST lowers deterministically
- Root + Element lowers deterministically
- Text lowers deterministically
- unsupported `Attribute` returns diagnostic
- unsupported `Binding` returns diagnostic
- unsupported `Action` returns diagnostic
- same input twice produces same output
- no conversion traits are required
- no renderer/runtime/verifier/VM is invoked

## Forbidden In First Implementation

- parser integration
- verifier integration
- VM/runtime integration
- renderer/backend integration
- WGPU/winit
- layout engine
- draw commands
- event loop
- widget framework
- command execution
- file I/O
- network access
- host effects
- Workbench/Studio coupling
- dependency additions
- conversion traits
- public stable API claim

## Required Gate Before Code

Before any code PR:

- this plan must be merged
- owner must explicitly approve implementation
- scope must be bounded to `crates/prom-ui`
- expected changed files must be declared
- no dependencies unless separately admitted
- local tests must be included
- no Workbench/Studio/runtime/renderer/parser/verifier/VM changes

## Relationship to Existing Documents

This plan references and stays subordinate to:

- `docs/roadmap/post_ui/r12_ui_model_invariants.md`
- `docs/roadmap/post_ui/r12_ui_ast_ir_boundary.md`
- `docs/roadmap/post_ui/r12_ui_lowering_contract.md`
- `docs/dna/SEMANTIC_UI_DNA.md`

Model invariants remain stronger than implementation convenience.
AST/IR boundary remains active.
Lowering contract remains active.
Semantic UI DNA remains the authority doctrine.

## Next Recommended Step

Recommended next step after this plan:

R12-UI-LOWERING-MINIMAL-PLAN-AUDIT

Then, only if audit passes and owner explicitly approves:

R12-UI-LOWERING-MINIMAL-SEED

No lowering implementation before separate owner approval.
No code PR is authorized by this document.

## Final Decision

Final decision:
READY — USE THIS PLAN ONLY AS A GATE BEFORE ANY MINIMAL UI LOWERING SEED
