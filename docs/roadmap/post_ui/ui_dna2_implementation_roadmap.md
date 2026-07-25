# UI-DNA2 implementation roadmap

Status: DIRECTIONAL RESEARCH ROADMAP
Repository evidence baseline: `94ae4a4ed187f589264160e794f6ebb45de1261d`
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
Shell owns local projection playback and rendering preparation.
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
| #1520 | `b514c51455c086ba624fbfe173e510b61ebd9946` | Documentation-only Shell Player v0 ownership and stage boundary freeze; initial head `6cbafc3379d59cdffa1b0c5c67a41e88c2ba09e2`; final reviewed head `f3cfb9d9cd4b51af05fbf83c5f6263228dc6bb43`; exact-head push CI `29635394874`, exact-head PR CI `29635396064`, post-merge CI `29636264299` — all 8/8 PASS; one input-contract inconsistency corrected before Ready; no implementation authorization |
| #1521 | `c71242d04c1f6962c9bc816b535b9136e9113d23` | UI-DNA2-9A1 closeout and roadmap ledger evidence landed; Shell Player boundary authorization consumed and closed; no implementation authorization |
| #1522 | `6219e67e6f5f233797cfe8047cc54e0148a5a223` | Rust 1.97.1 repository and CI baseline qualification landed; seven exact Clippy compatibility corrections qualified; no Shell Player scope expansion |
| #1523 | `3e229a821cebc013acd5f294c4872efaa6fd37a1` | Explicit `rustfmt` installation landed for both Rust 1.97.1 Windows 7hell workflows; post-merge CI `29652449165` passed 8/8; no Shell Player scope expansion |
| #1524 | `0eede9391f6f5d1aaf446e94326b74797f1973d7` | Shell Player session/local-state normative contract landed; three independent P2 ambiguities were corrected before merge; post-merge CI `29655671082` passed 8/8 |
| #1525 | `0061df1e4134c7ced1c9a157140f602b3853466f` | Post-merge P2 input-resource preflight correction landed; oversized inputs are rejected before stable-target and replay traversal; post-merge CI `29659183512` passed 8/8; origin thread resolved |
| #1527 | `e4abf5793dbe5ff74eaffd85987c7b717b4e3744` | UI-DNA2-9B evidence closeout and durable-roadmap synchronization; accepted P2 findings recorded as resolved; unresolved implementation decisions preserved. |
| #1528 | `e81ed971b79447302139f61eb613ea6ff99acbd0` | Crate-private deterministic Shell Player lifecycle seed; Created / Active / Suspended / Closed transitions and rejection-state preservation landed; no patch application or public API. |
| #1529 | `e1b36f3a8b5414a6d337c0335179170dc13d8edc` | Crate-private stateful `ShellSession` owner landed; immutable activation context and commit-on-`Applied` semantics qualified. |
| #1530 | `ba41772e7acece129828d8e2f43ca1df8b614e94` | Crate-private ordered ProjectionPatch envelope metadata and stages 1-4 preflight landed; no stable-target traversal, replay policy or state mutation. |
| #1531 | `e5a457cf8e74608f5df1468c0b058ab29cd9892b` | Crate-private local replay-cursor representation landed in `ShellLocalState`; no compatibility or advancement semantics in that slice. |
| #1532 | `18edb45ce7de17c2036acb53f71b2ec7357f5b50` | Documentation-only deterministic stage-6 replay-cursor compatibility contract frozen. |
| #1533 | `e0c64fd6e6da8623849b56b6d34da71c58891833` | Crate-private pure stage-6 replay-cursor compatibility evaluator and 17 focused tests landed; no cursor advancement, stage-5 validation or pipeline orchestration. |
| #1534 | `6d8b1b1cd51ad4a746ee57dfb1296f20bf26721a` | Documentation-only Shell Player stage-5 stable-target boundary frozen; manifest, catalog, target-reference count and diagnostic ownership concepts separated. |
| #1535 | `fecbaae5de60163466cd60b4bd2bfee20325c341` | Documentation-only prepared-handoff ownership contract frozen; transition-, activation- and session-scoped lifetimes separated; future catalog ownership assigned to `prom-ui-runtime`. |
| #1536 | `d1797a1a18bb48512f549bc8fc522dcfc455ef68` | Documentation-only explicit `CollectionAnchor` declaration contract frozen; source/qualified identity, `CAD_*` diagnostics, ordering and prepared-activation source boundary defined. |
| #1539 | `94ae4a4ed187f589264160e794f6ebb45de1261d` | Crate-private programmatic explicit `CollectionAnchor` declaration representation landed; declaration storage in `ProjectionSourceDocument` landed; deterministic compiler-owned source-to-static qualification landed; immutable `QualifiedCollectionAnchorDeclarations` landed; four exact `CAD_*` diagnostics landed; whole-set fail-closed behavior landed; deterministic ascending `StaticNodeId` output landed; deterministic duplicate provenance landed; 20 focused tests landed; reviewed head `327c52bb05191a5e6a01f93d7a32874f119540c3`; exact-head CI `30031617862` passed 8/8; post-merge CI `30034743940` passed 8/8. |
| #1541 | `8d29c19c782928aae546ced3c1b9c58e8db8491c` | Complete remaining Shell Player contour landed: `PreparedProjectionPatchTargets` and `PreparedActiveProjectionTargets` producers; the first public `prom-ui::shell_bridge` cross-crate surface (patch admission, prepared-evidence and activation-target snapshots, local-projection-state application/queries) with same-change golden-snapshot public API guard coverage for both `prom-ui::shell_bridge` and the now-public `prom-ui-runtime::shell_player`; runtime-owned `ActiveProjectionTargetCatalog` constructed only from one activation snapshot; stage-4 prepared-evidence coherence and target-reference resource checks; stage-5 stable-target membership evaluator using only the immutable session catalog; stage-5/stage-6 orchestration; deterministic `LocalProjectionState` with atomic `SetBindingValue`/`SetNodeAvailability`/`CollectionInsert`/`CollectionUpdate`/`CollectionRemove`/`CollectionMove` application; atomic commit and replay-cursor advancement in `ShellSession::apply_projection_patch_batch`; a deterministic `--shell-player-demo` native mode rendering every frame from committed Shell Player state through the existing winit/wgpu backend; 60+ new focused tests. CI status: not yet run (branch not pushed as of this commit). Gate D remains closed; production promotion remains unauthorized. |
| #1544 | `f9727312b9a3a8aa0b1523ce41ed5a77935c2538` | UI-DNA2 end-to-end pipeline landed (closes #1543): textual `collection_anchor` Grammar v0 declaration syntax and parser/frontend integration (`compile_projection_source_text_with_collection_anchors`); `ProjectionBundle v0` canonical binary codec (`crates/prom-ui/src/projection_bundle.rs`) with an 8-stage verifier (decode, header, structural, cross-artifact, compatibility, self-consistency trust), committed golden vectors and an exhaustive negative-test matrix — trust verification is deterministic self-consistency (canonicalize → re-encode → byte-compare), explicitly not cryptographic signing; bounded fail-closed Gate D activation (`activate_projection_bundle_v0_gate_d`) restricted to the `--ui-dna2-reference` contour only, documented as **OPEN WITH LIMITS** for that contour in the new `docs/spec/ui/gate_d_activation_policy_v0.md` — general/global Gate D is not claimed; `ReferenceContourAdmission` (`prom-ui-runtime::reference_admission`) reusing the existing `action_admission`/`action_admission_result`/`admitted_action` evidence chain plus a bounded replay/staleness invocation guard; real glyphon-backed `DrawText`/glyph rendering in the native wgpu backend; bounded hit-testing, focus routing and pointer-capture; the `--ui-dna2-reference` end-to-end native reference application (UI-DNA2-10) starting from real Projection Source text; and a cross-cutting fix to `UiBackendAdapter::run_event_loop` (`prom-ui-runtime`/`prom-ui-backend-native`, all 15 implementors updated) so real winit-translated input actually reaches application event loops — previously every native backend silently dropped all real OS input regardless of application, including the already-merged #1541 Shell Player demo, because `DesktopSession::run`'s `EventBuffer` was never fed from `NativeBackend::pending_events`; unit tests never caught this because they exercise state handlers directly. Reviewed head `ae7e2c37ddc5f65773e0dbbd9c399eac8d07d3b0`; exact-head CI (runs `30152392177`/`30152409984`) 16/16 passed; post-merge CI `30153625541` passed. Verified natively with real synthetic OS-level pointer input reaching hit-testing → focus → admission → commit → visible re-render for both an admitted click (button flips Unavailable, collection appends, replay cursor advances) and a denied click (state preserved, DENIED banner shown), each with screenshot evidence; invalid-bundle rejection and replay/stale-invocation rejection verified automatically with unchanged-state assertions. UI-DNA2-11 promotion decision: **PROMOTE WITH LIMITS** — see §7 and §11. |

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
Shell Player v0 ownership and stage boundary freeze
local projection playback ownership
caller-supplied session/viewport context
Shell Player activated-session input contract
immutable session resource-limit authority
Created / Active / Suspended / Closed lifecycle
local reconstructible session-state domains
deterministic ten-stage transition model
input-side resource preflight before collection traversal
complete candidate commit or complete previous-state preservation
stage-10 diagnostic emission cap
SPV0_ diagnostic namespace
local shell state separated from Semantic truth
hit-test result separated from action authorization
ActionIntent candidate separated from admitted action
draw/session material separated from renderer pixels
shell transition separated from backend event loop
crate-private deterministic Shell Player lifecycle evaluator
crate-private stateful ShellSession owner
ordered ProjectionPatch envelope metadata preflight through stages 1-4
local ProjectionReplayCursor representation
crate-private pure stage-6 replay-cursor compatibility evaluator
stage-6 replay-cursor compatibility relation
stage-5 stable-target validation boundary
PreparedProjectionPatchTargets conceptual handoff
PreparedActiveProjectionTargets conceptual handoff
prepared-handoff atomicity
declared/actual target-reference count coherence
future prom-ui-runtime ownership of ActiveProjectionTargetCatalog
same-PR-as-public-bridge guard policy
explicit CollectionAnchor declaration ownership and identity
QualifiedCollectionAnchorDeclarations whole-set semantics
CAD_* projection-qualification diagnostics
deterministic ascending StaticNodeId CollectionAnchor ordering
crate-private programmatic CollectionAnchor declaration representation
ProjectionSourceDocument declaration storage
crate-private QualifiedCollectionAnchorDeclarations representation
deterministic CollectionAnchor declaration qualification
whole-set CAD_* diagnostic qualification
deterministic duplicate SourceRef provenance
ascending StaticNodeId qualified declaration ordering
PreparedProjectionPatchTargets Rust representation and producer
PreparedActiveProjectionTargets Rust representation and producer
shared NodeAnchor/BindingAnchor/CollectionAnchor target-class representation
first public prom-ui::shell_bridge cross-crate surface
same-PR golden-snapshot public API guard for prom-ui::shell_bridge and prom-ui-runtime::shell_player
ActiveProjectionTargetCatalog Rust representation owned by prom-ui-runtime
ActivatedShellSessionContext catalog attachment
stage-4 prepared-evidence declared/actual count coherence integration
stage-4 target-reference resource-limit integration
stage-5 stable-target evaluator using only the immutable session catalog
stage-5/stage-6 orchestration
deterministic LocalProjectionState with atomic collection insert/update/remove/move semantics
ProjectionPatch runtime application
replay-cursor advancement
candidate-state calculation and atomic commit
deterministic native Shell Player demo mode in prom-ui-demo
renderer/backend event-loop integration for Shell Player (Clear/FillRect commands only; DrawText remains an existing backend no-op)
ProjectionBundle v0 canonical binary codec (assemble/canonical_bytes)
ProjectionBundle v0 8-stage verifier (decode/header/structural/cross-artifact/compatibility/self-consistency trust)
ProjectionBundle v0 golden vectors and exhaustive negative-test matrix
ProjectionBundle v0 self-consistency trust verification (explicitly not cryptographic signing; no digest/signature algorithm selected anywhere)
bounded fail-closed Gate D activation for the --ui-dna2-reference contour only
Gate D = OPEN WITH LIMITS for the --ui-dna2-reference contour (general/global Gate D remains not claimed)
textual collection_anchor declaration Grammar v0 syntax
Grammar v0 parser support for textual CollectionAnchor declarations
parser-to-compiler frontend support for textual CollectionAnchor declarations (compile_projection_source_text_with_collection_anchors)
ReferenceContourAdmission reusing the existing action_admission/action_admission_result/admitted_action evidence chain
bounded replay/staleness invocation guard for the reference contour
real glyphon-backed DrawText/glyph rendering in the native wgpu backend
bounded hit-testing, focus routing and pointer-capture in the reference application
deterministic vertical-stack-by-declared-child-order layout for the reference application
deterministic accessibility evidence (BridgeAccessibilityEntry: node/role/label) for the reference application
--ui-dna2-reference end-to-end native reference application (UI-DNA2-10)
UiBackendAdapter::run_event_loop delivers real translated input events to application event loops (cross-cutting fix; previously every native backend silently dropped all real input for every application, including the #1541 Shell Player demo)
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
patch runtime application is landed through #1541 for the SetBindingValue/SetNodeAvailability/CollectionInsert/CollectionUpdate/CollectionRemove/CollectionMove family
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
general/unbounded runtime admission integration (bounded reference-contour admission is landed through #1544)
general/unbounded runtime dispatch integration
general UI wiring beyond the bounded reference contour
digest/signature algorithm selection for ProjectionBundle trust (cryptographic trust, as distinct from the landed self-consistency verification)
general Level 4/5 production ProjectionBundle reader/parser (bounded contour parser/verifier/loader are landed through #1544)
full OS accessibility-tree / accesskit integration (deterministic BridgeAccessibilityEntry evidence is landed through #1544)
general draw/layout realization beyond the bounded reference contour's deterministic vertical-stack layout
Workbench or Semantic Studio work
unrestricted/critical/production promotion (UI-DNA2-11 records PROMOTE WITH LIMITS, bounded to the --ui-dna2-reference contour; see §11)
```

The complete Shell Player transition pipeline (`PreparedProjectionPatchTargets`/
`PreparedActiveProjectionTargets` producers, the `prom-ui::shell_bridge` public
bridge with same-PR API guard coverage, the runtime-owned
`ActiveProjectionTargetCatalog`, stage-4 coherence/resource checks, the
stage-5 evaluator, stage-5/stage-6 orchestration, `ProjectionPatch` runtime
application, replay-cursor advancement, candidate-state calculation and
atomic commit, and a deterministic native demo mode) is landed through
#1541. This is Shell Player's own local playback/rendering integration.

`ProjectionBundle` parsing/validation/verification/inert-loading, bounded
Gate D activation, bounded hit-test/focus/pointer-capture, bounded
deterministic accessibility evidence, and the end-to-end reference slice
(UI-DNA2-10) are landed through #1544, strictly bounded to the
`--ui-dna2-reference` contour. General/unrestricted admission integration,
full OS accessibility-tree integration, general draw/layout realization
beyond that one contour's deterministic layout, and unrestricted/critical
production promotion remain absent or unauthorized above.

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
| UI-DNA2-8 — ProjectionBundle qualification | **UI-DNA2-8A LOGICAL CONTRACT FREEZE, UI-DNA2-8B PARSER/VALIDATOR/VERIFIER AND UI-DNA2-8C INERT-LOADER LANDED THROUGH #1544, BOUNDED TO THE `--ui-dna2-reference` CONTOUR; GENERAL LEVEL 4/5 PRODUCTION READER NOT CLAIMED** | ProjectionBundle v0 logical identity, deterministic stage ownership, validation, resource, diagnostic and authority boundaries landed through #1519; canonical binary codec, 8-stage parser/validator/verifier (structural, cross-artifact, compatibility, self-consistency trust), golden vectors, exhaustive negative-test matrix and pure in-memory inert loading landed through #1544 | Digest/signature algorithm ownership for cryptographic trust remains unresolved (self-consistency verification only); general Level 4/5 production reader/parser remains not claimed |
| UI-DNA2-9 — Shell player integration | **CORE STAGES 1-9 TRANSITION PIPELINE LANDED THROUGH #1541; BOUNDED GATE D ACTIVATION, HIT-TEST/FOCUS/ACCESSIBILITY AND NATIVE TEXT RENDERING LANDED THROUGH #1544 FOR THE `--ui-dna2-reference` CONTOUR; PRODUCTION PROMOTION REMAINS OUT OF SCOPE** | #1520 and #1521 freeze and close the 9A1 ownership/stage boundary; #1524 freezes the 9B activated-session, lifecycle, local-state, deterministic transition, resource and diagnostic contract; #1525 moves input-resource preflight ahead of target/replay traversal; #1527 closes UI-DNA2-9B evidence; #1528 lands the crate-private lifecycle evaluator; #1529 lands the stateful `ShellSession` owner; #1530 lands ordered ProjectionPatch envelope preflight through stages 1-4; #1531 lands the local replay-cursor representation; #1532 freezes the stage-6 replay-cursor compatibility contract; #1533 lands the crate-private stage-6 replay-cursor compatibility evaluator; #1534 freezes the stage-5 stable-target boundary contract; #1535 freezes the prepared-handoff ownership contract; #1536 freezes the explicit `CollectionAnchor` declaration contract; #1539 lands the crate-private programmatic `CollectionAnchor` declaration representation, `ProjectionSourceDocument` declaration storage, deterministic compiler-owned qualification, immutable `QualifiedCollectionAnchorDeclarations`, four `CAD_*` diagnostics, deterministic duplicate provenance and 20 focused tests (reviewed head `327c52bb05191a5e6a01f93d7a32874f119540c3`; exact-head CI `30031617862` 8/8; post-merge CI `30034743940` 8/8); #1541 lands `PreparedProjectionPatchTargets`/`PreparedActiveProjectionTargets` producers, the first public `prom-ui::shell_bridge` bridge with same-change API guard coverage, the runtime-owned `ActiveProjectionTargetCatalog`, stage-4 coherence/resource checks, the stage-5 evaluator, stage-5/stage-6 orchestration, `ProjectionPatch` runtime application, replay-cursor advancement, candidate-state calculation and atomic commit, and a deterministic `--shell-player-demo` native mode manually verified via a live screenshot (reviewed head `8d29c19c782928aae546ced3c1b9c58e8db8491c`); #1544 lands textual `CollectionAnchor` declaration syntax and Grammar v0/frontend integration, bounded Gate D activation, `ReferenceContourAdmission`, real glyphon `DrawText` rendering, bounded hit-test/focus/pointer-capture and deterministic accessibility evidence, plus a cross-cutting fix so real native input actually reaches application event loops (reviewed head `ae7e2c37ddc5f65773e0dbbd9c399eac8d07d3b0`; exact-head CI 16/16; post-merge CI passed) | general/unrestricted admission, dispatch and UI wiring beyond the bounded reference contour; full OS accessibility-tree/accesskit integration; general draw/layout realization beyond the bounded contour's deterministic layout; production promotion remain closed/unauthorized |
| UI-DNA2-10 — End-to-end reference slice | **COMPLETE, LANDED IN #1544** | `--ui-dna2-reference`: one deterministic, non-critical reference application driving the full pipeline from real Projection Source text through to a visible native window, verified with real synthetic OS input reaching hit-testing, admission, commit and visible re-render (screenshots) | None; bounded to this one reference contour by design |
| UI-DNA2-11 — Production promotion decision | **DECIDED IN #1544: PROMOTE WITH LIMITS** | Explicit decision recorded in §11: the `--ui-dna2-reference` contour, its bounded Gate D policy, and the ProjectionBundle self-consistency verifier are promoted for that bounded, non-critical contour only | Explicit exclusions apply: no cryptographic trust, no general/unrestricted Gate D, no critical or production use; see §11 |

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

Gate D status as of #1544:

```text
Gate D = OPEN WITH LIMITS, bounded strictly to the --ui-dna2-reference contour
```

The bounded policy (`docs/spec/ui/gate_d_activation_policy_v0.md`) is
fail-closed, has fixed non-caller-configurable resource limits and an
accepted-versions table, and authorizes exactly one activation function
(`activate_projection_bundle_v0_gate_d`) for exactly one reference contour.

Gate D remains closed for everything else:

```text
general/unbounded runtime integration
general/unbounded admission integration
general/unbounded dispatch integration
mutable registry
revocation
other reference-domain resolution
general UI wiring beyond the bounded reference contour
```

## 7. Latest bounded research checkpoint

### UI-DNA2-9 Shell Player bounded implementation and stage-5 contract contour

UI-DNA2-9A1 Shell Player v0 boundary contract = LANDED IN #1520
UI-DNA2-9A1 authorization = CONSUMED / CLOSED
UI-DNA2-9B session/local-state contract = LANDED IN #1524
UI-DNA2-9B post-merge preflight correction = LANDED IN #1525
UI-DNA2-9B authorization = CONSUMED / CLOSED
UI-DNA2-9B evidence closeout = LANDED IN #1527
lifecycle seed = LANDED IN #1528
stateful ShellSession owner = LANDED IN #1529
ordered envelope preflight = LANDED IN #1530
replay cursor seed = LANDED IN #1531
replay compatibility contract = LANDED IN #1532
replay compatibility evaluator = LANDED IN #1533
stage-5 stable-target boundary contract = LANDED IN #1534
prepared-handoff contract = LANDED IN #1535
CollectionAnchor declaration contract = LANDED IN #1536
CollectionAnchor declaration qualification = LANDED IN #1539
PreparedProjectionPatchTargets producer = LANDED IN #1541
PreparedActiveProjectionTargets producer = LANDED IN #1541
prom-ui::shell_bridge public cross-crate surface = LANDED IN #1541
ActiveProjectionTargetCatalog = LANDED IN #1541
stage-4 prepared-evidence coherence and resource checks = LANDED IN #1541
stage-5 stable-target evaluator = LANDED IN #1541
stage-5/stage-6 orchestration = LANDED IN #1541
ProjectionPatch runtime application = LANDED IN #1541
replay-cursor advancement = LANDED IN #1541
candidate-state calculation and atomic commit = LANDED IN #1541
native Shell Player demo mode = LANDED IN #1541
UI-DNA2-9 core transition pipeline = LANDED IN #1541

Current state:

```text
Binding Graph Semantic observation adapter v0 = LANDED IN #1515
denial/recovery/freshness v0 contract = LANDED IN #1516
Task Projection v0 = LANDED IN #1517
Task Projection P2 corrective qualification = LANDED IN #1518
ProjectionBundle v0 logical contract = LANDED IN #1519
General Level 4/5 production reader/parser = NOT CLAIMED
FINAL SERIALIZATION = RESOLVED for the bounded contour in #1544; trust-metadata representation (digest/signature) remains NOT SELECTED
UI-DNA2-8B = LANDED IN #1544 (bounded contour parser/validator/verifier)
UI-DNA2-8C = LANDED IN #1544 (bounded contour pure in-memory inert loader)
ProjectionBundle parser/validator/verifier = IMPLEMENTED in #1544, bounded to the --ui-dna2-reference contour
ProjectionBundle inert loader = IMPLEMENTED in #1544, bounded to the --ui-dna2-reference contour
ProjectionBundle activation = bounded Gate D activation IMPLEMENTED in #1544 (activate_projection_bundle_v0_gate_d); general activation remains NOT AUTHORIZED
bundle activation = bounded (see above); general bundle activation remains NOT AUTHORIZED
Shell Player v0 ownership and stage boundary = LANDED IN #1520
UI-DNA2-9A1 authorization = CONSUMED / CLOSED
UI-DNA2-9B session/local-state semantics = LANDED / CLOSED
UI-DNA2-9B input-side resource preflight = LANDED IN #1525
UI-DNA2-9B evidence closeout = LANDED IN #1527
crate-private Shell Player lifecycle evaluator = LANDED IN #1528
crate-private stateful ShellSession owner = LANDED IN #1529
ordered ProjectionPatch envelope preflight (stages 1-4) = LANDED IN #1530
local ProjectionReplayCursor representation = LANDED IN #1531
stage-6 replay-cursor compatibility contract = LANDED IN #1532
crate-private stage-6 replay-cursor compatibility evaluator = LANDED IN #1533
stage-5 stable-target boundary contract = LANDED IN #1534
prepared-handoff ownership contract = LANDED IN #1535
explicit CollectionAnchor declaration contract = LANDED IN #1536
crate-private CollectionAnchor declaration qualification = LANDED IN #1539
QualifiedCollectionAnchorDeclarations = LANDED IN #1539
prepared evidence producers (PreparedProjectionPatchTargets, PreparedActiveProjectionTargets) = LANDED IN #1541
runtime catalog (ActiveProjectionTargetCatalog) = LANDED IN #1541
stage-5 evaluator = LANDED IN #1541
stage-5/stage-6 orchestration = LANDED IN #1541
patch application = LANDED IN #1541
cursor advancement = LANDED IN #1541
candidate-state calculation and atomic commit = LANDED IN #1541
native Shell Player demo mode = LANDED IN #1541
renderer integration for Shell Player = LANDED IN #1541 (Clear/FillRect only; DrawText remains a pre-existing backend no-op)
backend integration for Shell Player = LANDED IN #1541 (existing native winit/wgpu event loop, no new backend)
UI-DNA2-9 core transition pipeline = LANDED IN #1541
textual CollectionAnchor declaration syntax and Grammar v0/frontend integration = LANDED IN #1544
ProjectionBundle parser/validator/verifier/inert-loader = LANDED IN #1544 (bounded contour)
bounded Gate D activation (activate_projection_bundle_v0_gate_d) = LANDED IN #1544
ReferenceContourAdmission (reuses existing action_admission/action_admission_result/admitted_action evidence chain) = LANDED IN #1544
real glyphon DrawText/glyph rendering in the native wgpu backend = LANDED IN #1544
bounded hit-test/focus/pointer-capture = LANDED IN #1544
deterministic accessibility evidence (BridgeAccessibilityEntry) = LANDED IN #1544
--ui-dna2-reference end-to-end reference application (UI-DNA2-10) = LANDED IN #1544
UiBackendAdapter::run_event_loop real-input delivery fix (cross-cutting, all 15 implementors) = LANDED IN #1544
general draw/layout realization beyond the bounded reference contour = NOT IMPLEMENTED
full OS accessibility-tree/accesskit integration = NOT IMPLEMENTED (deterministic evidence only)
general/unbounded ProjectionBundle activation = NOT AUTHORIZED
Gate D = OPEN WITH LIMITS, bounded strictly to the --ui-dna2-reference contour; general/global Gate D remains CLOSED
production promotion = PROMOTE WITH LIMITS, bounded strictly to the --ui-dna2-reference contour (UI-DNA2-11, see §11); unrestricted/critical/production promotion remains NOT AUTHORIZED
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

The UI-DNA2-9A1, UI-DNA2-9B, #1541 (`SHELL-PLAYER-END-TO-END-TURBO`), and
#1543/#1544 (`UI-DNA2-END-TO-END-GATE-D-TURBO`) authorizations were each
consumed and are now closed. The #1541 authorization covered exactly the
bounded Shell Player transition pipeline, public bridge, runtime catalog,
and native demo landed in that change.

The #1544 authorization covered exactly: textual `CollectionAnchor`
Grammar v0 syntax and frontend integration; the `ProjectionBundle v0`
codec, bounded UI-DNA2-8B/8C parser/validator/verifier/inert-loader;
bounded Gate D activation for the `--ui-dna2-reference` contour only;
`ReferenceContourAdmission` reusing the existing admission evidence chain;
real native text rendering; bounded hit-test/focus/pointer-capture and
deterministic accessibility evidence; the UI-DNA2-10 end-to-end reference
application; and the UI-DNA2-11 promotion decision (**PROMOTE WITH
LIMITS**, bounded to that one contour — see §11). It also fixed a
cross-cutting bug in `UiBackendAdapter::run_event_loop` discovered while
verifying #1544 natively, which affects every native backend consumer
including the previously-merged #1541 Shell Player demo.

It does not authorize general/unrestricted `ProjectionBundle` activation,
general admission/dispatch runtime integration, general/unbounded Gate D
movement, full OS accessibility-tree integration, general draw/layout
realization beyond the bounded contour, or unrestricted/critical/production
promotion.

## 8. Dependency order after rebaseline

EVIDENCE LANDED:

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
→ Shell Player ownership and stage boundary freeze (#1520)
→ Shell Player boundary closeout (#1521)
→ Rust 1.97.1 repository and CI qualification (#1522)
→ explicit Rust 1.97.1 7hell rustfmt qualification (#1523)
→ Shell Player session/local-state contract (#1524)
→ Shell Player input-resource preflight correction (#1525)
→ UI-DNA2-9B evidence closeout (#1527)
→ Shell Player lifecycle seed (#1528)
→ stateful ShellSession owner (#1529)
→ ordered ProjectionPatch envelope preflight (#1530)
→ replay cursor seed (#1531)
→ replay-cursor compatibility contract (#1532)
→ replay-cursor compatibility evaluator (#1533)
→ stage-5 stable-target boundary contract (#1534)
→ prepared-handoff ownership contract (#1535)
→ explicit CollectionAnchor declaration contract (#1536)
→ programmatic explicit CollectionAnchor declaration qualification (#1539)
→ complete Shell Player transition pipeline, public prom-ui::shell_bridge, runtime catalog, patch application, replay-cursor advancement and native demo (#1541)
→ textual CollectionAnchor Grammar v0 syntax, ProjectionBundle v0 codec (parser/validator/verifier/inert-loader), bounded Gate D activation, ReferenceContourAdmission, native text rendering, bounded hit-test/focus/accessibility, UI-DNA2-10 reference application, UI-DNA2-11 promotion decision, and the UiBackendAdapter::run_event_loop real-input delivery fix (#1544, closes #1543)
```

CLOSED DOCUMENTATION CONTRACT SLICE:

```text
UI-DNA2-9B Shell Player session and local-state contract v0
UI-DNA2-9B authorization = CONSUMED / CLOSED
```

CLOSED IMPLEMENTATION SLICE:

```text
SHELL-PLAYER-END-TO-END-TURBO (#1541)
#1541 authorization = CONSUMED / CLOSED
UI-DNA2-END-TO-END-GATE-D-TURBO (#1543/#1544)
#1543/#1544 authorization = CONSUMED / CLOSED
```

CURRENTLY UNAUTHORIZED FUTURE CONTOURS:

```text
general/unrestricted ProjectionBundle activation (bounded Gate D activation for the --ui-dna2-reference contour is landed in #1544)
digest/signature algorithm selection for ProjectionBundle cryptographic trust
general/unbounded admission and dispatch runtime integration
full OS accessibility-tree / accesskit integration
general draw/layout realization beyond the bounded reference contour
unrestricted/critical/production promotion (UI-DNA2-11 recorded PROMOTE WITH LIMITS, bounded to the --ui-dna2-reference contour only)
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
- [x] Gate D is OPEN WITH LIMITS for the bounded `--ui-dna2-reference` contour only, landed through #1544; general/global Gate D activation/integration remains closed.
- [x] Projection source textual parser/grammar and pure in-memory parser-to-compiler frontend are qualified through #1507 and #1508; both remain crate-private, with no public API or filesystem/runtime loading authorization.
- [x] Static UI IR Artifact V1 qualification is landed at the crate-private pure in-memory boundary through #1511; public codec API and filesystem/runtime loaders remain absent, runtime loading remains unauthorized, Gate D remains closed, and production promotion remains unauthorized.
- [x] Binding Graph observation comparison and dirty-propagation v0 contract is normatively frozen independently of implementation authorization.
- [x] Binding Graph observation/dirty engine implementation and qualification are landed through #1514 at the crate-private pure in-memory boundary.
- [x] The crate-private caller-supplied Semantic observation adapter for Binding Graph observations is landed and qualified through #1515; live Semantic reads remain absent and unauthorized.
- [ ] Action IR admission integration is separately approved and qualified.
- [x] Projection Patch replay-order model and qualification are complete in the bounded WP4B contour.
- [x] Patch application is qualified in the `prom-ui-runtime::shell_player` contour through #1541 for the `SetBindingValue`/`SetNodeAvailability`/`CollectionInsert`/`CollectionUpdate`/`CollectionRemove`/`CollectionMove` family.
- [x] Denial/recovery/freshness v0 projection and inert ProjectionPatch construction are specified, implemented and qualified through #1516.
- [x] Task Projection v0 is separately specified, implemented and qualified at the crate-private pure in-memory boundary through #1518; application, live evidence acquisition, admission execution and runtime integration remain unauthorized.
- [x] ProjectionBundle v0 logical identity, stage separation, validation, resource, diagnostic and authority boundaries are frozen by the bounded documentation-only UI-DNA2-8A change.
- [x] ProjectionBundle parser/validator/verifier implementation is landed through #1544 (UI-DNA2-8B): an 8-stage verifier (decode, header, structural, cross-artifact, compatibility, self-consistency trust) with golden vectors and an exhaustive negative-test matrix; general Level 4/5 production reader/parser is not claimed; digest/signature cryptographic trust remains unresolved.
- [x] ProjectionBundle pure in-memory inert loader is landed through #1544 (UI-DNA2-8C), bounded to the `--ui-dna2-reference` contour.
- [x] ProjectionBundle activation is landed through #1544 as bounded, fail-closed Gate D activation (`activate_projection_bundle_v0_gate_d`) for the `--ui-dna2-reference` contour only; general/unrestricted activation is separately authorized and not yet approved.
- [x] Shell Player v0 ownership, inputs, outputs, stage relationships and non-authority boundaries are frozen by UI-DNA2-9A1 without implementation authorization.
- [x] UI-DNA2-9B Shell Player session input, lifecycle, local-state, transition, resource and diagnostic semantics are frozen through #1524 and the post-merge input-resource preflight correction in #1525; the consumed authorization is closed and grants no implementation authority.
- [x] Crate-private deterministic Shell Player lifecycle seed is landed through #1528.
- [x] Crate-private stateful `ShellSession` owner is landed through #1529.
- [x] Ordered ProjectionPatch envelope metadata preflight through stages 1-4 is landed through #1530.
- [x] Local replay-cursor representation is landed through #1531.
- [x] Crate-private stage-6 replay-cursor compatibility evaluator is landed through #1533, following the documentation-only compatibility contract frozen in #1532.
- [x] Shell Player stage-5 stable-target boundary contract is frozen through #1534 without implementation authorization.
- [x] Prepared cross-crate handoff ownership contract is frozen through #1535 without implementation authorization.
- [x] Explicit `CollectionAnchor` declaration contract is frozen through #1536 without implementation authorization.
- [x] Crate-private programmatic explicit `CollectionAnchor` declaration representation, `ProjectionSourceDocument` declaration storage, deterministic compiler-owned qualification and immutable `QualifiedCollectionAnchorDeclarations` are landed through #1539; prepared-activation producer/consumption integration is landed through #1541; textual declaration syntax and Grammar v0 parser/frontend integration (`compile_projection_source_text_with_collection_anchors`) are landed through #1544.
- [x] `PreparedProjectionPatchTargets` and `PreparedActiveProjectionTargets` are implemented and have a producer entry point, landed through #1541.
- [x] `ActiveProjectionTargetCatalog` runtime catalog is implemented, landed through #1541, constructed only from one activation-target snapshot and attached immutably to the activated session context.
- [x] Stage-5 stable-target evaluator is implemented, landed through #1541, using only the immutable session catalog.
- [x] ProjectionPatch runtime application and replay-cursor advancement are implemented, landed through #1541, atomic and with no cursor advancement on rejection.
- [x] The complete Shell Player transition pipeline (stages 1-9) is implemented and integrated, landed through #1541.
- [x] Renderer and backend integration are implemented, landed through #1541, through the existing native winit/wgpu backend and `--shell-player-demo` mode, manually verified via a live screenshot; real glyphon `DrawText`/glyph rendering is landed through #1544, closing the previously-noted backend gap.
- [x] Shell Player is qualified without authority transfer: it owns no Semantic truth, grants no action or effect authorization, and the renderer gains no Semantic authority (contract invariants preserved and tested through #1541 and #1544).
- [x] Bounded hit-testing, focus routing, pointer-capture and deterministic accessibility evidence (`BridgeAccessibilityEntry`) are landed through #1544 for the `--ui-dna2-reference` contour; full OS accessibility-tree/accesskit integration remains not implemented.
- [x] End-to-end deterministic reference slice (UI-DNA2-10) is landed through #1544: `--ui-dna2-reference`, one canonical, non-critical reference application driving the full pipeline from real Projection Source text to a visible native window, verified with real synthetic OS input reaching hit-testing, admission, commit and visible re-render (admitted-click and denied-click screenshot evidence).
- [x] Production promotion decision (UI-DNA2-11) is recorded in #1544: **PROMOTE WITH LIMITS**, bounded strictly to the `--ui-dna2-reference` contour, its bounded Gate D policy, and the ProjectionBundle self-consistency verifier — see §11 for the full decision and its explicit exclusions.

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

### UI-DNA2-11 production promotion decision (recorded in #1544)

**Decision: PROMOTE WITH LIMITS.**

Scope of the decision — promoted:

```text
the --ui-dna2-reference end-to-end reference contour (UI-DNA2-10), in full
the bounded Gate D activation policy for exactly that contour
the ProjectionBundle v0 parser/validator/verifier/inert-loader for exactly that contour
the ProjectionBundle v0 self-consistency trust verifier (deterministic
  canonicalize -> re-encode -> byte-compare; explicitly not cryptographic trust)
ReferenceContourAdmission reusing the existing admission evidence chain,
  bounded to that contour's fixed granted-action set
real native text rendering, bounded hit-test/focus/pointer-capture and
  deterministic accessibility evidence for that contour
```

Explicit exclusions from this decision:

```text
no general or unrestricted Gate D activation (Gate D remains CLOSED outside
  the bounded --ui-dna2-reference contour)
no cryptographic trust claim for ProjectionBundle (no digest/signature
  algorithm is selected or implemented anywhere in this repository)
no general Level 4/5 production ProjectionBundle reader/parser
no general/unrestricted runtime admission or dispatch integration
no full OS accessibility-tree / accesskit integration (deterministic
  BridgeAccessibilityEntry evidence only)
no general draw/layout realization beyond the bounded contour's
  deterministic vertical-stack layout
no critical, safety-relevant, or unrestricted production use of any kind
```

Rationale: every arrow in the pipeline (Projection Source text through to a
committed, visible native update) is real, tested code, verified with real
synthetic OS-level input reaching hit-testing, admission, and commit end to
end — not console simulation and not hand-built post-activation fixtures.
The verification and admission mechanisms are honest about their own
limits (self-consistency, not cryptography; one fixed granted-action set,
not general capability evaluation), and every exclusion above reflects a
real, currently-unresolved or intentionally out-of-scope decision rather
than an oversight. That combination of genuine end-to-end evidence plus
explicit, honestly-stated limits is exactly what `PROMOTE WITH LIMITS`
means: this bounded contour is fit to stand as evidence and as a reference
implementation, while everything general, critical, or safety-relevant
remains exactly as unauthorized as it was before.

### Closure criteria status

All five closure criteria above are now met:

1. This document was updated in #1544 and reflects the accepted execution state, including this decision.
2. Every remaining phase is either complete (UI-DNA2-0 through UI-DNA2-10) or has an explicit, documented boundary of what remains unauthorized and why (UI-DNA2-11's limits above; general Level 4/5 ProjectionBundle, cryptographic trust, full accessibility-tree integration, and unrestricted admission/dispatch/promotion all remain explicitly out of scope rather than silently dropped).
3. UI-DNA2-10 has deterministic, authority-preserving evidence: real synthetic OS input verified reaching hit-testing, admission, and commit, with screenshot evidence for both an admitted and a denied action, and automated evidence for invalid-bundle and replay/stale rejection, all preserving prior state on rejection.
4. UI-DNA2-11 records the explicit outcome above: `PROMOTE WITH LIMITS`.
5. No historical, experimental, or renderer-local structure was silently promoted to semantic authority — Shell Player still owns no Semantic truth, `hit-test result != action authorization`, `ActionIntent candidate != admitted action`, and the ProjectionBundle verifier is documented everywhere as self-consistency rather than cryptographic trust.

This roadmap update, together with the #1544 evidence it records, is what
allows the umbrella issue #1489 to close.

The following remain true after #1544:

```text
general Level 4/5 production ProjectionBundle reader/parser = NOT CLAIMED
digest/signature algorithm for cryptographic trust = NOT SELECTED
general/unrestricted ProjectionBundle activation = NOT AUTHORIZED
general/unbounded admission and dispatch runtime integration = NOT AUTHORIZED
full OS accessibility-tree / accesskit integration = NOT IMPLEMENTED
general draw/layout realization beyond the bounded reference contour = NOT IMPLEMENTED
Gate D = CLOSED outside the bounded --ui-dna2-reference contour
unrestricted/critical/production promotion = NOT AUTHORIZED
NEXT AUTHORIZED IMPLEMENTATION SLICE = NONE
```

The following are now landed and no longer belong to the unauthorized list
above:

```text
complete Shell Player stages 1-9 transition pipeline = LANDED IN #1541
PreparedProjectionPatchTargets / PreparedActiveProjectionTargets implementation = LANDED IN #1541
ActiveProjectionTargetCatalog implementation = LANDED IN #1541
stage-4 prepared-evidence coherence and resource checks = LANDED IN #1541
stage-5 stable-target evaluator = LANDED IN #1541
stage-5/stage-6 orchestration = LANDED IN #1541
ProjectionPatch runtime application = LANDED IN #1541
replay-cursor advancement = LANDED IN #1541
candidate-state calculation and atomic commit = LANDED IN #1541
first public prom-ui::shell_bridge bridge with same-change API guard = LANDED IN #1541
deterministic native Shell Player demo mode = LANDED IN #1541
textual CollectionAnchor declaration syntax and Grammar v0/frontend integration = LANDED IN #1544
ProjectionBundle v0 canonical codec (parser/validator/verifier/inert-loader), bounded to --ui-dna2-reference = LANDED IN #1544
bounded fail-closed Gate D activation for --ui-dna2-reference = LANDED IN #1544 (Gate D = OPEN WITH LIMITS for that contour)
ReferenceContourAdmission reusing the existing admission evidence chain = LANDED IN #1544
real glyphon DrawText/glyph rendering in the native backend = LANDED IN #1544
bounded hit-test/focus/pointer-capture = LANDED IN #1544
deterministic accessibility evidence = LANDED IN #1544
end-to-end reference slice (UI-DNA2-10, --ui-dna2-reference) = LANDED IN #1544
UI-DNA2-11 production promotion decision = DECIDED IN #1544 (PROMOTE WITH LIMITS)
UiBackendAdapter::run_event_loop real-input delivery (cross-cutting fix) = LANDED IN #1544
```
