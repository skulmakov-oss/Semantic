# R12 ALM Turbovec Backend Posture

## 1. Purpose

This document records future posture for using Turbovec as possible ALM acceleration backend. It does not authorize implementation or dependency addition.

## 2. Current Factual State

* ALM is a planned/future associative layer.
* No ALM Turbovec backend exists.
* No Turbovec dependency exists.
* No ALM runtime integration exists.
* Semantic UI indexing/vectorization remains future-only.
* Semantic admission/truth remains separate.

## 3. ALM Is Not LLM

* ALM should not be treated as probabilistic text generation.
* ALM should be a traceable associative memory / activation engine.
* LLM may be an operator/interface later, not source of truth.
* Semantic verifier/admission remains separate.

## 4. Why Turbovec Is Relevant

* similarity search
* activation spreading
* dense vector scoring
* packed state scanning
* conflict scan
* cache-friendly memory layout
* lightweight local inference support

## 5. Quad-State Fit

* ALM associations may carry N/F/T/S posture.
* Future packed representation may use T-plane and F-plane.
* known = T | F
* conflict = T & F
* unknown = !(T | F)
* packed form must preserve semantics.

## 6. Backend Boundary

* ALM core must own concepts, associations, traces, activation semantics.
* Turbovec may only be a backend candidate.
* Turbovec must not own truth.
* Turbovec must not own admission.
* Turbovec must not own Semantic memory semantics.
* backend replacement must not change ALM semantics.

## 7. Possible Future Architecture

* `alm-core`
* `alm-engine`
* `alm-backend-simple`
* `alm-backend-turbovec`
* `alm-cli`

All of these are future-only and not authorized here.

## 8. Dependency Gate

Future Turbovec integration requires:
* owner approval
* dependency audit
* license review
* no_std/alloc compatibility review if applicable
* benchmark plan
* deterministic test plan
* fallback simple backend
* clear boundary between ALM semantics and backend acceleration

## 9. Forbidden Behavior

* no dependency addition
* no Cargo changes
* no ALM code
* no backend code
* no Turbovec import
* no runtime integration
* no verifier/parser/VM integration
* no Workbench/Studio
* no UI pipeline change

## 10. Admission Guard Table

| Area | Current status | Admission Guard classification | Notes |
|---|---|---|---|
| ALM concept | Absent | FUTURE_ONLY_NOT_AUTHORIZED | requires spec |
| Turbovec as inspiration | Present | ADMITTED | architectural direction |
| Turbovec dependency | Absent | FUTURE_ONLY_NOT_AUTHORIZED | requires dependency audit |
| ALM backend trait | Absent | FUTURE_ONLY_NOT_AUTHORIZED | requires contract |
| ALM simple backend | Absent | FUTURE_ONLY_NOT_AUTHORIZED | requires seed |
| ALM Turbovec backend | Absent | FUTURE_ONLY_NOT_AUTHORIZED | requires backend contract |
| Quad-state packed ALM associations | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future feature |
| Semantic verifier/admission | Absent | FORBIDDEN | Turbovec must not own this |
| Workbench/Studio integration | Absent | FORBIDDEN | out of scope |

## 11. Relationship To Current R12 UI Work

* This document does not modify R12 UI.
* This document does not authorize UI indexing implementation.
* This document keeps ALM/Turbovec separate from prom-ui.
* Future UI indexing and future ALM Turbovec backend are separate gates.

## 12. Future Gates

* R12-ALM-DNA-SPEC
* R12-ALM-CORE-POSTURE
* R12-ALM-BACKEND-TRAIT-CONTRACT
* R12-ALM-SIMPLE-BACKEND-SEED
* R12-ALM-TURBOVEC-DEPENDENCY-AUDIT
* R12-ALM-TURBOVEC-BACKEND-CONTRACT

## 13. Final Decision

Final decision:
READY — TURBOVEC IS A FUTURE ALM BACKEND CANDIDATE, NOT A CURRENT DEPENDENCY OR SOURCE OF TRUTH
