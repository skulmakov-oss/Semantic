# Projection Source Model

Status: draft spec
Track: POST-UI / Intent-Driven Projection
Scope type: documentation only
Depends on:
- docs/dna/SEMANTIC_UI_DNA.md
- docs/dna/SEMANTIC_UI_DNA_v2.md
- docs/roadmap/post_ui/intent_driven_projection_roadmap.md
Related:
- #1310
- #1327
- #1328
- #1329
- #1330
- docs/spec/ui/projection_source_grammar_v0.md

The projection source model describes presentation intent for Semantic-owned UI projection.

It does not define semantic truth, verifier admission, VM behavior, runtime authority, renderer behavior, or production UI wiring.

The source model does not itself define parser grammar.

The bounded structural grammar contract is specified separately in
projection_source_grammar_v0.md.

The WP2C-P4 Grammar v0 parser/scanner is landed and remains crate-private.
WP2C-P5 adds only a crate-private, pure in-memory composition from source text
through that parser and the existing semantic compiler to Static UI IR.

## Source Role Representation Boundary

Projection Source AST stores authored role identifiers as owned source text.

RoleDictionary retains canonical static `RoleId` values.

Owned source text is resolved through RoleDictionary during Projection Source
semantic validation and lowering.

A syntactically valid unknown role remains representable in the source AST and
fails through `PS_UNKNOWN_ROLE`.

Source-role storage does not grant role validity, authority, runtime behavior,
renderer ownership, or admission.

```text
source role name != canonical RoleId
source storage != semantic acceptance
unknown source role != parser failure
known role resolution != authority
```

## Identifier Candidate and Role Resolution Boundary

Grammar v0 owns identifier candidate boundaries and parser diagnostic
selection. `RoleDictionary` owns known-role resolution after a complete parse.
A syntactically valid unknown role identifier such as `button` enters the AST
and later reports `PS_UNKNOWN_ROLE`. A malformed ASCII identifier candidate is
parser-owned and reports `PSP_INVALID_IDENTIFIER`; a forbidden non-ASCII scalar
reports `PSP_UNEXPECTED_CHAR` over that scalar.

```text
identifier candidate != accepted role
valid identifier != known role
malformed identifier != unknown role
PSP_INVALID_IDENTIFIER != PS_UNKNOWN_ROLE
parser diagnostic != semantic validation
tokenization != authority
```

The normative WP2C-P3 candidate, context, precedence and span rules remain in
Grammar v0 and are not duplicated here. The landed crate-private P4
parser/scanner owns its detailed tokenization and diagnostic selection. P5
composes that parser with the existing crate-private compiler without
publishing a parser or token API. Parse success remains distinct from semantic
validity.

```text
parser/scanner landed != public parser or token API
composition != file loading
composition != runtime loading
parse success != semantic validation
composition != activation or admission
composition != capability authority or shell mutation
composition != Gate D activation or production promotion
```

## Source Size and Provenance Representability

Projection Source provenance uses UTF-8 byte offsets represented by the landed
`SourceSpan` `u32` fields. The normative representability limit is
`u32::MAX = 4_294_967_295` source bytes. This model does not widen `SourceSpan`
to `u64`, `usize`, character offsets or line/column coordinates.

For the crate-private parser receiving `&str`, source-size preflight occurs
before parser ownership. A source longer than `u32::MAX` bytes is classified as
the input-domain error `ProjectionSourceInputError::SourceTooLarge`; it
produces no SourceSpan, PSP diagnostic, tokenization, AST or PS validation.
The conceptual error carries the caller-supplied `SourceId`, actual UTF-8 byte
length and maximum accepted byte length. The landed crate-private
implementation enforces this preflight without widening the public API.

The limit is a representability ceiling, not a recommended operational file
size. A future host or loader may impose a smaller memory, quota, transport or
sandbox limit, but that host resource rejection remains outside Grammar v0.

The normative representability ceiling is `u32::MAX` bytes on every platform.
On platforms where `usize` can represent values greater than `u32::MAX`, an
`&str` longer than `u32::MAX` bytes is rejected as `SourceTooLarge` before
lexical processing. On platforms where `usize` cannot represent a value
greater than `u32::MAX`, every constructible `&str` is within the
representability ceiling. The grammar contract does not vary with pointer
width; practical allocation capacity on a 32-bit platform is a separate
matter.

```text
normative maximum != usize::MAX
normative maximum != platform-dependent maximum
32-bit platform behavior != different grammar contract
64-bit platform capability != wider SourceSpan
```

After preflight succeeds, every parser position is in
`0..=input_byte_length` and therefore in `0..=u32::MAX`. Surface and node
declaration endpoints, token endpoints, zero-width missing-token positions,
duplicate-declaration keyword spans, offending-character or offending-token
spans, the EOF position and `PSP_UNEXPECTED_EOF` all use representable
`SourceSpan` endpoints. Representability is not validity and does not permit
unchecked conversion. Existing half-open span shapes remain unchanged.

```text
source byte length != character count
source-size acceptance != syntax validity
source-size acceptance != parse success
source-size acceptance != semantic validity
parse success != semantic validation success
semantic-validation success != runtime activation
input rejection != parser diagnostic
input rejection != runtime admission
provenance representability != authority
```

Source-size policy specified != parser implementation
source-size policy specified != parser qualification
source-size policy specified != runtime source loading
source-size policy specified != WP2C completion

The P1 source-size, P2 clause-context and P3 identifier-candidate diagnostic
contracts are landed and unchanged. The P4 parser/scanner implementation is
also landed and crate-private. P5 supplies only the pure in-memory composition
to the existing Static UI IR compiler. Repository state and ledger state grant
no runtime, activation, admission, capability or production authority.

No source-size check grants capability, performs admission, loads files,
allocates runtime buffers, activates a ProjectionBundle, mutates shell state,
selects a renderer, opens Gate D or claims production readiness.

## 1. Purpose

The projection source model exists to prevent the bad workflow:

```text
.sm + hand-written Rust UI glue
```

The intended workflow is:

```text
.sm owns meaning.
Projection source owns presentation intent.
Compiler emits UI IR.
Shell renders UI IR.
Semantic admission remains authoritative.
```

Projection source is a way to describe what should be projected without turning Semantic source into layout code.

## 2. Source File Posture

`.proj.sm` is the preferred v0 working name for projection source files.
The name is not a parser commitment until a grammar spec is approved.

A projection source is a companion to Semantic source.
It does not replace `.sm`.
It does not embed semantic business logic.
It does not override verifier or admission behavior.

### Inline projection rule

Inline projection inside `.sm` is forbidden in v0 unless separately approved by governance.

Semantic meaning must not be polluted by renderer, layout, or presentation lifecycle concerns.

## 3. Allowed Projection Intent

Projection source may express intent about:

- surfaces;
- semantic roles;
- state bindings;
- action affordances from admitted / action-offer sources;
- evidence outlets;
- denial outlets;
- recovery outlets;
- task projection contracts;
- freshness / connectivity display intent;
- accessibility labels and focus intent;
- priority / criticality hints;
- viewer-relative visibility policy.

These are intent declarations, not widgets.

Projection source may describe what a surface should present, what it should expose, and how it should be interpreted by non-visual and visual channels.

## 4. Forbidden Content

Projection source must not contain:

- business logic;
- verifier rules;
- admission rules;
- VM / runtime behavior;
- host effects;
- network calls;
- file system effects;
- renderer backend selection;
- absolute pixels;
- CSS-like layout;
- manual colors / fonts / themes;
- animation implementation;
- hand-written Rust UI glue;
- production shell wiring;
- dependency declarations;
- unsafe escape hatches.

Projection source is presentation intent, not a hidden implementation language.

## 5. Minimal Role Vocabulary

The following draft roles seed the projection vocabulary:

| Role | Meaning | Allowed use | Forbidden interpretation |
| --- | --- | --- | --- |
| `AppSurface` | top-level projected application surface | root surface container | renderer backend or app runtime |
| `Panel` | grouped projected region | structural partition | absolute layout object |
| `Section` | named area within a surface | logical grouping | business logic boundary |
| `FieldGroup` | related readouts or inputs | clustered form-like projection | widget toolkit ownership |
| `TextReadout` | textual projected value | labels, status, prose | semantic source of truth |
| `NumericReadout` | numeric projected value | counters, totals, results | computation engine |
| `StateBadge` | compact state indicator | status, mode, freshness | admission authority |
| `EvidencePanel` | evidence or trace surface | diagnostics, provenance, audit projection | audit authority |
| `DenialOutlet` | denial or refusal surface | projected denials, reasons, recovery hints | policy engine |
| `RecoveryOutlet` | recovery / retry projection | resume, retry, acknowledge | recovery implementation |
| `TaskPanel` | task-oriented projection area | progress, task state, controls | task engine |
| `ActionSlot` | location for an action affordance | button region or command affordance | direct effect execution |
| `SafeAction` | low-risk admitted action affordance | accepted control surface | bypass around capability checks |
| `GuardedAction` | constrained action affordance | capability-gated control | silent repeat or unchecked effect |
| `DangerAction` | high-risk action affordance | explicit risky control | implicit confirmation bypass |
| `ConnectivityBadge` | freshness or connection indicator | connected / stale / offline display | networking implementation |
| `List` | projected collection container | ordered item set | storage engine |
| `ListItem` | projected collection item | one stable item projection | anonymous visual fragment |

Roles are semantic projection roles, not renderer widgets.
A role must remain interpretable by non-visual surfaces such as CLI, logs, voice UI, or evidence reports.

## 6. Binding Model Boundary

Projection bindings observe Semantic state.
They do not mutate Semantic state directly.
Mutation attempts must become structured `ActionIntent` candidates and pass admission.

Binding categories include:

- state binding;
- evidence binding;
- action offer binding;
- task binding;
- connectivity binding.

Projection bindings describe relationships and visibility, not mutation behavior or implementation control flow.

## 7. Action Affordance Boundary

Projection source may declare where action affordances appear.
It must not invent actions.
It must not bypass `ActionOffers`, capability checks, or admission.

```text
UI proposes.
Semantic disposes.
Shell shows.
```

Action affordance declarations describe route and presentation, not execution authority.

## 8. Accessibility as Projection Contract

Accessibility is part of the source model, not renderer polish.

Accessibility declarations should include:

- human-readable label;
- role description;
- focus order intent;
- criticality;
- denial / recovery discoverability;
- non-visual interpretation;
- evidence provenance where practical.

Accessibility intent must remain visible in the projection contract even when the final renderer differs.

## 9. Quad-State Preservation

Projection source must preserve:

- `N` — unknown
- `F` — false
- `T` — true
- `S` — conflict

Projection source must not flatten Quad-state into boolean visibility, success/failure, or generic disabled UI.

Unknown remains unknown.
Conflict remains conflict.
Denial is not the same as false.

## 10. Non-Normative Example

Non-normative sketch — not parser syntax.
This sketch remains non-normative and may contain concepts outside grammar v0.

```text
projection CalculatorView for CalculatorState {
  surface main role AppSurface {
    readout display role NumericReadout bind result
    action add role SafeAction from ActionOffers.add
    evidence outlet EvidencePanel
    denial outlet DenialOutlet
  }
}
```

This sketch only illustrates intent flow and role placement.
It is not final grammar, not an implementation, and not a parser commitment.

## 11. Acceptance Criteria

The spec is acceptable when:

- it defines `.proj.sm` as preferred working name;
- it explains relation to `.sm`;
- it blocks inline projection in v0;
- it defines allowed projection intent;
- it defines forbidden content;
- it includes draft role vocabulary;
- it preserves Semantic authority;
- it preserves Quad-state meaning;
- it treats accessibility as contract;
- it does not implement parser grammar;
- it does not claim production readiness.

## 12. Explicit CollectionAnchor Declarations

Status: normative contract freeze.
Scope type: documentation only.
This section amends this document. It is not a second specification.

This section freezes the ownership, identity, validation, ordering,
diagnostic, resource, integration, mutation, and authority contract for
explicit projection-owned `CollectionAnchor` declarations. It removes the
remaining semantic blocker recorded against future
`PreparedActiveProjectionTargets` and `ActiveProjectionTargetCatalog`
implementation in `docs/spec/ui/shell_player_session_state_v0.md`.

This section does not implement Rust, does not modify parser grammar, does
not add public API, and does not modify the public API guard. It does not
authorize implementation of `PreparedActiveProjectionTargets`,
`ActiveProjectionTargetCatalog`, or the stage-5 evaluator.

### Ownership

- Projection Source owns authored collection-anchor declaration intent.
- `prom-ui` owns validation and deterministic lowering of that intent into
  projection-owned structural declaration evidence.
- The validated Static UI structural layer owns the stable target node
  identities against which declarations are qualified.
- `PreparedActiveProjectionTargets` may consume only the successfully
  qualified collection-anchor declaration set.
- `prom-ui-runtime` does not own source declarations, declaration
  validation, declaration lowering, or declaration identity.
- `ActiveProjectionTargetCatalog` remains owned by `prom-ui-runtime` only
  after prepared activation evidence crosses the controlled handoff.

Neither a composition caller nor Shell Player may become:

- declaration author;
- declaration validator;
- declaration lowerer;
- collection-anchor identity owner;
- projection source owner;
- Semantic authority;
- action-authorization authority.

No reverse dependency is authorized. No shared crate is authorized.

### The `CollectionAnchorDeclaration` concept

`CollectionAnchorDeclaration` is the conceptual name for one authored
declaration. A declaration means only:

```text
the identified projection-owned static node is explicitly declared and
qualified as an eligible collection patch target for local projection playback
```

This eligibility is projection-structural only. It is not Semantic admission,
verifier admission, capability admission, bundle admission, patch admission,
action admission, runtime activation, or effect authorization.

A declaration does not mean:

- the node currently contains collection elements;
- a Semantic collection exists;
- a `BindingValueDomain::Collection` binding exists;
- the node has a collection-like role name;
- a `CollectionKey` identifies the anchor;
- a collection patch has already targeted it;
- the collection is visible;
- the collection is mutable;
- an action is authorized;
- a patch is valid;
- a patch may be applied.

A declaration is projection structure, not runtime state.

### Source and lowered identities

Two identity stages are distinguished.

**Authored source target.** At the Projection Source level, an authored
declaration targets exactly one projection source node identity. Conceptually
this is a `ProjectionSourceNodeId`. This contract does not freeze grammar
syntax or an AST field name. The declaration must refer to a node declared in
the same `ProjectionSourceDocument`. A declaration must not target:

- a surface ID;
- a role name;
- a `CollectionKey`;
- a `BindingId`;
- a `BindingSlot`;
- a patch operation;
- an arbitrary numeric ID outside the source document.

**Qualified static target.** After deterministic lowering, the qualified
anchor identity is `StaticDocumentId` + `StaticNodeId`. The declaration set is
also bound to the corresponding `Revision` and `Epoch`.

The following are frozen explicitly:

- `CollectionAnchor` identity is the static target node within one exact
  static document version.
- `CollectionKey` is not part of `CollectionAnchor` identity.
- `SourceSpan` is provenance, not identity.
- Authored declaration order is not identity.
- Storage position is not identity.
- Map iteration position is not identity.

This contract does not freeze an assumption that raw `ProjectionSourceNodeId`
and `StaticNodeId` values must always remain numerically identical. The
compiler owns deterministic source-node-to-static-node mapping.

### The qualified declaration set

`QualifiedCollectionAnchorDeclarations` is the conceptual name for the set of
all successfully qualified collection anchors for exactly one validated
projection-owned static document version. It is bound coherently to
`StaticDocumentId`, `Revision`, and `Epoch`. It contains only stable
collection target coordinates. It is:

- immutable after qualification;
- deterministically ordered;
- local and reconstructible;
- non-authoritative;
- not caller-authored after qualification;
- not the runtime catalog;
- not prepared activation evidence by itself.

It must not be mixed across documents, revisions, epochs, or projection
activations. An empty declaration set is valid.

### Validation rules

Validation is fail-closed and deterministic.

**Rule 1 — Source target existence.** Every authored declaration target must
resolve to exactly one node declared in the same `ProjectionSourceDocument`.
A missing target rejects the declaration set and emits
`CAD_MISSING_TARGET_NODE`.

**Rule 2 — Duplicate declaration.** The same qualified static node must not
be declared more than once. A duplicate declaration rejects the declaration
set and emits `CAD_DUPLICATE_DECLARATION`. Duplicate detection is by
qualified collection-anchor identity, not by `SourceRef`, `CollectionKey`,
role, authored order, or object identity.

**Rule 3 — Static target existence.** After lowering, every qualified
declaration must reference a node present in the same validated
`StaticUiDocument`. Failure rejects the declaration set and emits
`CAD_MISSING_STATIC_NODE`. This is a defensive compiler/adapter coherence
diagnostic.

**Rule 4 — Document-version coherence.** Declarations and the target Static
UI document must agree on `StaticDocumentId`, `Revision`, and `Epoch`. A
mismatch rejects the declaration set and emits
`CAD_DOCUMENT_VERSION_MISMATCH`.

**Rule 5 — No inferred declarations.** A node must not become a
`CollectionAnchor` merely because of: `StaticNodeId` existence,
`ProjectionSourceNode` existence, `CollectionKey` existence, role text, a
`List`-like role, `BindingValueDomain::Collection`, `collection_key` on a
`BindingDeclaration`, `CollectionInsert`, `CollectionUpdate`,
`CollectionRemove`, `CollectionMove`, prior patch history, caller assertion,
host state, backend state, map membership, or map iteration order.

**Rule 6 — No extra role requirement.** A valid explicit declaration does not
additionally require a specific role name or binding domain unless a later
independent contract introduces such a requirement. This contract does not
silently add a `List`-role requirement.

**Rule 7 — No new reachability rule.** Collection-anchor validation must not
invent a new surface-reachability rule. It relies on the already validated
projection-owned structural document. This contract does not weaken existing
Static UI IR validation.

### Deterministic ordering

Qualified collection anchors must be ordered solely by stable qualified
target identity. The normative v0 order is: ascending `StaticNodeId` within
the one bound `StaticDocumentId`/version. Because the set is bound to one
document version, the document identity does not need to be repeated as a
per-entry sort component.

Forbidden ordering sources:

- authored declaration order;
- `Vec` insertion order;
- `HashMap` or `HashSet` iteration;
- source byte order as final identity order;
- role name;
- `CollectionKey`;
- patch order;
- runtime discovery order;
- backend order;
- host order;
- pointer address.

Qualification must produce the same ordered set for equivalent declarations
regardless of authored declaration order. This contract does not freeze a
Rust collection type.

### Diagnostic ownership

The declaration diagnostics belong to `prom-ui` projection qualification.
They are not Shell Player `SPV0_*` diagnostics. The following stable codes
are frozen:

| Code | Class |
| --- | --- |
| `CAD_MISSING_TARGET_NODE` | An authored declaration target does not resolve to a node in the same `ProjectionSourceDocument`. |
| `CAD_DUPLICATE_DECLARATION` | The same qualified static node is declared more than once. |
| `CAD_MISSING_STATIC_NODE` | A qualified declaration references a node absent from the validated `StaticUiDocument`. |
| `CAD_DOCUMENT_VERSION_MISMATCH` | Declarations and the target Static UI document disagree on `StaticDocumentId`, `Revision`, or `Epoch`. |

Diagnostic ordering must be deterministic. Conceptual ordering is consistent
with the existing diagnostic-coordinate posture: source provenance when
present, then stable diagnostic code, then primary stable node identity, then
secondary coordinate. Exact Rust diagnostic structs and field layouts remain
unresolved.

Any declaration diagnostic means: no `QualifiedCollectionAnchorDeclarations`
value, no `PreparedActiveProjectionTargets` value from that input, and no
runtime catalog construction from that input. No partial declaration set may
escape.

`CAD_*` diagnostics are not mapped to `SPV0_INVALID_TARGET`. `SPV0_INVALID_TARGET`
remains a stage-5 runtime membership failure after a valid catalog already
exists; `CAD_*` diagnostics are projection-qualification-time failures that
occur before any catalog exists.

### Resource posture

The v0 declaration set is structurally bounded by the number of unique nodes
in the validated static document. Therefore:

```text
qualified declaration count <= validated static node count
```

This contract does not introduce a separate host resource limit, an
arbitrary declaration quota, a new `ShellLifecycleLimits` field, or a stage-4
diagnostic. A future smaller operational limit requires a separate contract.
Representation-size overflow and allocation strategy remain unresolved.

### `PreparedActiveProjectionTargets` integration

`PreparedActiveProjectionTargets` obtains `CollectionAnchor` coordinates only
from `QualifiedCollectionAnchorDeclarations`. No other source may contribute
`CollectionAnchor` membership. The collection-anchor coordinates retain the
qualified deterministic order. `PreparedActiveProjectionTargets` may combine
the qualified collection anchors with independently qualified `NodeAnchor`
and `BindingAnchor` evidence, but it must not reinterpret or regenerate
collection declarations.

Absence rules:

- no explicit declaration means no `CollectionAnchor` membership;
- a later collection patch targeting that node means stage-5 membership
  rejection once the runtime catalog exists;
- a `CollectionKey` or collection patch operation never creates declaration
  evidence.

This contract does not authorize implementation of
`PreparedActiveProjectionTargets`, `ActiveProjectionTargetCatalog`, or the
stage-5 evaluator.

### Mutation and lifetime posture

Declarations belong to one projection structural version. They are not
mutated by `ProjectionPatch`, `ShellSession`, `ActiveProjectionTargetCatalog`,
collection inserts, collection updates, collection removals, collection
moves, renderer activity, backend activity, focus state, or event handling.

Changing declarations requires a newly qualified projection structural
version. A patch must not add or remove `CollectionAnchor` declarations. A
new activation may transport a new prepared declaration set only from a
newly qualified projection version.

### Authority boundaries

```text
CollectionAnchor declaration != Semantic collection truth
CollectionAnchor declaration != collection contents
CollectionAnchor declaration != action authorization
CollectionAnchor declaration != patch admission
CollectionAnchor declaration != patch application
CollectionAnchor declaration != renderer command
CollectionAnchor declaration != backend resource
CollectionAnchor declaration != runtime discovery
CollectionAnchor declaration != bundle trust
```

Declaration qualification does not load a bundle, activate a bundle, create
a `ShellSession`, construct the runtime catalog, validate replay
compatibility, apply a patch, advance the replay cursor, calculate candidate
state, authorize effects, or move Gate D.

### Explicitly unresolved

The following remain unresolved and unauthorized by this section:

- Projection Source grammar syntax;
- parser tokens;
- source AST field names;
- Rust declaration type names;
- Rust declaration-set type names;
- module paths;
- field layouts;
- constructors;
- visibility;
- public API;
- public re-exports;
- serialization;
- ABI;
- canonical byte format;
- source-to-static mapping representation;
- diagnostic Rust representation;
- storage collection;
- allocation strategy;
- representation-size overflow behavior;
- prepared activation producer;
- runtime catalog implementation;
- `ActivatedShellSessionContext` expansion;
- stage-5 evaluator;
- stage-5/stage-6 orchestration.

This section does not claim implementation readiness for any of the above.
