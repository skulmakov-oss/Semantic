# R12 ALM Association Model Contract

## 1. Purpose

Define future ALM association model boundaries before implementation. This document does not authorize code.

## 2. Current Factual State

* ALM DNA spec exists.
* ALM core posture exists.
* ALM remains future-only.
* No ALM core crate exists.
* No concept model exists.
* No association record model exists.
* No relation kind enum exists.
* No evidence/conflict posture model exists.
* No activation engine exists.
* No backend trait exists.
* Semantic verifier/admission remains separate.

## 3. Concept Identity Posture

* future `ConceptId` may identify local ALM concepts.
* `ConceptId` must not be Semantic truth.
* `ConceptId` must not be verifier admission.
* `ConceptId` must not be repository identity unless separately gated.
* concept identity must remain traceable to source/gate/spec when available.

## 4. Association Record Posture

Future association records may contain:
* source concept
* target concept
* relation kind
* strength/weight
* evidence posture
* conflict posture
* trace source
* optional scope/context

No association record is implemented here.

## 5. Relation Kind Posture

Future relation kinds may include:
* similar_to
* depends_on
* supports
* contradicts
* explains
* blocks
* requires_gate
* related_spec
* derived_from

Relation vocabulary is not final and must be separately gated before code.

## 6. Evidence Posture

* evidence posture may use N/F/T/S later.
* evidence is not truth by itself.
* evidence must remain explainable.
* evidence must not bypass verifier/admission.
* evidence may point to source docs, gates, tests, or traces.

## 7. Conflict Posture

* conflict is not ordinary failure.
* conflict must remain visible.
* conflicting associations must not be silently collapsed.
* conflict posture may use Quad-state later.
* conflict surfacing is future-only.

## 8. Strength / Weight Boundary

* weight may help ranking.
* weight must not become truth.
* weight must not override admission.
* weight must not hide conflict.
* numeric scoring is future-only and not authorized here.

## 9. Traceability Contract

Future association output should be able to explain:
* why this association exists;
* where it came from;
* what relation path activated it;
* what evidence posture it carries;
* whether conflict exists;
* which gate/spec/source supports it.

## 10. Backend Independence

* association semantics belong to ALM core.
* simple backend and Turbovec backend must not change association meaning.
* backend may accelerate lookup/scoring only after separate gate.
* backend must not own truth/admission.

## 11. LLM Operator Boundary

* LLM may later ask ALM for associations.
* LLM may format explanation.
* LLM must not invent association truth.
* LLM must not bypass Semantic verifier/admission.

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
| association model contract | Implemented | ADMITTED | contract definition |
| ConceptId | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| association records | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| relation kinds | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| evidence posture | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| conflict posture | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| strength/weight | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| trace source | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| backend acceleration | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| LLM operator use | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| Semantic verifier/admission ownership | Absent | FORBIDDEN | ALM/backend/LLM must not own |
| Workbench/Studio integration | Absent | FORBIDDEN | out of scope |

## 14. Future Gates

* R12-ALM-ASSOCIATION-MODEL-AUDIT
* R12-ALM-CONCEPT-ID-CONTRACT
* R12-ALM-RELATION-KIND-CONTRACT
* R12-ALM-EVIDENCE-POSTURE-CONTRACT
* R12-ALM-CONFLICT-POSTURE-CONTRACT
* R12-ALM-TRACEABILITY-CONTRACT
* R12-ALM-ASSOCIATION-MODEL-SEED

## 15. Final Decision

Final decision:
READY — ALM ASSOCIATIONS MAY BE DESIGNED AS TRACEABLE RELATION RECORDS, BUT IMPLEMENTATION REMAINS FUTURE-ONLY UNTIL SEPARATELY GATED
