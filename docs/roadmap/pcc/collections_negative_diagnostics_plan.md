# PCC Collections Negative Diagnostics Plan

Status: PCC-COLL-3A fixture corpus + diagnostics plan

## Goal

Define the first negative fixture corpus for Practical Core collections diagnostics.

## Scope

This plan covers negative fixtures for:

- unsupported map removal;
- unsupported map iteration;
- collection-to-text formatting;
- sequence element type mismatch;
- map key type mismatch;
- map value type mismatch;
- invalid sequence index type.

## Fixture Path

```text
tests/fixtures/pcc/collections/fail/
```

## Fixture Matrix

| Fixture | Expected failure |
| --- | --- |
| `map_remove_unsupported.sm` | `remove` is not part of the current PCC Map contract |
| `map_iteration_unsupported.sm` | map iteration is not part of the current PCC Collections contract |
| `to_text_sequence.sm` | `to_text(collection)` is out of scope |
| `to_text_map.sm` | `to_text(collection)` is out of scope |
| `sequence_contains_wrong_type.sm` | sequence element type mismatch |
| `map_set_wrong_key_type.sm` | map key type mismatch |
| `map_set_wrong_value_type.sm` | map value type mismatch |
| `sequence_index_wrong_type.sm` | invalid sequence index type |

## Harness Policy

The future harness should assert:

- the fixture fails;
- a broad diagnostic marker is present;
- no panic occurs.

It should not assert full diagnostic text or exact spans.

## Manual Probe Results

Record observed markers after running `smc check` manually.

| Fixture | Observed marker | Notes |
| --- | --- | --- |
| `map_remove_unsupported.sm` | `E0201` | unknown function `map_remove` |
| `map_iteration_unsupported.sm` | `E0201` | iterable `for x in collection` currently requires built-in `Sequence(type)`, `i32` range, or a direct record `Iterable` impl |
| `to_text_sequence.sm` | `E0201` | builtin `to_text` currently supports `text`, `bool`, `i32`, `u32`, and `quad` |
| `to_text_map.sm` | `E0201` | builtin `to_text` currently supports `text`, `bool`, `i32`, `u32`, and `quad` |
| `sequence_contains_wrong_type.sm` | `E0201` | builtin `contains` second argument type mismatch |
| `map_set_wrong_key_type.sm` | `E0201` | builtin `map_set` key type mismatch |
| `map_set_wrong_value_type.sm` | `E0201` | builtin `map_set` value type mismatch |
| `sequence_index_wrong_type.sm` | `E0201` | sequence indexing currently requires `i32` index |

## Follow-Up

- PCC-COLL-3B: add compile-fail harness for collections diagnostics;
- PCC-COLL-4: wire collections diagnostics into 7hell;
- PCC-COLL-5: collections closeout.

## Harness Status

`PCC-COLL-3B` adds:

```text
tests/pcc_collections_negative.rs
```

The harness asserts:

- each collections-negative fixture fails;
- broad diagnostic markers are present;
- no panic occurs.

It intentionally does not assert exact spans or full diagnostic rendering.

## 7hell Status

`PCC-COLL-4` wires the collections negative diagnostics harness into Hell 6:

```text
cargo test --test pcc_collections_negative
```

The 7hell runner remains:

- linear;
- hardcoded;
- fail-fast;
- without a group selector;
- without a fixture registry.

## Closeout

The final contour summary will live in `collections_core_closeout.md` once the
collections qualification path is fully closed.

## Explicit Non-Goals

- No diagnostics redesign.
- No `map_remove` addition.
- No map iteration contract.
- No collection formatting API.
- No 7hell integration before a harness exists.
- No exact span assertions yet.
