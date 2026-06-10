# R12 ALM Activation Contract

## 1. Purpose

Define future ALM activation boundaries before implementation. This document does not authorize code.

## 2. Current Factual State

* ALM DNA spec exists.
* ALM core posture exists.
* ALM association model contract exists.
* ALM remains future-only.
* No activation engine exists.
* No scoring/ranking engine exists.
* No backend trait exists.
* No simple backend exists.
* No Turbovec backend exists.
* No LLM operator integration exists.
* Semantic verifier/admission remains separate.

## 3. Activation Definition

Future activation as:
* selecting seed concepts;
* walking or propagating across associations;
* collecting candidate related concepts;
* preserving traceable paths;
* surfacing conflict/evidence posture;
* producing hints, not truth.

## 4. Seed Concept Posture

* seed concepts may come from user intent, editor context, selected code, spec name, diagnostic, or future LLM operator request.
* seed concepts are inputs to activation.
* seed concepts are not proof of truth.
* seed selection must remain traceable.

## 5. Propagation Posture

* future propagation may traverse associations.
* propagation may be bounded by depth, score, relation type, scope, or gate.
* propagation must be deterministic under same input/config.
* propagation must not call parser/verifier/runtime/renderer.
* propagation must not execute code.

## 6. Scoring / Ranking Boundary

* scoring may rank candidate hints.
* score is not truth.
* score must not override conflict.
* score must not override verifier/admission.
* scoring semantics require separate gate before code.

## 7. Conflict Surfacing

* conflict must remain visible.
* conflict must not be collapsed into low score.
* conflict must not be silently filtered.
* conflict posture may use Quad-state later.
* activation should be able to return “candidate with conflict” as distinct from failure.

## 8. Evidence Posture

* activated result may carry evidence posture.
* evidence is not admission.
* evidence must be explainable.
* evidence may point to docs/specs/tests/traces.
* evidence must not bypass Semantic verifier/admission.

## 9. Traceability Contract

Future activation output should explain:
* seed concepts;
* activated concepts;
* relation path;
* relation kinds;
* score/rank if present;
* evidence posture;
* conflict posture;
* source/gate references where available.

## 10. Determinism Contract

* same seed concepts + same association graph + same config = same activation output.
* no wall-clock time.
* no randomness.
* no file I/O.
* no network access.
* no host effects.
* no global mutable state.

## 11. Backend Boundary

* activation semantics belong to ALM core/engine contract.
* backend may accelerate propagation/scoring only after separate gate.
* backend replacement must not change activation meaning.
* Turbovec backend remains future-only.
* backend must not own truth/admission.

## 12. LLM Operator Boundary

* LLM may later request activation hints.
* LLM may format activation results.
* LLM must not invent activation truth.
* LLM must not bypass Semantic verifier/admission.

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
| activation contract | Implemented | ADMITTED | contract definition |
| seed concepts | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| propagation | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| scoring/ranking | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| conflict surfacing | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| evidence posture | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| traceability output | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| backend acceleration | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| LLM operator use | Absent | FUTURE_ONLY_NOT_AUTHORIZED | future gate |
| Semantic verifier/admission ownership | Absent | FORBIDDEN | ALM/backend/LLM must not own |
| Workbench/Studio integration | Absent | FORBIDDEN | out of scope |

## 15. Future Gates

* R12-ALM-ACTIVATION-AUDIT
* R12-ALM-ACTIVATION-SEED-CONTRACT
* R12-ALM-ACTIVATION-TRACE-CONTRACT
* R12-ALM-SCORING-CONTRACT
* R12-ALM-CONFLICT-SURFACING-CONTRACT
* R12-ALM-ACTIVATION-ENGINE-SEED
* R12-ALM-BACKEND-TRAIT-CONTRACT

## 16. Final Decision

Final decision:
READY — ALM ACTIVATION MAY BE DESIGNED AS TRACEABLE ASSOCIATION PROPAGATION, BUT IMPLEMENTATION REMAINS FUTURE-ONLY UNTIL SEPARATELY GATED
