## Summary

Implements the R12 UI Intent Admission and Dispatch boundary document.

This PR establishes the strict architectural boundary between a declared UI intent (`SemanticIntent`) and an admitted actionable operation (`InteractionAdmittedSemanticActionId`). It ensures that UI intentions are evaluated against system policies (capabilities, lifecycle) before any authority to execute or dispatch is granted.

## Scope

Adds to `docs/roadmap/post_ui/`:
- `r12_ui_intent_admission_and_dispatch_boundary.md`

## Design decision

The bridge from declaration to execution is strictly guarded by Admission Gates. 
`SemanticIntent` holds zero authority and cannot be dispatched directly. It must pass through Capability and Lifecycle checks within the UI runtime to be upgraded to an `InteractionAdmittedSemanticActionId`. Only this ID can be dispatched to the host or state manager to trigger effects. This prevents "authority leakage" where the UI runtime might otherwise bypass business logic rules.

## Boundary discipline

This PR only defines the governance and boundary constraints. It does not contain the Rust source implementation.

## Verification

```text
cargo check
cargo fmt
pwsh -File scripts/local_ci.ps1
```

## Next gate

After this PR is merged, the next logical step is to implement the source logic for this boundary:

```text
R12-UI-INTENT-ADMISSION-AND-DISPATCH-SOURCE-PR
```
