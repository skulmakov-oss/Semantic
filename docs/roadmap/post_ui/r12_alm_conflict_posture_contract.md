# R12 ALM Conflict Posture Contract

## 1. Purpose

Define future ALM conflict posture boundaries before implementation. This document does not authorize code.

## 2. Current Factual State

* ALM DNA spec exists.
* ALM core posture exists.
* ALM association model contract exists.
* ALM activation contract exists.
* ALM traceability contract exists.
* ALM evidence posture contract exists.
* ALM remains future-only.
* No conflict model exists.
* No evidence model exists.
* No trace model exists.
* No activation engine exists.
* No backend trait exists.
* No LLM operator integration exists.
* Semantic verifier/admission remains separate.

## 3. Conflict Definition

Future ALM conflict means an association, evidence item, activation path, or hint carries incompatible or disputed posture.
Conflict is a signal requiring visibility, not silent suppression.

## 4. Conflict Is Not Ordinary Failure

* conflict is not ordinary failure.
* conflict is not low confidence.
* conflict is not merely false.
* conflict is not absence.
* conflict must not be silently collapsed into score.
* conflict must not be silently filtered from ALM output.

## 5. Conflict Posture Shape

Future conflict posture may include:
* conflicting association reference
* disputed evidence reference
* conflict source
* conflict relation kind
* conflict severity if separately gated
* trace path to conflict
* optional scope/context

No conflict structure is implemented here.

## 6. Quad-State Conflict Posture

* future conflict may use N/F/T/S.
* S/conflict must remain explicit.
* unknown is not conflict.
* false is not conflict by itself.
* true is not admission.
* packed representation remains future-only.

## 7. Conflict And Evidence

* evidence may support conflict detection later.
* disputed evidence must remain inspectable.
* conflicting evidence must not be hidden by ranking.
* evidence conflict must not become admission failure by itself.
* verifier/admission remains separate.

## 8. Conflict And Activation

* activation may return candidate-with-conflict.
* candidate-with-conflict is distinct from failure.
* activation must preserve conflict path where available.
* activation must not hide conflict behind low score.
* conflict surfacing is future-only and separately gated.

## 9. Conflict And Scoring

* score may rank candidates later.
* score must not erase conflict.
* high score must not override conflict.
* low score must not replace conflict explanation.
* scoring remains future-only.

## 10. Conflict And Traceability

* trace should explain where conflict appeared.
* trace should identify relation/evidence/source where available.
* trace must not convert conflict into admission.
* trace must not hide conflict from LLM formatting.

## 11. Backend Boundary

* conflict semantics belong to ALM core/contract.
* backend may accelerate conflict scans only after separate gate.
* backend replacement must not change conflict meaning.
* Turbovec backend remains future-only.
* backend must not own truth/admission.

## 12. LLM Operator Boundary

* LLM may later format conflict explanations.
* LLM may not hide conflict.
* LLM may not invent conflict.
* LLM may not convert conflict into final truth.
* LLM output remains subordinate to Semantic verifier/admission.

## 13. Forbidden Behavior

* no implementation
* no dependency addition
* no ALM code
* no backend code
* no Turbovec import
* no parser/verifier/runtime/renderer integration
* no Workbench/Studio
* no source-of-truth claim

## 14. Admission Guard Table

| Area | Current status | Admission Guard classification | Notes |
|---|---|---|---|
| conflict posture contract | Implemented | ADMITTED | contract definition |
| conflict structure | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| disputed evidence reference | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| conflict trace path | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| Quad-state conflict | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| conflict activation surfacing | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| conflict scoring | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| backend conflict scan | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| LLM conflict formatting | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| Semantic verifier/admission ownership | Absent | FORBIDDEN | ALM/backend/LLM must not own |
| Workbench/Studio integration | Absent | FORBIDDEN | out of scope |

## 15. Future Gates

* R12-ALM-CONFLICT-POSTURE-AUDIT
* R12-ALM-CONFLICT-SOURCE-CONTRACT
* R12-ALM-CONFLICT-QUAD-STATE-CONTRACT
* R12-ALM-CONFLICT-SURFACING-CONTRACT
* R12-ALM-CONFLICT-TRACE-SEED
* R12-ALM-LLM-OPERATOR-BOUNDARY

## 16. Final Decision

Final decision:
READY — ALM CONFLICT MUST REMAIN VISIBLE AS CONFLICT, NOT FAILURE, LOW SCORE, OR SILENT FILTERING
