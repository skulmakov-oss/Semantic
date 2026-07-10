# CORE-QUAD-GPU-TILE-TRANSPORT

## Starting main commit

`890f17439eaaa8f89440cfdd017a8c7afb31f584`

## Branch, issue, and parent

- Branch: `core-quad/gpu-tile-transport`
- Issue: #1417
- Parent: #1404
- Slice: 1 of 2; this PR does not close #1417.

## Exact allowed boundary

```text
.harness/current.task.yaml
.harness/reports/CORE-QUAD-GPU-TILE-TRANSPORT.md
Cargo.lock
crates/prom-ui-backend-native/Cargo.toml
crates/prom-ui-backend-native/src/lib.rs
crates/prom-ui-backend-native/src/quad_tile_upload.rs
crates/prom-ui-backend-native/tests/gpu_quad_tile128_transport.rs
docs/roadmap/core_quad/gpu_quad_tile128_transport.md
```

`Cargo.lock` was reviewed and did not require a diff because the local package
inventory already contained `semantic-core-quad`; no unrelated resolution
change was made.

## Existing layout and WGPU owner evidence

Core `QuadTile128` remains `repr(C, align(16))`, 32 bytes, aligned to 16 bytes,
with truth/falsity plane offsets 0/16. Its direct plane accessors and register
conversion APIs remain the semantic source of truth. WGPU context, surface,
pipeline, and existing bytemuck vertex usage remain owned by
`prom-ui-backend-native::wgpu_integration`.

## Dependency direction and feature wiring

```text
prom-ui-backend-native -> semantic-core-quad
```

The optional path dependency is enabled only by the existing `wgpu-backend`
feature. Default and no-default dependency trees do not acquire
`semantic-core-quad`; the WGPU tree does. No dependency was added to
`semantic-core-quad`.

## Transport shape and layout

`GpuQuadTile128` is `repr(C, align(16))` with public `t: [u32; 4]` and
`f: [u32; 4]` fields, deriving only ordinary value traits. Compile-time and
integration assertions qualify size 32, alignment 16, and offsets 0/16.

## Word order and conversions

`split_u128` and `join_u128` use least-significant 32-bit word first. Core-to-
transport reads `true_plane()`/`false_plane()`; transport-to-core calls
`QuadTile128::from_planes` after joining words. Both are deterministic and
allocation-free.

## Integration-test inventory

- layout freeze;
- known least-significant-word ordering;
- deterministic split/join roundtrips;
- core plane roundtrip;
- transport fields matching core planes;
- four-register construction preserving lanes;
- zero/default transport.

Tests use no adapter, device, window, display server, surface, or physical GPU.

## Default/no-default posture

The default native backend remains unchanged. No-default dependency inspection
passes and confirms no core-quad dependency. The requested no-default compile
is currently blocked by pre-existing `prom-ui` no_std errors for missing
`alloc` imports (`Vec`, `String`, `vec!`, `format!`, and `ToString`) in files
outside this PR's allowed boundary. No such files were modified.

## Deferred boundaries

No Pod/Zeroable implementation, byte casting, byte-slice upload helper, WGPU
buffer, WGSL mirror, shader source, runtime upload, buffer submission, or
render integration was added. Those belong to the final #1417 slice.

## Exact verification commands and results

```text
cargo +1.93.1 fmt --all --check                                      PASS
cargo +1.93.1 clippy --workspace --all-targets -- -D warnings        PASS
cargo +1.93.1 clippy -p prom-ui-backend-native --all-targets --features wgpu-backend -- -D warnings PASS
cargo +1.93.1 test -p prom-ui-backend-native --features wgpu-backend --test gpu_quad_tile128_transport -- --nocapture PASS (7 tests)
cargo +1.93.1 test -p prom-ui-backend-native --features wgpu-backend --quiet PASS
cargo +1.93.1 check -p prom-ui-backend-native --no-default-features   BLOCKED by pre-existing prom-ui no_std errors
cargo +1.93.1 check -p prom-ui-backend-native --all-features          PASS
cargo +1.93.1 test -p semantic-core-quad --quiet                      PASS (134 inline, 9 integration)
cargo +1.93.1 test -p semantic-core-capsule --quiet                   PASS (8 tests)
cargo +1.93.1 test --test legacy_guards --quiet                       PASS (10 tests)
cargo +1.93.1 test --all-targets --quiet                              PASS
git diff --check                                                       PASS
dependency tree checks                                                PASS
Cargo.lock unrelated-resolution review                                PASS (no diff)
```

## Explicit non-changes

No `semantic-core-quad` source, `prom-ui-runtime`, `prom-ui`, spec, shader,
WGPU buffer, byte upload, Pod/Zeroable trait, WGSL mirror, renderer behavior,
VM/runtime behavior, or unrelated untracked file was modified or staged.

## Second-slice boundary

The final #1417 slice remains responsible for Pod/Zeroable qualification,
byte-slice boundary helpers, WGSL mirror, byte-order tests, and issue closeout.
