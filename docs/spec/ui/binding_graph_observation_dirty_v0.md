# Binding Graph Observation and Dirty Propagation V0

Status: NORMATIVE CONTRACT FREEZE; IMPLEMENTATION NOT YET AUTHORIZED
Track: UI DNA v2
Owner: `prom-ui::binding_graph`
Runtime activation: NOT AUTHORIZED

## 1. Purpose

This document freezes the deterministic crate-private v0 contract for
converting caller-supplied Binding Graph observations into a canonical dirty
Binding set.

The conceptual flow is:

```text
validated BindingGraphDocument
+ previous observation set
+ current observation set
+ caller-supplied resource limits

→ resource and content validation
→ temporal classification
→ deterministic dirty seeds
→ dependency propagation
→ canonical dirty result
```

This is a contract freeze, not a Rust implementation or public API. It defines
observable behavior that a future separately authorized implementation must
qualify without selecting its internal data structures or traversal algorithm.

## 2. Ownership and authority

`prom-ui::binding_graph` owns:

- Binding declarations;
- dependency structure;
- observation comparison contract;
- dirty derivation;
- deterministic propagation evidence.

Semantic source adapters own:

- obtaining authoritative Semantic values;
- assigning observation revision and epoch;
- selecting the observed scope.

`prom-ui-runtime` owns:

- runtime consumption;
- shell mutation;
- Projection Patch application;
- activation.

Required authority distinctions:

```text
Binding Graph observation != Semantic observation authority
observation possession != authority over observed meaning
dirty result != permission to mutate UI
dirty result != admission
dirty result != Projection Patch
dirty derivation != Projection Patch application
verification or derivation != loading
loading != activation
```

The caller remains responsible for selecting a valid observation scope and for
supplying observations obtained through its authoritative boundary. The
Binding Graph compares caller-supplied evidence; it does not read Semantic
state or decide which Semantic facts are authoritative.

## 3. Conceptual v0 types

The following are conceptual contract types. Exact Rust names, field layout and
module visibility remain implementation-owned.

```text
BindingObservation
  binding_id: BindingId
  epoch: Epoch
  revision: Revision
  quad_state: optional exact Quad N/F/T/S carrier

BindingObservationSet
  complete caller-selected observation snapshot
  unique observations keyed by BindingId

DirtyReason
  reason kind
  originating seed BindingId
  direct or propagated evidence

DirtyBinding
  binding_id: BindingId
  canonical DirtyReason collection

DirtyDerivationResult
  canonical DirtyBinding collection
  deterministic reason evidence
  no runtime commands
```

For v0, a non-Quad observation carries change identity through its epoch and
revision tuple. The optional payload is reserved for the exact Quad state and
is forbidden for non-Quad bindings. Payload equality therefore means exact
equality of the optional Quad carrier; no hidden scalar, text or collection
value comparison is implied.

No type in this contract is a public API claim.

## 4. Snapshot and scope contract

Each `BindingObservationSet` is a complete snapshot for one caller-selected
scope. The previous and current sets supplied to one derivation must describe
the same scope.

The scope may contain all graph bindings or a caller-selected subset. The
scope selection remains caller-owned and does not transfer Semantic authority
to the Binding Graph. A binding absent from both snapshots is outside the
comparison. A binding present in only one snapshot is an added or removed
observation, not an implicit unknown value.

The contract does not define a filesystem source, runtime subscription or live
Semantic read for either snapshot.

## 5. Observation structural and domain rules

Before temporal comparison:

1. each observation set is canonicalized by ascending `BindingId`;
2. every `BindingId` is unique within its set;
3. every observation refers to a Binding declared by the supplied validated
   `BindingGraphDocument`;
4. every Quad-domain Binding carries exactly one explicit `N`, `F`, `T` or `S`;
5. every non-Quad Binding omits the Quad carrier.

Violations are errors:

```text
duplicate BindingId → duplicate observation
undeclared BindingId → unknown binding
Quad binding without quad_state → missing Quad state
non-Quad binding with quad_state → unexpected Quad state
```

Input storage order must not affect validation outcome, temporal
classification, dirty membership, reason evidence or canonical output.
Equivalent duplicate diagnostics are coalesced canonically rather than exposed
in caller storage order.

Added and removed observations are meaningful changes. Neither is silently
normalized to unknown or absence of evidence.

## 6. Temporal comparison

Temporal comparison is performed independently per `BindingId` over the union
of the previous and current canonical snapshot keys.

For a Binding present in both snapshots, compare `(epoch, revision)`
lexicographically:

```text
current tuple < previous tuple
→ stale observation rejection

current tuple == previous tuple and payload equal
→ exact replay / unchanged

current tuple == previous tuple and payload differs
→ conflicting replay rejection

current tuple > previous tuple
→ changed observation

previous absent and current present
→ added observation

previous present and current absent
→ removed observation
```

An epoch increase may reset revision numbering. A revision decrease inside the
same epoch is stale. A lower epoch is stale regardless of revision. Equal
temporal identity with different payload is conflicting evidence, not a new
revision.

Temporal validation covers the complete compared scope before dirty seed
derivation. Any stale observation or conflicting replay rejects the entire
derivation and produces no partial `DirtyDerivationResult`.

## 7. Exact Quad preservation

For `BindingValueDomain::Quad`, comparison and evidence preserve the exact
four-state value:

```text
N remains N
F remains F
T remains T
S remains S
```

The following are forbidden:

- `N → false`;
- `S → true`;
- unknown → absent;
- conflict → error merely because the value is `S`;
- binary collapse;
- implicit normalization.

`N`, `F`, `T` and `S` are valid distinct payloads. A transition between any
two distinct Quad states is `QuadValueChanged` when the temporal tuple also
permits a changed observation. Dirty reason evidence retains the previous and
current exact states where that evidence is exposed; it must not retain a
collapsed boolean substitute.

## 8. Direct dirty seeds

A direct dirty seed is produced for every applicable direct reason:

- `Added`;
- `Removed`;
- `RevisionChanged`;
- `EpochChanged`;
- `QuadValueChanged`.

`Added` and `Removed` are exclusive with tuple-comparison reasons. For a
Binding present in both snapshots, all applicable changed evidence is retained:
an epoch change, revision-number change and Quad-state change may therefore be
coalesced into one dirty Binding with multiple direct reasons.

No direct dirty seed is produced for:

- `Unchanged`;
- `ExactReplay`.

Stale observations and conflicting replays are errors, not dirty reasons.

## 9. Dependency propagation

The existing `BindingDeclaration.dependencies` direction remains normative:

```text
if B.dependencies contains A,
then B depends on A,
and a dirty A propagates to B.
```

If Binding A changes, every Binding transitively dependent on A becomes dirty.
Propagation must be semantically equivalent to a deterministic fixed point
over reverse dependency edges.

Required behavior:

- no duplicate dirty bindings;
- multiple origins are coalesced;
- every originating seed remains visible in reason evidence;
- every applicable direct reason of an origin remains visible;
- cycles are not handled here because `BindingGraphDocument` is already
  validated and acyclic;
- input order does not affect the result;
- propagation produces no runtime command or patch operation.

This contract does not freeze breadth-first search, depth-first search, queue
shape, index representation or another internal algorithm.

## 10. Canonical result and evidence order

The final dirty collection is ordered by ascending `BindingId`.

Within each dirty Binding, reasons are ordered by:

1. reason kind;
2. originating seed `BindingId`.

The v0 reason-kind order is:

```text
Added
Removed
RevisionChanged
EpochChanged
QuadValueChanged
```

Direct evidence is attached to its own seed Binding. Propagated evidence
retains the same reason kind and originating seed Binding. Equivalent reason
records are coalesced. Canonicalization must not erase a distinct origin or a
distinct direct reason.

`DirtyDerivationResult` contains evidence only. It contains no shell command,
renderer command, ActionIntent, admission verdict or Projection Patch.

## 11. Error stages and precedence

The required stage order is:

```text
1. resource preflight
2. observation structural validation
3. domain and Quad carrier validation
4. temporal validation
5. dirty seed derivation
6. dependency propagation
7. canonical result construction
```

Precedence rules:

- resource failure precedes content validation;
- structural diagnostics precede domain diagnostics;
- structural and domain diagnostics precede temporal diagnostics;
- seed derivation begins only after temporal validation succeeds for the
  complete compared scope;
- failure at any stage produces no partial `DirtyDerivationResult`;
- diagnostics are coalesced by conceptual identity and ordered by stage,
  optional `BindingId`, then error identity.

Stable conceptual error identities are:

```text
BGOD_RESOURCE_LIMIT_EXCEEDED
BGOD_DUPLICATE_OBSERVATION
BGOD_UNKNOWN_BINDING
BGOD_MISSING_QUAD_STATE
BGOD_UNEXPECTED_QUAD_STATE
BGOD_STALE_OBSERVATION
BGOD_CONFLICTING_REPLAY
BGOD_DIRTY_RESULT_LIMIT_EXCEEDED
BGOD_PROPAGATION_WORK_LIMIT_EXCEEDED
```

These identities are normative classifications. Exact Rust enum and variant
names remain implementation-owned but must map to these distinctions without
collapsing their meaning or precedence.

## 12. Caller-supplied resource limits

The caller supplies finite limits for at least:

- maximum observations per snapshot;
- maximum dirty bindings;
- maximum propagation-edge visits;
- maximum retained dirty reasons.

Rules:

- each snapshot count is checked before sorting or allocation proportional to
  untrusted input;
- combined accounting uses checked arithmetic;
- dirty-Binding capacity is checked before inserting a new result;
- every reverse dependency edge visit consumes propagation work;
- retained reason capacity is checked before adding distinct evidence;
- limits are checked before unbounded growth;
- unchecked arithmetic is forbidden;
- a limit failure rejects the complete derivation and returns no partial
  result.

Required distinctions:

```text
host quota rejection != invalid BindingGraphDocument
host quota rejection != invalid observation structure
host quota rejection != stale observation
dirty-result limit != propagation-work limit
```

Exceeding observation or reason capacity is
`BGOD_RESOURCE_LIMIT_EXCEEDED`. Exceeding dirty-Binding capacity is
`BGOD_DIRTY_RESULT_LIMIT_EXCEEDED`. Exceeding reverse-edge work is
`BGOD_PROPAGATION_WORK_LIMIT_EXCEEDED`.

## 13. Explicit non-goals and final posture

This contract does not authorize:

- Rust implementation;
- Semantic source adapters;
- live Semantic reads;
- Action IR admission;
- ActionIntent dispatch;
- Projection Patch construction or application;
- shell-local state;
- runtime mutation;
- renderer commands;
- filesystem loading;
- runtime loading;
- public API;
- Gate D transition;
- production promotion.

Final posture:

```text
Binding Graph observation contract = FROZEN
Binding Graph dirty engine implementation = NOT YET AUTHORIZED
public Binding Graph observation API = ABSENT
runtime consumption = NOT AUTHORIZED
Projection Patch application = NOT AUTHORIZED
Gate D = CLOSED
production promotion = NOT AUTHORIZED
NEXT AUTHORIZED IMPLEMENTATION SLICE = NONE
```

This contract is a prerequisite for a future bounded implementation proposal.
It is not that proposal and grants no implementation permission.
