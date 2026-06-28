## Summary

Implements the R12 UI Action Mapping source gate.

This PR adds a pure runtime-side action mapping scaffold that converts routed UI interaction context into semantic intent metadata without executing actions, mutating state, or granting runtime authority.

## Scope

Adds to `prom-ui-runtime`:

- `SemanticIntent`
- `UiActionMapper`
- mapping from `UiIrNodeId` to `InteractionActionName`
- source closeout document for the action mapping gate

## Design decision

`SemanticIntent` stores `InteractionActionName`, not `InteractionAdmittedSemanticActionId`.

Reason:

```text
InteractionActionName
  = requested semantic action name before admission

InteractionAdmittedSemanticActionId
  = assigned only after capability/lifecycle/admission gates
```

This keeps action mapping pure and prevents pre-admission authority leakage.

## Boundary discipline

This PR performs mapping only.

## Explicit non-scope

* no admitted action id allocation
* no capability admission
* no lifecycle gate
* no action execution
* no effect execution
* no state mutation
* no VM call
* no Host ABI call
* no renderer changes
* no backend draw
* no frame presentation
* no Workbench/Studio integration

## Verification

```text
cargo check
cargo fmt
pwsh -File scripts/local_ci.ps1
```

## Next gate

After this PR is merged, the next logical boundary is:

```text
R12-UI-INTENT-ADMISSION-AND-DISPATCH-BOUNDARY-PR
```

That boundary must define how mapped semantic intents are checked before any dispatch or authority is allowed.
