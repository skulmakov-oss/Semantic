# Bank Truth-Map Helpers

- **Owner:** `semantic-core-quad`
- **Issue:** `#1411` (slice 2 of 2, completes the issue)
- **Related:** `#1413`, `#1404`

## Objective
This completes the second and final #1411 implementation slice by adding in-place truth-map helpers to `QuadroBank<N>` and `QuadTileBank<N>`.

## Supported Bank Helpers
Both owner types (`QuadroBank<N>`, `QuadTileBank<N>`) now support the following helpers:

* `map_not_inplace(&mut self)`
* `map_xor_inplace(&mut self, other: &Self)`
* `map_and_inplace(&mut self, other: &Self)`
* `map_or_inplace(&mut self, other: &Self)`
* `map_implies_inplace(&mut self, other: &Self)`
* `map_nand_inplace(&mut self, other: &Self)`
* `map_nor_inplace(&mut self, other: &Self)`

## Semantics
1. **Elementwise Ordering:** Operations are applied elementwise. Bank element order is fully preserved.
2. **Resource Constraints:** No allocation is performed. Fully `no_std` compatible.
3. **Lattice Separation:**
   - AND and OR differ behaviorally from meet and join.
   - NOT and inverse currently coincide extensionally for N/F/T/S, but they remain separate operation families and APIs.
   - Future policy changes must not silently alias one implementation contract to the other.
4. **EQUIV Deferral:** `map_equiv_inplace` remains explicitly deferred under the v1 compatibility policy.
5. **Zero-length Behavior:** Operations on length-zero banks are no-ops and safely complete without panic.
6. **Length Mismatch:** Not possible since both operands share the same const generic `N`.

## Preserved Shapes
The public struct shapes of `QuadroBank<N>` and `QuadTileBank<N>` remain identical.

## Explicit Non-Changes
- No EQUIV.
- No direct plane optimization.
- No GPU transport implementation.
- No benchmarks.
- No VM/runtime changes.

## #1411 Completion
With this change, #1411 is complete:
- reg maps exist;
- tile lifting exists;
- bank lifting exists;
- optimization and benchmarks are separate future work.
