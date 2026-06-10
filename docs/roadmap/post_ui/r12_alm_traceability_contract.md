# R12 ALM Traceability Contract

## 1. Purpose

Define future ALM traceability boundaries before implementation. This document does not authorize code.

## 2. Current Factual State

* ALM DNA spec exists.
* ALM core posture exists.
* ALM association model contract exists.
* ALM activation contract exists.
* ALM remains future-only.
* No trace model exists.
* No activation engine exists.
* No backend trait exists.
* No LLM operator integration exists.
* Semantic verifier/admission remains separate.

## 3. Traceability Definition

Future ALM traceability means ALM output can explain:
* what was activated;
* why it was activated;
* from which seed concept;
* through which association path;
* with what evidence posture;
* with what conflict posture;
* with what source/gate/spec reference where available.

## 4. Trace Output Shape

Future trace output may include:
* seed concepts
* activated concepts
* relation path
* relation kinds
* evidence posture
* conflict posture
* score/rank if present
* source/gate references
* diagnostic/source links where available

No trace output structure is implemented here.

## 5. Trace Is Not Truth

* trace explains association, not correctness.
* trace does not prove Semantic validity.
* trace does not prove verifier admission.
* trace does not prove runtime readiness.
* trace does not override conflict.
* trace does not replace Local Admission Guard.

## 6. Evidence Trace Boundary

* evidence trace may point to docs/specs/tests/audits/future memory.
* evidence is not admission.
* evidence must remain inspectable.
* evidence must not be silently collapsed into score.

## 7. Conflict Trace Boundary

* conflict trace must remain visible.
* conflict is not ordinary failure.
* conflict must not be hidden by ranking.
* conflict must not be silently filtered from explanation.
* conflict posture may use Quad-state later.

## 8. Determinism Contract

* same activation input + same association graph + same trace config = same trace output.
* no wall-clock time.
* no randomness.
* no file I/O.
* no network access.
* no host effects.
* no global mutable state.

## 9. Backend Boundary

* trace semantics belong to ALM core/engine contract.
* backend may accelerate trace collection only after separate gate.
* backend replacement must not change trace meaning.
* Turbovec backend remains future-only.
* backend must not own truth/admission.

## 10. LLM Operator Boundary

* LLM may later format trace output.
* LLM may not invent trace evidence.
* LLM may not hide conflict.
* LLM may not convert trace into admission.
* LLM output remains subordinate to Semantic verifier/admission.

## 11. Forbidden Behavior

* no implementation
* no dependency addition
* no ALM code
* no backend code
* no Turbovec import
* no parser/verifier/runtime/renderer integration
* no Workbench/Studio
* no source-of-truth claim

## 12. Admission Guard Table

| Area | Current status | Admission Guard classification | Notes |
|---|---|---|---|
| traceability contract | Implemented | ADMITTED | contract definition |
| trace output shape | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| evidence trace | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| conflict trace | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| deterministic trace output | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| backend trace acceleration | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| LLM trace formatting | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| Semantic verifier/admission ownership | Absent | FORBIDDEN | ALM/backend/LLM must not own |
| Workbench/Studio integration | Absent | FORBIDDEN | out of scope |

## 13. Future Gates

* R12-ALM-TRACEABILITY-AUDIT
* R12-ALM-TRACE-OUTPUT-CONTRACT
* R12-ALM-EVIDENCE-POSTURE-CONTRACT
* R12-ALM-CONFLICT-POSTURE-CONTRACT
* R12-ALM-TRACEABILITY-SEED
* R12-ALM-LLM-OPERATOR-BOUNDARY

## 14. Final Decision

Final decision:
READY — ALM TRACEABILITY MAY EXPLAIN ASSOCIATION PATHS, BUT IT IS NOT TRUTH, ADMISSION, OR RUNTIME READINESS
