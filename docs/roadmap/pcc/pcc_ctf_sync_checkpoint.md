# PCC / CTF Sync Checkpoint

## Status

Checkpoint after closing the first Practical Core phase.

This document synchronizes the completed PCC contours with the Core Trust Freeze
lane.

It does not introduce new language features.

## Purpose

The first PCC practical phase closed the following contours:

- Control Flow Core
- Text Core
- Collections v0
- Stdlib v0

This checkpoint verifies whether those practical surfaces require updates to:

- runtime value registry;
- trap taxonomy;
- determinism matrix;
- verifier expectations;
- golden trace policy;
- capability/effect boundary;
- 7hell qualification posture.

## Source Checkpoint

This sync is based on:

- `docs/roadmap/pcc/practical_core_phase_checkpoint.md`
- `docs/roadmap/pcc/control_flow_core_closeout.md`
- `docs/roadmap/pcc/text_core_closeout.md`
- `docs/roadmap/pcc/collections_core_closeout.md`
- `docs/roadmap/pcc/stdlib_v0_closeout.md`

## PCC Contours Reviewed

| PCC contour | Status | Trust impact |
| --- | ---: | --- |
| Control Flow Core | closed | verifier / control-flow / trap / determinism relevant |
| Text Core | closed | runtime value / string table / output relevant |
| Collections v0 | closed | runtime value / container / trap / quota relevant |
| Stdlib v0 | closed | helper / capability / builtin boundary relevant |

## RuntimeValue Registry Sync

### Reviewed Surfaces

The completed PCC phase uses:

- `quad`
- `bool`
- `i32`
- `u32`
- `text`
- `Sequence(T)`
- `Map(K, V)`
- `Option(T)`
- `Result(T, E)`
- function-local mutable variables
- canonical `fn main()` without return type

### Sync Questions

- Are all PCC-admitted value families represented in the current runtime value
  registry?
- Are `text`, `Sequence(T)`, `Map(K, V)`, `Option(T)`, and `Result(T, E)`
  documented as runtime-relevant surfaces?
- Are current collection carriers described clearly enough?
- Are helper-return types such as `len`, `contains`, `map_get`, and `pop`
  aligned with runtime value expectations?

### Current Decision

```text
SYNC-PASS-WITH-FOLLOWUPS
```

## Trap Taxonomy Sync

### Reviewed Negative Contours

Current PCC negative harnesses cover:

- control-flow misuse;
- text misuse;
- collection misuse;
- stdlib / helper misuse.

Current negative harnesses:

- `tests/pcc_control_flow_negative.rs`
- `tests/pcc_text_negative.rs`
- `tests/pcc_collections_negative.rs`
- `tests/pcc_stdlib_negative.rs`

### Trap / Diagnostic Areas

Potential trust-relevant trap or diagnostic families:

- non-bool `if` / `while` condition;
- `break` / `continue` outside loop;
- missing match fallback arm;
- missing return path;
- invalid text operation;
- invalid `to_text(...)` target;
- invalid `print(...)` target;
- invalid sequence index type;
- invalid collection element / key / value type;
- unsupported map operation;
- unsupported collection formatting;
- missing map key behavior;
- empty `pop` behavior;
- out-of-bounds sequence access.

### Sync Questions

- Which failures are compile-time diagnostics only?
- Which failures should eventually become runtime traps?
- Are missing-key, empty-pop, and out-of-bounds semantics intentionally
  unresolved?
- Are current broad diagnostic markers acceptable for PCC, or do any need
  stable diagnostic codes?

### Current Decision

```text
SYNC-PASS-WITH-FOLLOWUPS
```

## Determinism Matrix Sync

### Reviewed Deterministic Surfaces

PCC examples now rely on deterministic behavior for:

- `match` branch selection;
- `while` / `loop`;
- `break;` / `continue;`;
- text equality and concatenation;
- `to_text(...)` for admitted scalar families;
- sequence helper behavior;
- map helper behavior;
- `assert`;
- `print(text)` as observable output.

### Sync Questions

- Is text concatenation deterministic across platforms?
- Is `to_text(...)` formatting deterministic for admitted scalar families?
- Is map behavior deterministic enough for canonical examples?
- Is iteration over `Sequence(T)` deterministic?
- Is any map iteration intentionally out of scope to avoid ordering ambiguity?
- Is `print(text)` deterministic enough as an observable effect, or must it
  remain capability-bound only?

### Current Decision

```text
SYNC-PASS-WITH-FOLLOWUPS
```

## Verifier Expectations Sync

### PCC Surfaces That May Affect Verifier Expectations

- loops and jumps;
- terminal return paths;
- `match` lowering;
- `Option(T)` / `Result(T, E)` constructor flow;
- text constants and string table use;
- collection helper calls;
- builtin / helper calls;
- `print(text)` effect path;
- `assert` behavior.

### Sync Questions

- Does verifier admission already cover the control-flow shapes now
  canonicalized?
- Are loop / jump targets adequately verified?
- Are helper calls checked through a stable builtin / helper path?
- Are text constants / string references validated?
- Are `print(text)` and other helper calls clearly separated from host ABI
  widening?
- Do any PCC helpers require capability metadata now, or only later?

### Current Decision

```text
SYNC-PASS-WITH-FOLLOWUPS
```

## Golden Trace Policy Sync

### Candidate Golden Traces

Potential future golden traces:

- `match_control_flow`
- `option_result_control_flow`
- `loop_control_flow`
- `text_core`
- `collections_core`
- `stdlib_v0_helpers`

### Sync Questions

- Should canonical examples produce golden traces now?
- Should traces include helper calls?
- Should `print(text)` output be included in trace expectations?
- Should negative diagnostics have golden output snapshots, or only broad
  marker assertions?
- Should 7hell consume golden traces later?

### Current Decision

```text
SYNC-PASS-WITH-FOLLOWUPS
```

## Capability / Effect Boundary Sync

### Reviewed Helper

- `print(text)`

### Current PCC Boundary

`print(text)` is canonical-safe as practical surface, but it must remain
capability-aware in principle.

This checkpoint does not widen host ABI or PROMETHEUS effect policy.

### Sync Questions

- Is `print(text)` currently treated as a builtin helper, host call, or
  provisional output surface?
- Does it require explicit capability declaration now?
- Should future stdlib contract move `print(text)` under `debug` / `io` /
  capability-bound namespace?
- Does any canonical example risk implying unrestricted host output?

### Current Decision

```text
SYNC-PASS-WITH-FOLLOWUPS
```

## 7hell Sync

Hell 6 currently runs:

```text
cargo test --test pcc_control_flow_negative
cargo test --test pcc_text_negative
cargo test --test pcc_collections_negative
cargo test --test pcc_stdlib_negative
```

The runner remains:

- linear;
- hardcoded;
- fail-fast;
- without `--group` selector;
- without fixture registry.

### Sync Questions

- Is the current fixed Hell 6 model sufficient for PCC phase closeout?
- Should group registry remain deferred?
- Should positive canonical examples also be explicitly named in 7hell docs?
- Should 7hell eventually become the single qualification entrypoint for PCC?

### Current Decision

```text
SYNC-PASS-WITH-FOLLOWUPS
```

## Known Accepted Boundaries

The following boundaries remain accepted after the PCC phase:

### Control Flow

- canonical `fn main()` uses no return type;
- current `match` requires explicit `_` fallback arm;
- expression-valued `match` is out of scope;
- exhaustiveness checking is future work;
- `break expr`, labeled loops, iterators, and advanced loop forms are out of scope.

### Text

- interpolation is out of scope;
- multiline / raw strings are out of scope;
- `text + scalar` is out of scope;
- `to_text(record)` and `to_text(collection)` are out of scope;
- text ordering and locale-aware comparison are out of scope;
- host-facing text ABI widening is out of scope.

### Collections

- `map_remove` is out of scope;
- map iteration is out of scope;
- collection formatting is out of scope;
- generic collection traits are out of scope;
- missing-key, out-of-bounds, and empty-pop trap semantics are not fully finalized.

### Stdlib

- final stdlib module layout is out of scope;
- `core.*`, `text.*`, `seq.*`, `map.*` namespacing is future work;
- formatting API is out of scope;
- debug / logging framework is out of scope;
- host ABI widening is out of scope.

## Sync Outcome

### Current Result

```text
SYNC-PASS-WITH-FOLLOWUPS
```

### Follow-Up Summary

The PCC phase is aligned with the current CTF lane, but the trust layer should
record follow-ups for:

- runtime value registry naming / documentation for the new practical surfaces;
- trap taxonomy entries for unresolved collection edges;
- determinism and replay coverage for the remaining collection edge cases;
- capability wording for `print(text)` and helper output surfaces;
- 7hell / trace policy wording for canonical anchors.

### Possible Outcomes

| Outcome | Meaning |
| --- | --- |
| `SYNC-PASS` | PCC phase is aligned with current CTF lane. |
| `SYNC-PASS-WITH-FOLLOWUPS` | PCC phase is acceptable, but follow-up CTF docs/issues are needed. |
| `SYNC-BLOCKED` | PCC phase exposed trust-layer gaps that must be resolved before opening the next contour. |

## Recommended Next Step

Do not open a new PCC practical contour until this checkpoint records one of:

```text
SYNC-PASS
SYNC-PASS-WITH-FOLLOWUPS
```

If the result is `SYNC-BLOCKED`, resolve the blocking trust-layer items first.
