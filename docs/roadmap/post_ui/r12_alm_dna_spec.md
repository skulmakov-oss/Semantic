# R12 ALM DNA Spec

## 1. Purpose

This document defines ALM doctrine and boundaries before any implementation.

## 2. Current Factual State

* ALM is future-only.
* No ALM implementation exists.
* No ALM backend exists.
* No ALM Turbovec dependency exists.
* Semantic verifier/admission remains separate.
* R12 UI work remains separate from ALM.

## 3. ALM Definition

* traceable associative memory;
* activation engine;
* hint/navigation layer;
* semantic association surface;
* explainable path generator.

## 4. ALM Is Not LLM

* ALM does not perform probabilistic text generation.
* ALM does not replace LLM.
* LLM may later be an operator/interface.
* ALM must remain traceable and bounded.

## 5. ALM Is Not Semantic

* ALM does not own Semantic truth.
* ALM does not decide verifier admission.
* ALM does not execute code.
* ALM does not decide release readiness.
* ALM does not own Local Admission Guard.

## 6. Intended Future Roles

* Semantic coding assistant support;
* concept association;
* rule/gate navigation;
* activation path explanation;
* conflict surfacing;
* related-spec discovery;
* user guidance without writing opaque code.

## 7. Quad-State Association Posture

* ALM associations may carry N/F/T/S posture in the future.
* Unknown is not absence.
* Conflict is not ordinary failure.
* Denied is not false.
* Not admitted is not invalid source.
* Any future packed representation must preserve meaning.

## 8. Traceability Contract

Future ALM outputs should include:
* activated concepts;
* relation path;
* evidence posture;
* conflict posture if present;
* source/gate references where available.

## 9. Backend Boundary

* ALM core semantics must be backend-agnostic.
* backend-simple may exist first in future.
* backend-turbovec may exist later only after dependency gate.
* backend replacement must not change ALM meaning.
* backend must not own truth/admission.

## 10. Turbovec Relationship

* Turbovec is future backend candidate/inspiration.
* This document does not authorize Turbovec dependency.
* Turbovec integration requires separate dependency audit and backend contract.

## 11. LLM Operator Boundary

* LLM may request ALM hints later.
* LLM may format user-facing text later.
* LLM must not become ALM source of truth.
* LLM output must remain subordinate to Semantic verifier/admission.

## 12. Forbidden Behavior

* no implementation;
* no dependency addition;
* no Turbovec import;
* no parser/verifier/runtime/renderer integration;
* no Workbench/Studio;
* no code generation authority;
* no source-of-truth claim.

## 13. Admission Guard Table

| Area | Current status | Admission Guard classification | Notes |
|---|---|---|---|
| ALM doctrine | Implemented | ADMITTED | doctrine definition |
| ALM implementation | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| ALM backend trait | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| backend-simple | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| backend-turbovec | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| Turbovec dependency | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| LLM operator | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| Semantic verifier/admission | Absent | FORBIDDEN | ALM/LLM/backend must not own |
| Workbench/Studio | Absent | FORBIDDEN | out of scope |

## 14. Future Gates

* R12-ALM-DNA-AUDIT
* R12-ALM-CORE-POSTURE
* R12-ALM-ASSOCIATION-MODEL-CONTRACT
* R12-ALM-BACKEND-TRAIT-CONTRACT
* R12-ALM-SIMPLE-BACKEND-SEED
* R12-ALM-TURBOVEC-DEPENDENCY-AUDIT
* R12-ALM-TURBOVEC-BACKEND-CONTRACT
* R12-ALM-LLM-OPERATOR-BOUNDARY

## 15. Final Decision

Final decision:
READY — ALM IS A TRACEABLE ASSOCIATIVE LAYER, NOT LLM, NOT VERIFIER, NOT RUNTIME, AND NOT SOURCE OF TRUTH
