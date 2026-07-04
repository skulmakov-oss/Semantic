# Intent-Driven Projection Closeout

Status: closeout / index
Track: POST-UI / Intent-Driven Projection
Scope type: documentation only
Implementation status: blocked
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
- docs/roadmap/post_ui/ui_shell_kit_projection_alignment.md
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
- #1339

This document closes the POST-UI Intent-Driven Projection documentation contour.

It is an index and boundary record, not an implementation plan.
It does not authorize parser work, compiler work, UI IR runtime types, ProjectionBundle loading, runtime patch pipelines, shell player behavior, production UI wiring, Workbench integration, or ui-shell-kit promotion.

Implementation remains blocked until a separate implementation issue, Harness task, allowed_paths declaration, and review boundary are approved.

## 1. Closeout Summary

The POST-UI Intent-Driven Projection docs/spec stack is closed.

```text
Meaning first.
Intent projection second.
UI IR third.
Rendering last.
```

The stack is ready for controlled future implementation planning.
It is not permission to implement by default.

## 2. Completed Stack

| PR | Document | Role | Status |
| --- | --- | --- | --- |
| #1328 | `docs/dna/SEMANTIC_UI_DNA_v2.md` | doctrine extension | closed |
| #1329 | `docs/dna/SEMANTIC_UI_DNA.md` cross-link | original DNA discoverability | closed |
| #1330 | `docs/roadmap/post_ui/intent_driven_projection_roadmap.md` | roadmap | closed |
| #1331 | `docs/spec/ui/projection_source_model.md` | projection source model | closed |
| #1332 | `docs/spec/ui/ui_ir_schema.md` | UI IR schema | closed |
| #1333 | `docs/spec/ui/action_ir_routing.md` | Action IR / ActionIntent routing | closed |
| #1334 | `docs/spec/ui/projection_patch_model.md` | Binding Graph / patch streams | closed |
| #1335 | `docs/spec/ui/denial_recovery_projection.md` | denial / partial batch / recovery | closed |
| #1336 | `docs/spec/ui/task_projection_model.md` | long-running tasks | closed |
| #1337 | `docs/spec/ui/multi_client_freshness_model.md` | multi-client / freshness | closed |
| #1338 | `docs/spec/ui/projection_bundle_delivery.md` | ProjectionBundle delivery | closed |
| #1339 | `docs/roadmap/post_ui/ui_shell_kit_projection_alignment.md` | ui-shell-kit alignment | closed |

## 3. Reading Order for Future Codex Tasks

Required reading order:

1. `docs/dna/SEMANTIC_UI_DNA.md`
2. `docs/dna/SEMANTIC_UI_DNA_v2.md`
3. `docs/roadmap/post_ui/intent_driven_projection_roadmap.md`
4. `docs/spec/ui/projection_source_model.md`
5. `docs/spec/ui/ui_ir_schema.md`
6. `docs/spec/ui/action_ir_routing.md`
7. `docs/spec/ui/projection_patch_model.md`
8. `docs/spec/ui/denial_recovery_projection.md`
9. `docs/spec/ui/task_projection_model.md`
10. `docs/spec/ui/multi_client_freshness_model.md`
11. `docs/spec/ui/projection_bundle_delivery.md`
12. `docs/roadmap/post_ui/projection_bundle_fixture_inventory.md`
13. `docs/spec/ui/projection_bundle_basis.md`
14. `docs/spec/ui/projection_bundle_reader_parser_entry_gate.md`
15. `docs/roadmap/post_ui/ui_shell_kit_projection_alignment.md`
16. `docs/roadmap/post_ui/intent_driven_projection_closeout.md`

```text
Read the doctrine before the mechanism.
Read the mechanism before implementation.
Read the closeout before opening new work.
```

The closeout comes last so future tasks do not treat the docs stack as an implementation license.

## 4. Authority Map

Ownership remains split:

- Semantic owns meaning.
- Projection owns presentation intent.
- UI IR owns structure.
- Action IR owns affordance routing.
- Binding Graph owns deterministic dependency mapping.
- Patch streams own projection updates.
- Shell owns rendering behavior.
- Renderer owns pixels.
- Verifier / admission owns semantic admission decisions.
- Capability / audit authority owns capability checks, host-effect permission, critical action authorization, and audit evidence boundaries.
- Runtime owns execution / scheduling only where explicitly specified.

```text
No layer may silently absorb authority from another layer.
```

This map is the standing boundary contract for the POST-UI stack.

## 5. What Is Now Specified

The following concepts are now specified in docs:

- Projection Source Model;
- `.proj.sm` as preferred v0 working name, not parser commitment;
- UI IR schema;
- Action IR and ActionIntent routing;
- `ActionIntentBatch`;
- Binding Graph;
- patch streams;
- denial taxonomy;
- recovery taxonomy;
- long-running task projection;
- `TaskStatePatch`;
- viewer-relative projection;
- freshness states;
- ProjectionBundle delivery;
- pinned critical UI;
- verified dynamic UI;
- ui-shell-kit alignment.

This is the full POST-UI Intent-Driven Projection doctrine / spec contour.

## 6. What Remains Explicitly Not Implemented

No parser.
No compiler.
No `.proj.sm` implementation.
No UI IR Rust runtime types.
No Action IR runtime implementation.
No Binding Graph implementation.
No patch stream implementation.
No denial / recovery runtime handling.
No task engine integration.
No multi-client sync.
No freshness tracking.
No ProjectionBundle loader.
No bundle verification implementation.
No shell player.
No runtime patch pipeline.
No production UI wiring.
No Workbench dependency.
No ui-shell-kit promotion.
No verifier / VM / SemCode changes.
No capability / audit authority widening.

```text
Specified does not mean implemented.
Closed docs do not open runtime authority.
```

The docs stack records contracts and boundaries; it does not itself create runtime behavior.

## 7. Implementation Remains Blocked

Implementation is still blocked.

Implementation may only start after:

- separate issue;
- separate Harness task;
- explicit allowed paths;
- explicit forbidden paths;
- one narrow implementation target;
- validation plan;
- rollback boundary;
- no production wiring in first slice;
- no global refactor;
- no authority widening.

```text
No implementation without a task.
No task without allowed paths.
No authority change without review.
```

This closeout is a navigation aid for future PRs and Codex tasks.
It does not replace the individual specs.

## 8. Allowed Future Work Categories

Allowed future categories may include:

- docs-only fixture inventory;
- non-executing `ProjectionBundle` fixture format;
- UI IR minimal subset design;
- `ActionIntent` envelope fixture examples;
- Binding Graph fixture examples;
- patch stream golden examples;
- denial / recovery golden examples;
- task projection golden examples;
- ui-shell-kit evidence inventory;
- CI / golden test planning.

```text
Future work starts with fixtures and evidence before runtime.
```

Future work must stay evidence-led rather than jumping straight into runtime wiring.

## 9. Forbidden Default Next Steps

Do not start with:

- production UI;
- Workbench;
- a `ProjectionBundle` loader;
- a runtime patch pipeline;
- UI IR runtime types;
- parser / compiler work;
- promoting `ui-shell-kit`.

These are the wrong first moves after reading the closeout.

## 10. First Safe Next Slice Recommendation

Recommended first future slice:

```text
define a non-executing ProjectionBundle fixture inventory and golden evidence plan.
```

No loader.
No runtime.
No shell player.
No production wiring.

`ProjectionBundle Basis v0` is the claim boundary for the current fixture evidence contour.
It records that the current achieved level is Level 3 only and that reader/parser, loader, runtime, and production UI behavior are not claimed.

`ProjectionBundle Reader/Parser Entry Gate v0` is the pre-Level-4 gate for any future reader/parser-adjacent work.
It does not claim Level 4; it defines what must exist before Level 4 may be attempted.

Do not create that fixture in this PR.

## 11. Ledger Note

This closeout is intended to support `#1310` ledger readability.

It gives future PRs and Codex tasks a single reading map, a closed stack list, and a hard implementation boundary.

## 12. Acceptance Criteria

The closeout document is acceptable when:

- it lists the full completed stack;
- it defines reading order;
- it restates authority map;
- it summarizes specified concepts;
- it lists non-implemented items;
- it states implementation remains blocked;
- it defines allowed future work categories;
- it defines forbidden default next steps;
- it recommends a fixture / evidence-first future slice;
- it does not modify code;
- it does not modify existing specs;
- it does not claim production readiness.
