# GPU `QuadTile128` Transport Representation

## Purpose

Define the first controlled transport slice for the qualified core tile while
keeping semantic storage and visual transport ownership separate.

## Issue

- Slice 1 of 2 for #1417.
- Parent: #1404.

This PR does not close #1417.

## Starting main commit

`890f17439eaaa8f89440cfdd017a8c7afb31f584`

## Owner boundary

- Core semantic owner: `semantic-core-quad` and `QuadTile128`.
- Transport owner: `prom-ui-backend-native` and `GpuQuadTile128`.

The transport type does not own Semantic truth, admission, execution, or quad
operations. It is a representation consumed by a later admitted visual path.

## Dependency direction

```text
prom-ui-backend-native -> semantic-core-quad
```

The reverse dependency is forbidden. `semantic-core-quad` remains free of
WGPU and byte-upload assumptions.

## Core layout

`QuadTile128` remains canonical CPU/core semantic storage with `repr(C,
align(16))`, size 32 bytes, alignment 16 bytes, truth plane at offset 0, and
falsity plane at offset 16. Its `u128` planes remain owned by the core.

## GPU transport layout

Feature-gated `GpuQuadTile128` is:

```rust
#[repr(C, align(16))]
pub struct GpuQuadTile128 {
    pub t: [u32; 4],
    pub f: [u32; 4],
}
```

It is 32 bytes, aligned to 16 bytes, and has field offsets 0 and 16. The
transport fields represent truth and falsity planes as portable words.

## Word ordering

`split_u128` and `join_u128` define least-significant 32-bit word first:

```text
words[0] = bits 0..31
words[1] = bits 32..63
words[2] = bits 64..95
words[3] = bits 96..127
```

This is a word-order contract, not a host byte-order claim.

## Conversion APIs

Core-to-transport conversion reads only `true_plane()` and `false_plane()` and
splits them into words. Transport-to-core conversion joins the two word arrays
and calls `QuadTile128::from_planes`. Both conversions are deterministic and
allocation-free.

## Static layout assertions

The transport module asserts size 32, alignment 16, truth offset 0, and falsity
offset 16 at compile time. Integration tests repeat these assertions through
the public feature-gated surface.

## Feature gating

The optional `semantic-core-quad` dependency is enabled only by the existing
`wgpu-backend` feature. The transport module and its re-exports are gated by
the same feature. Default and no-default builds do not acquire the core-quad
dependency.

## No-std/default-feature posture

The default native backend remains unchanged. `--no-default-features` remains a
core-free native-backend configuration. The WGPU feature is an opt-in visual
transport path and does not alter core feature ownership.

## Semantic non-authority

`GpuQuadTile128` does not perform semantic operations, admission, execution,
rendering, buffer submission, or effect authorization. A transport value is not
Semantic truth, and successful conversion is not semantic validation.

## Why transport uses `[u32; 4]`

Four 32-bit words make alignment and layout explicit for a portable transport
boundary and avoid assuming that a graphics API consumes Rust `u128` layout
directly.

## Why core keeps `u128` planes

The core representation preserves the existing semantic-storage contract and
its efficient plane operations. Transport concerns do not redefine that core
representation.

## Byte-cast deferral

This slice does not derive or implement Pod/Zeroable, authorize raw byte
casting, or add byte-slice upload helpers. Layout qualification precedes any
byte-cast decision.

## WGSL deferral

No WGSL mirror or shader source is added. Word-array shader mapping remains part
of the final #1417 slice.

## Testing inventory

The feature-gated integration suite covers layout, known word order, split/join
vectors, core plane roundtrip, field-to-plane correspondence, register-array
roundtrip, and zero/default transport. Tests require no adapter, device,
window, display server, native surface, or physical GPU.

## Explicit non-changes

No `semantic-core-quad` source, `prom-ui-runtime`, shader, WGPU buffer, byte
upload, Pod/Zeroable implementation, renderer behavior, or unrelated crate is
changed.

## Second slice

The final #1417 slice remains responsible for Pod/Zeroable qualification,
byte-slice boundary helpers, WGSL mirror, byte-order tests, and issue closeout.
