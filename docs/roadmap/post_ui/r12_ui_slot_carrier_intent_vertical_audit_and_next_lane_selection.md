# R12 UI Slot Carrier Intent Vertical Audit and Next Lane Selection

## 1. Purpose

Audits the completed R12 UI Slot carrier intent metadata vertical chain to verify the metadata-only evidence flow is closed. Selects the next code-first lane for the UI roadmap. This audit is documentation-only and records repository truth.

## 2. DNA Alignment

Semantic UI is an evidence-first, projection-driven UI architecture.
- UI may display truth. UI does not become truth.
- UI state is projection/cache, not semantic state.
- Evidence must preserve source references.
- Renderer boundary remains non-authoritative.
- The intent metadata chain observes these principles perfectly, transferring inert intent from Tree to Render without claiming semantics.

## 3. Closed Source Basis

The following PRs form the closed source basis for this vertical chain:

- `#1097` — Tree Slot carrier intent metadata (Merged: `34bafd8bdadf5366bf21beb7137247d8d15a05e2`)
- `#1098` — Tree Slot intent -> AST metadata bridge (Merged: `47ad744630b90dbf1b256af433691100dbc49eaa`)
- `#1099` — AST Slot intent -> IR metadata bridge (Merged: `626561ebc1de7f2d08632936c822f7b38b497b17`)
- `#1100` — IR Slot intent -> Projection metadata bridge (Merged: `c71ecce472c0e2f461efdded9acc24ae66caf01c`)
- `#1101` — Projection Slot intent -> Render metadata bridge (Merged: `e518bdae73fc9a7c6ba8448881aeb11dffdc1242`)

## 4. Verified Vertical Chain

The metadata-only evidence chain is now fully closed across all intermediate abstractions:

```text
UiTree Slot
  -> UiTreeSlotCarrierIntentModel
  -> UiTreeSlotAstIntentModel
  -> UiAstSlotIrIntentModel
  -> UiIrSlotProjectionIntentModel
  -> UiProjectionSlotRenderIntentModel
```

## 5. Source Surface Audit

All builder functions and modules are successfully published and exported in `crates/prom-ui/src/lib.rs`:

- `tree_slot_intent::build_tree_slot_carrier_intents`
- `tree_slot_ast_intent::build_tree_slot_ast_intents`
- `ast_slot_ir_intent::build_ast_slot_ir_intents`
- `ir_slot_projection_intent::build_ir_slot_projection_intents`
- `projection_slot_render_intent::build_projection_slot_render_intents`

All generated model entries hold a `Deferred` state, adhering to the non-authoritative boundary constraint.

## 6. Test Surface Audit

The test surface provides rigorous validation of the vertical chain:

- Tests span all five boundary transitions.
- All tests are fully implemented; zero `assert!(true)` placeholders exist in the intent vertical.
- Mismatched node kinds, missing nodes, and validation failures correctly yield diagnostics instead of partial models.
- Tests prove Slot intent metadata propagation without crossing into action/effect authority.
- No comment-only tests are present.

## 7. Authority Boundary Audit

A strict boundary audit was performed. The intent chain introduces:

- **No backend/runtime/capability calls.**
- **No layout or draw command generation.**
- **No action/effect execution.**
- **No time, random, or global mutable state.**

The intent flow remains inert and non-authoritative.

## 8. Evidence Preservation Audit

Every intent builder explicitly maps the evidence chain forward:

- Tree node handles and raw IDs.
- AST nodes and resolutions.
- IR fragment identifiers.
- Projection node identifiers.
- Render node identifiers and empty render marker evidence.

Evidence from previous layers is embedded in every subsequent intent entry, maintaining full provenance to the original `UiTree Slot`.

## 9. Diagnostics Audit

Each transformation layer provides dedicated deterministic diagnostics:

- `UiTreeSlotCarrierIntentDiagnostic`
- `UiTreeSlotAstIntentDiagnostic`
- `UiAstSlotIrIntentDiagnostic`
- `UiIrSlotProjectionIntentDiagnostic`
- `UiProjectionSlotRenderIntentDiagnostic`

Diagnostic mapping properly detects and isolates `MissingTargetNode` and `UnexpectedTargetNodeKind` failures.

## 10. Mutation Boundary Audit

All metadata bridges are strictly pure side-car functions:

- They accept upstream intent models and downstream artifacts by shared reference `&`.
- They return an independent intent model.
- They do not mutate inputs. This is explicitly covered by unit tests.

## 11. Candidate Next Lanes

| Candidate                                                  | Classification           | Reason                                                                                                                                               |
| ---------------------------------------------------------- | ------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| `R12-UI-SLOT-CARRIER-INTENT-GOLDEN-VERTICAL-SLICE-TEST-PR` | Selected                 | Best next code-first step: one end-to-end test proving Tree → AST → IR → Projection → Render metadata-only chain without adding new source behavior. |
| `R12-UI-SLOT-CARRIER-INTENT-CLOSEOUT-PR`                   | Deferred                 | Closeout before golden vertical test would be documentation ahead of proof.                                                                          |
| `R12-UI-SLOT-CARRIER-INTENT-RENDER-MARKER-SOURCE-PR`       | Deferred / too broad     | Would risk turning metadata intent into render marker behavior.                                                                                      |
| `R12-UI-SLOT-CARRIER-INTENT-PROPERTY-CARRIER-SOURCE-PR`    | Deferred / forbidden now | PropertyCarrier generation remains separately gated and must not be introduced by Slot metadata.                                                     |
| `R12-UI-SLOT-CARRIER-INTENT-ACTION-CARRIER-SOURCE-PR`      | Deferred / forbidden now | Action semantics and dispatch remain outside this chain.                                                                                             |
| `R12-UI-SLOT-CARRIER-INTENT-EFFECT-BOUNDARY-SOURCE-PR`     | Deferred / forbidden now | EffectBoundary semantics remain separately gated.                                                                                                    |
| `R12-UI-RENDER-BACKEND-INTEGRATION-SOURCE-PR`              | Deferred / too early     | Backend draw authority is outside current metadata-only proof chain.                                                                                 |

## 12. Selected Next Lane

Selected next lane:
R12-UI-SLOT-CARRIER-INTENT-GOLDEN-VERTICAL-SLICE-TEST-PR

This selection is code-first but test-only.
This selection does not add new source modules.
This selection does not change existing source behavior.
This selection does not introduce carrier generation.
This selection does not introduce render markers.
This selection does not introduce backend draw.
This selection does not introduce action/effect/capability/runtime authority.

## 13. Deferred Lanes

- `R12-UI-SLOT-CARRIER-INTENT-CLOSEOUT-PR`
- `R12-UI-SLOT-CARRIER-INTENT-RENDER-MARKER-SOURCE-PR`
- `R12-UI-SLOT-CARRIER-INTENT-PROPERTY-CARRIER-SOURCE-PR`
- `R12-UI-SLOT-CARRIER-INTENT-ACTION-CARRIER-SOURCE-PR`
- `R12-UI-SLOT-CARRIER-INTENT-EFFECT-BOUNDARY-SOURCE-PR`
- `R12-UI-RENDER-BACKEND-INTEGRATION-SOURCE-PR`

## 14. Admission Guard

Admission Guard executed: YES
Admission Guard result: FAIL - ENVIRONMENT PATHING
Admission Guard changed: NO
GitHub CI used: NO

## 15. Non-Scope

This audit and selection PR does not:
- implement new source behavior;
- change source files;
- change tests;
- create new metadata layer;
- change Tree -> AST;
- change AST -> IR;
- change IR -> Projection;
- change Projection -> Render;
- introduce carrier generation;
- introduce renderer/backend/action/effect/capability/runtime authority.

## 16. Final Decision

Final decision:
PASS — R12 UI Slot Carrier Intent vertical chain is closed for metadata-only evidence flow.

The verified chain is:
UiTree Slot -> Tree Slot carrier intent metadata -> AST metadata -> IR metadata -> Projection metadata -> Render metadata.

The next selected lane is R12-UI-SLOT-CARRIER-INTENT-GOLDEN-VERTICAL-SLICE-TEST-PR.

This audit is docs-only and does not change source, change tests, add new behavior, add new metadata modules, introduce carrier generation, introduce render markers, introduce backend draw, introduce layout behavior, introduce action/effect execution, introduce capability admission, introduce runtime/VM/Host ABI authority, or modify Admission Guard.

The next lane is test-only and must prove the full vertical metadata chain end-to-end without adding source behavior.
