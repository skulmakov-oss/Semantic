# PCC Stdlib Negative Diagnostics Plan

Status: PCC-STDLIB-3A fixture corpus + PCC-STDLIB-3B harness

## Goal

Define the first negative fixture corpus for Practical Core Stdlib v0 diagnostics.

## Scope

This plan covers negative fixtures for:

- non-text `print(...)`;
- collection printing;
- `to_text(record)`;
- `to_text(collection)`;
- premature stdlib namespace usage.

## Fixture Path

```text
tests/fixtures/pcc/stdlib/fail/
```

## Fixture Matrix

| Fixture | Expected failure |
| --- | --- |
| `print_i32.sm` | `print` only admits `text` |
| `print_bool.sm` | `print` only admits `text` |
| `print_quad.sm` | `print` only admits `text` |
| `print_sequence.sm` | no collection printing / formatting |
| `print_map.sm` | no map printing / formatting |
| `to_text_record.sm` | `to_text(record)` out of scope |
| `to_text_sequence.sm` | `to_text(collection)` out of scope |
| `unknown_std_namespace.sm` | stdlib module namespace not admitted yet |

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
| `print_i32.sm` | `E0201` | builtin `print` expects text, got I32 |
| `print_bool.sm` | `E0201` | builtin `print` expects text, got Bool |
| `print_quad.sm` | `E0201` | builtin `print` expects text, got Quad |
| `print_sequence.sm` | `E0201` | builtin `print` expects text, got Sequence |
| `print_map.sm` | `E0201` | builtin `print` expects text, got Map |
| `to_text_record.sm` | `E0201` | builtin `to_text` does not yet support record type `Sensor` |
| `to_text_sequence.sm` | `E0201` | builtin `to_text` currently supports text, bool, i32, u32, and quad |
| `unknown_std_namespace.sm` | `E0201` | unknown enum type `text` in constructor |

## Follow-Up

- PCC-STDLIB-3B: add compile-fail harness for stdlib diagnostics;
- PCC-STDLIB-4: wire stdlib diagnostics into 7hell;
- PCC-STDLIB-5: stdlib v0 closeout.

## Harness Status

`PCC-STDLIB-3B` adds:

```text
tests/pcc_stdlib_negative.rs
```

The harness asserts:

- each stdlib-negative fixture fails;
- broad diagnostic markers are present;
- no panic occurs.

It intentionally does not assert exact spans or full diagnostic rendering.

## 7hell Status

`PCC-STDLIB-4` wires the stdlib negative diagnostics harness into Hell 6:

```text
cargo test --test pcc_stdlib_negative
```

The 7hell runner remains:

- linear;
- hardcoded;
- fail-fast;
- without a group selector;
- without a fixture registry.

## Explicit Non-Goals

- No expansion of `print`.
- No expansion of `to_text`.
- No `text::`, `core::`, or `seq::` namespaces.
- No import policy redesign.
- No formatting API.
- No host ABI widening.
- No harness wiring yet.
- No exact span assertions yet.

## Closeout

The completed contour summary lives in
[stdlib_v0_closeout.md](stdlib_v0_closeout.md).
