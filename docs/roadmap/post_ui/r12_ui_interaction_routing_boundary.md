# R12 UI Interaction Routing Boundary

## 1. Purpose
This boundary document defines the future architectural expectations for interaction routing in Semantic UI. It dictates how raw physical events (like pointer movement and clicks) captured in the backend map to specific semantic targets (Node IDs) via hit-testing.

It does not implement hit-testing, action execution, event mapping, or UI layout solvers.
It introduces no source code, tests, Cargo changes, dependencies, or runtime mutations.

## 2. Closed Basis
| PR | Role | Status |
|----|------|--------|
| #1120 | First Visible Surface Boundary | MERGED |
| #1122 | Backend Native Baseline Ledger | MERGED |
| #1125 | Windowing Boundary | MERGED |
| #1133 | Winit Run Loop Integration Boundary | MERGED |
| #1136 | Frame Presentation Boundary | MERGED |
| #1137 | Frame Presentation Source | MERGED |
| #1138 | Static Visible Demo | MERGED |
| #1139 | Raw Event Capture Boundary | MERGED |
| #1140 | Raw Event Capture Source | MERGED |

## 3. Boundary Summary
With physical events now securely captured into the `RawBackendEvent` representation, the runtime must route these spatial/physical events to logical nodes.

This boundary requires that hit-testing uses the cached `UiLayoutRectModel` (or equivalent layout projection evidence) to translate a physical coordinate `(X, Y)` into a semantic `NodeId`. The interaction routing boundary explicitly prohibits immediately executing actions or mutating semantic state upon finding a target. The output of this boundary is simply the routing metadata: "this event targeted this NodeId."

## 4. SEMANTIC_UI_DNA Compliance
PASS - Hit-testing relies on projection evidence (layout cache), not semantic truth.
PASS - Routing identifies targets but does not act on them.
PASS - Direct capability execution remains forbidden.
PASS - Unknown/Conflict geometry returns explicit non-matches instead of forcing a target.

docs/dna inspected: YES
DNA files inspected:
- [SEMANTIC_UI_DNA.md](../../dna/SEMANTIC_UI_DNA.md)

DNA alignment:
- The UI remains a read-only projection. Hit-testing reads the layout cache inertly.
- Locating a node by physical position is distinct from expressing user intent.

## 5. Proposed Future Source Shape
A future source gate will define hit-testing interfaces against the layout model:

```rust
pub trait UiHitTester {
    fn find_target_at(&self, layout: &UiLayoutRectModel, x: f64, y: f64) -> Option<NodeId>;
}

pub struct RoutedInteraction {
    pub target: NodeId,
    pub event: RawBackendEvent,
}
```

The future PR will implement logic that walks the geometry projection to find the intersecting node.

## 6. Allowed Semantics
Allowed future semantics, if admitted by a later source PR:
- Walking a `UiLayoutRectModel` or similar projection artifact to test coordinate intersection.
- Returning an inert `NodeId` representing the found element.
- Emitting a `RoutedInteraction` struct that bundles the raw event with its destination.

## 7. Forbidden Semantics
Forbidden in this boundary and immediate future source gates:
- No hit-testing source code is written in this PR.
- Hit-testing must not mutate the `UiLayoutRectModel` or the `UiTree`.
- Hit-testing must not return or trigger a semantic `Action`.
- The router must not bypass `prom-ui-runtime` capability gates.

## 8. Hit-Testing Rules
Interaction routing observes geometry; it does not dictate it. If an element is physically occluded but semantically present, hit-testing must respect the visible Z-order defined in the projection. Z-order collision semantics must fail gracefully without panicking the UI thread.

## 9. Dependency Boundary Rules
- Hit-testing logic lives strictly within `prom-ui` or `prom-ui-runtime`.
- The physical `winit` backend must not perform semantic hit-testing. It only provides the raw spatial coordinates.

## 10. Future-Gated Work
- `R12-UI-INTERACTION-ROUTING-SOURCE-PR`
  - Defines the `UiHitTester` implementation against layout projections.
- `R12-UI-ACTION-MAPPING-BOUNDARY-PR`
  - Defines how a `RoutedInteraction` (a raw event + a NodeId) turns into a semantic intent/Action.

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
PASS — R12 UI Interaction Routing Boundary defined.

This PR defines the future boundary for semantic hit-testing and event routing.
It introduces no source code, tests, semantic mappings, or execution authority.

## 13. Recommended Next Lane
`R12-UI-INTERACTION-ROUTING-SOURCE-PR`

Do not start it in this PR.
