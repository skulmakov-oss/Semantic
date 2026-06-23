# R12 UI Intent Dispatch Boundary

## 1. Purpose
This boundary document establishes the architectural constraints for securely dispatching a `SemanticIntent` into the Semantic UI runtime execution environment. It defines how inert UI intentions are transitioned into execution capabilities without compromising UI purity.

It does not implement the dispatcher code, capability gates, or execution environment.
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

## 3. Boundary Summary
The `UiActionMapper` from the previous phase produces a `SemanticIntent`, which couples an `InteractionActionBindingId` to a `UiProjectedNodeId`. The **Intent Dispatch** phase bridges the gap between this inert intention and the actual execution of side effects.

The `UiIntentDispatcher` acts as a secure switchboard. It takes the `SemanticIntent` and delegates it to the appropriate capability gate within `prom-ui-runtime` based on the requested action. It **never** executes closures directly from the UI tree. 

## 4. SEMANTIC_UI_DNA Compliance
PASS - The UI tree and projection engine contain no executable logic or closures.
PASS - Side-effects are deferred entirely to capability gates hosted in the runtime environment.
PASS - The dispatcher operates synchronously on intentions but strictly enforces capability checks.

docs/dna inspected: YES
DNA files inspected:
- [SEMANTIC_UI_DNA.md](../../dna/SEMANTIC_UI_DNA.md)

DNA alignment:
- Enforces strict capability admission for all UI actions.
- Preserves the "inert model" property of the UI layer: interactions generate intents, but only the runtime fulfills them.

## 5. Proposed Future Source Shape
A future source PR will implement the dispatcher trait, likely within `prom-ui-runtime` to allow access to capability mechanisms:

```rust
pub trait UiIntentDispatcher {
    /// Attempts to securely dispatch a semantic intent, evaluating required capabilities.
    fn dispatch_intent(&self, intent: SemanticIntent) -> Result<(), IntentDispatchError>;
}
```

The runtime will match the `action_id` within the `SemanticIntent` against predefined actions (e.g., `CloseWindow`, `ExpandCapabilityGate`) and invoke the corresponding secure handler.

## 6. Allowed Semantics
Allowed future semantics, if admitted by a later source PR:
- Implementing the `UiIntentDispatcher` interface.
- Reading the `SemanticIntent` and inspecting its target node and action.
- Routing the intent to pre-registered capability gates in `prom-ui-runtime`.
- Rejecting intents if the target node or user lacks necessary permissions.

## 7. Forbidden Semantics
Forbidden in this boundary and immediate future source gates:
- No execution code or dispatcher logic is written in this PR.
- The UI layer (`prom-ui`) must not define capability gates or assume execution semantics; the dispatcher logic lives in `prom-ui-runtime`.
- The dispatcher must not accept ad-hoc lambda functions or closures from the UI tree. All dispatched actions must resolve to well-known, pre-compiled capability paths.

## 8. Capability Resolution Rules
1. **Binding lookup**: The dispatcher uses the `InteractionActionBindingId` to determine the specific action requested.
2. **Gate lookup**: The requested action maps to a specific capability gate (e.g., `capability_gate_close_window`).
3. **Evaluation**: The gate evaluates context (e.g., node identity, module scope) and admits or denies the execution.
4. **Execution/Effect**: If admitted, the runtime initiates the associated side effect.

## 9. Dependency Boundary Rules
- The `UiIntentDispatcher` interface may be defined in `prom-ui` for abstractness, but its meaningful implementation MUST reside in `prom-ui-runtime`.
- The dispatcher implementation MUST depend on `prom-cap` (or similar capability evaluation primitives) to securely gate execution.

## 10. Future-Gated Work
- `R12-UI-INTENT-DISPATCH-SOURCE-PR`
  - Defines the `UiIntentDispatcher` trait and its implementation within the runtime.
  - Links the dispatcher to the initial set of inert capability gates.

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
PASS — R12 UI Intent Dispatch Boundary defined.

This PR establishes the architectural boundary for securely executing mapped semantic intents via capability gates. It introduces no executable logic or mutations.

## 13. Recommended Next Lane
`R12-UI-INTENT-DISPATCH-SOURCE-PR`

Do not start it in this PR.
