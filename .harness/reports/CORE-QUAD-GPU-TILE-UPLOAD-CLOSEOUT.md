# CORE-QUAD-GPU-TILE-UPLOAD-CLOSEOUT

Starting main commit: `1644fd8643937427265e39f415ce6b553044f4be`

Branch: `core-quad/gpu-tile-upload-closeout`

Issue: #1417. Parent: #1404.

Exact five-file boundary:

- `.harness/current.task.yaml`
- `.harness/reports/CORE-QUAD-GPU-TILE-UPLOAD-CLOSEOUT.md`
- `crates/prom-ui-backend-native/src/quad_tile_upload.rs`
- `crates/prom-ui-backend-native/tests/gpu_quad_tile128_transport.rs`
- `docs/roadmap/core_quad/gpu_quad_tile128_upload_closeout.md`

## Scope evidence

PR #1485's transport state was reviewed: `GpuQuadTile128`, split/join helpers, deterministic core conversions, and static layout qualification were already present. This slice retains those guarantees and adds Pod/Zeroable qualification, a read-only byte-slice helper, and the WGSL mirror layout contract.

`bytemuck` was already an optional dependency under the existing `wgpu-backend` feature. No Cargo or `Cargo.lock` file changed, and `semantic-core-quad` was not modified.

The helper is `gpu_quad_tiles_as_bytes(&[GpuQuadTile128]) -> &[u8]` and uses `bytemuck::cast_slice`. It allocates no memory, creates no GPU buffer, submits no runtime upload, and has no mutable byte variant. The WGSL mirror is a string contract only.

## Tests and boundaries

Static size, alignment, and field-offset assertions remain. Existing split/join and conversion tests remain. New tests cover:

- Pod and Zeroable compile proof and zeroed default;
- byte-slice length, including an empty slice;
- typed byte exposure roundtrip;
- required WGSL mirror shape;
- core tile conversion before byte exposure and roundtrip preservation.

No GPU buffer, shader, renderer, or runtime-upload integration was added. `semantic-core-quad` remains the semantic owner; `GpuQuadTile128` remains visual transport state.

## Dependency isolation and warning

The no-default dependency tree was checked and does not contain `semantic-core-quad`. The `wgpu-backend` dependency tree contains `semantic-core-quad`. The pre-existing native no-default compilation blocker outside this PR was not run or claimed as a pass; dependency-tree isolation is the relevant guard for this slice.

## Verification

The exact verification sequence is:

```text
cargo +1.93.1 tree -p prom-ui-backend-native --no-default-features
cargo +1.93.1 tree -p prom-ui-backend-native --features wgpu-backend
cargo +1.93.1 fmt --all --check
cargo +1.93.1 clippy --workspace --all-targets -- -D warnings
cargo +1.93.1 clippy -p prom-ui-backend-native --all-targets --features wgpu-backend -- -D warnings
cargo +1.93.1 test -p prom-ui-backend-native --features wgpu-backend --test gpu_quad_tile128_transport -- --nocapture
cargo +1.93.1 test -p prom-ui-backend-native --features wgpu-backend --quiet
cargo +1.93.1 check -p prom-ui-backend-native --all-features
cargo +1.93.1 test -p semantic-core-quad --quiet
cargo +1.93.1 test -p semantic-core-capsule --quiet
cargo +1.93.1 test --test legacy_guards --quiet
cargo +1.93.1 test --all-targets --quiet
git diff --check
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/harness-check.ps1
```

The dependency isolation checks, formatting, clippy, transport tests, feature tests, core tests, capsule tests, legacy guards, root all-target tests, diff check, and harness check completed successfully. The release/published CI result is intentionally not recorded here.

## Acceptance mapping for #1417

- Core/visual memory boundary documented -> module rustdoc and roadmap closeout
- `QuadTile128` alignment decision explicit -> retained static assertions
- No WGPU dependency added to `semantic-core-quad` -> unchanged dependency boundary
- No bytemuck dependency added to `semantic-core-quad` -> visual/backend-only derive
- GPU-facing layout uses `[u32; 4]` -> transport fields and WGSL mirror
- Byte-casting permitted only after layout tests -> Pod/Zeroable and byte tests
- Static layout assertions exist -> retained size/alignment/offset checks
- Conversion helpers deterministic and tested -> retained conversion inventory
- Existing tile/bank behavior remains intact -> no core or runtime changes

This PR closes #1417. EQUIV, renderer/runtime integration, GPU buffers, shader files, and full WGPU upload submission remain outside this slice. #1404 is not complete.
