# R12 UI Intent State Update Boundary

## 1. Purpose
This boundary document establishes the architectural constraints for executing state mutations or side effects derived from UI intents. It defines the strict requirement that any state update must consume a verified and audited `InteractionAdmittedSemanticAction` token, forming the final step of the intent execution "Staircase".

It introduces no source code, tests, Cargo changes, dependencies, or runtime mutations.

## 2. Closed Basis
| PR | Role | Status |
|----|------|--------|
| TBD | Intent Capability Boundary | MERGED (Assumed) |
| TBD | Intent Capability Source | MERGED (Assumed) |
| TBD | Intent Audit Boundary | MERGED (Assumed) |
| TBD | Intent Audit Source | MERGED (Assumed) |

## 3. Boundary Summary
In the Semantic UI architecture, the UI layer is a pure projection and cannot mutate state directly. The `RuntimeIntentDispatcher` coordinates the processing of a `SemanticIntent` through capability evaluation and audit tracing.

Once an intent is successfully evaluated (Capability Boundary) and durably logged (Audit Boundary), it yields an `InteractionAdmittedSemanticAction`. The State Update Boundary asserts that this unforgeable token is the *exclusive* mechanism by which the runtime may perform business logic, state mutations, or request host effects.

## 4. SEMANTIC_UI_DNA Compliance
PASS - Maintains complete decoupling between intention and state mutation.
PASS - State mutation is physically isolated from UI dispatch and rendering.
PASS - Enforces a secure, token-based execution model where no UI action can bypass capability and audit layers.

docs/dna inspected: YES
DNA files inspected:
- [SEMANTIC_UI_DNA.md](../../dna/SEMANTIC_UI_DNA.md)

DNA alignment:
- "UI is a pure projection, side-effects are runtime capabilities."
- This boundary ensures that side-effects strictly consume authorized capability tokens.

## 5. Proposed Future Source Shape
A future source PR will implement the state execution primitives, likely defining an interface that the `RuntimeIntentDispatcher` invokes after the audit phase:

```rust
pub trait RuntimeStateUpdater {
    /// Consumes an admitted semantic action and executes the corresponding state update or effect.
    fn execute_admitted_action(&self, action: InteractionAdmittedSemanticAction) -> Result<(), StateUpdateError>;
}
```

## 6. Allowed Semantics
Allowed future semantics, if admitted by a later source PR:
- Implementing the `RuntimeStateUpdater` to handle application-specific logic.
- Dispatching events to host ABIs (e.g., window management, system dialogs).
- Transitioning the application state model based on the admitted action.

## 7. Forbidden Semantics
Forbidden in this boundary and immediate future source gates:
- No execution or state mutation logic is written in this PR or the upcoming State Update source PR (Wave 0 remains inert).
- Generating fake `InteractionAdmittedSemanticAction` tokens to bypass capability and audit checks.
- Direct UI-to-State mutations without dispatcher coordination.

## 8. State Update Rules
1. **Token Required**: The state update layer strictly requires an `InteractionAdmittedSemanticAction`.
2. **Execute**: The `RuntimeStateUpdater` processes the token and updates the state.
3. **Completion**: Once execution concludes, the dispatcher returns success. If execution fails (e.g., effect layer unavailable), a `StateUpdateError` is yielded.

## 9. Dependency Boundary Rules
- The state update abstractions must reside in `prom-ui-runtime` or the core application layer, entirely outside of `prom-ui`.
- The update logic MUST NOT depend on layout, rendering, or specific widget states.

## 10. Future-Gated Work
- `R12-UI-INTENT-STATE-UPDATE-SOURCE-PR`
  - Defines the `RuntimeStateUpdater` trait.
  - Wires an inert version of the updater into the `RuntimeIntentDispatcher` to complete the scaffold pipeline.

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
PASS — R12 UI Intent State Update Boundary defined.

This PR establishes the architectural requirement that state mutations must consume authorized capability tokens. It introduces no executable logic or mutations.

## 13. Recommended Next Lane
`R12-UI-INTENT-STATE-UPDATE-SOURCE-PR`

Do not start it in this PR.
