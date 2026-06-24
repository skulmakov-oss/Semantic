# R12 UI Intent Admission and Dispatch Boundary

## 1. Problem Statement
The action mapping layer translates a raw user interaction into a `SemanticIntent`. However, a `SemanticIntent` is merely a declaration of a user's wish (e.g., "submit form", "delete item"). If the UI runtime directly executes this intent or dispatches it to the state management layer, it bypasses security, component capabilities, and lifecycle rules. This creates an "authority leak" where the UI acts blindly, potentially triggering actions on disabled, unmounted, or unauthorized components.

## 2. The Boundary Definition
To prevent authority leakage, a strict **Intent Admission and Dispatch Boundary** must be enforced. The boundary isolates the pure declaration of an intent from the authority to execute it.

The critical bridge is defined as follows:
```text
SemanticIntent → [ Admission Gates ] → InteractionAdmittedSemanticActionId → [ Dispatch ]
```

### 2.1 The Admission Gates
Before a `SemanticIntent` is granted authority, it must be evaluated by the admission system within `prom-ui-runtime`. The system checks:
1. **Capability Gate**: Does the targeted semantic node possess the capability required for the requested action? (e.g., is the `Button` capability currently enabled, or is it disabled?)
2. **Lifecycle Gate**: Is the node fully mounted and active? Actions cannot be admitted for nodes that are unmounting, detached, or hidden by policy.

### 2.2 Authority Minting
If the intent passes all admission gates, the runtime mints an `InteractionAdmittedSemanticActionId` (or equivalent validated struct, like `InteractionAdmittedSemanticAction`). 
- **Rule**: `SemanticIntent` has zero authority. `InteractionAdmittedSemanticActionId` possesses execution authority.

### 2.3 Inert Dispatch
Once admitted, the action is dispatched across the UI boundary.
- **Rule**: The UI runtime MUST NOT execute business logic, side effects, or VM mutations itself.
- **Rule**: The UI runtime merely acts as a pipeline, outputting the `InteractionAdmittedSemanticActionId` to an abstract `ActionDispatcher` trait implemented by the Host or State Manager.

## 3. DNA Enforcement
- **No Direct Execution**: The `map_interaction` result must immediately enter the admission gates.
- **Strict Typing**: Dispatch traits must accept `InteractionAdmittedSemanticActionId` (or similar admitted types), NEVER a raw `SemanticIntent`. This enforces admission at compile time.
- **State Integrity**: Mutating application state is strictly the domain of the Host/VM, which receives the dispatched, admitted action.

## 4. Next Steps (Implementation)
The implementation of this boundary (`R12-UI-INTENT-ADMISSION-AND-DISPATCH-SOURCE-PR`) will introduce the `AdmissionFacade` or `AdmissionGuard`, the `ActionDispatcher` trait, and wire the output of `UiActionMapper` through these gates to dispatch.
