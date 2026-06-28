# ADT Payload Ownership Paths

## Context
Semantic ADT execution supports `MakeAdt`, `AdtTag`, and `AdtGet` opcodes, providing foundational ability to create and move ADT values. However, fully unblocking Practical Core Completion (PCC) for ADTs requires the ability to borrow and trace ownership paths into ADT payloads (e.g. `match x { MyEnum::Variant(ref value) => ... }`).

## The Problem
When borrowing from a tuple or record, the compiler emits an ownership section path (e.g., `TupleIndex(0)` or `FieldSymbol(foo)`), and the VM ensures no overlapping mutable paths exist. Before this capability, SemCode and the runtime engine only supported paths for tuples and fields. Thus, taking a reference to an ADT payload could not be expressed in the binary contract or enforced at runtime.

## Solution
To solve this cleanly, we introduce ADT Payload ownership path vocabulary *before* altering lowering or VM semantics:

1. **`sm-format`**: Introduced `OWNERSHIP_PATH_COMPONENT_ADT_PAYLOAD`. The binary format represents this as `[tag: 1 byte] + [variant_symbol_id: 4 bytes] + [payload_index: 2 bytes]`.
2. **`sm-runtime-core`**: The runtime representation `AccessPath` uses `PathComponent::AdtPayload { variant: SymbolId, index: u16 }` to describe stable paths into ADTs.
3. **`sm-verify`**: The verifier structurally accepts the `AdtPayload` component and asserts that the `variant` symbol ID is within bounds of the string table.

### Why not just an `index`?
A naive solution would use `AdtPayloadIndex(u16)`, analogous to a tuple index. But different variants in the same enum can have payloads with identical indices (`Some(T)` vs `Data(T)`). Dropping the variant tag creates paths that can falsely overlap or improperly alias if the variant changes, rendering it unsuitable for the stable runtime ownership checker. Including the `variant` ensures a safe, non-colliding alias path (e.g. `Option::Some.0` does not collide with `Result::Err.0`).

## What remains
This document describes only the *vocabulary* layer.

- **Lowering**: `sm-ir` does not yet emit these paths. (Pending ADT-2)
- **Runtime Borrows**: `sm-vm` does not yet process these paths in match expressions. (Pending ADT-3)
