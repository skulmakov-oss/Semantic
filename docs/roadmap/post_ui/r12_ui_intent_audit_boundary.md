# R12 UI Intent Audit Boundary

## 1. Purpose
This boundary document establishes the architectural constraints for auditing UI intent admission and denial. It ensures that the result of any capability evaluation is durably logged into an audit trace *before* the intent is allowed to cause any runtime mutation or side effect.

It introduces no source code, tests, Cargo changes, dependencies, or runtime mutations.

## 2. Closed Basis
| PR | Role | Status |
|----|------|--------|
| TBD | Intent Dispatch Boundary | MERGED (Assumed) |
| TBD | Intent Dispatch Source | MERGED (Assumed) |
| TBD | Intent Capability Boundary | MERGED (Assumed) |
| TBD | Intent Capability Source | MERGED (Assumed) |

## 3. Boundary Summary
The `RuntimeIntentDispatcher` currently evaluates intents using a `UiCapabilityEvaluator`. The result of this evaluation—whether it grants an `InteractionAdmittedSemanticAction` or rejects with an `IntentCapabilityError`—must be logged.

The Audit Boundary acts as the mandatory observational phase. It intercepts the admission result, formats a standardized trace report, and sends it to the central audit sink. Only upon successful audit serialization does the flow proceed to the next execution boundary (State Update).

## 4. SEMANTIC_UI_DNA Compliance
PASS - Maintains complete decoupling between intention, capability, and state.
PASS - State mutation remains strictly forbidden at the audit trace layer.
PASS - Fosters an explicit, auditable security model for UI interactions, ensuring zero "silent" mutations or denials.

docs/dna inspected: YES
DNA files inspected:
- [SEMANTIC_UI_DNA.md](../../dna/SEMANTIC_UI_DNA.md)

DNA alignment:
- "UI is a pure projection, side-effects are runtime capabilities."
- This boundary ensures that the utilization (or denial) of runtime capabilities is fully traceable.

## 5. Proposed Future Source Shape
A future source PR will implement the audit tracing primitives, likely defining an interface that the `RuntimeIntentDispatcher` can use:

```rust
pub trait UiAuditLogger {
    /// Records the result of a capability evaluation.
    fn record_admission_result(&self, result: &InteractionActionAdmissionResult) -> Result<(), AuditError>;
}
```

## 6. Allowed Semantics
Allowed future semantics, if admitted by a later source PR:
- Implementing the `UiAuditLogger` and formatting traces.
- Translating `InteractionAdmittedSemanticAction` or `IntentCapabilityError` into human/machine-readable trace reports.
- Emitting traces to external or internal diagnostic sinks.

## 7. Forbidden Semantics
Forbidden in this boundary and immediate future source gates:
- No execution or state mutation logic is written in this PR or the upcoming audit source PR.
- Skipping the audit layer for "trusted" intents is strictly forbidden.
- The `UiAuditLogger` MUST NOT alter the `InteractionAdmittedSemanticAction` token.

## 8. Capability & Audit Rules
1. **Capability Checked**: The `UiCapabilityEvaluator` produces an `InteractionActionAdmissionResult`.
2. **Audit Handled**: The dispatcher routes the result to the `UiAuditLogger`.
3. **Trace Emitted**: The logger records the admission or denial.
4. **Halt or Proceed**: If admitted, the process halts here until the subsequent State Update boundaries are established. If denied, the request safely terminates.

## 9. Dependency Boundary Rules
- The audit tracing traits and records should reside in `prom-ui-runtime` or a dedicated `prom-audit` crate.
- The tracing logic MUST NOT depend on layout, rendering specifics, or the physical origin of the event.

## 10. Future-Gated Work
- `R12-UI-INTENT-AUDIT-SOURCE-PR`
  - Defines the `UiAuditLogger` trait.
  - Wires the logger into the `RuntimeIntentDispatcher`.

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
PASS — R12 UI Intent Audit Boundary defined.

This PR establishes the architectural requirement for logging capability evaluations before execution. It introduces no executable logic or mutations.

## 13. Recommended Next Lane
`R12-UI-INTENT-AUDIT-SOURCE-PR`

Do not start it in this PR.
