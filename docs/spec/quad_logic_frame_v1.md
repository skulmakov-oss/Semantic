# Quad Logic Frame v1

Status: Frozen spec draft  
Owner: `crates/semantic-core-quad`  
Compatibility crate: `crates/ton618-core`

## Purpose

This specification formally defines the Quad Logic Frame v1 before behavior-changing implementation work begins in `semantic-core-quad`. It establishes the state encoding, primitive vs derived operations, execution policies, and mask isolation requirements.

## Ownership Boundary

The canonical owner of the Quad Logic Frame implementation is `crates/semantic-core-quad`. The legacy crate `crates/ton618-core` is retained exclusively for backward compatibility purposes.

## State Encoding

Quad states are strictly encoded using a 2-bit representation per semantic lane:

- `N = 00` (Null)
- `F = 01` (Strict False)
- `T = 10` (Strict True)
- `S = 11` (Conflict / Super)

## Planes and Masks

The encoding structurally separates logical information into two distinct 1-bit planes:
- **Falsity plane** (Bit 0): Defines the absence of strict truth or presence of conflict.
- **Truth plane** (Bit 1): Defines the presence of strict truth or conflict.

## State Predicates

Built-in state predicates operate by querying the presence of values within the planes:
- Null (`N`)
- Strict True (`T`)
- Strict False (`F`)
- Conflict / Super (`S`)
- Known / Non-null (any state other than `N`)

## Operation Families

The Quad Logic Frame defines operations that evaluate across multiple semantic states, grouped into distinct families. Mixing operations between truth-table families and knowledge-lattice families by name is strictly forbidden.

### Truth-table operations

Standard boolean logic operations extended over the quad domain (e.g., AND, OR, NOT).

### Knowledge-lattice operations

Information-theoretic operations dealing with the accretion and intersection of semantic knowledge (e.g., JOIN, MEET).

### Diagnostic operations

Operations providing visibility into internal execution states or validity boundaries without mutating quad logic states.

### Event and delta operations

Operations determining changes or state transitions between subsequent evaluation frames.

## Primitive vs Derived Operations

| Operation | Origin | v1 Policy |
|---|---|---|
| NOT | primitive | Implemented as primitive truth-table complement |
| AND | primitive or generated policy table | Base truth-table intersection |
| OR | primitive or generated policy table | Base truth-table union |
| XOR | raw-code derived or primitive | Base truth-table difference |
| IMPLIES | current derived semantics retained | Retain current execution semantics: `A -> B = NOT(A) join B` |
| EQUIV | decision deferred or separately named | Do not overload; requires explicit named API |
| NAND | derived from NOT(AND) | Formally derived execution |
| NOR | derived from NOT(OR) | Formally derived execution |

## Mask Model Policy

Raw ambiguous `u64` masks must not be exposed in public v1 APIs. The architecture requires typed wrappers separating dense lane masks from physical LSB-aligned bitmasks. The final internal canonical representation remains deferred behind explicit typed bridge names.

## IMPLIES Policy

The existing `IMPLIES` execution semantics (`A -> B = NOT(A) join B`) must be retained to preserve backward compatibility. Primitive LUT implication must not silently replace this execution behavior. If primitive implication is implemented later, it must be exposed via a separately named API or introduced through a dedicated semantic-breaking update.

## Backward Compatibility Policy

Modifications to core semantic structures, state encodings, or mask evaluation policies are bound by the backward compatibility rules of `ton618-core`. Any change altering truth-table outputs must explicitly undergo review and provide a migration mapping.

## Non-goals

This specification does not define LUT (Look-Up Table) implementations, SWAR formulas, or modifications to VM execution, opcodes, or cryptographic trust verification.

## Acceptance Checklist

- [x] State encoding defined (`N=00`, `F=01`, `T=10`, `S=11`)
- [x] Primitive vs Derived operation table defined
- [x] `IMPLIES` policy explicitly documented
- [x] Mask policy strictly isolates raw `u64`
- [x] Operations formally grouped into non-mixing families
