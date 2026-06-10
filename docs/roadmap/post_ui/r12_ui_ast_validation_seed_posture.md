# R12 UI AST Validation Seed Posture

Status: Draft
Track: R12 / POST-UI / Semantic UI Model
Scope type: factual posture note
Implementation status: no new pipeline implementation authorized by this document

## 1. Purpose

* This document records the factual posture after the minimal Semantic UI AST validation seed.
* It prevents interpreting validation seed as complete UI admission, parser, verifier, runtime, renderer, or application pipeline.
* It does not authorize parser integration.
* It does not authorize verifier/VM/runtime integration.
* It does not authorize renderer/backend integration.
* It does not authorize Workbench or Semantic Studio implementation.
* It does not authorize indexing or vectorized storage work.

## 2. What Exists Now

* `crates/prom-ui/src/validation.rs`
* `validate_ast`
* `UiAstValidationConfig`
* `UiAstValidationDiagnosticKind`
* `UiAstValidationDiagnostic`
* `UiAstValidationDiagnostics`
* `UiAstValidationResult`
* `crates/prom-ui/src/lib.rs` validation exports
* local pure validation tests

Current implemented diagnostics:
* `DuplicateNodeId`
* `MissingParentTarget`
* `MissingChildTarget`
* `InconsistentParentChild`
* `SelfParent`
* `SelfChild`
* `MultipleRoots`

## 3. What The Validation Seed Does

* It performs a pure local structural check over `UiAst`.
* It treats empty AST as valid in the first seed.
* It allows zero or one Root in the first seed.
* It diagnoses multiple Roots.
* It diagnoses duplicate node IDs.
* It diagnoses missing parent targets.
* It diagnoses missing child targets.
* It diagnoses inconsistent parent/child relationships.
* It diagnoses self-parent.
* It diagnoses self-child.
* It returns structured diagnostics.
* It remains deterministic.
* It does not mutate AST.
* It does not produce `UiIr`.

## 4. Unsupported By Design

* Cycle detection is not implemented.
* AST indexing is not implemented.
* Dense NodeIndex mapping is not implemented.
* HashMap-based lookup is not implemented.
* SoA storage is not implemented.
* CSR adjacency storage is not implemented.
* Quad-state packed validation overlays are not implemented.
* Turbovec integration is not implemented.
* Parser integration is not implemented.
* Verifier/VM/runtime integration is not implemented.
* Renderer/backend integration is not implemented.

## 5. What Still Does Not Exist

* Semantic UI parser
* parser-produced AST contract
* AST validation admission wrapper
* UI verifier admission
* Local Admission Guard integration for UI validation
* VM/runtime execution
* renderer adapter
* WGPU/winit backend
* layout engine
* draw commands
* event loop
* widget framework
* Workbench product implementation
* Semantic Studio implementation
* full UI pipeline

## 6. What This Seed Proves

* `UiAst` can be structurally checked locally.
* Structural diagnostics can be collected deterministically.
* AST validation can stay separate from lowering.
* AST validation can stay separate from parser/verifier/runtime/renderer.
* Parent/child consistency can be checked without mutating the AST.
* The validation layer can remain dependency-free.

## 7. What This Seed Does Not Prove

* It does not prove UI source syntax.
* It does not prove parser correctness.
* It does not prove semantic validity.
* It does not prove verifier admission.
* It does not prove Local Admission Guard admission.
* It does not prove runtime readiness.
* It does not prove renderer readiness.
* It does not prove layout semantics.
* It does not prove event handling.
* It does not prove application shell capability.
* It does not prove vectorized UI graph storage.
* It does not prove Turbovec integration.
* It does not make Workbench or Semantic Studio authorized product applications.

## 8. Relationship To Lowering

* Validation and lowering are separate layers.
* Current `validate_ast` does not call `lower_ast_to_ir`.
* Current `lower_ast_to_ir` does not require validation yet.
* Future admitted lowering may require prior validation only after a separate gate.
* Validation success does not imply successful lowering.
* Validation success does not imply verifier admission.
* Validation success does not imply runtime readiness.
* Validation success does not imply renderer readiness.

## 9. Authority Boundary

UI may display truth. UI does not become truth.

* validation output is not semantic truth.
* validation output is not source validity.
* validation output is not verifier admission.
* validation output is not Local Admission Guard admission.
* validation output is not runtime readiness.
* validation output is not renderer readiness.
* validation output is not release readiness.

## 10. State Boundary

UI state is projection/cache, not semantic state.

* AST validation state is not Semantic state.
* AST validation state is not runtime state.
* AST validation state is not renderer state.
* AST validation state is not Workbench/Studio state.
* AST validation must not mutate repository truth.

## 11. Quad-State Boundary

* The minimal AST validation seed does not implement Quad-state UI semantics yet.
* Future validation must preserve N/F/T/S semantics where applicable if Quad-state UI markers are introduced.
* Unknown must not be dropped.
* Conflict must not be flattened into ordinary failure.
* Denied must not be treated as false.
* Not admitted must not be treated as invalid source.

## 12. Indexing And Vectorization Posture

* Current validation seed intentionally uses simple local structure checks.
* Current validation seed may have O(N²) lookup behavior.
* O(N²) behavior is accepted for the first seed.
* Future internal indexing may be considered separately.
* Future indexing must not change public `UiAst` semantics without a separate gate.
* Future dense NodeIndex / SoA / CSR / packed Quad-state overlays require separate posture/contract.
* Turbovec may be studied as architectural inspiration or future backend candidate only through a separate gate.
* This document does not authorize indexing, SoA, CSR, HashMap, dense NodeIndex, or Turbovec work.

## 13. Future Gates Still Required

* AST validation diagnostics hardening
* AST validation indexing posture
* AST validation indexing contract
* cycle detection contract
* parser boundary spec
* UI verifier admission spec
* runtime boundary spec
* renderer adapter contract
* no Workbench/Studio product work until Semantic can author UI shells through its own UI model

## 14. Admission Guard Table

| Area | Current status | Admission Guard classification | Notes |
|---|---|---|---|
| validation.rs exists | Implemented | ADMITTED | pure local structural seed |
| validate_ast exists | Implemented | ADMITTED | pure local structural seed |
| validation public types exist | Implemented | ADMITTED | pure local structural seed |
| empty AST valid | Implemented | ADMITTED | part of first seed |
| zero/one Root valid | Implemented | ADMITTED | part of first seed |
| multiple Roots diagnostic | Implemented | ADMITTED | part of first seed |
| duplicate ID diagnostic | Implemented | ADMITTED | part of first seed |
| parent/child diagnostics | Implemented | ADMITTED | part of first seed |
| self-parent/self-child diagnostics | Implemented | ADMITTED | part of first seed |
| cycle detection | Absent | FUTURE_ONLY_NOT_AUTHORIZED | requires separate contract |
| parser integration | Absent | FUTURE_ONLY_NOT_AUTHORIZED | requires parser boundary spec |
| verifier/VM/runtime integration | Absent | FUTURE_ONLY_NOT_AUTHORIZED | requires UI verifier admission |
| renderer/backend integration | Absent | FUTURE_ONLY_NOT_AUTHORIZED | requires renderer adapter contract |
| Workbench/Studio implementation | Absent | FORBIDDEN | product UI blocked until Semantic authors UI shells |
| indexing / SoA / CSR / Turbovec integration | Absent | FUTURE_ONLY_NOT_AUTHORIZED | requires separate posture/contract |

## 15. Next Recommended Step

Recommended next step after this posture note and local hardening:

R12-UI-AST-VALIDATION-POSTURE-AND-HARDENING-REVIEW

Then choose one bounded path:

* R12-UI-AST-VALIDATION-DIAGNOSTICS-HARDENING-2
* or R12-UI-AST-INDEXING-POSTURE

* no parser/runtime/renderer work is authorized by this document.
* no indexing/vectorization work is authorized by this document.

## 16. Final Decision

Final decision:
READY — TREAT AST VALIDATION AS A LOCAL STRUCTURAL SEED, NOT UI ADMISSION OR A COMPLETE UI PIPELINE
