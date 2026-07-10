# Core Quad Exact-State and Plane Delta Split

## Ownership
* **Owner:** `semantic-core-quad`

## API Split Definition
The original `StateDelta32` conceptually mixed plane changes (entering/leaving truth and falsity) and exact state transitions. To formalize the API:
* **Plane Delta (`PlaneDelta32`):** Describes changes in truth-plane and falsity-plane membership.
* **Exact-State Delta (`ExactStateDelta32`):** Describes entry into and exit from each exact Quad state independently.

## State Mapping
The exact state APIs use the established crate terminology:
* `N` (00) -> `neither`
* `F` (01) -> `strict_false`
* `T` (10) -> `strict_true`
* `S` (11) -> `super`

## Formulas

### Plane Formulas
* `entered_truth = current_truth AND NOT previous_truth`
* `left_truth = previous_truth AND NOT current_truth`
* `entered_falsity = current_false AND NOT previous_false`
* `left_falsity = previous_false AND NOT current_false`

### Exact-State Formulas
For any state `Q` in `{N, F, T, S}`:
* `entered_Q = current_exact_Q AND NOT previous_exact_Q`
* `left_Q = previous_exact_Q AND NOT current_exact_Q`

## Representative Transition Table

| Transition | Exact-state result | Plane result |
| ---------- | ------------------------ | ------------------------- |
| `T → S` | left strict T; entered S | falsity entered |
| `S → T` | left S; entered strict T | falsity left |
| `F → S` | left strict F; entered S | truth entered |
| `S → F` | left S; entered strict F | truth left |
| `N → S` | left N; entered S | truth and falsity entered |
| `S → N` | left S; entered N | truth and falsity left |

## Compatibility Posture
* The public data shape and from_regs computation of `StateDelta32` remain unchanged. Rust documentation was clarified.
* The four legacy truth/falsity plane fields of `StateDelta32` are semantically equivalent to the corresponding fields of `PlaneDelta32`. `StateDelta32` remains a broader mixed compatibility structure and also contains exact-super/conflict and aggregate changed/known events.
* Inventory of `StateDelta32` legacy fields:
  * Plane subset: `entered_true`, `left_true`, `entered_false`, `left_false`
  * Exact S/conflict subset: `entered_super`, `left_super`, `became_conflicted`, `resolved_conflict`
  * Aggregate subset: `changed`, `became_known`, `became_unknown`

## Rationale
All four states (including `N`) must be covered by `ExactStateDelta32` so transitions such as `N -> S` or `S -> N` are modeled fully as entry and exit events across the logic domain, preventing missing event coverage for variables dropping into or out of nullity. This satisfies issue #1409 under compatibility policy #1413.

## Out of Scope
Explicitly deferred:
* 128-lane APIs (`StateDelta128`, `QuadTile128`) are deferred to a subsequent tile-level modification slice.
* No changes made to mask, tile layout, bank semantics, parser, verifier, or VM behavior.
