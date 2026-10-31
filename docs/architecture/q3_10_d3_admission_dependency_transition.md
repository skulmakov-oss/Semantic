# Q3.10 D3 - Admission Dependency Transition

Status: PROPOSED - dependency transition not authorized

Baseline: 4207f72551190662c14961deb083bdf3bf33d3d2

## Observed graph

cargo metadata --no-deps confirms that sm-format owns wire decoding, sm-verify
owns bytecode admission, sm-vm owns execution, sm-front and sm-sema own source
analysis, and smc-cli coordinates compiler-side operations.

Required negative edges:

    sm-verify -X-> sm-front
    sm-verify -X-> sm-sema
    sm-verify -X-> sm-admission

No semantic typechecking rules may be duplicated in the gateway and no
alternative bytecode verifier may be introduced.

## Proposed minimal graph

    sm-front ---> sm-sema
        \           |
         \          v
          ---> sm-admission <--- sm-format
                        |
                        +---> sm-verify
                        +---> sm-ir
                        +---> sm-emit

The arrows describe allowed use by a future gateway. sm-admission would
coordinate existing certified owners: front/sema for source observations,
format for structural data, verify for bytecode admission, and ir/emit for
deterministic reproduction.

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

The future patch must prove that cargo metadata has no forbidden reverse edge
or cycle, downstream Rust cannot construct the opaque token, source A plus
artifact B is rejected, structural rev27 decode remains untrusted without the
gateway context, legacy V0-V25 behavior is unchanged, and no opcode,
capability, VM privilege or verifier bypass is introduced.

This document does not authorize creating sm-admission or editing Cargo files.
