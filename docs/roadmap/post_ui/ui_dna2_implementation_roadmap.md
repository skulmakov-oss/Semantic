# UI-DNA2 implementation roadmap

Status: DIRECTIONAL RESEARCH ROADMAP
Repository evidence baseline: `d395e5708ddca696d296003e9182fde1f43f651c`
Live tracker: `#1489`

## Document role

This document is a directional research roadmap for UI DNA v2.

It is not a canonical source of repository truth, architectural authority,
implementation authorization, release posture, or production readiness.

It records the current research direction, working phase decomposition,
landed evidence, unresolved hypotheses, and likely sequencing.

Because UI DNA v2 is partly an exploratory engineering programme, this
roadmap may be revised, reordered, split, collapsed, or rewritten when
implementation and qualification evidence changes the model.

Canonical ownership and forbidden-boundary decisions remain in the accepted
ownership and compatibility documents.

Actual implementation evidence is established by landed code, tests,
qualification artifacts, and merged pull requests.

The active issue ledger records current coordination state, but roadmap
position alone never authorizes a task.

```text
roadmap direction != canonical truth
roadmap phase != automatic authorization
roadmap ordering != immutable architecture
implementation evidence != production promotion
landed code != stable promise
qualification != shell integration
```

Issue #1489 is the active coordination ledger.

This roadmap is the durable directional research map.

The issue and roadmap may be synchronized for clarity, but neither overrides
accepted ownership boundaries, landed code, qualification evidence, or
explicit promotion decisions.

## 1. WP4B observation baseline

WP4B accepted starting baseline:

```text
5f9549690505968743f111851a7ae3b087433e9e
docs(ui): add UI DNA v2 implementation roadmap (#1498)
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
| #1498 | `5f9549690505968743f111851a7ae3b087433e9e` | Directional UI DNA v2 implementation roadmap, WP4B ownership-boundary correction, Gate D closure preserved |
| #1500 | `fa6215bd55fa5a67cfce411e35f8eac3b6e1df60` | Projection Source Grammar v0 normative contract and Projection Source model |
| #1507 | `0ac0d980228dae5fb907b900aeb7c334a6916fd3` | Crate-private Grammar v0 parser/scanner implementation; all 12 `PSP_*` diagnostics, exact UTF-8 spans, source-size preflight and fail-fast precedence qualified |
| #1508 | `184103ec27cd99ff8a91b9e7255e9179cbd49606` | Crate-private pure in-memory Projection Source parser-to-compiler frontend; semantic-validation boundary and Static UI IR composition qualified |
| #1513 | `57a9332b9de6bca1a545eada8df0db3b7f64d2ea` | Projection Source qualification lifecycle reconciled; Binding Graph observation/dirty v0 normative contract frozen; public APIs, Semantic adapters and runtime integration remain unauthorized |
| #1514 | `5546272351a3913fcb76d62ada7dd44b2102a68e` | Crate-private pure in-memory Binding Graph observation/dirty engine v0 and executable qualification landed; exact Quad preservation, deterministic reverse-dependency propagation and bounded stage precedence qualified; public API, Semantic adapters and runtime integration remained absent from that slice |
| #1515 | `64bbaa6397b01ed220dbfec0c9df8564ec49525f` | Crate-private caller-supplied Semantic observation adapter v0 and executable adapter-to-dirty-engine composition qualification landed; live Semantic reads, public APIs, runtime consumption and Projection Patch integration remain absent |
| #1516 | `b6c151f0525d9052435814df5f0e6a1789ee8ae2` | Denial/recovery/freshness v0 contract, crate-private projection implementation, inert ProjectionPatch construction and executable qualification landed; Task Projection remained separately bounded |
| #1517 | `a056f220dcfc73c1d4731b315138f8b148cfe1bd` | Task Projection v0 contract, crate-private pure in-memory implementation, canonical representation and inert ProjectionPatch construction landed; reviewed head `f4df2990036f3206c54a939c7d19bee11b61fbf5`; exact-head CI `29583403595` and post-merge CI `29587270649` succeeded |
| #1518 | `547e00c98c24079f2a01f02ead9088c333cbb8da` | Task Projection P2 corrective qualification landed; reviewed head `bf258b89969636244edb4912b6969c219445c40c`; exact-head CI `29595540320` and post-merge CI `29598533948` succeeded; exact aggregate text bounds and lossless `TaskRecordRef(u64)` projection qualified |
| #1519 | `d395e5708ddca696d296003e9182fde1f43f651c` | UI-DNA2-8A ProjectionBundle v0 logical contract freeze; documentation-only; deterministic and non-overlapping structural, cross-artifact, compatibility, trust-verification and inert-loading boundaries; final serialization unresolved; parser implementation blocked; general Level 4 not claimed; reviewed head `b9409868781ddb564ab60d4bcbe7f097c85c96f0`; exact-head push CI `29632061124`, exact-head PR CI `29632062257`, post-merge CI `29632178545` — all 8/8 |

## 4. Current landed contract state

The following foundations are active in `main`:

```text
Projection Source AST foundation
Projection Source Grammar v0 contract
crate-private Projection Source parser/scanner qualification
crate-private pure in-memory parser-to-compiler frontend qualification
Role Dictionary
neutral contract primitives
Static UI IR document foundation
Projection Source → Static UI IR deterministic lowering
bounded legacy UiIr adapter
Binding Graph declarations
Binding Graph observation/dirty v0 normative contract
crate-private Binding Graph observation/dirty engine v0 qualification
crate-private Binding Graph caller-supplied Semantic observation adapter v0 qualification
Action IR route declarations
ActionIntent transport contracts
prom-refs authority-free value contracts
exact CapabilityRef lookup in prom-cap
Projection Patch contract foundation
denial/recovery/freshness v0 projection qualification
Task Projection v0 canonical representation
crate-private pure in-memory Task Projection v0 qualification
Task Projection inert ProjectionPatch construction
Task Projection P2 corrective qualification through #1518
ProjectionBundle v0 logical contract freeze
deterministic structural, cross-artifact, compatibility and trust ownership
caller-supplied ProjectionBundle resource-bound categories
inert bundle loading separated from activation
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
ProjectionBundle parser/validator/verifier implementation
ProjectionBundle inert-loader implementation
ProjectionBundle activation
Shell Player implementation
ProjectionPatch runtime application
Workbench or Semantic Studio work
production promotion
```

## 5. Current research checkpoint matrix

This matrix is a working decomposition, not an immutable architecture.
Future evidence may split, combine, reorder, or retire phases.

| Phase | Current status | Landed evidence | Remaining closure work |
| --- | --- | --- | --- |
| UI-DNA2-0 — Reconciliation | **COMPLETE** | #1488, reconciliation document, #1490 | None; preserve as evidence baseline |
| UI-DNA2-1 — Ownership and contract freeze | **COMPLETE** | ownership/compatibility freeze, D01-D11, #1490 | Changes require a separate owner decision |
| UI-DNA2-2 — Projection source front-end | **GRAMMAR V0 PARSER AND FRONTEND QUALIFICATION LANDED; PUBLIC API AND LOADING NOT AUTHORIZED** | programmatic Projection Source AST, source normalization and diagnostics in #1490; normative Grammar v0 contract in #1500; crate-private parser/scanner with all 12 `PSP_*` diagnostics, exact UTF-8 spans, source-size preflight and fail-fast precedence qualified in #1507; crate-private pure in-memory semantic-validation and Static UI IR composition qualified in #1508 | Public parser/frontend APIs and filesystem/runtime loaders remain absent; runtime loading, Gate D and production promotion remain unauthorized |
| UI-DNA2-3 — Canonical Static UI IR | **CRATE-PRIVATE ARTIFACT V1 QUALIFICATION LANDED; LOADING AND ACTIVATION NOT AUTHORIZED** | versioned wrapper, stable structure, semantic child ordering, lowering and qualification bytes in #1490; normative Artifact V1 contract in #1510; crate-private pure in-memory verifier, two committed golden vectors, all 22 normative invalid-artifact rows, deterministic rejection mutations, exhaustive minimal-vector truncation and exact canonical re-encoding equality in #1511 (`ddf28436c1c4ab0a961c007e89c757deae87dcfe`); exact-head and post-merge CI succeeded | Public codec API and filesystem/runtime loaders remain absent; runtime loading, Gate D and production promotion remain unauthorized; no next implementation slice is authorized |
| UI-DNA2-4 — Binding Graph | **FOUNDATION, OBSERVATION/DIRTY V0 ENGINE AND CALLER-SUPPLIED SEMANTIC OBSERVATION ADAPTER QUALIFICATION LANDED** | deterministic declarations, cycle validation and diagnostics in #1491; normative observation/dirty contract frozen in #1513; crate-private pure in-memory dirty engine and executable qualification landed in #1514; caller-supplied evidence adapter contract, crate-private implementation and executable composition qualification landed in #1515 | live Semantic reads and subscriptions remain absent; runtime consumption and Projection Patch application remain unauthorized behind Gate D; UI-DNA2-4 is not complete |
| UI-DNA2-5 — Action IR integration | **CONTRACT FOUNDATION LANDED** | static routes, `ActionIntent`, invocation context, structural mapper, #1491 | Explicit adapter to existing admission boundary, accepted/denied traces, stale revision, idempotency and capability evidence; Gate D required |
| UI-DNA2-6 — Projection patch model and runtime | **WP4A FOUNDATION + WP4B REPLAY-ORDER CHECKPOINT COMPLETE** | #1497 — Projection Patch contract foundation<br>#1499 — deterministic replay-order model and qualification | actual patch application remains deferred to the separately gated UI-DNA2-9 prom-ui-runtime::shell_player contour |
| UI-DNA2-7 — Denial, recovery, task and freshness projection | **UI-DNA2-7A DENIAL/RECOVERY/FRESHNESS V0, UI-DNA2-7B TASK PROJECTION V0 AND P2 CORRECTIVE QUALIFICATION THROUGH #1518 LANDED** | denial/recovery/freshness v0 contract, crate-private implementation, inert ProjectionPatch construction and qualification landed in #1516; Task Projection v0 contract, crate-private pure in-memory implementation, canonical representation and inert patch construction landed in #1517; exact aggregate text bounds and lossless `TaskRecordRef(u64)` projection qualified in #1518 | Task Projection application, admission execution, runtime integration, Gate D and production promotion remain unauthorized |
| UI-DNA2-8 — ProjectionBundle qualification | **UI-DNA2-8A LOGICAL CONTRACT FREEZE LANDED; GENERAL LEVEL 4, IMPLEMENTATION, INERT LOADING AND ACTIVATION NOT AUTHORIZED** | ProjectionBundle v0 logical identity, deterministic stage ownership, validation, resource, diagnostic and authority boundaries landed through #1519 | Resolve final serialization and other blocking decisions; separate authorization for UI-DNA2-8B parser/validator/verifier implementation and qualification; separate authorization for UI-DNA2-8C pure in-memory inert-loader qualification; separate activation decision |
| UI-DNA2-9 — Shell player integration | **UI-DNA2-9A1 SHELL PLAYER OWNERSHIP AND STAGE BOUNDARY CARRIED BY THIS DOCUMENTATION-ONLY CHANGE; IMPLEMENTATION, PATCH APPLICATION, ACTIVATION, DRAW-SEAM RUNTIME, BACKEND INTEGRATION AND PRODUCTION PROMOTION NOT AUTHORIZED** | canonical O26/O27 ownership and dependency boundaries; experimental `ui-shell-kit` evidence inventory; Shell Player v0 boundary contract | Resolve the implementation-blocking decisions named by `shell_player_boundary_v0.md`; separate authorization for detailed local-state and transition contracts; separate authorization for ProjectionPatch runtime-application semantics; complete UI-DNA2-8B and UI-DNA2-8C prerequisites where required; separate authorization for bounded crate-private Shell Player implementation; separate Gate D and production-promotion decisions |
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

## 7. Latest bounded research checkpoint

### UI-DNA2-9A1 Shell Player v0 boundary contract

AUTHORIZED BOUNDED DOCUMENTATION-ONLY CHANGE —
OWNERSHIP AND STAGE BOUNDARY FREEZE CARRIED HERE;
IMPLEMENTATION, PATCH APPLICATION, ACTIVATION, DRAW-SEAM RUNTIME,
BACKEND INTEGRATION AND PROMOTION NOT AUTHORIZED

Current state:

```text
Binding Graph Semantic observation adapter v0 = LANDED IN #1515
denial/recovery/freshness v0 contract = LANDED IN #1516
Task Projection v0 = LANDED IN #1517
Task Projection P2 corrective qualification = LANDED IN #1518
ProjectionBundle v0 logical contract = LANDED IN #1519
General Level 4 = NOT CLAIMED
FINAL SERIALIZATION = UNRESOLVED
UI-DNA2-8B = NOT AUTHORIZED
UI-DNA2-8C = NOT AUTHORIZED
ProjectionBundle parser/validator/verifier = NOT IMPLEMENTED
ProjectionBundle inert loader = NOT IMPLEMENTED
ProjectionBundle activation = NOT AUTHORIZED
bundle activation = NOT AUTHORIZED
Shell Player v0 ownership and stage boundary = CARRIED BY THIS DOCUMENTATION-ONLY CHANGE
Shell Player implementation = NOT AUTHORIZED
ProjectionPatch application = NOT AUTHORIZED
renderer integration = NOT AUTHORIZED
backend integration = NOT AUTHORIZED
runtime integration = NOT AUTHORIZED
Gate D = CLOSED
production promotion = NOT AUTHORIZED
NEXT AUTHORIZED IMPLEMENTATION SLICE = NONE
```

Required invariants:

```text
Shell Player owns local projection playback.
Shell Player does not own Semantic truth.
local shell state != Semantic truth
patch application != Semantic mutation
hit-test result != action authorization
ActionIntent candidate != admitted action
draw material != pixels
shell transition != backend event loop
```

The current authorization is consumed by the Shell Player v0 ownership and
stage boundary freeze. UI-DNA2-8B, UI-DNA2-8C, Shell Player implementation,
activation, patch application, admission/runtime integration, or any later
contour requires a new explicit bounded authorization and a new harness task.

## 8. Dependency order after rebaseline

EVIDENCE LANDED OR CARRIED BY THE CURRENT CHANGE:

```text
0 → 1 → WP2 foundation → WP3 foundation
→ D0B → D0C → D0D → D0E
→ WP4A → WP4B replay-order checkpoint
→ UI-DNA2-3A Artifact V1 contract
→ UI-DNA2-3B crate-private Artifact V1 qualification
→ UI-DNA2-2 Grammar v0 parser/frontend qualification reconciliation
→ Binding Graph observation/dirty v0 normative contract freeze
→ Binding Graph observation/dirty v0 crate-private engine implementation and qualification (#1514)
→ Binding Graph caller-supplied Semantic observation adapter v0 implementation and qualification (#1515)
→ denial/recovery/freshness v0 projection (#1516)
→ task projection v0 (#1517)
→ Task Projection P2 corrective qualification (#1518)
→ ProjectionBundle v0 logical contract freeze (#1519)
→ Shell Player ownership and stage boundary freeze (UI-DNA2-9A1)
```

CURRENTLY UNAUTHORIZED FUTURE CONTOURS:

```text
ProjectionBundle parser/validator/verifier implementation
ProjectionBundle inert-loader implementation
ProjectionBundle activation
Shell Player implementation
ProjectionPatch runtime application
bundle activation
end-to-end reference slice
production-promotion decision
```

This order is directional rather than immutable.

Research evidence may justify:
- reordering;
- splitting a phase;
- merging phases;
- adding a prerequisite;
- removing an obsolete contour.

No roadmap movement may bypass an accepted ownership or authority boundary.

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
- [x] Projection source textual parser/grammar and pure in-memory parser-to-compiler frontend are qualified through #1507 and #1508; both remain crate-private, with no public API or filesystem/runtime loading authorization.
- [x] Static UI IR Artifact V1 qualification is landed at the crate-private pure in-memory boundary through #1511; public codec API and filesystem/runtime loaders remain absent, runtime loading remains unauthorized, Gate D remains closed, and production promotion remains unauthorized.
- [x] Binding Graph observation comparison and dirty-propagation v0 contract is normatively frozen independently of implementation authorization.
- [x] Binding Graph observation/dirty engine implementation and qualification are landed through #1514 at the crate-private pure in-memory boundary.
- [x] The crate-private caller-supplied Semantic observation adapter for Binding Graph observations is landed and qualified through #1515; live Semantic reads remain absent and unauthorized.
- [ ] Action IR admission integration is separately approved and qualified.
- [x] Projection Patch replay-order model and qualification are complete in the bounded WP4B contour.
- [ ] Patch application is separately qualified in the `prom-ui-runtime::shell_player` contour.
- [x] Denial/recovery/freshness v0 projection and inert ProjectionPatch construction are specified, implemented and qualified through #1516.
- [x] Task Projection v0 is separately specified, implemented and qualified at the crate-private pure in-memory boundary through #1518; application, live evidence acquisition, admission execution and runtime integration remain unauthorized.
- [x] ProjectionBundle v0 logical identity, stage separation, validation, resource, diagnostic and authority boundaries are frozen by the bounded documentation-only UI-DNA2-8A change.
- [ ] ProjectionBundle parser/validator/verifier implementation is separately authorized and qualified.
- [ ] ProjectionBundle pure in-memory inert loader is separately authorized and qualified.
- [ ] ProjectionBundle activation is separately authorized.
- [x] Shell Player v0 ownership, inputs, outputs, stage relationships and non-authority boundaries are frozen by UI-DNA2-9A1 without implementation authorization.
- [ ] Shell player is qualified without authority transfer.
- [ ] End-to-end deterministic reference slice is complete.
- [ ] Production promotion decision is explicit.

## 11. Definition of Done

This roadmap does not independently declare UI DNA v2 complete.

Completion or promotion decisions require explicit evidence and governance
outside roadmap progression, including:

- landed implementation evidence;
- qualification results;
- accepted ownership compliance;
- integration evidence where applicable;
- explicit release or promotion decisions.

This umbrella issue may close only when:

1. `docs/roadmap/post_ui/ui_dna2_implementation_roadmap.md` exists and matches the accepted execution state;
2. every remaining phase is either completed or represented by an approved child issue with explicit ownership and gates;
3. the end-to-end reference slice has deterministic and authority-preserving evidence;
4. Phase UI-DNA2-11 records an explicit promotion outcome;
5. no historical, experimental or renderer-local structure has been silently promoted to semantic authority.

Until then:

```text
UI-DNA2-8B = NOT AUTHORIZED
UI-DNA2-8C = NOT AUTHORIZED
Shell Player implementation = NOT AUTHORIZED
ProjectionPatch runtime application = NOT AUTHORIZED
bundle activation = NOT AUTHORIZED
Gate D = CLOSED
production promotion = NOT AUTHORIZED
NEXT AUTHORIZED IMPLEMENTATION SLICE = NONE
```
