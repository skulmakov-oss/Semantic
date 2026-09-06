# ADT Payload Ownership Paths

## Context
Semantic ADT execution supports `MakeAdt`, `AdtTag`, and `AdtGet` opcodes, providing foundational ability to create and move ADT values. However, fully unblocking Practical Core Completion (PCC) for ADTs requires the ability to borrow and trace ownership paths into ADT payloads (e.g. `match x { MyEnum::Variant(ref value) => ... }`).

## The Problem
When borrowing from a tuple or record, the compiler emits an ownership section path (e.g., `TupleIndex(0)` or `FieldSymbol(foo)`), and the VM ensures no overlapping mutable paths exist. Before this capability, SemCode and the runtime engine only supported paths for tuples and fields. Thus, taking a reference to an ADT payload could not be expressed in the binary contract or enforced at runtime.

## Solution
To solve this cleanly, we introduce ADT Payload ownership path vocabulary *before* altering lowering or VM semantics:

1. **`sm-format`**: Introduced `OWNERSHIP_PATH_COMPONENT_ADT_PAYLOAD`. The binary format represents this as `[tag: 1 byte] + [variant_symbol_id: 4 bytes] + [payload_index: 2 bytes]`.
2. **`sm-runtime-core`**: The runtime representation `AccessPath` uses `PathComponent::AdtPayload { variant: SymbolId, index: u16 }` to describe stable paths into ADTs.
3. **`sm-verify`**: The verifier structurally accepts the `AdtPayload` component in `Borrow` events. The `variant` `SymbolId` is **not** bounds-checked against the string table - it is used exclusively as an opaque, root-gated equality key inside the VM's overlap check (`access_paths_overlap`), the same treatment `#1725` already established for `Field`'s `SymbolId`. A `Write` event carrying an `AdtPayload` component is rejected unconditionally, under every header, regardless of capability (#1718).

### Why not just an `index`?
A naive solution would use `AdtPayloadIndex(u16)`, analogous to a tuple index. But different variants in the same enum can have payloads with identical indices (`Some(T)` vs `Data(T)`). Dropping the variant tag creates paths that can falsely overlap or improperly alias if the variant changes, rendering it unsuitable for the stable runtime ownership checker. Including the `variant` ensures a safe, non-colliding alias path (e.g. `Option::Some.0` does not collide with `Result::Err.0`).

## Current status (updated for #1718)

This document originally described only the *vocabulary* layer, before
lowering or VM semantics existed for it. Both now exist and are qualified:

- **Lowering**: `sm-ir` emits `Borrow(AdtPayload)` for real source
  (`match value { MyEnum::Variant(ref inner) => ... }`), proven by a
  real-source positive E2E test
  (`vm_runs_adt_payload_ownership_positive_e2e_path`,
  `crates/sm-vm/src/semcode_vm.rs`). No source syntax produces
  `Write(AdtPayload)` - the language has no mutable ADT-payload
  reassignment - so lowering never emits it, and the producer boundary
  (`emit_semcode` in `crates/sm-ir/src/legacy_lowering.rs`) fails closed if
  one is ever constructed internally/synthetically.
- **Runtime Borrows**: `sm-vm` processes `Borrow(AdtPayload)` in match
  expressions and enforces overlap against later writes through the same
  `AccessPath`/`access_paths_overlap` machinery used for tuple/record/
  sequence paths.
- **Admission contract (#1718)**: `Borrow(AdtPayload)` requires
  `CAP_OWNERSHIP_ADT_BORROW_PATHS`, carried starting `HEADER_V21`/`SEMCOD21`
  (rev 22). `Write(AdtPayload)` is rejected unconditionally, under every
  header including `SEMCOD21`, regardless of capability - this is not a
  missing-capability gap; promoting it requires a new, separately authorized
  contract change. Full evidence and rationale:
  `docs/roadmap/stable_foundation/ssf08_1718_path_family_contract_decision.md`
  and `docs/spec/runtime_ownership.md`.
