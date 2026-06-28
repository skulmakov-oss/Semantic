# R12 UI Action Mapping Boundary

## 1. Purpose
This boundary document defines the architectural expectations for translating physical interactions into semantic intents within Semantic UI. It dictates how a `RoutedInteraction` (a raw backend event coupled with a hit-tested semantic `NodeId`) is mapped to an actionable semantic `Intent` or `Action`.

It does not implement event mapping logic, the registry of intents, or execute any UI actions.
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

## 3. Boundary Summary
The hit-testing phase produces a `RoutedInteraction`, establishing *where* a physical event occurred logically. The action mapping phase translates *what* that event means semantically for that specific node.

This boundary mandates that mapping logic must remain decoupled from the layout projection engine. The layout engine provides geometry for hit-testing, while a separate mechanism (e.g., an `ActionMap` or `IntentBinder` associated with the tree node) handles the mapping. Action Mapping is a pure translation layer; it emits an `Intent` or `Action` but **does not execute it**.

## 4. SEMANTIC_UI_DNA Compliance
PASS - Action Mapping relies on predefined bindings, not immediate inline execution closures.
PASS - Translating a routed interaction to an intent is a pure data transformation.
PASS - Direct capability execution remains strictly governed by `prom-ui-runtime`.
PASS - The UI projection remains a read-only artifact during mapping.

docs/dna inspected: YES
DNA files inspected:
- [SEMANTIC_UI_DNA.md](../../dna/SEMANTIC_UI_DNA.md)

DNA alignment:
- The UI layer emits structured intents rather than mutating state directly upon interaction.
- The separation of geometry resolution (routing) and semantic interpretation (action mapping) prevents tangled "smart widgets".

## 5. Proposed Future Source Shape
A future source gate will define intent mapping interfaces:

```rust
/// An abstract representation of a mapped user intention.
pub struct SemanticIntent {
    pub target_node: NodeId,
    pub action_id: ActionId,
    // Optional payload data extracted from the RawBackendEvent
}

pub trait UiActionMapper {
    /// Purely translates a RoutedInteraction into an actionable SemanticIntent, if a binding exists.
    fn map_interaction(&self, interaction: RoutedInteraction) -> Option<SemanticIntent>;
}
```

The future PR will implement logic that looks up the `NodeId` in an action binding registry and applies the relevant mapping rule for the given `RawBackendEvent`.

## 6. Allowed Semantics
Allowed future semantics, if admitted by a later source PR:
- Looking up interaction bindings associated with a semantic `NodeId`.
- Translating `RawBackendEvent` variants (e.g., `PointerDown`) into abstract `SemanticIntent`s (e.g., `Activate`, `Focus`).
- Emitting the `SemanticIntent` for downstream evaluation by the runtime capability gates.

## 7. Forbidden Semantics
Forbidden in this boundary and immediate future source gates:
- No action mapping source code is written in this PR.
- The action mapper must **not** execute the resulting `SemanticIntent`. Execution is strictly the domain of the runtime.
- The action mapper must **not** mutate the `UiTree` or projection models directly.
- Mapping logic must not introduce synchronous blocking operations or arbitrary external side effects.

## 8. Mapping Rules
Action Mapping defines meaning, not execution. If a `NodeId` has no bindings for a specific event type, the mapper simply returns `None`. The mapper may extract necessary geometric/event data (e.g., scroll deltas) into the `SemanticIntent` payload, but it does not apply those updates.

## 9. Dependency Boundary Rules
- Action Mapping logic lives strictly within `prom-ui` or `prom-ui-runtime`.
- The native backend (`prom-ui-backend-native`) remains completely unaware of `SemanticIntent`s or `ActionId`s.

## 10. Future-Gated Work
- `R12-UI-ACTION-MAPPING-SOURCE-PR`
  - Defines the `UiActionMapper` trait and intent data structures.
- `R12-UI-INTENT-DISPATCH-BOUNDARY-PR`
  - Defines how the runtime evaluates, gates, and ultimately executes a `SemanticIntent`.

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
PASS — R12 UI Action Mapping Boundary defined.

This PR establishes the architectural boundary for translating interactions to semantic intents.
It introduces no source code, tests, execution logic, or authority.

## 13. Recommended Next Lane
`R12-UI-ACTION-MAPPING-SOURCE-PR`

Do not start it in this PR.
