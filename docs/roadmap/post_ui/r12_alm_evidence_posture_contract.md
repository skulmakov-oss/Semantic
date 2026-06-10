# R12 ALM Evidence Posture Contract

## 1. Purpose
Define future ALM evidence posture boundaries before implementation. This document does not authorize code.

## 2. Current Factual State
* ALM DNA spec exists.
* ALM core posture exists.
* ALM association model contract exists.
* ALM activation contract exists.
* ALM traceability contract exists.
* ALM remains future-only.
* No evidence model exists.
* No trace model exists.
* No activation engine exists.
* No backend trait exists.
* No LLM operator integration exists.
* Semantic verifier/admission remains separate.

## 3. Evidence Definition
Future ALM evidence means explainable support for an association or activation result.
Evidence may point to:
* docs
* specs
* audits
* tests
* traces
* user-provided context
* future semantic memory records

Evidence supports explanation. Evidence does not create truth.

## 4. Evidence Is Not Truth
* evidence does not prove Semantic validity.
* evidence does not prove verifier admission.
* evidence does not prove runtime readiness.
* evidence does not override conflict.
* evidence does not replace Local Admission Guard.
* evidence does not authorize code execution.

## 5. Evidence Posture Shape
Future evidence posture may include:
* source reference
* source type
* confidence/strength if separately gated
* freshness/epoch if separately gated
* relation to association
* relation to activation trace
* conflict marker if evidence is disputed

No evidence structure is implemented here.

## 6. Quad-State Evidence Posture
* future evidence may use N/F/T/S.
* unknown evidence is not absence.
* false evidence is not denial of all association.
* true evidence is not semantic truth.
* conflict evidence must remain visible.
* packed representation remains future-only.

## 7. Evidence Source Boundary
* source references must remain inspectable.
* source references must not be fabricated.
* source references must not be silently collapsed into score.
* evidence source must not imply admission.
* source availability must not be treated as source validity.

## 8. Evidence And Scoring
* evidence may influence score later.
* score is not truth.
* score must not hide conflict.
* score must not override verifier/admission.
* scoring is future-only and separately gated.

## 9. Evidence And Traceability
* evidence should connect to trace output.
* trace may explain why evidence was attached.
* evidence must remain visible in explanation.
* evidence must not be stripped by LLM formatting.
* evidence must not become admission.

## 10. Backend Boundary
* evidence semantics belong to ALM core/contract.
* backend may accelerate evidence lookup only after separate gate.
* backend replacement must not change evidence meaning.
* Turbovec backend remains future-only.
* backend must not own truth/admission.

## 11. LLM Operator Boundary
* LLM may later format evidence explanations.
* LLM may not invent evidence.
* LLM may not hide disputed evidence.
* LLM may not convert evidence into admission.
* LLM output remains subordinate to Semantic verifier/admission.

## 12. Forbidden Behavior
* no implementation
* no dependency addition
* no ALM code
* no backend code
* no Turbovec import
* no parser/verifier/runtime/renderer integration
* no Workbench/Studio
* no source-of-truth claim

## 13. Admission Guard Table

| Area | Current status | Admission Guard classification | Notes |
|---|---|---|---|
| evidence posture contract | Implemented | ADMITTED | contract definition |
| evidence source reference | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| evidence source type | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| confidence/strength | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| freshness/epoch | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| disputed evidence marker | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| Quad-state evidence | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| evidence scoring | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| evidence trace output | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| backend evidence lookup | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| LLM evidence formatting | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| Semantic verifier/admission ownership | Absent | FORBIDDEN | ALM/backend/LLM must not own |
| Workbench/Studio integration | Absent | FORBIDDEN | out of scope |

## 14. Future Gates
* R12-ALM-EVIDENCE-POSTURE-AUDIT
* R12-ALM-EVIDENCE-SOURCE-CONTRACT
* R12-ALM-EVIDENCE-QUAD-STATE-CONTRACT
* R12-ALM-EVIDENCE-SCORING-CONTRACT
* R12-ALM-EVIDENCE-TRACE-SEED
* R12-ALM-LLM-OPERATOR-BOUNDARY

## 15. Final Decision

Final decision:
READY — ALM EVIDENCE MAY SUPPORT EXPLANATION, BUT IT IS NOT TRUTH, ADMISSION, OR EXECUTION AUTHORITY
