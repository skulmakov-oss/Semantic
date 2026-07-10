# GPU QuadTile128 Upload Boundary Closeout

## Purpose

This document closes the controlled GPU transport qualification slice for `GpuQuadTile128`. It qualifies byte exposure at the visual/backend boundary while preserving semantic-core ownership of quad meaning.

## Issue

This PR closes #1417. The work belongs to parent #1404; the umbrella remains open for its remaining closeout work.

## Starting main commit

The starting `main` commit was `1644fd8643937427265e39f415ce6b553044f4be`.

## Prior slice

PR #1485 introduced `GpuQuadTile128` in `prom-ui-backend-native`, including its split/join helpers, deterministic conversions, and layout qualification. That slice established the visual transport representation without byte-cast authorization.

## Upload boundary

This slice adds the read-only `gpu_quad_tiles_as_bytes` boundary for already-converted `GpuQuadTile128` values. It is a borrowed byte view only: it allocates nothing, creates no GPU buffer, performs no WGPU submission, and does not own transfer or use lifetime.

## Pod/Zeroable qualification

`GpuQuadTile128` derives `bytemuck::Pod` and `bytemuck::Zeroable` under the existing `wgpu-backend` feature. The derive is validated by compile-proof tests and a zeroed-value equality test. No manual unsafe implementation is used.

## Byte-slice helper

The public helper uses `bytemuck::cast_slice` and accepts only `&[GpuQuadTile128]`. There is no mutable byte helper. Core `QuadTile128` must first be converted into the visual transport type.

## WGSL mirror

`GPU_QUAD_TILE128_WGSL` contains the layout mirror:

```text
struct GpuQuadTile128 {
    t: vec4<u32>,
    f: vec4<u32>,
};
```

This is a layout contract only. It is not a shader file, shader module, or bind-group declaration.

## Core versus transport

`semantic-core-quad::QuadTile128` remains canonical semantic storage and is not byte-cast qualified. `GpuQuadTile128` is owned by the visual/backend crate and is the only upload-surface type in this boundary. Conversion to and from core tiles remains deterministic and tested.

## Dependency direction

The existing optional dependency direction is from `prom-ui-backend-native` to `semantic-core-quad` through `wgpu-backend`. No dependency was added to `semantic-core-quad`, and no Cargo or lockfile change is part of this slice.

## Feature gating

The transport module and its tests remain behind `wgpu-backend`. The existing optional `bytemuck` dependency supplies the derive and cast operations; no new dependency is required.

## No semantic authority

Byte exposure transports already-qualified values. It does not grant the visual/backend layer semantic authority, alter truth-policy behavior, or change core execution.

## No renderer/runtime integration

This PR adds no renderer integration, GPU buffers, shader files, runtime upload submission, frame rendering, or visual metadata. Production renderer readiness and a full WGPU upload pipeline are explicit non-claims.

## Testing inventory

The prior seven transport tests remain, covering size/alignment/offsets, word ordering, split/join, core-plane roundtrip, field correspondence, register-array roundtrip, and zero/default transport. New tests cover Pod/Zeroable, byte-slice length, typed byte roundtrip, WGSL mirror presence, and the core-versus-transport upload boundary.

## No-default warning

The pre-existing native no-default compile blocker is outside this PR and is not claimed as a pass. The relevant guard for this slice is dependency-tree isolation: `semantic-core-quad` is absent from the no-default tree and present in the `wgpu-backend` tree.

## Acceptance mapping for #1417

- Core/visual memory boundary documented -> module rustdoc and this closeout
- `QuadTile128` alignment decision explicit -> retained static layout assertions
- No WGPU dependency added to `semantic-core-quad` -> Cargo boundary unchanged
- No bytemuck dependency added to `semantic-core-quad` -> derive remains visual/backend-owned
- GPU-facing layout uses `[u32; 4]` -> `GpuQuadTile128` fields and WGSL mirror
- Byte-casting permitted only after layout tests -> Pod/Zeroable and byte-boundary tests
- Static layout assertions exist -> size, alignment, and field-offset assertions retained
- Conversion helpers deterministic and tested -> split/join and core/transport tests retained
- Existing tile/bank behavior remains intact -> no core or runtime behavior changes

This PR closes #1417. It does not claim production renderer readiness or a complete WGPU upload pipeline.

## Explicit non-claims

This closeout does not claim GPU buffer creation, upload submission, shader compilation, render integration, runtime behavior changes, semantic authority, or cross-platform byte-endianness beyond typed `u32` word order.

## Remaining work

The remaining tracked work is the #1404 umbrella closeout. No additional renderer or runtime work is included here.
