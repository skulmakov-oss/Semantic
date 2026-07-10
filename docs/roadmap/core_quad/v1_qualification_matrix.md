# Quad Logic Engine v1 Public Qualification Matrix

## Scope

- **Owner:** `semantic-core-quad`
- **Issue:** #1412
- **Parent:** #1404
- **Slice:** qualification slice 1 of 2

This PR qualifies the landed v1 public API through an external-consumer
integration test. It does not close #1412.

## Why an integration suite

`crates/semantic-core-quad/tests/v1_qualification.rs` imports
`semantic_core_quad` as a downstream crate and uses only public constructors,
methods, fields, and accessors. It complements rather than replaces the inline
unit tests: inline tests can inspect implementation-local behavior, while this
matrix proves that the qualified contract is reachable through the public
surface.

## Existing inline unit-test inventory

The review covered the existing tests for frozen state encoding, mask bridge
roundtrips and type distinction, odd physical-mask rejection, scalar/SWAR map
equivalence for all seven maps, the 4x4 delta matrices, register/tile
roundtrips, tile map oracles, register and tile bank elementwise maps, and the
repeated-state bank matrix. The new suite exercises those contracts as public
downstream behavior rather than copying their internal helpers.

## Public qualification layers

The integration matrix covers:

- state encoding through `bits()` and `from_bits()`;
- scalar, SWAR, and default register truth maps;
- the dense/physical typed-mask bridge and rejection rules;
- exact-state and plane deltas across the complete 4x4 state matrix;
- tile lifting through four public register values;
- register-bank and tile-bank in-place lifting;
- truth-map versus knowledge-lattice API-family separation.

## Deterministic vectors and matrices

The suite uses exactly these canonical raw register vectors:

```text
0x0000_0000_0000_0000
0xFFFF_FFFF_FFFF_FFFF
0x5555_5555_5555_5555
0xAAAA_AAAA_AAAA_AAAA
0x0123_4567_89AB_CDEF
0xE4E4_E4E4_E4E4_E4E4
0xBADC_0FFE_DEAD_BEEF
```

Register equivalence covers the complete 7x7 binary matrix. Delta assertions
cover every previous/current pair in `[N, F, T, S]` and isolate lane 9. Tile
and bank fixtures contain observably distinct elements. No randomness, wall
clock, environment-specific CPU feature, or platform-dependent value is used.

## Policy boundaries

EQUIV remains deferred and is not part of the qualified v1 API. This PR adds no
benchmark implementation. The second #1412 slice is reserved for benchmark
inventory, an available no-new-dependency execution method, relative-output
policy, and issue closeout without absolute performance promises.

## Feature posture and smoke path

- `std`: tested.
- `no_std`: check-qualified through `--no-default-features`.
- `serde`: compile/test-qualified under `--all-features`.
- `semantic-core-capsule`: retained as the minimum downstream smoke consumer.

## Explicit non-changes

No production source, Cargo configuration, public API, specification, other
crate, dependency, visual/GPU layer, runtime behavior, EQUIV surface, or
benchmark code is changed. This PR does not close #1412.
