# no_std Support Matrix

This document defines the current `no_std` and `alloc` boundaries in the workspace-crate layout.

## Build Modes

- `std` (default): full toolchain (`smc-cli` + workspace compiler pipeline).
- `no_std` (`--no-default-features`): core/runtime-safe components only.
- `alloc` (crate-level): parser/semantics core logic without OS/file-system dependencies.

## Crate Matrix

- `crates/ton618-core`: `no_std` + `alloc` native.
- `crates/sm-front`: `alloc`-capable; `std` optional.
- `crates/sm-sema`:
  - `alloc_core` module: alloc-native smc core.
  - `std_adapters` module: std-only glue (module loading, path/IO, diagnostic rendering context).
- `crates/smc-cli`: `std` only.

## Semantics Split (Contract)

- `alloc_core` contains:
  - type/symbol tables and smc policies
  - import/export policy core checks
  - re-export/cycle/collision core helpers
  - pure law/when helpers and folding detectors
- `std_adapters` contains only:
  - provider/path orchestration for module loading
  - conversion of core policy errors into rendered diagnostics
  - CLI-facing std integration points

No file-system/path canonicalization is allowed inside `alloc_core`.

## Required Checks

Run before merge:

```powershell
cargo test -q
cargo check --no-default-features --quiet
cargo check -p sm-front --no-default-features --features alloc --quiet
cargo check -p sm-sema --no-default-features --features alloc --quiet
```

`sm-front`'s alloc-only check is enforced in CI (`check-no-std` job). In that
mode `sm-profile` is built with `alloc` only; `std` (and serde) are enabled
only through `sm-front/std` (#1666).

## Scope

`no_std` is intended for embedding core compiler/runtime primitives into VectorOS contexts.
`smc-cli` and std adapters remain host-side orchestration layers.
