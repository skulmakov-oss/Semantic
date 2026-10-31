# Q3.10 D3 - Admission Dependency Transition

Status: PROPOSED - dependency transition not authorized

Baseline: 4207f72551190662c14961deb083bdf3bf33d3d2

## Observed graph

cargo metadata --no-deps plus the package manifests confirm these production
edges. In this document A -> B means package A depends on package B. Features
are listed separately and are not additional packages. The logical alias
`legacy-language-core` below means the compatibility-core package named in the
current `crates/sm-front/Cargo.toml` and `crates/sm-sema/Cargo.toml`; the
manifests remain the authoritative package-name source.

    sm-front -> legacy-language-core, sm-profile, sm-diagnostic
    sm-sema -> legacy-language-core, sm-front, sm-profile, sm-diagnostic
    sm-format -> (none)
    sm-ir -> sm-front, sm-profile, sm-format
    sm-emit -> sm-format, sm-ir
    sm-verify -> sm-runtime-core, sm-format, sm-diagnostic

sm-verify -> sm-ir exists only under sm-verify dev-dependencies for tests; it
is not a production dependency. Existing feature wiring for std/alloc and
profile features remains owned by each manifest.

Required negative edges:

    sm-verify must not depend on sm-front
    sm-verify must not depend on sm-sema
    sm-verify must not depend on sm-admission

No semantic typechecking rules may be duplicated in the gateway and no
alternative bytecode verifier may be introduced.

## Proposed minimal graph

    sm-admission -> sm-front
    sm-admission -> sm-sema
    sm-admission -> sm-format
    sm-admission -> sm-ir
    sm-admission -> sm-emit
    sm-admission -> sm-verify

This is a proposed graph, not a compiled workspace graph. sm-admission would
coordinate existing certified owners: front/sema for source observations,
format for structural data, verify for bytecode admission, and ir/emit for
deterministic reproduction. No existing owner may depend back on
sm-admission.

## Minimum future Cargo transition

The first implementation patch would require:

1. add crates/sm-admission to root workspace members;
2. add crates/sm-admission/Cargo.toml;
3. add only approved existing workspace dependencies;
4. update Cargo.lock if resolution changes;
5. add dependency-cycle and public API tests;
6. integrate with smc-cli only in a later scoped patch.

The current Harness forbids dependency and Cargo manifest changes. This
transition is therefore blocked here. Future authorization must explicitly
permit the root manifest, new crate manifest and any lockfile update. No
Harness weakening is acceptable.

## Ownership rules

- sm-admission must not expose a public constructor accepting caller claims,
  digests, booleans or callbacks as semantic proof.
- sm-verify remains the owner of SemCode admission and must not depend on
  source-semantic crates.
- sm-format remains the owner of structural decoding only.
- sm-front and sm-sema remain owners of parsing and typechecking.
- The gateway may issue an opaque token only after approved checks and exact
  artifact equality.

## Qualification before transition

The future patch must add sm-admission to a temporary or task-authorized
workspace, then run cargo metadata with dependency resolution and inspect the
resolved production graph. cargo metadata --no-deps for the current graph is
not proof that the future graph is cycle-free. The patch must also prove that
downstream Rust cannot construct the opaque token, source A plus artifact B
is rejected, structural rev27 decode remains untrusted without gateway
context, legacy V0-V25 behavior is unchanged, and no opcode, capability, VM
privilege or verifier bypass is introduced.

This document does not authorize creating sm-admission or editing Cargo files.
