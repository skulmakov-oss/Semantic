# Roadmap: Tile Truth Map Lifting (#1411)

## Metadata
* **Owner:** `semantic-core-quad`
* **Issue:** #1411
* **Context:** This is the first slice of #1411. Relates to #1413 (API compatibility) and #1404 (v1 roadmap).

## Supported Tile Truth Maps
The following default truth maps have been lifted to `QuadTile128`:
- NOT
- XOR
- AND
- OR
- IMPLIES
- NAND
- NOR

## Explicit Omission of EQUIV
`map_equiv` is intentionally omitted from this layer to adhere to the v1 compatibility policy.

## Implementation Strategy
This is the deterministic reference path:
1. Decompose the 128-lane tile into four 32-lane `QuadroReg32` registers.
2. Apply the default register-level map to each.
3. Reconstruct the tile from the four resulting registers.

This avoids duplicating the complex boolean combinations and provides an indisputable source of truth.

## Distinction from Lattice Methods
Truth maps apply pure logic operations on states. They are distinct from the knowledge lattice operations:
- `join`: least upper bound in the knowledge lattice
- `meet`: greatest lower bound
- `inverse`: truth and falsity swap
- `raw_delta`: XOR of plane bitmasks

## Posture
- **Allocation:** None.
- **Environment:** `no_std` compatible.
- **Layout:** The 32-byte, 16-byte-aligned `QuadTile128` layout remains unchanged.

## Deferred Optimization
- Direct plane formulas for `QuadTile128`.
- Benchmarking of reference vs direct implementations.

## Deferred Second Slice (#1411)
- Additive helpers for `QuadroBank<N>`
- Additive helpers for `QuadTileBank<N>`

## Explicit Non-Changes
- No GPU transport implementation
- No visual layer or wgpu
- No changes to underlying masks or tile layouts