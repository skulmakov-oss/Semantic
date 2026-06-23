# R12 UI Intent Capability Boundary

## 1. Purpose
This boundary document establishes the architectural constraints for UI intent capability gating. It defines how a `SemanticIntent` requests and receives authority before any execution (state mutation or side-effect) is permitted. 

It does not implement the capability gate logic, authorization providers, or execution handlers.
It introduces no source code, tests, Cargo changes, dependencies, or runtime mutations.

## 2. Closed Basis
| PR | Role | Status |
|----|------|--------|
| #1136 | Frame Presentation Boundary | MERGED |
| #1137 | Frame Presentation Source | MERGED |
| #1138 | Static Visible Demo | MERGED |
| #1139 | Raw Event Capture Boundary | MERGED |
| #1140 | Raw Event Capture Source | MERGED |
| #1141 | Interaction Routing Boundary | MERGED |
| TBD | Interaction Routing Source | MERGED (Assumed) |
| TBD | Action Mapping Boundary | MERGED (Assumed) |
| TBD | Action Mapping Source | MERGED (Assumed) |
| TBD | Intent Dispatch Boundary | MERGED (Assumed) |
| TBD | Intent Dispatch Source | MERGED (Assumed) |

## 3. Boundary Summary
The `RuntimeIntentDispatcher` currently denies all execution by default (`CapabilityDenied`). To progress towards functional interactions, we must establish a capability boundary. This boundary dictates that an intent must be matched against a registered `UiCapabilityKind` associated with the caller or the specific node.

Capability evaluation acts as an admission phase. It intercepts the dispatched intent, checks authority against a capability registry or policy, and only upon success does it forward the intent to an execution phase (e.g., state update or host effect).

## 4. SEMANTIC_UI_DNA Compliance
PASS - Maintains complete decoupling between intention and execution authority.
PASS - State mutation remains strictly forbidden at the dispatch and capability evaluation layers.
PASS - Fosters an explicit, auditable security model for UI interactions.

docs/dna inspected: YES
DNA files inspected:
- [SEMANTIC_UI_DNA.md](../../dna/SEMANTIC_UI_DNA.md)

DNA alignment:
- "UI is a pure projection, side-effects are runtime capabilities."
- This boundary formalizes the "runtime capabilities" portion, ensuring intents cannot self-authorize.

## 5. Proposed Future Source Shape
A future source PR will implement the capability evaluation primitives, likely defining an interface that the `RuntimeIntentDispatcher` can consult:

```rust
pub trait UiCapabilityEvaluator {
    /// Evaluates if the given intent is authorized to execute its requested action.
    fn evaluate_intent(&self, intent: &SemanticIntent) -> Result<AdmittedAction, IntentCapabilityError>;
}
```

The `AdmittedAction` serves as an unforgeable token proving that authority was granted, which is then consumed by the execution layer (State Update or Effect).

## 6. Allowed Semantics
Allowed future semantics, if admitted by a later source PR:
- Implementing capability evaluators and registries.
- Mapping `InteractionActionBindingId` to specific `UiCapabilityKind`s.
- Returning unforgeable admission tokens (`AdmittedAction`) upon successful capability checks.
- Generating audit traces for both admitted and denied capability requests.

## 7. Forbidden Semantics
Forbidden in this boundary and immediate future source gates:
- No execution or state mutation logic is written in this PR or the upcoming capability source PR.
- Capability evaluators MUST NOT execute the action themselves; they only grant authority (admission).
- Bypassing the capability evaluator to directly mutate state or invoke VM/Host ABI is strictly prohibited.

## 8. Capability Evaluation Rules
1. **Intercept**: The dispatcher receives the `SemanticIntent`.
2. **Consult**: The dispatcher passes the intent to the `UiCapabilityEvaluator`.
3. **Verify**: The evaluator checks if the required capability for the intent's action is held by the triggering context.
4. **Admit/Deny**: If verified, an `AdmittedAction` is returned. If not, an `IntentCapabilityError` (leading to a denial trace) is returned.
5. **Halt**: After admission, the process halts here until the subsequent State Update or Effect boundaries are established.

## 9. Dependency Boundary Rules
- The capability concepts (e.g., `UiCapabilityKind`) and the evaluator interface should reside in a core capability crate (`prom-cap` or `prom-ui-runtime`).
- The evaluation logic MUST NOT depend on layout or rendering specifics.

## 10. Future-Gated Work
- `R12-UI-INTENT-CAPABILITY-SOURCE-PR`
  - Defines the capability evaluation trait and an initial restrictive policy.
  - Plugs the evaluator into the `RuntimeIntentDispatcher`, transitioning it from a hardcoded "deny all" to a policy-based "deny by default".

## 11. Repository Scope
- source files changed: NO
- test files changed: NO
- docs changed: YES
- `Cargo.toml` changed: NO
- `Cargo.lock` changed: NO
- `docs/dna` changed: NO
- Admission Guard changed: NO
- GitHub CI used: NO

## 12. Final Decision
PASS — R12 UI Intent Capability Boundary defined.

This PR establishes the architectural boundary for evaluating authority of UI intents before any state mutation occurs. It introduces no executable logic or mutations.

## 13. Recommended Next Lane
`R12-UI-INTENT-CAPABILITY-SOURCE-PR`

Do not start it in this PR.
