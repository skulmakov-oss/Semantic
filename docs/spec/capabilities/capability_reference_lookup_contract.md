# Capability Reference Lookup Contract

Status: D0D specification
Owner: prom-cap
Gate: Gate D remains closed

This specification authorizes no Rust implementation.

## Purpose

This specification defines lookup of an untrusted `CapabilityRef` inside the
`prom-cap` capability domain.

The purpose of lookup is limited to mapping an exact capability reference to a
non-authoritative capability-domain record.

Lookup MUST NOT:

- grant authority;
- perform admission;
- dispatch an operation;
- execute an effect;
- validate an `ActionIntent`;
- select a latest generation;
- resolve any other reference domain.

## Ownership

| Concept | Owner |
| --- | --- |
| `ReferenceToken` representation | `prom-refs` |
| `CapabilityRef` representation | `prom-refs` |
| capability lookup contract | `prom-cap` |
| capability lookup storage/view | `prom-cap` |
| capability grant check | `prom-cap` |
| runtime dispatch | existing runtime owner, outside D0D |

`prom-refs` MUST remain representation-only.

`prom-refs` MUST NOT gain:

- registry behavior;
- lookup behavior;
- resolve behavior;
- grant behavior;
- admission behavior;
- dispatch behavior;
- runtime dependencies.

No capability registry currently exists.

The gate registry in `prom-gates` is a separate domain and MUST NOT be treated
as a capability registry or as a partial capability-reference lookup
implementation.

## Trust model

Every `CapabilityRef` MUST be treated as untrusted input.

Possession of a `CapabilityRef` grants no permission.

Successful lookup grants no permission.

Lookup proves only that an exact reference is represented in the supplied
capability lookup view.

Authorization MUST require a later explicit capability check.

This specification makes no cryptographic authenticity claim.

Required pipeline:

```text
untrusted CapabilityRef
    ↓
prom-cap exact lookup
    ↓
non-authoritative resolved capability record
    ↓
separate explicit grant or policy check
    ↓
allowed or CapabilityDenied
```

## Reference coordinates

The reference coordinates are:

- `issuer: u64`
- `namespace: u32`
- `generation: u32`
- `value: u64`

### issuer

`issuer` is an opaque capability-domain authority coordinate.

It is not authentication.

It is not a globally unique identity.

Rules:

- comparison MUST be exact;
- lookup MUST NOT fall back between issuers;
- identical remaining coordinates under different issuers MUST NOT alias;
- cross-issuer numeric collisions MAY exist.

### namespace

`namespace` is an issuer-local partition coordinate.

Rules:

- `namespace` is meaningful only under its issuer;
- lookup MUST NOT fall back between namespaces;
- equal namespace values under different issuers MAY coexist.

### generation

`generation` is an owner-defined anti-reuse or incarnation coordinate.

Rules:

- `generation` MUST be part of exact reference identity;
- `generation` is not time;
- `generation` is not freshness by itself;
- `generation` is not globally monotonic;
- `generation` MUST NOT be compared across unrelated coordinates;
- lookup MUST NEVER silently substitute another generation.

D0D v0 decision:

A generation mismatch MUST be reported publicly as `UnknownReference`.

This specification MUST NOT introduce a distinct public stale or revoked
classification because lifecycle and revocation semantics do not yet exist.

### value

`value` is an opaque capability-domain handle coordinate.

Rules:

- ordering has no semantic meaning beyond deterministic key ordering;
- zero has no reserved meaning in D0D;
- value reuse is distinguishable only through `generation`;
- lookup MUST NOT use `value` alone.

## Registry key

The external lookup key is the complete token:

```text
issuer + namespace + generation + value
```

Normative requirements:

- all four fields MUST participate in exact identity;
- lookup MUST NOT fall back by partial key;
- lookup MUST NOT select another generation;
- lookup MUST NOT search another issuer;
- lookup MUST NOT search another namespace;
- registry sharding MUST NOT alter observable key semantics.

Deterministic lexicographic ordering MUST be:

1. `issuer`
2. `namespace`
3. `generation`
4. `value`

This ordering is for lookup storage and validation only. It does not imply
semantic precedence or authority.

## Minimal lookup entry

The normative conceptual entry is:

```text
CapabilityLookupEntry {
    reference: CapabilityRef,
    kind: CapabilityKind,
}
```

This is a specification-level shape only.

The entry:

- identifies the exact stored reference;
- associates it with canonical `CapabilityKind`;
- contains no grant set;
- contains no `CapabilityManifest`;
- contains no `CapabilityChecker`;
- contains no callback;
- contains no dispatcher;
- contains no runtime handle;
- contains no admission state;
- confers no authority.

## Lookup result

The observable lookup result is the exact stored reference together with its
`CapabilityKind`.

The following conceptual shape is permitted:

```text
ResolvedCapability {
    reference: CapabilityRef,
    kind: CapabilityKind,
}
```

A conforming implementation MAY return `&CapabilityLookupEntry` directly.

`ResolvedCapability` is an optional conceptual equivalent only.

`ResolvedCapability` MUST NOT be interpreted as a mandatory second public Rust
type.

`ResolvedCapability` MUST NOT add authority, grant state, runtime state, or a
distinct identity beyond the stored `CapabilityLookupEntry`.

A later implementation review MUST NOT duplicate the public API with both
`CapabilityLookupEntry` and `ResolvedCapability` unless a separate invariant is
proven.

A caller MUST still perform a separate grant check appropriate to the requested
operation.

`CapabilityManifest` MUST NOT be used as the lookup result.

`CapabilityChecker` MUST NOT be used as the lookup result.

`CapabilityDenied` MUST NOT be used as the lookup result.

## Minimal registry model

D0D v0 is frozen to an immutable borrowed lookup view.

Conceptual model:

```text
CapabilityLookupView<'a> {
    entries: &'a [CapabilityLookupEntry],
}
```

Normative requirements:

- entries MUST be strictly sorted by the full-token key;
- duplicate full-token keys MUST be invalid;
- lookup itself MUST require no allocation;
- lookup MUST NOT mutate state;
- lookup MUST be deterministic;
- the view MUST NOT own runtime synchronization;
- the view MUST NOT define revocation;
- the view MUST NOT advance generations;
- the view MUST NOT serialize itself.

An implementation MAY use linear search or binary search when result and error
semantics remain identical.

Algorithmic complexity is not a normative authority or correctness property in
this contract.

## Registry construction validation

Validation MUST be deterministic.

Invalid input conditions are:

- entries not strictly ordered;
- duplicate full-token key.

Conceptual construction errors are:

- `CapabilityLookupBuildError::UnsortedEntries`
- `CapabilityLookupBuildError::DuplicateReference`

Exact Rust names MAY be confirmed later in the implementation slice, but these
two error classes are normative.

Error precedence MUST be:

1. inspect entries in slice order;
2. report the first adjacent invalid pair;
3. when equal keys are encountered, classify as duplicate;
4. otherwise classify descending order as unsorted.

## Lookup error

The minimal public v0 lookup error is:

```text
CapabilityLookupError::UnknownReference
```

The public v0 contract MUST collapse:

- unknown issuer;
- unknown namespace;
- unknown generation;
- unknown value;

into:

```text
UnknownReference
```

Reasons:

- no lifecycle model exists;
- revocation is not modeled;
- stale-reference semantics are not settled;
- detailed errors may leak registry structure;
- exact full-token lookup needs only success or exact miss.

`CapabilityLookupError` is not `CapabilityDenied`.

D0D MUST NOT define:

- `StaleReference`
- `RevokedReference`
- `UnknownIssuer`
- `UnknownNamespace`
- `RegistryUnavailable`

## Existing CapabilityDenied conflation

`CapabilityManifest::require` currently maps some manifest-validation failure
into `CapabilityDeniedCode::MissingCapability`.

That behavior is existing capability-policy behavior outside the D0D lookup
contract.

The D0D implementation MUST NOT reuse `CapabilityDenied` or
`CapabilityDeniedCode` for lookup failures or registry-construction failures.

This specification does not modify `CapabilityManifest::require`.

A separate future audit SHOULD classify that existing behavior independently.

## Lookup semantics

Conceptual behavior:

```text
lookup(view, reference)
```

Required steps:

1. compare the complete `CapabilityRef`;
2. return the exact associated entry when present;
3. return `UnknownReference` otherwise;
4. perform no fallback;
5. perform no grant check;
6. perform no logging requirement;
7. perform no mutation;
8. perform no allocation;
9. perform no runtime dispatch.

Successful lookup MUST preserve the exact input reference.

## Determinism

For identical entries and identical `CapabilityRef` input, lookup MUST return
the same resolved entry or the same error.

The contract requires:

- stable build-error precedence;
- no hash-randomization-dependent behavior;
- no thread-schedule-dependent result;
- no implicit latest-generation selection;
- no iteration-order-dependent ambiguity;
- no partial-key fallback.

Registry iteration is not public in D0D and has no separate semantic contract
beyond retained slice order.

## no_std and allocation posture

D0D does not establish or qualify `no_std` support for `prom-cap`.

The D0D lookup operation MUST require no allocation.

A future D0D implementation MUST NOT silently widen the crate's `std`
requirements relative to the accepted baseline.

Any `no_std` qualification for `prom-cap` requires a separate qualification
slice and is outside this contract.

The future implementation MAY depend on `prom-refs`.

The future implementation MUST NOT depend on `prom-ui` or
`prom-ui-runtime`.

Alloc-backed mutable registries are deferred.

The immutable borrowed view is the approved minimal v0 model.

## Dependency direction

The only new dependency edge permitted for a future implementation is:

```text
prom-cap -> prom-refs
```

Forbidden dependency directions:

- `prom-refs -> prom-cap`
- `prom-cap -> prom-ui`
- `prom-cap -> prom-ui-runtime`
- `prom-refs -> prom-ui`
- `prom-refs -> prom-ui-runtime`

This specification itself introduces no dependency edge.

## Security considerations

This contract MUST treat the following as untrusted-input risks:

- forged issuer;
- forged namespace;
- forged generation;
- forged value;
- replayed old generation;
- cross-issuer collision;
- cross-namespace collision;
- value reuse;
- error-based enumeration;
- lookup-result misuse as authority.

Required mitigations:

- complete-token exact match;
- no partial fallback;
- no detailed public miss reason;
- no implicit generation upgrade;
- explicit separation from grant checking;
- no cryptographic claims.

Revocation and authenticity are deferred.

## Explicit non-goals

This specification does not define:

- mutable registry;
- global registry;
- registry singleton;
- revocation;
- generation advancement;
- latest-generation lookup;
- serialization;
- persistence;
- concurrent mutation;
- runtime ownership;
- runtime dispatch;
- `ActionIntent` integration;
- UI admission;
- host effects;
- generic `Resolver<T>`;
- workspace-wide lookup trait;
- `ActorRef` resolution;
- `SessionRef` resolution;
- `ClientRef` resolution;
- `RevisionRef` resolution;
- `EpochRef` resolution.

## Public API posture

This specification is normative before Rust API creation.

The future implementation SHOULD use concrete `prom-cap` types.

No generic resolver trait is permitted.

No workspace-wide lookup trait is permitted.

A concrete borrowed view with an inherent lookup method is preferred.

Exact Rust naming remains subject to later implementation review.

Any future public Rust surface requires same-series updates to:

- `tests/public_api_contracts.rs`;
- `tests/golden_snapshots/public_api/prom_cap_lib.txt`;
- focused compile-contract tests.

No `prom-refs` public API snapshot change should be required.

## Required future test matrix

| Test | Required invariant |
| --- | --- |
| exact match | exact full-token entry resolves |
| unknown issuer | `UnknownReference` |
| unknown namespace | `UnknownReference` |
| unknown generation | `UnknownReference` |
| unknown value | `UnknownReference` |
| cross-issuer collision | no alias |
| cross-namespace collision | no alias |
| same value, different generation | no fallback |
| duplicate full token | deterministic build failure |
| unsorted entries | deterministic build failure |
| first-invalid-pair precedence | when multiple adjacent violations exist, construction reports the earliest invalid adjacent pair only |
| repeated lookup | identical result |
| lookup does not grant | explicit policy check remains necessary |
| allocation-free lookup | no allocation required by operation |
| `no_std` qualification check | required only when `prom-cap` no_std qualification is separately in scope |
| public API guard | exact approved surface only |

Revocation tests are deferred until revocation semantics exist.

## Gate conditions for Rust implementation

| Condition | Required result |
| --- | --- |
| ownership | `prom-cap` |
| target | non-authoritative reference + kind record |
| registry key | complete `CapabilityRef` |
| generation behavior | exact key; mismatch becomes unknown |
| registry model | immutable borrowed sorted view |
| mutation | out of scope |
| revocation | out of scope |
| lookup error | `UnknownReference` |
| build errors | duplicate and unsorted |
| grant separation | explicit |
| `no_std` posture | no new qualification claim; no silent widening relative to the accepted baseline |
| allocation posture | allocation-free lookup |
| dependency direction | `prom-cap -> prom-refs` only |
| public API posture | concrete API, no generic trait |
| tests | normative matrix present |

Completion of this specification permits only a later implementation proposal.

It does not itself authorize implementation.

## Generic resolver rejection

This specification normatively rejects:

```text
trait Resolver<T>
```

and equivalent workspace abstractions.

Reasons:

- domain owners differ;
- target types differ;
- errors differ;
- five reference domains have no canonical targets;
- `resolve` may imply authority;
- the abstraction would freeze an unsupported common model.
