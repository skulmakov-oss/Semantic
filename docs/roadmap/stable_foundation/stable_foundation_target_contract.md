# Semantic Stable Foundation Target Contract

Status: SSF-00 candidate contour; not a stable release claim
Contract version: `SSF-TARGET-0`
Evidence base: `89a014b66e7c1e40502dbd764c94bf5f9445677f`

## Purpose

This contract freezes the smallest coherent target that later SSF phases may
stabilize and qualify. It does not promote current-main behavior. A feature is
in the final Stable Foundation only after its owning phase freezes the contract,
the implementation satisfies it end to end, SSF-12 qualifies it, and a human
release owner explicitly promotes it.

## Architectural invariant

```text
source specification
  -> frontend and semantic analysis
  -> deterministic IR / SemCode
  -> verifier admission
  -> deterministic VM/runtime
  -> explicit capability and audit boundary
  -> CLI / language server / UI projections
  -> compatibility and release decision
```

The verifier may reject any artifact before execution. Package identity, CLI,
editor, Workbench, and Studio layers cannot override that decision. Following
the Semantic UI DNA, UI is a projection of canonical meaning and evidence, not
an alternate language/runtime authority.

## Candidate Foundation contour

The target is deliberately narrower than current `main`.

### Language core

SSF-01 resolves this candidate into the narrower versioned contract in
`docs/spec/foundation_source_profile_v1.md`. Later phases may consume that
contract but may not silently widen it.

- one versioned Rust-like executable profile;
- functions and an explicit program entrypoint;
- immutable and mutable bindings with deterministic assignment rules;
- value-producing blocks, `if`/`else`, bounded `match`, loops, exits, and
  returns;
- records, tuples, bounded enums/ADTs, `Option`, and `Result`;
- native `quad`, `bool`, `text`, `i32`, `u32`, `f64`, `fx`, and `unit`, each
  only to the subset selected by SSF-01 and SSF-07;
- bounded `Sequence` and `Map` contracts;
- immutable first-wave closures;
- deterministic monomorphised generics and static traits only to the subset
  selected by SSF-07;
- bounded patterns/destructuring and direct local module imports;
- only those function contracts that can be documented, rejected, lowered,
  verified, and executed consistently.

### Profile coherence

Rust-like Semantic is the executable-profile candidate. SSF-02 selected Model B:
Logos is the separate experimental declarative profile
`semantic.logos.declarative/0.1`, with parse/semantic/inspection support and no
SemCode/verifier/VM execution path. The decision record is
`rustlike_logos_coherence_decision.md`.

### Minimal deterministic library

The candidate library families are `std.core`, `std.quad`, `std.math`,
`std.text`, `std.seq`, `std.map`, `std.option`, `std.result`, `std.serde`, and
`std.rand`. Existing builtins are inputs, not proof that these modules or their
compatibility contracts already exist. SSF-03 owns the final minimal APIs.

### Controlled application boundary

The target admits only explicitly declared, granted, audited, and replay-aware
effects. SSF-04 owns the final capability names and profiles. The minimum use
case is a deterministic CLI file transform. Networking, process spawning, and
implicit ambient authority remain excluded.

### Reproducible projects and local packages

The target includes:

- one canonical manifest direction;
- deterministic project roots, source roots, entrypoints, and path handling;
- `check`, `compile`, `verify`, `run`, and `test` project flows;
- reproducible local path dependencies, package identities, cycles, hashes,
  capability-request inventory, and a minimal provenance/lock record.

It does not require a public registry, remote solver, build scripts, or install
hooks.

### Runtime and ownership

Execution remains verifier-first, deterministic, quota-bounded, and explicit
about traps versus admission failures. SSF-08 must choose an honest public
position. The default candidate is a bounded deterministic VM language; no
Rust-equivalent lifetime or region claim is implied.

### Tooling and compatibility

The target includes canonical machine-readable diagnostics, deterministic
idempotent formatting, a diagnostics-first language server, at least one
documented external editor path, explicit compatibility/migration policies,
and inspectable artifact identity and provenance. Canonical tooling must reuse
compiler truth rather than reimplement it.

### Proof and onboarding

The final contour must be exercised by the canonical application families in
SSF-11 and the exact qualification gates in SSF-12. Native UI may demonstrate a
consumer path but is not language, verifier, runtime, or Foundation authority.

## Exclusions

The target excludes async/await, general concurrency/distribution, macros,
unrestricted reflection or dynamic dispatch, a garbage-collected object
runtime, unrestricted filesystem/network/process effects, a public registry,
plugin marketplace, ALM, autonomous source mutation, Studio completion,
Andromeda, and broad permanent ABI/ISA promises.

## Unresolved decisions and owners

| Decision | Owning phase | Entry assumption |
|---|---|---|
| Exact stable source grammar/profile and included feature subsets | SSF-01 | Start from the candidate core above; do not inherit all of current `main`. |
| Executable versus declarative Logos | SSF-02 | Resolved as Model B; Logos remains experimental and non-executable. |
| Builtin-to-stdlib boundary and APIs | SSF-03 | Existing builtins may be wrapped, renamed, narrowed, or deferred. |
| Capability names, grants, denial results, audit, replay profiles | SSF-04 | Existing `print` is evidence, not the finished boundary. |
| Canonical manifest, project layout, discovery, and command shapes | SSF-05 | Both current manifests are evidence; neither wins by implication. |
| Package identity, provenance, and lock record | SSF-06 | Local dependencies remain unpromoted. |
| Numeric, text, collection, closure, generic, trait, and pattern limits | SSF-07 | Prefer the smallest ordinary-program contour. |
| Ownership Position A or B | SSF-08 | Position A is the conservative candidate, not a pre-decided result. |
| Diagnostic schema, formatter, LSP, editor baseline | SSF-09 | `smc check` remains source of truth. |
| Compatibility windows, migration, artifact trust, checksums/signing | SSF-10 | No stable publication is currently evidenced. |
| Canonical application and onboarding proof | SSF-11 | Examples must declare maturity and remain executable. |
| Promotion verdict | SSF-12 plus human release owner | Green gates permit a recommendation, never automatic publication. |

## Change control

Every later phase must reference `SSF-TARGET-0` and the matrix row(s) it owns.
Changing the contour requires an explicit contract revision with rationale,
dependency impact, and qualification impact. Implementation presence, a green
unit test, or a UI demonstration cannot silently widen the target.

## SSF-01 entry conditions

SSF-01 may begin only after:

1. the matrix, this target contract, and the dependency map are accepted and
   linked from issue #1571;
2. current-facing status documents no longer claim unverified stable
   publication;
3. the SSF-00 drift check and repository documentation guards pass;
4. the SSF-00 PR is reviewed, green on its exact head, and merged to `main`;
5. issue #1571 records the exact merge commit and closes its exit gate;
6. `.harness/current.task.yaml` is advanced separately to authorize only
   SSF-01.

Until all six are true, language implementation remains unauthorized.
