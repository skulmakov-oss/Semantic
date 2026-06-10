# R12 UI AST Indexing Posture And Contract

## 1. Purpose

This document records the future indexing/vectorization posture after the minimal AST validation seed and posture/hardening work. It does not authorize implementation.

## 2. Current Factual State

* `validate_ast` exists.
* validation is local and structural.
* validation does not call lowering.
* current validation may use linear lookup.
* parser/verifier/runtime/renderer do not exist for UI.
* indexing/vectorization are not implemented.

## 3. Why Indexing Is Future Work

* current seed optimizes correctness and reviewability;
* performance indexing is a separate internal layer;
* validation semantics must be stable before storage optimization.

## 4. Accepted Current Cost

* O(N²) lookup posture is accepted for the current seed;
* this is not a defect;
* it is a conscious seed-stage tradeoff.

## 5. Future Dense Index Direction

* `UiAstNodeId` remains public handle.
* future internal `NodeIndex` may map nodes into dense `0..N` space.
* dense index is internal, not semantic truth.
* dense index must not become repository identity.
* dense index must not imply admission.

## 6. Preferred Future Storage Shapes

* sorted id-to-index table;
* dense id-to-index table if IDs are dense;
* SoA for node properties;
* CSR for parent/children adjacency;
* bitplanes for Quad-state overlays.

HashMap is not the preferred first target.
HashMap may only be considered behind a separate gate if needed.

## 7. Topology vs State

* topology: parent/children, edges, adjacency;
* state: validation/admission/evidence/conflict;
* topology should likely use SoA/CSR;
* Quad-state should use T/F bitplanes where applicable.

## 8. Quad-State Overlay Posture

* future N/F/T/S overlays may use T-plane and F-plane packing;
* unknown/conflict must not be flattened;
* packed representation must preserve Semantic meaning.

## 9. Turbovec Posture

* Turbovec is currently inspiration / possible future backend candidate.
* This document does not authorize Turbovec dependency.
* No Turbovec integration is allowed here.
* Any future Turbovec backend requires separate contract, owner approval, and dependency gate.

## 10. Validation Relationship

* current `validate_ast` remains simple.
* future indexing may accelerate validation.
* future indexing must not change validation semantics.
* future indexing must not make validation call parser, lowering, verifier, runtime, or renderer.

## 11. Forbidden Behavior

* no implementation;
* no dependencies;
* no parser/verifier/runtime/renderer;
* no Workbench/Studio;
* no indexing code;
* no SoA/CSR code;
* no Turbovec code.

## 12. Admission Guard Table

| Area | Current status | Admission Guard classification | Notes |
|---|---|---|---|
| current O(N²) validation | Implemented | ADMITTED | accepted seed tradeoff |
| dense NodeIndex | Absent | FUTURE_ONLY_NOT_AUTHORIZED | requires separate contract |
| SoA | Absent | FUTURE_ONLY_NOT_AUTHORIZED | requires separate contract |
| CSR | Absent | FUTURE_ONLY_NOT_AUTHORIZED | requires separate contract |
| packed Quad bitplanes | Absent | FUTURE_ONLY_NOT_AUTHORIZED | requires separate contract |
| HashMap indexing | Absent | FUTURE_ONLY_NOT_AUTHORIZED | behind separate gate if needed |
| Turbovec | Absent | FUTURE_ONLY_NOT_AUTHORIZED | requires dependency gate |
| parser integration | Absent | FORBIDDEN | out of scope |
| verifier/runtime/renderer | Absent | FORBIDDEN | out of scope |
| Workbench/Studio | Absent | FORBIDDEN | out of scope |

## 13. Future Gates

* R12-UI-AST-INDEXING-AUDIT
* R12-UI-AST-INDEXED-LOOKUP-CONTRACT
* R12-UI-AST-INDEXED-LOOKUP-SEED
* R12-UI-QUAD-BITPLANE-POSTURE
* R12-ALM-TURBOVEC-BACKEND-POSTURE

## 14. Final Decision

Final decision:
READY — INDEXING, SOA, CSR, QUAD BITPLANES, AND TURBOVEC REMAIN FUTURE-ONLY UNTIL SEPARATELY GATED
