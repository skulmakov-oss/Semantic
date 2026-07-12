# UI-DNA2 implementation roadmap

Status: ACTIVE EXECUTION ROADMAP
Repository baseline: `f28d8d37417301c04058bd13909a9b18b7460c2a`
Live tracker: `#1489`

This document is the durable repository mirror of issue `#1489`.

Issue `#1489` is the live execution ledger.

`ui_dna2_implementation_roadmap.md` is the durable repository mirror.

Neither replaces the other.

This document records landed evidence and remaining gated execution contours.

It does not authorize implementation by itself.

## 1. Current execution baseline

Current main baseline:

```text
f28d8d37417301c04058bd13909a9b18b7460c2a
feat(ui): add projection patch contract foundation (#1497)
```

Current governing rule:

```text
roadmap progress != automatic authorization
contract foundation != runtime integration
lookup != authority
reference != referenced truth
reference slice != production promotion
```

Gate D activation and integration remain closed.

## 2. Governing formula

```text
Meaning
  → Projection intent
  → Static UI IR
  → Binding Graph + Action IR
  → ProjectionBundle
  → Patch-driven shell
  → Renderer
```

Authority remains:

```text
Semantic owns meaning.
Projection owns presentation intent.
UI IR owns structure.
Shell owns rendering.
Renderer owns pixels.
```

## 3. Rebaseline evidence

### Completed prerequisite

- [x] `#1488` — UI DNA v2 / `prom-ui` reconciliation completed.
- [x] `docs/roadmap/post_ui/ui_dna2_prom_ui_reconciliation.md` landed.
- [x] `docs/roadmap/post_ui/ui_dna2_ownership_and_compatibility_freeze.md` landed.
- [x] Ownership decisions D01-D11 approved and preserved.

### Merged implementation and contract slices

| PR | Merge commit | Result |
| --- | --- | --- |
| #1490 | `5c81047edfadc5b1add30b9baad4480fcae981c2` | Projection Source AST, Role Dictionary, contract primitives, Static UI IR foundation, deterministic lowering, legacy adapter, WP2 qualification |
| #1491 | `f5e02b4ac64c8278434c0a2c8ef0f13e2d2b7552` | Binding Graph and Action IR contract foundation, `ActionIntent` transport contracts, deterministic diagnostics, WP3 qualification |
| #1492 | `48b243e050bb34bbf6be4a3d9cf8f7d72a71d443` | `prom-refs`, `ReferenceToken`, six authority-free reference wrappers, `prom-ui -> prom-refs` boundary |
| #1493 | `7b17ee4769d7f1440c8c4252803ca657eb3d5eae` | Public reference value specification |
| #1494 | `eb1876a82969924238d18b8cc9ec5b0eb4d7a5fc` | `prom-refs` public API snapshot and negative authority/conversion guards |
| #1495 | `56df8ab37ae4bea23ced24bced6c28555319f7cf` | Exact capability-reference lookup specification owned by `prom-cap` |
| #1496 | `5bbda87455f0bdc65cd6f6013df8cbad799eae15` | Immutable borrowed exact capability-reference lookup implementation and qualification |
| #1497 | `f28d8d37417301c04058bd13909a9b18b7460c2a` | Crate-internal Projection Patch contract foundation, deterministic validation, exact diagnostic coordinate preservation, WP4A qualification |

## 4. Current landed contract state

The following foundations are active in `main`:

```text
Projection Source AST foundation
Role Dictionary
neutral contract primitives
Static UI IR document foundation
Projection Source → Static UI IR deterministic lowering
bounded legacy UiIr adapter
Binding Graph declarations
Action IR route declarations
ActionIntent transport contracts
prom-refs authority-free value contracts
exact CapabilityRef lookup in prom-cap
Projection Patch contract foundation
```

Current reference and lookup invariants:

```text
ReferenceToken = issuer + namespace + generation + value
reference possession grants no authority
lookup uses the complete key
lookup returns the exact stored borrowed entry
lookup miss != CapabilityDenied
lookup hit != grant
lookup != admission
lookup != dispatch
```

Current patch-foundation invariants:

```text
patch = inert projection contract data
patch validation is deterministic
patch diagnostics preserve exact coordinates
patch order is explicit
patch runtime application is not yet landed
patch != Semantic truth
patch != capability
patch != admission
patch != ActionIntent dispatch
patch != shell mutation
patch != renderer command
patch != runtime queue
```

The following remain absent or unauthorized:

```text
generic Resolver<T>
mutable authority registry
latest-generation fallback
implicit reference-domain resolution
runtime admission integration
runtime dispatch integration
UI wiring
Projection Patch runtime application
ProjectionBundle production loader or activation
shell player promotion
Workbench or Semantic Studio work
production promotion
```

## 5. Rebaselined phase matrix

| Phase | Current status | Landed evidence | Remaining closure work |
| --- | --- | --- | --- |
| UI-DNA2-0 — Reconciliation | **COMPLETE** | #1488, reconciliation document, #1490 | None; preserve as evidence baseline |
| UI-DNA2-1 — Ownership and contract freeze | **COMPLETE** | ownership/compatibility freeze, D01-D11, #1490 | Changes require a separate owner decision |
| UI-DNA2-2 — Projection source front-end | **FOUNDATION LANDED** | programmatic Projection Source AST, source normalization, diagnostics, #1490 | Approved textual grammar/parser, parser goldens, invalid syntax and forbidden-content qualification |
| UI-DNA2-3 — Canonical Static UI IR | **FOUNDATION LANDED** | versioned wrapper, stable structure, semantic child ordering, lowering, qualification bytes, #1490 | Final canonical artifact/serialization policy, compatibility surface and full invalid-artifact matrix |
| UI-DNA2-4 — Binding Graph | **CONTRACT FOUNDATION LANDED** | deterministic declarations, cycle validation, diagnostics, #1491 | Approved Semantic source adapters, revision/epoch observation rules, dirty-propagation integration and Quad preservation evidence |
| UI-DNA2-5 — Action IR integration | **CONTRACT FOUNDATION LANDED** | static routes, `ActionIntent`, invocation context, structural mapper, #1491 | Explicit adapter to existing admission boundary, accepted/denied traces, stale revision, idempotency and capability evidence; Gate D required |
| UI-DNA2-6 — Projection patch model and runtime | **WP4A CONTRACT FOUNDATION LANDED** | `#1497`, crate-internal Projection Patch contract foundation and qualification | WP4B deterministic replay-order model and qualification; actual patch application remains deferred to the separately gated UI-DNA2-9 shell-player contour |
| UI-DNA2-7 — Denial, recovery, task and freshness projection | **NOT STARTED** | specifications only | Bounded contracts and evidence after deterministic patch replay-order qualification |
| UI-DNA2-8 — ProjectionBundle qualification | **NOT STARTED** | fixture and draft-tool evidence only | Parser, validators, verifier, inert loader and activation separation |
| UI-DNA2-9 — Shell player integration | **NOT STARTED** | experimental `ui-shell-kit` evidence only | Separate promotion audit and bounded shell-player implementation |
| UI-DNA2-10 — End-to-end reference slice | **NOT STARTED** | no complete pipeline | One deterministic non-critical reference application |
| UI-DNA2-11 — Production promotion decision | **NOT STARTED** | no promotion claim | Explicit `PROMOTE / PROMOTE WITH LIMITS / KEEP EXPERIMENTAL / REWORK / STOP` decision |

## 6. Gate D0 reference and lookup subtrack

The bounded pre-integration reference subtrack is complete:

```text
D0B — authority-free reference value contracts: COMPLETE
D0B spec sync: COMPLETE
D0C — public API enforcement: COMPLETE
D0D — exact capability-reference lookup specification: COMPLETE
D0E — minimal exact capability-reference lookup implementation: COMPLETE
```

Current active dependency boundaries:

```text
prom-ui -> prom-refs
prom-cap -> prom-refs
```

`prom-refs` remains a neutral zero-dependency representation crate.

`prom-cap` owns capability lookup only.

Gate D remains closed for:

```text
runtime integration
admission integration
dispatch integration
mutable registry
revocation
other reference-domain resolution
UI wiring
```

## 7. Next proposed execution slice

### UI-DNA2-WP4B — deterministic patch replay-order model and qualification

This is the next proposed slice. This document does not authorize it by itself; it requires a separate bounded implementation issue and explicit activation.

Current state:

```text
WP4A contract foundation = LANDED
WP4B deterministic patch replay-order model and qualification = PROPOSED, NOT AUTHORIZED
Gate D = CLOSED
production promotion = NOT AUTHORIZED
```

#### Proposed narrow scope

```text
owner: crates/prom-ui::projection_patch
input: validated ProjectionPatch / ProjectionPatchSet
output: deterministic replay-order model,
ordered replay trace,
and qualification evidence
```

#### Required invariants

```text
declared patch order remains explicit
declared operation order remains explicit
same validated input produces the same replay-order evidence
no patch application
no projected UI state mutation
no shell-local state
no shell mutation
no renderer commands
no runtime queue ownership
no admission or capability authority
no host effects
```

Ownership boundary:

```text
prom-ui::projection_patch owns patch vocabulary and replay order.

prom-ui-runtime::shell_player owns patch application when separately
authorized under the shell-player integration phase.

WP4B does not implement shell application.
```

#### Explicitly forbidden in WP4B

```text
shell mutation runtime
ProjectionBundle loader
bundle activation
renderer/backend integration
networking
admission or capability policy
ActionIntent dispatch
Workbench
Semantic Studio
ui-shell-kit promotion
public API expansion without separate review
```

## 8. Dependency order after rebaseline

```text
COMPLETE:
0 → 1 → WP2 foundation → WP3 foundation → D0B → D0C → D0D → D0E → WP4A

NEXT PROPOSED:
WP4B deterministic patch replay-order model and qualification

REMAINING:
2 parser qualification
3 final artifact qualification
4 source/dirty integration
5 admission integration behind Gate D
7 denial/task/freshness
8 bundle qualification
9 shell player
10 reference slice
11 promotion decision
```

No later phase may be used to bypass an unfinished earlier authority or determinism requirement.

## 9. Governance boundaries

Workbench and Semantic Studio remain governed by `#675`.

`ui-shell-kit` remains governed by `#1310` and remains experimental.

Historical R12/Aldente structures remain evidence only unless separately revived.

ProjectionBundle claim levels remain separate:

```text
fixture reader != production parser
parser != validator
validator != verifier
verifier != loader
loader != activation
activation != production promotion
```

Reference and truth boundaries remain separate:

```text
lookup != authority
reference != referenced truth
reference slice != production promotion
```

## 10. Updated acceptance criteria

- [x] `#1488` reconciliation is complete and referenced.
- [x] Existing and target models are not silently collapsed.
- [x] Canonical ownership and compatibility decisions are frozen.
- [x] Projection Source AST and Static UI IR foundations are landed.
- [x] Binding Graph and Action IR contract foundations are landed.
- [x] Authority-free reference values are separated into `prom-refs`.
- [x] Reference public API and forbidden authority surfaces are guarded.
- [x] Exact capability-reference lookup is specified and implemented in `prom-cap`.
- [x] Lookup remains non-authoritative and separate from admission and dispatch.
- [x] Projection Patch contract foundation is landed in `main`.
- [x] `docs/roadmap/post_ui/ui_dna2_implementation_roadmap.md` exists from this rebaseline.
- [x] Workbench and Semantic Studio remain outside scope.
- [x] `ui-shell-kit` remains experimental.
- [x] Gate D activation/integration remains closed.
- [ ] Projection source textual parser/grammar is qualified.
- [ ] Static UI IR artifact/serialization qualification is complete.
- [ ] Binding Graph source and dirty-propagation integration is qualified.
- [ ] Action IR admission integration is separately approved and qualified.
- [ ] Projection Patch replay-order model and qualification are complete.
- [ ] Patch application is separately qualified in the `prom-ui-runtime::shell_player` contour.
- [ ] Denial/recovery/task/freshness projection is qualified.
- [ ] ProjectionBundle parser/validator/verifier/loader sequence is qualified.
- [ ] Shell player is qualified without authority transfer.
- [ ] End-to-end deterministic reference slice is complete.
- [ ] Production promotion decision is explicit.

## 11. Definition of Done

This umbrella issue may close only when:

1. `docs/roadmap/post_ui/ui_dna2_implementation_roadmap.md` exists and matches the accepted execution state;
2. every remaining phase is either completed or represented by an approved child issue with explicit ownership and gates;
3. the end-to-end reference slice has deterministic and authority-preserving evidence;
4. Phase UI-DNA2-11 records an explicit promotion outcome;
5. no historical, experimental or renderer-local structure has been silently promoted to semantic authority.

Until then:

```text
Issue #1489 = OPEN ACTIVE EXECUTION UMBRELLA
Gate D = CLOSED
production promotion = NOT AUTHORIZED
```
