# ui-shell-kit Projection Alignment

Status: roadmap alignment
Track: POST-UI / Intent-Driven Projection
Scope type: documentation only
Depends on:
- docs/dna/SEMANTIC_UI_DNA.md
- docs/dna/SEMANTIC_UI_DNA_v2.md
- docs/roadmap/post_ui/intent_driven_projection_roadmap.md
- docs/spec/ui/projection_source_model.md
- docs/spec/ui/ui_ir_schema.md
- docs/spec/ui/action_ir_routing.md
- docs/spec/ui/projection_patch_model.md
- docs/spec/ui/denial_recovery_projection.md
- docs/spec/ui/task_projection_model.md
- docs/spec/ui/multi_client_freshness_model.md
- docs/spec/ui/projection_bundle_delivery.md
Related:
- #1310
- #1328
- #1329
- #1330
- #1331
- #1332
- #1333
- #1334
- #1335
- #1336
- #1337
- #1338

ui-shell-kit is current reference infrastructure for POST-UI experimentation.

It is not production UI.
It is not Semantic authority.
It is not Workbench.
It is not a required app authoring framework.
It is not yet a ProjectionBundle player.

This document does not promote ui-shell-kit, wire it into production, implement ProjectionBundle loading, implement UI IR execution, or change runtime behavior.

## 1. Purpose

This alignment note exists to prevent the bad workflow:

```text
ui-shell-kit exists -> promote it directly into product UI
```

The intended posture is:

```text
ui-shell-kit remains reference infrastructure.
Intent-Driven Projection remains the architecture.
ProjectionBundle player remains future work.
Production UI wiring remains blocked.
```

```text
Reference shell first.
Projection player later.
Product UI only after gates.
```

ui-shell-kit is useful as a reference surface, but it is not a shortcut around the POST-UI doctrine stack.

## 2. Current Role

Current ui-shell-kit role:

- isolated research / experiment surface;
- Semantic-owned UI shell primitives;
- deterministic UI evidence substrate;
- reference shell behavior;
- layout / geometry / painting / input / focus / accessibility experiments;
- calculator scenario evidence;
- visual smoke bridge / manual smoke evidence where used.

The current module surface exposes calculator controller, layout, event, focus, paint, snapshot, theme, and shell helpers.

```text
ui-shell-kit is an evidence substrate, not a product surface.
```

Current evidence-oriented pieces include the calculator controller / scene / focus / snapshot pipeline, which are useful for deterministic projection experiments without implying production UI ownership.

## 3. Non-Authority Rule

ui-shell-kit must not own or redefine:

- Semantic truth;
- verifier admission;
- VM / runtime behavior;
- UI DNA;
- Projection Source Model;
- UI IR schema;
- Action IR routing;
- Binding Graph / patch model;
- denial / recovery policy;
- task engine behavior;
- multi-client freshness policy;
- ProjectionBundle delivery policy;
- Workbench product direction;
- production release readiness.

```text
ui-shell-kit may render projection evidence.
It does not become projection authority.
```

It can help show projection evidence, but it must not become the place where authority lives.

## 4. Alignment with Intent-Driven Projection

The following table maps current ui-shell-kit areas to future projection concepts.

| ui-shell-kit area | Current role | Future projection concept | Boundary |
| --- | --- | --- | --- |
| reference shell | calculator shell / reference UI experiment | `ProjectionBundle` player seed shell boundary | evidence substrate only, not product UI |
| geometry / layout primitives | `calculator_layout`, centering, grid cell placement | UI IR surfaces / nodes / layout-derived projection structure | deterministic layout evidence only |
| painting / rendering primitives | `UiFrame`, drawing helpers, theme-driven frame output | renderer adapter / `RenderPatch` consumption | not backend authority |
| input routing | `UiEvent`, pointer handling, scene-bounded hit testing | `ActionIR` / `ActionIntent` boundary | local shell input only |
| focus / action trace | `FocusRing`, button press / focus change actions | `ActionIntent` observability | not admission authority |
| accessibility experiments | labels, focus intent, readable state presentation | accessibility contract / role interpretation | not a complete accessibility framework |
| snapshot evidence | frame-to-snapshot output | `EvidencePatch` / renderer-independent projection evidence | deterministic evidence only, not production renderer proof |
| hit-test stability | button-center routing and stable local targeting | local event boundary / shell routing | not Semantic mutation |
| motion phase evidence | phase naming / deterministic frame-state evidence | `RenderPatch` / projection-state evidence | not animation backend contract |
| visual smoke bridge | manual operator smoke / screenshot-backed sanity checks | renderer adapter smoke | manual visual sanity check, not production wiring |
| calculator reference scenario | `7 + 3 = 10` repeatable scenario | minimal projection scenario evidence | repeatable reference surface, not app authoring model |

The mapping is intentionally conservative: present-day evidence artifacts inform future projection concepts, but they do not instantiate them yet.

## 5. Mapping to Future ProjectionBundle Player Seed

ui-shell-kit may later seed a `ProjectionBundle` player.

That future role requires separate approved specs and implementation tasks.

Potential future mapping:

- `ProjectionBundle` activation;
- UI IR interpretation;
- Binding Graph / patch application;
- Action IR route display;
- ActionIntent proposal formation;
- `EvidencePanel` / `DenialOutlet` / `RecoveryOutlet` display;
- `TaskPanel` projection;
- freshness / connectivity display;
- renderer adapter boundary.

```text
Future player seed does not mean current promotion.
```

This is a possible later destination, not a current claim.

## 6. Existing Evidence Mapping

Existing evidence and its future concept mapping:

| Existing evidence | Future concept | Meaning | Boundary |
| --- | --- | --- | --- |
| snapshot evidence | `EvidencePatch` / renderer-independent projection evidence | deterministic visual / state record | not production renderer proof |
| focus / action trace | `ActionIR` / `ActionIntent` route observability | input route trace | not admission authority |
| hit-test stability | local event boundary / shell routing | deterministic local input targeting | not Semantic mutation |
| motion phase evidence | `RenderPatch` phase naming / projection state | deterministic phase evidence | not animation backend contract |
| visual smoke bridge | renderer adapter smoke | manual visual sanity check | not production wiring |
| calculator reference scenario | minimal projection scenario evidence | repeatable reference surface | not app authoring model |

Where a local artifact is not yet dedicated as a test, treat the category as a future mapping target rather than as a claim of implemented architecture.

## 7. What Must Not Be Promoted Yet

Do not promote:

- production UI wiring;
- Workbench dependency;
- `ProjectionBundle` loader;
- UI IR interpreter;
- Binding Graph implementation;
- patch stream implementation;
- `ActionIntent` runtime route;
- admission integration;
- task engine integration;
- freshness tracking implementation;
- renderer backend commitment;
- public release widening.

```text
No production UI wiring.
No Workbench dependency.
No ProjectionBundle loader.
No UI IR interpreter.
No Binding Graph implementation.
No patch stream implementation.
No ActionIntent runtime route.
```

ui-shell-kit remains a reference substrate until all later gates are explicitly opened.

## 8. Required Gates Before Promotion

No promotion without a gate.
No gate without evidence.

Required gates before ui-shell-kit can move beyond reference infrastructure:

- `ProjectionBundle` Delivery spec approved;
- ui-shell-kit alignment doc approved;
- separate implementation issue opened;
- allowed paths declared through Harness;
- no production wiring in first implementation slice;
- UI IR interpreter design approved;
- patch application model approved for implementation;
- `ActionIntent` boundary approved for implementation;
- denial / recovery display approved for implementation;
- freshness / control gating approved for implementation;
- tests / golden evidence defined before runtime promotion.

These gates keep the future player seed bounded and reviewable.

## 9. Near-Term Allowed Work

Allowed future tasks may be docs/spec-only or tiny research tasks such as:

- inventory current ui-shell-kit evidence;
- map current tests to spec concepts;
- define a non-executing `ProjectionBundle` fixture format;
- define reference snapshot expectations;
- define shell-player requirements without implementation.

This PR does not prescribe implementation work.

## 10. Forbidden Next Steps

Do not start by:

- implementing a `ProjectionBundle` loader;
- wiring ui-shell-kit into production;
- creating Rust UI IR runtime types;
- replacing `prom-ui`;
- making Workbench depend on ui-shell-kit.

```text
Do not start with the loader.
Do not start with production wiring.
Do not start with runtime types.
Do not start with a replacement.
Do not start with Workbench coupling.
```

## 11. Open Questions

Open questions for the next phase:

- Should the first player seed be fixture-only or runtime-capable?
- What is the minimum UI IR subset for a safe player seed?
- Which evidence tests become golden?
- What renderer profile should the reference shell claim?
- How should visual smoke remain non-authoritative?
- What must remain CLI / non-visual compatible?

These questions are intentionally unresolved here.

## 12. Acceptance Criteria

The alignment note is acceptable when:

- it defines current ui-shell-kit role;
- it states ui-shell-kit is not production UI;
- it states ui-shell-kit is not Semantic authority;
- it maps current evidence categories to future projection concepts;
- it describes possible future `ProjectionBundle` player seed role;
- it keeps promotion blocked behind gates;
- it defines forbidden next steps;
- it preserves Intent-Driven Projection doctrine;
- it does not modify code;
- it does not claim production readiness.
