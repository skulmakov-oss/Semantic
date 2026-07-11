# UI-DNA2 Reference Value Contract

Status: draft v0

This document defines the canonical value contract for authority-free opaque references inside the Semantic UI architecture, specifically owned by the `prom-refs` crate.

## Purpose of `prom-refs`

The `prom-refs` crate serves as the neutral, policy-free owner of Semantic reference values. It exists solely to provide deterministic structural shapes for provenance coordinates, formally stripped of any implicit authentication, authorization, or trust semantics.

## Dependency Boundary

```text
prom-ui -> prom-refs
```

`prom-refs` must remain a leaf-like, `#![no_std]`, zero-allocation crate. It has zero external dependencies and does not depend on UI, runtime, capability, audit, or VM crates.

## The ReferenceToken

The canonical underlying structural shape is the four-part `ReferenceToken`.

It consists of exactly four primitive components:

```rust
ReferenceToken {
    issuer: u64,
    namespace: u32,
    generation: u32,
    value: u64,
}
```

- `issuer`: Deterministic identifier of the trusted domain owner or subsystem that issued the claim.
- `namespace`: Scoped partition within the issuer's domain.
- `generation`: Freshness or sequence counter to detect stale or recycled values.
- `value`: The core opaque correlation payload.

## Domain Wrappers

To provide type-safety, six nominally distinct domain wrappers are strictly defined around the `ReferenceToken`:

- `CapabilityRef`: Capability candidate reference.
- `ActorRef`: Authority-scoped correlation handle for an Actor.
- `SessionRef`: Session-scoped handle.
- `ClientRef`: Session-scoped correlation handle for a Client.
- `RevisionRef`: Versioned state token for a Revision.
- `EpochRef`: Versioned execution token for an Epoch.

Each wrapper provides a `const fn new(ReferenceToken)` constructor and a `const fn token() -> ReferenceToken` getter.

## Policy Prohibitions

1. **Possession Grants No Authority**: The values are untrusted correlation claims. Possession of a reference is never an authentication or capability grant.
2. **No Implicit Authentication**: The token does not cryptographically or intrinsically prove the authenticity of the `issuer`.
3. **No Resolution or Admission**: There is strictly no resolution logic, admission policy, or capability verdict present. These checks occur outside the crate in authoritative runtime boundaries.
4. **No Binary Guarantees**: There is no stable binary wire-format, stable ABI (`repr(C)`), or persistence guarantee.
5. **No Raw Conversion**: Single integer raw casting, `From<u64>`, `Into<u64>`, and arbitrary cross-domain conversions between wrappers are explicitly prohibited.

This document serves as a specialized value-contract extension beneath the existing ActionIntent architecture.
