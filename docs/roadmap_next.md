# Roadmap Next (Post-Base Stabilization)

Release-freeze checkpoint after `v0.1`/`v0.2`/`v0.3` close-out:

- `docs/roadmap/language_maturity/release_freeze_post_v03_checkpoint.md`
- `docs/roadmap/language_maturity/release_version_cut_decision.md`
- `docs/roadmap/language_maturity/import_reexport_full_scope.md`
- `docs/roadmap/language_maturity/persistent_incremental_packs_full_scope.md`
- `docs/roadmap/language_maturity/root_legacy_cleanup_full_scope.md`
- `docs/roadmap/language_maturity/arena_first_crystalfold_full_scope.md`

This document tracks the next four closure tracks to move Semantic from "working" to "production-disciplined".

For the next implementation slice, see the internal packed-state substrate roadmap:

- `docs/roadmap/roadmap_pulsar.md`

Pulsar is a planned internal packed-state substrate roadmap. It does not widen the active Core Trust Freeze contour.

Current status:
- `NEXT-1..NEXT-4` are completed for the current post-`v1.1.1` closure layer.
- this document now serves as a close-out record for those tracks, not as an
  active feature queue.

## NEXT-1: Import / Re-export v0.2 to FULL

Status:
- completed for the current import/re-export surface

Goal:
- Close policy edge-cases, symbol-level cycle behavior, collision matrix, and deterministic resolution docs/tests.

Tasks:
1. Fix and document lookup/export order contract.
2. Complete edge-case matrix:
   - missing select symbol
   - kind mismatch
   - alias collisions
   - re-export collisions
   - symbol-level cycles
   - wildcard ambiguity (or explicit "not supported in v0.2")
3. Finalize error pages: `E0242..E0245`.

Acceptance:
- Fixtures + snapshots for all edge-cases.
- Stable deterministic behavior documented in `docs/imports.md` and `docs/exports.md`.

## NEXT-2: Persistent Incremental Packs to FULL

Checkpoint:
- `docs/roadmap/language_maturity/persistent_incremental_packs_full_scope.md`

Status:
- completed for the current CLI cache surface

Goal:
- Complete dependency-aware reuse/invalidation across Ast/Sem/Ir/Exb packs.

Tasks:
1. Dependency-aware `CacheIndex` + graph model.
2. Formal `--trace-cache` reason codes (`SOURCE_CHANGED`, `DEP_CHANGED`, etc.).
3. Strict downstream invalidation by dependency graph.
4. Reuse smoke/perf scenarios.

Acceptance:
- Rebuild reasons are explicit and deterministic.
- Leaf change rebuilds only downstream.

## NEXT-3: Root Legacy Cleanup to FULL

Goal:
- Root contains only shim + bins; no legacy backend sources.

Status:
- completed; see `docs/roadmap/language_maturity/root_legacy_cleanup_full_scope.md`

Acceptance:
- `root/src` clean by policy; guard tests enforce it.

## NEXT-4: Arena-first + CrystalFold as Complete Stages

Checkpoint:
- `docs/roadmap/language_maturity/arena_first_crystalfold_full_scope.md`

Status:
- completed for the current arena/lowering/optimizer contract

Goal:
- Arena-first AST/semantic/lowering invariants finalized.
- CrystalFold formalized as deterministic optimization pass.

Tasks:
1. Remove remaining non-arena ownership tails.
2. Add arena invariants tests.
3. Define CrystalFold stage placement and pass interface.
4. Add correctness/warning/determinism test pack.

Acceptance:
- Stable hash-equivalent outputs for same inputs.
- Stable warning spans and diagnostics.
- Pass contract documented in `docs/opts.md`.
