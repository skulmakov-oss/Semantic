# UI DNA v2 Ownership and Compatibility Freeze

Status: proposed Gate A ownership freeze
Milestone: UI-DNA2-WP1
Task: UI-DNA2-WP1-FINAL-CLOSEOUT
Issue context: #1488 / #1489
Reconciliation evidence: `docs/roadmap/post_ui/ui_dna2_prom_ui_reconciliation.md`

This document is the canonical UI DNA v2 ownership authority for Gate A review.
It freezes target owners, compatibility posture, dependency direction, activation gates, and implementation package boundaries.

This document does not authorize implementation.

## 1. Status and baseline

| Field | Value |
| --- | --- |
| Repository | `skulmakov-oss/Semantic` |
| Required working directory | `C:\Users\said3\Desktop\EXOcode\Semantic_phase1_prom_ui` |
| Baseline branch | `main` |
| Baseline HEAD | `928d260fdcf18afdac54636badeaeca56e376610` |
| Scope | docs-only ownership freeze |
| Canonical doctrine anchor | `docs/dna/SEMANTIC_UI_DNA_v2.md` |
| Evidence ledger | `docs/roadmap/post_ui/ui_dna2_prom_ui_reconciliation.md` |

UI DNA v2 remains governed by the doctrine ordering:

```text
Meaning first.
Intent projection second.
UI IR third.
Rendering last.
```

The ownership chain remains:

```text
Semantic owns meaning.
Projection owns presentation intent.
UI IR owns structure.
Shell owns rendering.
Renderer owns pixels.
```

The safety rule remains:

```text
One meaning.
Many projections.
Truth does not move into UI.
```

## 2. Approved decisions D01-D11

| Decision | Status | Binding result |
| --- | --- | --- |
| D01 | APPROVED | Existing `UiIr` remains a structural substrate behind a separate versioned Static UI IR document contract. |
| D02 | APPROVED | Projection Source AST is dedicated and must not alias `UiAst`. |
| D03 | APPROVED | `ActionIntent` is a new boundary contract. `SemanticIntent` compatibility requires an explicit adapter. Action IR owns no admission authority. |
| D04 | APPROVED | `ProjectionBundle` progresses through separate claim levels. |
| D05 | APPROVED WITH RESTRICTION | `ui-shell-kit` remains experimental and cannot be promoted implicitly. |
| D06 | APPROVED | R12/Aldente evidence may be reused selectively, but its authority model is superseded. |
| D07 | APPROVED | A dedicated `projection_compile` owner performs deterministic lowering from Projection Source AST into Static UI IR. |
| D08 | APPROVED | `contract_primitives` is a neutral leaf owner for shared identifiers, source coordinates, revisions, epochs, and schema/version primitives. |
| D09 | APPROVED | `semantic_refs` owns opaque UI-side references to external Semantic facts. A reference never owns the referenced truth. |
| D10 | APPROVED | `admission_contract` owns UI-side transport envelopes only. External Semantic authority retains policy, capability evaluation, acceptance, denial, and revision validity. |
| D11 | APPROVED | The canonical ownership matrix contains target contracts, existing substrates, and exact interface boundaries only. Compatibility, experimental evidence, and superseded authority are registries inside this document. |

D01-D11 are not reopened here.

## 3. Normative ownership principles

- UI DNA v2 ownership is a contract boundary, not permission for crate-wide edits.
- Crate-level ownership never authorizes future work outside exact task `allowed_paths`.
- Compatibility evidence is not ownership.
- Experimental evidence is not ownership.
- Superseded authority is not ownership.
- External Semantic policy authority is not a UI compile/import node.
- Static UI IR must be usable without Projection Source AST availability.
- Projection Source AST types must not appear as fields in the canonical serialized Static UI IR document.
- Provenance must cross the source-to-IR boundary through neutral `SourceRef`-style contracts.
- `reference != referenced truth`.
- Action IR describes routes; it does not admit actions.
- `ActionIntent` carries a request; it does not represent acceptance.
- `AdmissionPort` forwards requests and transports results; it does not own policy.
- Dispatcher consumes admitted actions only; it cannot admit retroactively.
- Shell owns local interaction state and projection playback; it owns no Semantic truth.
- Renderer/backend evidence is backend-local evidence, not Semantic truth.
- Implementation activation remains unauthorized until a separate Gate B decision.

## 4. Canonical ownership matrix

The matrix contains target implementation contracts, existing substrates, and exact interface boundaries only.

| ID | Contract or substrate | Canonical owner | Kind | Status | Owns | Must not own | Compatibility posture |
| --- | --- | --- | --- | --- | --- | --- | --- |
| O01 | Contract primitives | `crates/prom-ui::contract_primitives` | target module | PROPOSED LOGICAL LOCATION - no implementation exists yet | `SchemaVersion`, `ContractVersion`, `SourceSpan`, `SourceRef`, `Revision`, `Epoch`, stable opaque IDs, deterministic diagnostic coordinates | projection, IR, binding, action, runtime, Semantic internals, shell, renderer, backend, admission policy | new neutral leaf |
| O02 | Role Dictionary | `crates/prom-ui::role_dictionary` | target module | PROPOSED LOGICAL LOCATION - no implementation exists yet | projection roles, role IDs, role version identifiers, role compatibility metadata | source AST, Static UI IR authority, renderer behavior, Semantic meaning | new provider contract |
| O03 | Projection Source | `crates/prom-ui::projection_source` | target module | LANDED — crate-private Grammar v0 parser/scanner and qualification through #1507; public API and loading absent | `.proj.sm` tokens, source AST, source refs, source diagnostics, source normalization | Static UI IR document authority, renderer layout, Semantic meaning, admission | dedicated owner; not `UiAst` |
| O04 | Projection lowering | `crates/prom-ui::projection_compile` | target module | LANDED — deterministic lowering through #1490 and crate-private pure in-memory parser-to-compiler frontend qualification through #1508; runtime activation absent | deterministic Projection Source AST to Static UI IR lowering, lowering order, provenance transfer, source-to-IR diagnostics | source language ownership, Static UI IR serialization authority, runtime activation | D07 owner |
| O05 | Static UI IR document | `crates/prom-ui::static_ir` | target module | PROPOSED LOGICAL LOCATION - no implementation exists yet | versioned renderer-independent projected structure, canonical ordering, IR digest, document wrapper | Projection Source AST fields, Semantic truth, renderer/backend behavior | may adapt existing `UiIr` substrate |
| O06 | Opaque Semantic references | `crates/prom-ui::semantic_refs` | target module | PROPOSED LOGICAL LOCATION - no implementation exists yet | `SemanticStateRef`, `EvidenceRef`, `ActionOfferRef`, `TaskRecordRef`, `ConnectivityFactRef`, `ActorRef`, `SessionRef`, `ClientRef` | referenced truth, Semantic internals, runtime policy, renderer/backend behavior | D09 owner |
| O07 | Binding Graph | `crates/prom-ui::binding_graph` | target module | PROPOSED LOGICAL LOCATION - no implementation exists yet | read-side relationships between structure, roles, semantic refs, action surfaces, evidence anchors | patch stream, denial/recovery, task/control semantics, runtime effects, admission | may adapt slot-intent carrier evidence |
| O08 | Action IR | `crates/prom-ui::action_ir` | target module | PROPOSED LOGICAL LOCATION - no implementation exists yet | compiled route declarations, route identity, route lookup contracts | request envelope, actor/session/client context, admission authority, capability policy | new route contract |
| O09 | ActionIntent | `crates/prom-ui::action_intent` | target module | PROPOSED LOGICAL LOCATION - no implementation exists yet | request envelope, route reference, actor/session/client attribution, source revision and epoch, idempotency, freshness evidence, requested action representation | route table ownership, acceptance, denial, policy, capability evaluation | new boundary contract |
| O10 | SemanticIntent compatibility adapter | `crates/prom-ui::compat` | target module | PROPOSED LOGICAL LOCATION - no implementation exists yet | explicit mapping between legacy carrier and `ActionIntent` without context erasure | implicit `From`/`Into`, admission authority, historical authority revival | adapter only |
| O11 | Admission transport contract | `crates/prom-ui::admission_contract` | target module | PROPOSED LOGICAL LOCATION - no implementation exists yet | `AdmissionRequestId`, `AdmissionOutcome`, `AdmittedAction`, `AdmissionDenied`, correlation metadata, revision/freshness evidence | policy, capability evaluation, acceptance logic, denial logic, revision validity, Semantic truth | D10 owner |
| O12 | AdmissionPort adapter boundary | `crates/prom-ui-runtime::intent_admission` | runtime module owning a port contract | PROPOSED LOGICAL LOCATION - no implementation exists yet | `AdmissionPort`, request forwarding, result transport, trace correlation, mapping to `admission_contract` envelopes | Semantic policy, capability evaluation, acceptance, denial, revision validity | exact UI-facing port owned by this module |
| O13 | Admitted-action dispatcher | `crates/prom-ui-runtime::intent_dispatch` | target module | PROPOSED LOGICAL LOCATION - no implementation exists yet | dispatch of `admission_contract::AdmittedAction` after admission | retroactive admission, policy, capability checks | consumes admitted action only |
| O14 | Projection Patch | `crates/prom-ui::projection_patch` | target module | PROPOSED LOGICAL LOCATION - no implementation exists yet | deterministic patch vocabulary and replay order over Static UI IR and Binding Graph outputs | Binding Graph authority, Semantic truth, runtime effects | patch stream contract |
| O15 | Denial/recovery projection | `crates/prom-ui::denial_recovery` | target module | PROPOSED LOGICAL LOCATION - no implementation exists yet | denial projection, recovery outlets, local/session denial distinction, acknowledgement/retry/resume/cancel-suffix presentation contracts | admission policy, invented recovery behavior, Semantic truth | consumes admission outcomes and graph anchors |
| O16 | Task projection | `crates/prom-ui::task_projection` | target module | PROPOSED LOGICAL LOCATION - no implementation exists yet | `TaskRecord` projection, progress presentation, task control projection, task-scope visibility | task truth, task engine, capability policy | consumes opaque refs and patches |
| O17 | Connectivity/freshness projection | `crates/prom-ui::connectivity_projection` | target module | PROPOSED LOGICAL LOCATION - no implementation exists yet | Fresh/stale/offline projection, observation-only and cache-only presentation, `PendingUnknown`, `Resyncing` projection | connection truth, control authority, offline critical action queue | consumes opaque refs and patches |
| O18 | Backend-neutral interaction contracts | `crates/prom-ui::interaction` | existing or proposed module boundary | PARTIAL EXISTING SUBSTRATE | raw UI event normalization contracts and interaction intent surface before action routing | shell local state, runtime sessions, native event loop, admission | backend-neutral provider |
| O19 | Bundle manifest | `crates/prom-ui-runtime::bundle::manifest` | target module | PROPOSED LOGICAL LOCATION - no implementation exists yet | manifest metadata, schema IDs, artifact IDs, role dictionary version IDs, digest references | parser, validation, verification, loading, activation | claim-level owner |
| O20 | Bundle parser | `crates/prom-ui-runtime::bundle::parser` | target module | PROPOSED LOGICAL LOCATION - no implementation exists yet | deterministic parse from approved artifact format into parsed bundle structure | structural compatibility, integrity verification, loading, activation | claim-level owner |
| O21 | Bundle structural validator | `crates/prom-ui-runtime::bundle::validate::structural` | target module | PROPOSED LOGICAL LOCATION - no implementation exists yet | structural acceptance/rejection of parsed bundle | compatibility validation, integrity verification, loading, activation | claim-level owner |
| O22 | Bundle compatibility validator | `crates/prom-ui-runtime::bundle::validate::compatibility` | target module | PROPOSED LOGICAL LOCATION - no implementation exists yet | role, renderer profile, schema, and version compatibility checks | integrity verification, loading, activation | claim-level owner |
| O23 | Bundle verifier | `crates/prom-ui-runtime::bundle::verify` | target module | PROPOSED LOGICAL LOCATION - no implementation exists yet | hash/signature/trust material verification for bundles | Semantic admission authority, activation, production promotion | claim-level owner |
| O24 | Bundle inert loader | `crates/prom-ui-runtime::bundle::loader` | target module | PROPOSED LOGICAL LOCATION - no implementation exists yet | loading verified bundle into inert runtime representation | activation, runtime side effects, production rollout | claim-level owner |
| O25 | Bundle activation boundary | `crates/prom-ui-runtime::bundle_activation` | target module | PROPOSED LOGICAL LOCATION - no implementation exists yet | safe activation boundary, rollback/quarantine contract | production promotion, Semantic authority | claim-level owner |
| O26 | Shell player | `crates/prom-ui-runtime::shell_player` | target module | PROPOSED LOGICAL LOCATION - no implementation exists yet | local focus, hit testing, accessibility realization, patch application, draw-command production | Semantic truth, admission policy, renderer pixel authority, backend event loop | may consume experimental shell evidence after approval |
| O27 | Runtime draw/session seam | `crates/prom-ui-runtime::draw_seam` | target module | PROPOSED LOGICAL LOCATION - no implementation exists yet | backend-neutral draw/session boundary consumed by native backends | native backend implementation, pixels, Semantic truth | runtime-owned seam |
| O28 | Native backend boundary | `crates/prom-ui-backend-native` | crate boundary | EXISTING SUBSTRATE | native facade, platform event bridge, backend-local rendering evidence | UI model authority, Semantic truth, runtime policy, Static UI IR authority | backend provider only |
| O29 | Existing `UiIr` structural substrate | existing `crates/prom-ui` `UiIr` owner | existing substrate | EXISTING SUBSTRATE | inert structural patterns and typed IDs usable behind v2 wrapper | complete UI DNA v2 document authority | ADAPTER |
| O30 | Existing `UiAst` substrate patterns | existing `crates/prom-ui` `UiAst` owner | existing substrate | EXISTING SUBSTRATE | reusable AST implementation patterns and diagnostics style | canonical Projection Source AST identity | ADAPTER |
| O31 | Existing `SemanticIntent` carrier | `crates/prom-ui::action_mapping` | existing substrate | EXISTING SUBSTRATE | legacy/minimal carrier type available to explicit compatibility mapper | `ActionIntent` replacement, admission authority, hidden context erasure | ADAPTER through `compat` |
| O32 | Existing UI model substrate | `crates/prom-ui::model` | existing substrate | EXISTING SUBSTRATE | neutral UI model identifiers and existing model vocabulary usable by Static UI IR through narrow contracts | Projection Source AST, lowering, renderer behavior, Semantic meaning | substrate only |

## 5. Logical module map

```text
crates/prom-ui
  contract_primitives
  role_dictionary
  model
  projection_source
  projection_compile
  static_ir
  semantic_refs
  binding_graph
  action_ir
  action_intent
  compat
  admission_contract
  projection_patch
  denial_recovery
  task_projection
  connectivity_projection
  interaction

crates/prom-ui-runtime
  intent_admission::AdmissionPort
  intent_admission
  intent_dispatch
  bundle::manifest
  bundle::parser
  bundle::validate::structural
  bundle::validate::compatibility
  bundle::verify
  bundle::loader
  bundle_activation
  shell_player
  draw_seam

crates/prom-ui-backend-native
  native backend boundary
```

Required projection boundary:

```text
projection_source
  owns tokens, source AST, source references, source diagnostics

projection_compile
  consumes projection_source
  emits static_ir

static_ir
  owns versioned renderer-independent projected structure
  does not import projection_source
```

Required action boundary:

```text
action_ir
  owns compiled route declarations, route identity, route lookup contracts

action_intent
  owns request envelope, route reference, actor/session/client attribution,
  revision, epoch, idempotency, freshness, and requested-action representation

admission_contract
  owns UI-side transport envelopes only

intent_admission::AdmissionPort
  owns adapter interface and result transport only

intent_dispatch
  consumes admitted actions only
```

Required runtime/backend direction:

```text
prom-ui-backend-native
  -> prom-ui-runtime
  -> prom-ui
```

Required interaction/shell direction:

```text
crates/prom-ui::interaction
  owns backend-neutral raw-event and interaction contracts

crates/prom-ui-runtime::shell_player
  consumes interaction contracts
  owns local shell state and projection playback
```

## 6. Compile/import dependency model

Arrow semantics:

```text
Consumer -> Provider
```

Only independently owned logical modules or contract modules may appear as nodes.
Container crate nodes are represented separately in the crate dependency table.
Nested trait/type contracts remain owned by their module unless they have independent logical ownership.
External Semantic policy authority is not a compile/import node.

### Compile node registry

| Node | Classification | Exact logical owner/path | Importable contract description | Status |
| --- | --- | --- | --- | --- |
| N01 | MODULE | `crates/prom-ui::contract_primitives` | shared schema/version/source/revision/epoch/ID primitives | proposed |
| N02 | MODULE | `crates/prom-ui::role_dictionary` | role definitions and role version identifiers | proposed |
| N03 | MODULE | `crates/prom-ui::model` | neutral existing UI model substrate contracts | existing |
| N04 | MODULE | `crates/prom-ui::projection_source` | Projection Source tokens, AST, source refs, diagnostics | landed crate-private; Grammar v0 parser/scanner qualified |
| N05 | MODULE | `crates/prom-ui::projection_compile` | deterministic lowering boundary from Projection Source to Static UI IR | landed crate-private; pure in-memory frontend qualified |
| N06 | MODULE | `crates/prom-ui::static_ir` | versioned renderer-independent Static UI IR document contract | proposed |
| N07 | MODULE | `crates/prom-ui::semantic_refs` | opaque non-authoritative references to external Semantic facts | proposed |
| N08 | MODULE | `crates/prom-ui::binding_graph` | read-side binding relationship graph | proposed |
| N09 | MODULE | `crates/prom-ui::action_ir` | action route declarations and route lookup contracts | proposed |
| N10 | MODULE | `crates/prom-ui::action_intent` | structured action request envelope | proposed |
| N11 | MODULE | `crates/prom-ui::compat` | explicit compatibility adapters | proposed |
| N12 | MODULE | `crates/prom-ui::action_mapping` | existing `SemanticIntent` carrier provider | existing |
| N13 | MODULE | `crates/prom-ui::admission_contract` | UI-side admission request/outcome transport envelopes | proposed |
| N14 | MODULE | `crates/prom-ui::projection_patch` | deterministic projection patch vocabulary and replay contract | proposed |
| N15 | MODULE | `crates/prom-ui::denial_recovery` | denial and recovery projection contracts | proposed |
| N16 | MODULE | `crates/prom-ui::task_projection` | task projection and task-control presentation contracts | proposed |
| N17 | MODULE | `crates/prom-ui::connectivity_projection` | connectivity/freshness projection contracts | proposed |
| N18 | MODULE | `crates/prom-ui::interaction` | backend-neutral interaction contracts | existing/proposed |
| N19 | MODULE | `crates/prom-ui-runtime::intent_admission` | admission adapter implementation boundary owning `AdmissionPort` | proposed |
| N20 | MODULE | `crates/prom-ui-runtime::intent_dispatch` | admitted-action dispatch boundary | proposed |
| N21 | MODULE | `crates/prom-ui-runtime::bundle::manifest` | bundle manifest contract | proposed |
| N22 | MODULE | `crates/prom-ui-runtime::bundle::parser` | bundle parser contract | proposed |
| N23 | MODULE | `crates/prom-ui-runtime::bundle::validate::structural` | structural validation contract | proposed |
| N24 | MODULE | `crates/prom-ui-runtime::bundle::validate::compatibility` | compatibility validation contract | proposed |
| N25 | MODULE | `crates/prom-ui-runtime::bundle::verify` | integrity/signature verification contract | proposed |
| N26 | MODULE | `crates/prom-ui-runtime::bundle::loader` | inert bundle loading contract | proposed |
| N27 | MODULE | `crates/prom-ui-runtime::bundle_activation` | safe activation boundary contract | proposed |
| N28 | MODULE | `crates/prom-ui-runtime::shell_player` | shell playback, focus, hit-test, accessibility, patch application, draw-command production | proposed |
| N29 | MODULE | `crates/prom-ui-runtime::draw_seam` | backend-neutral draw/session seam | proposed |

### Compile/import edge table

| Consumer | Provider | Reason |
| --- | --- | --- |
| `crates/prom-ui::role_dictionary` | `crates/prom-ui::contract_primitives` | role versions use neutral version/ID primitives |
| `crates/prom-ui::projection_source` | `crates/prom-ui::contract_primitives` | source refs and diagnostics use neutral source coordinates |
| `crates/prom-ui::projection_source` | `crates/prom-ui::role_dictionary` | source projection roles use role dictionary contracts |
| `crates/prom-ui::static_ir` | `crates/prom-ui::contract_primitives` | IR IDs, schema versions, source refs, revisions, and digests use neutral primitives |
| `crates/prom-ui::static_ir` | `crates/prom-ui::role_dictionary` | Static UI IR references role dictionary contracts |
| `crates/prom-ui::static_ir` | `crates/prom-ui::model` | Static UI IR may reuse neutral existing UI model substrate contracts |
| `crates/prom-ui::projection_compile` | `crates/prom-ui::projection_source` | lowering consumes Projection Source AST |
| `crates/prom-ui::projection_compile` | `crates/prom-ui::static_ir` | lowering emits Static UI IR |
| `crates/prom-ui::projection_compile` | `crates/prom-ui::contract_primitives` | lowering transfers neutral source provenance and diagnostics |
| `crates/prom-ui::projection_compile` | `crates/prom-ui::role_dictionary` | lowering resolves approved projection roles |
| `crates/prom-ui::semantic_refs` | `crates/prom-ui::contract_primitives` | opaque refs use stable IDs, epochs, revisions, and coordinates |
| `crates/prom-ui::binding_graph` | `crates/prom-ui::static_ir` | graph binds projected structure |
| `crates/prom-ui::binding_graph` | `crates/prom-ui::semantic_refs` | graph stores opaque references to external facts |
| `crates/prom-ui::binding_graph` | `crates/prom-ui::role_dictionary` | graph binds role identities |
| `crates/prom-ui::binding_graph` | `crates/prom-ui::contract_primitives` | graph uses stable IDs and revisions |
| `crates/prom-ui::action_ir` | `crates/prom-ui::static_ir` | routes attach to projected structure |
| `crates/prom-ui::action_ir` | `crates/prom-ui::binding_graph` | routes reference graph anchors |
| `crates/prom-ui::action_ir` | `crates/prom-ui::role_dictionary` | routes reference approved role contracts |
| `crates/prom-ui::action_ir` | `crates/prom-ui::contract_primitives` | route IDs and versions use neutral primitives |
| `crates/prom-ui::action_intent` | `crates/prom-ui::action_ir` | request envelope references action routes |
| `crates/prom-ui::action_intent` | `crates/prom-ui::semantic_refs` | request envelope carries actor/session/client and evidence refs |
| `crates/prom-ui::action_intent` | `crates/prom-ui::contract_primitives` | request envelope carries revision, epoch, idempotency, and freshness IDs |
| `crates/prom-ui::compat` | `crates/prom-ui::action_mapping` | explicit SemanticIntent adapter imports the exact legacy carrier provider |
| `crates/prom-ui::compat` | `crates/prom-ui::action_intent` | compatibility mapper produces or consumes ActionIntent explicitly |
| `crates/prom-ui::compat` | `crates/prom-ui::contract_primitives` | compatibility mapping preserves revision/epoch/source primitives |
| `crates/prom-ui::admission_contract` | `crates/prom-ui::contract_primitives` | request IDs, revisions, epochs, and freshness evidence use neutral primitives |
| `crates/prom-ui::admission_contract` | `crates/prom-ui::semantic_refs` | outcomes carry opaque fact/evidence/action-offer refs |
| `crates/prom-ui::admission_contract` | `crates/prom-ui::action_intent` | admission transport envelopes carry or reference requested action intent |
| `crates/prom-ui::projection_patch` | `crates/prom-ui::static_ir` | patches target Static UI IR structures |
| `crates/prom-ui::projection_patch` | `crates/prom-ui::binding_graph` | patches reference graph outputs |
| `crates/prom-ui::projection_patch` | `crates/prom-ui::contract_primitives` | patch IDs and replay order use neutral primitives |
| `crates/prom-ui::denial_recovery` | `crates/prom-ui::projection_patch` | denial/recovery emits projection patches |
| `crates/prom-ui::denial_recovery` | `crates/prom-ui::binding_graph` | denial/recovery anchors to graph outputs |
| `crates/prom-ui::denial_recovery` | `crates/prom-ui::semantic_refs` | denial/recovery carries opaque evidence/action refs |
| `crates/prom-ui::denial_recovery` | `crates/prom-ui::admission_contract` | denial/recovery consumes admission outcomes |
| `crates/prom-ui::task_projection` | `crates/prom-ui::projection_patch` | task progress emits projection patches |
| `crates/prom-ui::task_projection` | `crates/prom-ui::binding_graph` | task projection anchors to graph outputs |
| `crates/prom-ui::task_projection` | `crates/prom-ui::semantic_refs` | task projection carries opaque task/action-offer refs |
| `crates/prom-ui::connectivity_projection` | `crates/prom-ui::projection_patch` | freshness and connectivity emit projection patches |
| `crates/prom-ui::connectivity_projection` | `crates/prom-ui::binding_graph` | connectivity projection anchors to graph outputs |
| `crates/prom-ui::connectivity_projection` | `crates/prom-ui::semantic_refs` | connectivity projection carries opaque connectivity refs |
| `crates/prom-ui::interaction` | `crates/prom-ui::contract_primitives` | interaction contracts use stable source/session IDs where required |
| `crates/prom-ui-runtime::intent_admission` | `crates/prom-ui::admission_contract` | adapter maps results to admission envelopes |
| `crates/prom-ui-runtime::intent_admission` | `crates/prom-ui::action_intent` | adapter forwards structured requests |
| `crates/prom-ui-runtime::intent_admission` | `crates/prom-ui::semantic_refs` | adapter preserves opaque external references |
| `crates/prom-ui-runtime::intent_admission` | `crates/prom-ui::contract_primitives` | adapter preserves correlation, revision, and freshness primitives |
| `crates/prom-ui-runtime::intent_dispatch` | `crates/prom-ui::admission_contract` | dispatcher consumes only admitted actions |
| `crates/prom-ui-runtime::bundle::manifest` | `crates/prom-ui::contract_primitives` | manifest uses schema/version/digest identifiers |
| `crates/prom-ui-runtime::bundle::manifest` | `crates/prom-ui::static_ir` | manifest references Static UI IR artifact/schema identifiers |
| `crates/prom-ui-runtime::bundle::manifest` | `crates/prom-ui::role_dictionary` | manifest references Role Dictionary version identifiers |
| `crates/prom-ui-runtime::bundle::parser` | `crates/prom-ui-runtime::bundle::manifest` | parser reads manifest contract |
| `crates/prom-ui-runtime::bundle::validate::structural` | `crates/prom-ui-runtime::bundle::parser` | structural validator consumes parsed bundle structure |
| `crates/prom-ui-runtime::bundle::validate::structural` | `crates/prom-ui-runtime::bundle::manifest` | structural validator checks manifest-declared structure |
| `crates/prom-ui-runtime::bundle::validate::structural` | `crates/prom-ui::static_ir` | structural validator checks Static UI IR structure |
| `crates/prom-ui-runtime::bundle::validate::compatibility` | `crates/prom-ui-runtime::bundle::parser` | compatibility validator consumes parsed bundle structure |
| `crates/prom-ui-runtime::bundle::validate::compatibility` | `crates/prom-ui-runtime::bundle::manifest` | compatibility validator checks manifest-declared compatibility inputs |
| `crates/prom-ui-runtime::bundle::validate::compatibility` | `crates/prom-ui::role_dictionary` | compatibility validator checks role compatibility |
| `crates/prom-ui-runtime::bundle::verify` | `crates/prom-ui-runtime::bundle::manifest` | verifier checks manifest-declared trust material |
| `crates/prom-ui-runtime::bundle::verify` | `crates/prom-ui::contract_primitives` | verifier uses digest/version identifiers |
| `crates/prom-ui-runtime::bundle::loader` | `crates/prom-ui-runtime::bundle::parser` | inert loader consumes parsed bundle representation |
| `crates/prom-ui-runtime::bundle::loader` | `crates/prom-ui-runtime::bundle::validate::structural` | inert loader requires structural validation |
| `crates/prom-ui-runtime::bundle::loader` | `crates/prom-ui-runtime::bundle::validate::compatibility` | inert loader requires compatibility validation |
| `crates/prom-ui-runtime::bundle::loader` | `crates/prom-ui-runtime::bundle::verify` | inert loader requires integrity/signature verification |
| `crates/prom-ui-runtime::bundle_activation` | `crates/prom-ui-runtime::bundle::loader` | activation consumes inert loaded bundle |
| `crates/prom-ui-runtime::shell_player` | `crates/prom-ui::interaction` | shell consumes normalized interaction contracts |
| `crates/prom-ui-runtime::shell_player` | `crates/prom-ui-runtime::bundle::loader` | shell consumes inert loaded bundle representation |
| `crates/prom-ui-runtime::shell_player` | `crates/prom-ui::projection_patch` | shell applies projection patches |
| `crates/prom-ui-runtime::shell_player` | `crates/prom-ui::action_ir` | shell performs local route lookup |
| `crates/prom-ui-runtime::shell_player` | `crates/prom-ui::action_intent` | shell creates structured requests |
| `crates/prom-ui-runtime::shell_player` | `crates/prom-ui::static_ir` | shell reads static projected structure |
| `crates/prom-ui-runtime::shell_player` | `crates/prom-ui-runtime::draw_seam` | shell emits draw/session material through the backend-neutral draw seam |
| `crates/prom-ui-runtime::draw_seam` | `crates/prom-ui::contract_primitives` | draw/session seam may carry stable opaque IDs |

### Crate dependency table

| Consumer crate | Provider crate | Purpose |
| --- | --- | --- |
| `prom-ui-runtime` | `prom-ui` | runtime consumes UI contract modules without moving authority into runtime |
| `prom-ui-backend-native` | `prom-ui-runtime` | native backend consumes backend-neutral runtime seams |
| `prom-ui-backend-native` | `prom-ui` | native backend may consume stable UI transport/event contracts without owning UI semantics |

Forbidden crate dependencies:

```text
prom-ui -> prom-ui-runtime
prom-ui-runtime -> prom-ui-backend-native
prom-ui -> prom-ui-backend-native
```

Forbidden compile/import edges:

```text
static_ir -> projection_source
projection_source -> static_ir
binding_graph -> projection_patch
interaction -> shell_player
prom-ui -> prom-ui-runtime
prom-ui-runtime -> prom-ui-backend-native
any UI compile node -> external Semantic policy authority
```

## 7. Runtime/data flow

Arrow semantics:

```text
data or request moves from A to B
```

Canonical runtime/data flow:

```text
RawUiEvent
  -> interaction normalization
  -> Action IR route lookup
  -> ActionIntent
  -> AdmissionPort adapter
  -> external Semantic admission authority
  -> AdmissionOutcome
  -> admitted dispatcher or denial projection
  -> ProjectionPatch
  -> shell player
  -> runtime draw seam
  -> native backend
```

Expanded runtime notes:

- Raw events are local until normalized into backend-neutral interaction contracts.
- Action IR lookup selects a route; it does not admit.
- `ActionIntent` carries request context; it does not represent acceptance.
- `AdmissionPort` forwards the request to external Semantic admission authority and transports the result.
- External Semantic admission authority may accept, deny, reject for freshness, or return evidence.
- Admitted results flow to `intent_dispatch`.
- Denied results flow to `denial_recovery`.
- Projection changes flow through `projection_patch`.
- Shell player applies patches and produces draw commands.
- Native backend receives backend-local draw/session material only.

## 8. Authority flow

Arrow semantics:

```text
decision authority remains with the named owner
```

Canonical authority flow:

```text
Semantic owns meaning.
External Semantic admission authority owns policy and acceptance.
Projection owns presentation intent.
Static UI IR owns structure.
Binding Graph owns read-side dependencies.
Action IR owns routes.
ActionIntent carries requests.
AdmissionContract carries outcomes.
Shell owns local interaction state.
Renderer/backend owns pixels.
```

Authority non-transfer rules:

- Projection Source does not own Semantic meaning.
- Static UI IR does not own Semantic meaning.
- Binding Graph does not own Semantic truth.
- Action IR does not own admission.
- `ActionIntent` does not own acceptance.
- `admission_contract` does not own policy.
- `intent_admission` does not own capability evaluation or revision validity.
- Shell does not own Semantic truth.
- Renderer/backend does not own projection authority or Semantic truth.
- Snapshot evidence is not Semantic truth.

External Semantic admission authority is recorded only in runtime/data flow and authority flow.
It is intentionally absent from the compile/import graph.

## 9. Public/internal contract policy

Default posture:

| Contract | Default visibility | Promotion requirements |
| --- | --- | --- |
| `contract_primitives` | internal first; selected primitive types may become public only by explicit approval | versioning, public API guard, compatibility policy |
| `projection_source` | internal | parser/source diagnostics tests, no `.proj.sm` public claim before approval |
| `projection_compile` | internal | deterministic lowering tests, source-to-IR provenance tests |
| `static_ir` | internal document contract first | serialization/digest tests, compatibility wrapper, public API guard |
| `semantic_refs` | boundary-visible where required | opacity tests and no Semantic internals exposure |
| `binding_graph` | internal | deterministic construction tests and negative cycle/missing-key tests |
| `action_ir` | internal | route lookup tests and admission non-authority tests |
| `action_intent` | boundary type where required | context-preservation tests and explicit compatibility mapping |
| `admission_contract` | boundary type where required | acceptance/denial envelope tests and no policy ownership |
| `intent_admission::AdmissionPort` | runtime boundary | adapter-only tests and trace-correlation tests |
| `ProjectionBundle` stages | internal first by claim level | parser/validator/verifier/loader separation evidence |
| `shell_player` | experimental/internal first | deterministic shell evidence and no authority widening |
| native backend | existing boundary | backend-local tests only |

A contract may become public only after stable ownership, deterministic behavior where relevant, positive and negative tests, versioning, compatibility policy, public API guard, and explicit architect approval.

## 10. Compatibility Registry

| Entity | Classification | Permitted compatibility | Forbidden compatibility | Required seam |
| --- | --- | --- | --- | --- |
| `UiIr` | ADAPTER | structural substrate behind v2 Static UI IR wrapper/lowering | treating legacy `UiIr` as complete v2 document authority | `crates/prom-ui::static_ir` |
| `UiAst` patterns | ADAPTER | implementation patterns and diagnostics style | aliasing `UiAst` as Projection Source AST | `crates/prom-ui::projection_source` |
| `SemanticIntent` | ADAPTER | explicit mapper to/from `ActionIntent` preserving actor/session/client/revision/epoch/idempotency/freshness | implicit context-erasing `From`/`Into`; admission authority | `crates/prom-ui::compat` importing `crates/prom-ui::action_mapping` |
| slot-intent carriers | ADAPTER | metadata and carrier evidence for Binding Graph migration | Binding Graph authority | `crates/prom-ui::binding_graph` |
| ProjectionBundle fixture reader | QUARANTINED | fixture evidence and negative-probe input | parser, validator, loader, activation authority | future `bundle::parser` tests only |
| historical compatibility adapters | QUARANTINED | deterministic evidence where mapped to exact owner | new architecture authority | narrow removable adapters |

Compatibility totals:

| Classification | Count |
| --- | ---: |
| ADAPTER | 4 |
| QUARANTINED | 2 |
| SUPERSEDED | 0 |

## 11. Experimental Evidence Registry

| Evidence source | Status | Permitted evidence | Forbidden authority | Required prerequisites | Promotion gate |
| --- | --- | --- | --- | --- | --- |
| `experiments/ui-shell-kit` | QUARANTINED | reference shell ideas, deterministic calculator shell evidence, focus/hit-test/paint/snapshot evidence, possible future ProjectionBundle player seed | production dependency, app authoring framework requirement, Semantic authority, admission authority, runtime policy, renderer/backend authority | Static UI IR, Binding Graph, Action IR, ActionIntent/admission seam, patch model, bundle parser basis, deterministic shell evidence | separate explicit architect approval after Gate B/C evidence |

Registry totals:

| Classification | Count |
| --- | ---: |
| QUARANTINED | 1 |

## 12. Superseded Authority Registry

| Historical authority | Status | Permitted use | Forbidden use | Revival rule |
| --- | --- | --- | --- | --- |
| R12/Aldente authority model | SUPERSEDED | historical context and separately mapped deterministic evidence | target owner, compile dependency, default UI DNA v2 authority, revival through compatibility adapter | requires a new explicit architecture decision; not part of WP1 |

Superseded authority rules:

- A superseded authority has no target owner.
- A superseded authority is not a compile dependency.
- A superseded authority cannot be revived through a compatibility adapter.
- Deterministic evidence associated with superseded work may be listed in the Compatibility or Experimental Evidence Registry only when mapped to an exact current owner.

Registry totals:

| Classification | Count |
| --- | ---: |
| SUPERSEDED | 1 |

## 13. Migration policy

| Source entity | Target owner | Migration rule | Required evidence | Stop condition |
| --- | --- | --- | --- | --- |
| `UiIr` | `crates/prom-ui::static_ir` | wrap or lower through an explicit v2 Static UI IR document contract | deterministic wrapper/lowering tests, serialization/digest evidence | legacy type treated as full v2 authority |
| `UiAst` | `crates/prom-ui::projection_source` | reuse implementation patterns only; do not alias identity | source AST/source-ref/diagnostic tests | Projection Source becomes a `UiAst` rename |
| `SemanticIntent` | `crates/prom-ui::compat` and `crates/prom-ui::action_intent` | explicit adapter preserving full context | positive and negative mapping tests | implicit context-erasing conversion |
| slot-intent metadata | `crates/prom-ui::binding_graph` | migrate carrier evidence to graph construction where useful | graph construction tests and migration cases | slot-intent chain becomes graph authority |
| ProjectionBundle fixtures | `crates/prom-ui-runtime::bundle::parser` | use as parser evidence only after parser basis approval | valid/invalid fixture tests | fixture reader claimed as parser/loader |
| `ui-shell-kit` | `crates/prom-ui-runtime::shell_player` evidence only | evaluate as experimental seed after prerequisites | deterministic shell evidence | implicit promotion |
| R12/Aldente deterministic evidence | exact current owner per case | reuse only through narrow owner-specific mapping | owner-specific regression or golden tests | authority model revived |

Migration must replace obsolete authority text rather than preserve parallel authority beside the new owner model.

## 14. Determinism ownership

| Determinism concern | Owner |
| --- | --- |
| contract identifier stability | `crates/prom-ui::contract_primitives` |
| source normalization | `crates/prom-ui::projection_source` |
| source diagnostic ordering | `crates/prom-ui::projection_source` |
| lowering order | `crates/prom-ui::projection_compile` |
| Static UI IR canonical ordering | `crates/prom-ui::static_ir` |
| Static UI IR serialization/digest | `crates/prom-ui::static_ir` |
| Binding Graph construction order | `crates/prom-ui::binding_graph` |
| patch replay order | `crates/prom-ui::projection_patch` |
| bundle interpretation/digest | `bundle::manifest`, `bundle::parser`, validators, `bundle::verify`, and `bundle::loader` by stage |
| shell snapshot determinism | `crates/prom-ui-runtime::shell_player` |
| backend-local rendering evidence | `crates/prom-ui-backend-native` |

Snapshot evidence must not be described as Semantic truth.
End-to-end determinism must not be assigned to one undifferentiated runtime layer.

## 15. ProjectionBundle claim levels

| Level | Name | Permitted claim | Forbidden claim | Required evidence | Owner |
| ---: | --- | --- | --- | --- | --- |
| 0 | Fixture evidence | inert fixture and draft-tool evidence exists | parser, validator, loader, activation, production readiness | fixture files, negative probes, claim-boundary docs | fixture evidence only |
| 1 | Parser basis | parser requirements and claim boundaries approved | parser implementation exists | parser-basis doc and negative fixture plan | `crates/prom-ui-runtime::bundle::parser` |
| 2 | Parser | parser consumes approved artifact format deterministically | validation, verification, loading, activation | parser tests and diagnostics | `crates/prom-ui-runtime::bundle::parser` |
| 3 | Structural validation | structural validator accepts/rejects parsed structure | compatibility, integrity, loading, activation | valid/invalid structural tests | `crates/prom-ui-runtime::bundle::validate::structural` |
| 4 | Compatibility validation | role/renderer/profile compatibility is checked | integrity, loading, activation | compatibility matrix tests | `crates/prom-ui-runtime::bundle::validate::compatibility` |
| 5 | Integrity/signature verification | bundle trust material is verified/rejected | Semantic admission authority, activation | hash/signature tests and rejection cases | `crates/prom-ui-runtime::bundle::verify` |
| 6 | Inert loading | verified bundle loads into inert representation | activation, runtime side effects, production UI | no-side-effect loader tests | `crates/prom-ui-runtime::bundle::loader` |
| 7 | Activation | safe activation gate is implemented | production promotion | safe update boundary and rollback/quarantine tests | `crates/prom-ui-runtime::bundle_activation` |
| 8 | Production promotion | production use is authorized under separate gate | automatic promotion from activation | release/promotion evidence and explicit architect approval | separate production promotion authority |

Parser is not validator. Validator is not verifier. Verifier is not loader. Loader is not activation. Activation is not production promotion.

## 16. Implementation work packages

This section defines large implementation work packages, not micro-PR sequencing.

Normative execution rule:

```text
one completed work package
  -> one final review
  -> one PR only after explicit publication authorization
```

An internal checkpoint is not a separate PR.
Gate B must be granted separately for every work package.

| Work package | Title | Includes | Required prior gate | Explicit non-goals |
| --- | --- | --- | --- | --- |
| UI-DNA2-WP2 | Projection Front-End and Static IR Foundation | `contract_primitives`, Role Dictionary, Projection Source tokens/AST/source refs/diagnostics, `projection_compile`, Static UI IR document wrapper, stable IDs, stable collection keys | Gate B for WP2 | no compiler activation, no shell, no renderer, no admission |
| UI-DNA2-WP3 | Binding, Action, and Admission Boundary | `semantic_refs`, Binding Graph, Action IR, `ActionIntent`, `admission_contract`, `SemanticIntent` compatibility mapper, `AdmissionPort` adapter, admitted-action dispatcher boundary | Gate B for WP3 after relevant WP2 contracts | no admission policy ownership, no runtime effects, no Semantic authority movement |
| UI-DNA2-WP4 | Projection State and Control Semantics | Projection Patch, denial/recovery, task projection, freshness/connectivity projection | Gate B for WP4 after relevant WP2/WP3 contracts | no renderer command streaming, no UI-local task engine, no offline critical action queue |
| UI-DNA2-WP5 | ProjectionBundle Qualification | parser basis, manifest, parser, structural validator, compatibility validator, integrity/signature verifier, inert loader, activation-boundary contract | Gate B for WP5 after required contracts | no live production promotion, no dynamic unchecked critical UI streaming |
| UI-DNA2-WP6 | Shell and Backend Evidence Alignment | shell player, patch application, focus, hit testing, accessibility realization, draw-command production, native backend boundary evidence, `ui-shell-kit` experimental evaluation | Gate B for WP6 after required contracts | no Workbench/Studio product work, no production promotion, no Semantic authority |

## 17. Implementation issue contract

Every future UI-DNA2 task must include the exact working directory:

```text
C:\Users\said3\Desktop\EXOcode\Semantic_phase1_prom_ui
```

Required preflight:

```powershell
Set-Location 'C:\Users\said3\Desktop\EXOcode\Semantic_phase1_prom_ui'
git rev-parse --show-toplevel
```

Root mismatch is a mandatory stop condition.

Every future task must include:

- work package and checkpoint ID;
- canonical owner;
- exact allowed files/modules;
- forbidden paths;
- authority impact;
- public API impact;
- compatibility impact;
- determinism requirements;
- positive tests;
- negative tests;
- golden evidence where relevant;
- no_std/alloc/std posture;
- migration rule;
- non-goals;
- stop conditions;
- completion report.

Crate-level ownership never authorizes crate-wide edits.
No task may invent or widen ownership for convenience.

## 18. Gates A-E

| Gate | Name | Meaning | Satisfied by WP1 approval |
| --- | --- | --- | --- |
| Gate A | architecture ownership freeze approved | canonical owners, dependency direction, compatibility posture, and activation constraints accepted | YES |
| Gate B | large implementation work package authorized | a bounded implementation work package is explicitly approved | NO |
| Gate C | implementation evidence accepted | local/CI tests and evidence for a work package are accepted | NO |
| Gate D | integration authorized | integration with adjacent layers is explicitly approved | NO |
| Gate E | production promotion authorized | public/production use is explicitly approved | NO |

UI-DNA2-WP1 approval satisfies Gate A only.

Gate A does not imply Gate B.

Implementation activation remains unauthorized.

## 19. Required non-changes

This freeze does not change:

- Semantic grammar;
- `.sm` parser;
- `.proj.sm` parser;
- compiler;
- verification;
- VM;
- capability model;
- admission implementation or behavior;
- runtime implementation or behavior;
- renderer behavior;
- native backend behavior;
- public Rust APIs;
- Cargo manifests;
- dependencies;
- features;
- tests;
- fixtures;
- CI;
- Workbench;
- Semantic Studio;
- `ui-shell-kit` status;
- ProjectionBundle implementation;
- GitHub state.

This freeze does not create:

- ADR files;
- separate compatibility registry files;
- separate experimental registry files;
- separate dependency graph documents;
- separate decision logs;
- separate Gate A reports;
- new UI-DNA2 specifications.

## 20. Final Gate A recommendation

READY FOR GATE A APPROVAL

Implementation activation remains unauthorized.
