# Binding Graph Semantic Observation Adapter V0

Status: NORMATIVE V0 CONTRACT; CRATE-PRIVATE IMPLEMENTATION AND QUALIFICATION
INCLUDED IN THIS BOUNDED CHANGE

## 1. Purpose

This specification freezes the deterministic, pure in-memory adapter that
maps caller-supplied observations keyed by `SemanticValueRef` into the exact
crate-private inputs consumed by the Binding Graph observation/dirty engine.

The adapter does not obtain Semantic values. It does not decide whether an
observation is authoritative, fresh or admissible. It only validates the
supplied carrier, matches references within a caller-selected Binding scope
and constructs inert `BindingObservation` values.

Conceptual flow:

```text
validated BindingGraphDocument
+ caller-selected BindingObservationScope
+ previous SemanticBindingObservationSet
+ current SemanticBindingObservationSet
+ caller-supplied SemanticBindingAdapterLimits

→ deterministic reference matching
→ structural and carrier validation
→ canonical BindingObservation inputs
```

The adapter does not invoke dirty derivation. Its output may be supplied to
`derive_dirty_bindings` by a caller as a separate operation.

## 2. Authority and ownership

Semantic owns the observed truth.

The caller owns:

- obtaining Semantic observations;
- deciding whether those observations are authoritative;
- supplying `Epoch` and `Revision`;
- selecting the Binding scope.

The adapter owns only:

- deterministic reference matching;
- structural validation;
- exact value-carrier mapping;
- construction of inert Binding Graph observation inputs.

Required boundaries:

```text
adapter input = caller-supplied evidence
adapter success != Semantic truth
reference match != authority
Epoch/Revision transport != freshness validation
adapter output != dirty result
dirty result != UI mutation permission
adapter output != Projection Patch
adapter output != admission
implementation != public API
implementation != runtime integration
```

## 3. Conceptual types

### 3.1 SemanticBindingObservation

```text
SemanticBindingObservation
  semantic_value_ref: SemanticValueRef
  epoch: Epoch
  revision: Revision
  quad_state: optional exact BindingQuadState
```

`Epoch`, `Revision` and `BindingQuadState` are transported exactly. The
adapter must not infer, normalize or rewrite them.

### 3.2 SemanticBindingObservationSet

```text
SemanticBindingObservationSet
  caller-supplied observations keyed by SemanticValueRef
```

Input storage order is not semantic. Each snapshot is canonicalized by
`SemanticValueRef`. A duplicate reference within one snapshot is invalid.

### 3.3 SemanticBindingAdapterLimits

```text
SemanticBindingAdapterLimits
  max_scoped_bindings
  max_observations_per_snapshot
  max_indexed_semantic_references
  max_fanout_mappings
  max_mapped_observations_per_snapshot
```

Every limit is caller-supplied. A resource rejection does not classify a
Semantic observation as false, stale or unauthorized.

### 3.4 AdaptedBindingObservationInputs

```text
AdaptedBindingObservationInputs
  canonical BindingObservationScope
  previous BindingObservationSet
  current BindingObservationSet
  deterministic fanout_work count
```

The result is inert. It contains no runtime command, dirty result,
`ProjectionPatch`, admission result or authority token.

## 4. Scope contract

The scope contains exact `BindingId` values.

Required behavior:

- scope IDs are canonicalized in ascending `BindingId` order;
- duplicate scope IDs are invalid;
- unknown scope IDs are invalid;
- every scoped Binding must contain `semantic_value`;
- a scoped Binding without `semantic_value` is not silently omitted;
- previous and current snapshots are interpreted against the same scope.

Bindings outside the selected scope are not mapped. Selecting a scope does
not grant authority over the referenced Semantic values.

## 5. Semantic reference index and fanout

The adapter builds deterministic evidence sufficient to distinguish graph
membership from scope membership, followed by this scoped mapping:

```text
SemanticValueRef → one or more scoped BindingId values
```

Rules:

- one Semantic observation may fan out to multiple scoped Bindings;
- fanout Binding IDs are ascending;
- the same reference may map to multiple non-Quad domains;
- the same reference may map to multiple Quad Bindings;
- a reference shared by Quad and non-Quad scoped Bindings is incompatible;
- incompatible fanout is rejected before snapshot payload mapping;
- fanout work is one unit per unique `(SemanticValueRef, scoped BindingId)`
  pair.

For an input Semantic observation:

```text
reference unused by every Binding in the graph
→ BGSA_UNKNOWN_SEMANTIC_REF

reference exists in the graph but no scoped Binding uses it
→ BGSA_OBSERVATION_OUT_OF_SCOPE
```

This classification is structural. It does not resolve or validate Semantic
truth.

## 6. Snapshot mapping

For each previous and current snapshot independently:

```text
one SemanticValueRef observation
→ one BindingObservation for every scoped Binding using that reference
```

The adapter preserves exactly:

- `Epoch`;
- `Revision`;
- `N`;
- `F`;
- `T`;
- `S`.

Absence of one Semantic observation means absence for every scoped Binding
mapped to that reference in that snapshot. Previous and current absence are
preserved independently. The adapter must not fabricate tombstones,
revisions or Quad values.

## 7. Domain carrier rules

Fanout domain determines the required carrier:

```text
Quad-only fanout     → quad_state required
non-Quad-only fanout → quad_state forbidden
Quad + non-Quad      → incompatible fanout-domain error
```

The exact four-state carrier is preserved:

```text
N remains N
F remains F
T remains T
S remains S
```

Binary collapse, absence normalization and interpreting `S` as an adapter
error are forbidden.

## 8. Deterministic stages and precedence

The stage order is normative:

1. resource preflight;
2. scope validation;
3. Semantic-reference index and fanout validation;
4. snapshot structural validation;
5. domain-carrier validation;
6. canonical observation mapping.

Only diagnostics from the earliest failing stage are returned. Within that
stage diagnostics are sorted canonically and deduplicated. Failure produces
no partial `AdaptedBindingObservationInputs`.

Stable conceptual codes:

| Code | Owning stage |
| --- | --- |
| `BGSA_RESOURCE_LIMIT_EXCEEDED` | resource preflight or bounded reference-index construction |
| `BGSA_DUPLICATE_SCOPE_BINDING` | scope validation |
| `BGSA_UNKNOWN_SCOPE_BINDING` | scope validation |
| `BGSA_BINDING_MISSING_SEMANTIC_REF` | scope validation |
| `BGSA_INCOMPATIBLE_FANOUT_DOMAIN` | reference-index and fanout validation |
| `BGSA_DUPLICATE_SEMANTIC_OBSERVATION` | snapshot structural validation |
| `BGSA_UNKNOWN_SEMANTIC_REF` | snapshot structural validation |
| `BGSA_OBSERVATION_OUT_OF_SCOPE` | snapshot structural validation |
| `BGSA_MISSING_QUAD_STATE` | domain-carrier validation |
| `BGSA_UNEXPECTED_QUAD_STATE` | domain-carrier validation |
| `BGSA_FANOUT_LIMIT_EXCEEDED` | reference-index and fanout validation |
| `BGSA_MAPPED_OBSERVATION_LIMIT_EXCEEDED` | canonical observation mapping |

An error exposes deterministic evidence where applicable:

- stage and code;
- `BindingId`;
- `SemanticValueRef`;
- previous/current snapshot side;
- limit kind;
- actual and maximum values.

Exact Rust enum names remain implementation-owned. The codes and stage
classification do not.

## 9. Resource behavior

Conforming implementations must use deterministic `no_std + alloc`
structures. Randomized hashing is forbidden.

Required behavior:

- scope and snapshot lengths are checked before proportional allocation;
- arithmetic is checked;
- the reference-index limit is checked before retaining an exceeding unique
  reference;
- the fanout limit is checked before retaining the exceeding mapping;
- mapped-observation limits are checked before inserting the exceeding
  observation;
- input order does not affect success, failure or output;
- caller-controlled input must not cause a panic;
- failure returns no partial result.

The same graph, scope, snapshots and limits must always produce the same
result or the same canonical diagnostics.

## 10. Dirty-engine composition

Adapter output may supply the scope, previous snapshot and current snapshot
to the landed crate-private Binding Graph observation/dirty engine.

The adapter does not perform:

- lexicographic `(Epoch, Revision)` temporal validation;
- stale-observation rejection;
- conflicting-replay rejection;
- dirty seed derivation;
- reverse-dependency propagation.

Those behaviors remain owned by `derive_dirty_bindings` and its frozen
contract. Adapter mapping does not alter the dirty engine's temporal,
propagation, ordering or resource semantics.

## 11. Implementation and qualification mapping

| Contract role | Repository file |
| --- | --- |
| validated Binding Graph declarations | `crates/prom-ui/src/binding_graph.rs` |
| exact Binding observation carriers and dirty engine | `crates/prom-ui/src/binding_graph_observation.rs` |
| Semantic-reference adapter | `crates/prom-ui/src/binding_graph_semantic_adapter.rs` |
| executable adapter and composition qualification | `crates/prom-ui/src/ui_dna2_binding_semantic_adapter_qualification_tests.rs` |
| crate-private module registration | `crates/prom-ui/src/lib.rs` |

Qualification covers scope and reference classification, storage-order
invariance, fanout, resource precedence, exact Epoch/Revision transport,
exhaustive `N/F/T/S` carrier preservation and adapter-to-engine composition.

## 12. Explicit non-goals and final posture

This bounded change does not authorize:

- live Semantic reads or subscriptions;
- validation of Semantic truth or caller authority;
- Semantic freshness authority;
- a public adapter or observation API;
- runtime consumption;
- Action IR admission or dispatch;
- `ProjectionPatch` construction or application;
- UI mutation;
- filesystem or runtime loading;
- Gate D transition;
- production promotion.

Final posture:

```text
Binding Graph Semantic observation adapter v0 contract = FROZEN
crate-private adapter implementation and qualification = INCLUDED IN THIS BOUNDED CHANGE
adapter input = CALLER-SUPPLIED EVIDENCE
live Semantic reads = ABSENT
public adapter API = ABSENT
runtime consumption = NOT AUTHORIZED
Projection Patch integration = NOT AUTHORIZED
AUTHORIZED SLICE CONSUMED BY THIS CHANGE:
Binding Graph Semantic observation adapter implementation and qualification
FOLLOW-ON AUTHORIZED IMPLEMENTATION SLICE = NONE
Gate D = CLOSED
production promotion = NOT AUTHORIZED
```

This implementation evidence consumes only the explicit authorization for
the adapter. It does not authorize runtime integration or another slice.
