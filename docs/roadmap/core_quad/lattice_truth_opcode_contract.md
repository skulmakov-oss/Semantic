# Core Quad VM: Lattice vs Truth-Table Opcode Contract

## 1. Decision
Preserve current VM `QAnd`/`QOr` semantics as lattice meet/join.

`semantic-core-quad` truth-table maps are canonical for explicit truth-map operations only.
They are not the backend for current `QAnd`/`QOr` VM opcodes.

Future truth-table VM support must use distinct opcode names and explicit lowering.
No hidden adapter, remapping, or falsity-plane inversion is allowed.

## 2. Definitions
- **Lattice operations**: Operations that treat the 4-state quad values as a mathematical lattice. These are primarily bitwise operations over the 2-bit state encoding, where operations map identically to mathematical `meet` and `join`.
- **Truth-table maps**: Operations that treat the 4-state quad values as strict truth and falsity planes, mapping exactly to Belnap logic (e.g. truth-plane intersection, falsity-plane union).

## 3. Current VM Contract
The existing VM opcodes `QAnd`, `QOr`, `QNot`, and `QImpl` strictly abide by lattice logic:
- `QAnd` behaves as lattice meet (bitwise AND).
- `QOr` behaves as lattice join (bitwise OR).
- `QNot` behaves as lattice inverse (T/F plane swap).

## 4. semantic-core-quad Truth-Map Contract
The operations provided by `semantic-core-quad` (`map_and`, `map_or`, `map_implies`, `map_not`) strictly abide by truth-table logic mapping and serve as the canonical reference for any truth-table operations within the language and runtime.

## 5. Non-Substitution Rule
Lattice operations and truth-table maps are mathematically distinct across the 4-state domain. 
Under no circumstances may a `semantic-core-quad` truth-table mapping be silently substituted as a backend for a lattice opcode (e.g. `QAnd` / `QOr`). Bridging adapters that artificially negate planes to emulate lattice operations via truth-table maps are strictly prohibited.

## 6. Future Opcode Naming Policy
To prevent semantic leakage and clarify intent, future opcode layers must adhere to explicit naming conventions separating the two families of operations.

Candidate Names:

**Lattice Layer (Compatibility & Mathematical):**
- `QMeet` (Maps to legacy `QAnd`)
- `QJoin` (Maps to legacy `QOr`)
- `QInverse` (Maps to legacy `QNot`)

**Truth-Table Layer (Strict Belnap Logic):**
- `QTruthAnd`
- `QTruthOr`
- `QTruthImpl`
- `QTruthNot` (Required to strictly align with the truth-table family, avoiding mix-and-match).

## 7. Migration Rule
Any migration of VM logic to use canonical `semantic-core-quad` behavior must involve explicit introduction of the new truth-table opcodes (`QTruth*`), along with explicit lowering from the frontend IR.

## 8. Out of Scope
This contract does not govern the actual implementation of the new opcodes, nor does it mandate immediate deprecation of the `sm-vm` legacy opcodes. It solely establishes the boundary and naming strategy moving forward.
