# R12 ALM Core Posture

## 1. Purpose

Define future ALM core boundaries before implementation. This document does not authorize code.

## 2. Current Factual State

* ALM DNA spec exists.
* ALM remains future-only.
* No ALM core crate exists.
* No association model exists.
* No activation engine exists.
* No backend trait exists.
* No simple backend exists.
* No Turbovec backend exists.
* No LLM operator integration exists.
* Semantic verifier/admission remains separate.

## 3. ALM Core Responsibility

Future ALM core may eventually own:
* concept handles
* association records
* relation kinds
* activation state
* trace records
* conflict/evidence posture
* explanation paths

But only after separate implementation gate.

## 4. ALM Core Must Not Own

ALM core must not own:
* Semantic truth
* verifier admission
* Local Admission Guard
* runtime execution
* source validity
* release readiness
* renderer/UI output
* LLM-generated authority

## 5. Association Model Posture

Future associations may include:
* source concept
* target concept
* relation kind
* strength/weight
* evidence posture
* conflict posture
* trace source

No association model is implemented here.

## 6. Activation Engine Posture

Future activation may include:
* seed concepts
* spreading activation
* ranked hints
* conflict surfacing
* traceable paths

No activation engine is implemented here.

## 7. Quad-State Posture

Future ALM may use N/F/T/S for association/evidence/conflict state.
Unknown is not absence.
Conflict is not ordinary failure.
Denied is not false.
Not admitted is not invalid source.
Packed representation remains future-only.

## 8. Backend Boundary

* ALM core semantics must be backend-agnostic.
* backend-simple is future-only.
* backend-turbovec is future-only.
* backend replacement must not change ALM semantics.
* backend must not own truth/admission.

## 9. LLM Operator Boundary

* LLM may later query ALM.
* LLM may format ALM hints.
* LLM must not become source of truth.
* LLM output remains subordinate to Semantic verifier/admission.

## 10. Admission Guard Table

| Area | Current status | Admission Guard classification | Notes |
|---|---|---|---|
| ALM core posture | Implemented | ADMITTED | posture definition |
| concept handles | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| association records | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| activation engine | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| trace records | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| backend trait | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| simple backend | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| Turbovec backend | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| LLM operator | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| Semantic verifier/admission | Absent | FORBIDDEN | ALM/backend/LLM must not own |
| Workbench/Studio | Absent | FORBIDDEN | out of scope |

## 11. Future Gates

* R12-ALM-CORE-POSTURE-AUDIT
* R12-ALM-ASSOCIATION-MODEL-CONTRACT
* R12-ALM-ACTIVATION-CONTRACT
* R12-ALM-TRACEABILITY-CONTRACT
* R12-ALM-BACKEND-TRAIT-CONTRACT
* R12-ALM-SIMPLE-BACKEND-SEED
* R12-ALM-LLM-OPERATOR-BOUNDARY
* R12-ALM-TURBOVEC-DEPENDENCY-AUDIT

## 12. Final Decision

Final decision:
READY — ALM CORE MAY BE DESIGNED AS A TRACEABLE ASSOCIATIVE LAYER, BUT IMPLEMENTATION REMAINS FUTURE-ONLY UNTIL SEPARATELY GATED
