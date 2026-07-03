# PCC Collections Core Closeout

Status: closed for the current PCC Collections v0 contour

## Completed Slices

- PCC-COLL-0: current collections admitted surface audit
- PCC-COLL-1: collections core contract
- PCC-COLL-2: standalone canonical `collections_core` example
- PCC-COLL-3A: collections negative diagnostics fixture corpus
- PCC-COLL-3B: collections negative diagnostics harness
- PCC-COLL-4: 7hell collections diagnostics wiring

## Practical-Safe Collections Surface

The following collections surface is qualified for the current PCC contour.

### Sequence

- `Sequence(T)`;
- sequence literals where admitted;
- indexing with admitted integer index type;
- `for` over admitted sequence forms;
- `len`;
- `is_empty`;
- `contains`;
- `push`;
- `prepend`;
- `pop`.

### Map

- `Map(K, V)`;
- `map_empty`;
- `map_set`;
- `map_get`;
- `map_contains`.

## Canonical Examples

Primary standalone collections example:

- `examples/canonical/collections_core/src/main.sm`

Mixed practical anchors:

- `examples/canonical/text_collections_toolbox/src/main.sm`
- `examples/canonical/cli_batch_core/src/main.sm`

## Negative Diagnostics Coverage

Covered by:

- `tests/pcc_collections_negative.rs`

Fixtures:

- `tests/fixtures/pcc/collections/fail/map_remove_unsupported.sm`
- `tests/fixtures/pcc/collections/fail/map_iteration_unsupported.sm`
- `tests/fixtures/pcc/collections/fail/to_text_sequence.sm`
- `tests/fixtures/pcc/collections/fail/to_text_map.sm`
- `tests/fixtures/pcc/collections/fail/sequence_contains_wrong_type.sm`
- `tests/fixtures/pcc/collections/fail/map_set_wrong_key_type.sm`
- `tests/fixtures/pcc/collections/fail/map_set_wrong_value_type.sm`
- `tests/fixtures/pcc/collections/fail/sequence_index_wrong_type.sm`

## 7hell Coverage

Hell 6 now runs:

```bash
cargo test --test pcc_collections_negative
```

The 7hell runner remains:

- linear;
- hardcoded;
- fail-fast;
- without `--group` selector;
- without fixture registry.

## Documented Current Boundaries

Still out of scope for current Collections v0:

- `map_remove` / remove;
- map iteration;
- collection formatting;
- `to_text(collection)`;
- `print(collection)`;
- generic collection abstraction layer;
- collection traits;
- sets;
- ordered maps;
- iterator protocol;
- collection serialization;
- host-facing collection ABI widening;
- finalized missing-key trap semantics;
- finalized out-of-bounds trap semantics;
- finalized `pop` empty semantics.

## Current Observed Negative Markers

- `map_remove_unsupported.sm`:
  - `E0201`
  - unknown function `map_remove`
- `map_iteration_unsupported.sm`:
  - `E0201`
  - `Iterable` contract boundary for `for`
- `to_text_sequence.sm` / `to_text_map.sm`:
  - `E0201`
  - `to_text` supports only `text`, `bool`, `i32`, `u32`, `quad`
- `sequence_contains_wrong_type.sm`:
  - `E0201`
  - `contains` second argument type mismatch
- `map_set_wrong_key_type.sm`:
  - `E0201`
  - key type mismatch
- `map_set_wrong_value_type.sm`:
  - `E0201`
  - value type mismatch
- `sequence_index_wrong_type.sm`:
  - `E0201`
  - sequence indexing requires `i32`

Exact diagnostic text and spans are intentionally not over-specified.

## Validation

Passed:

```bash
cargo test -q --test pcc_collections_negative
powershell -ExecutionPolicy Bypass -File .\\tools\\7hell\\run.ps1
```

Also covered through:

- `tests/canonical_examples.rs`
- `tests/cli_public_smoke_matrix.rs`

## Next PCC Contour

Recommended next practical contour:

```text
Stdlib v0
```

Reason:

Control Flow, Text Core, and Collections v0 are now qualified enough for current PCC. The next practical need is to define which helpers are part of the canonical public stdlib surface and which are only current builtins or provisional helpers.
