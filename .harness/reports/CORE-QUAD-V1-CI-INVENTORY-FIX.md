# Harness Report: CORE-QUAD-V1-CI-INVENTORY-FIX

## Status
* **Starting Commit:** `e8d926f985c4d92f341d840a83a5749f952767b6`
* **Changed Files:**
  * `.harness/current.task.yaml`
  * `.harness/reports/CORE-QUAD-V1-CI-INVENTORY-FIX.md`
  * `tests/legacy_guards.rs`

## Root Cause Analysis
The CI boundary enforcement test `legacy_guards.rs` explicitly checks for the legacy compatibility crate name across all tracked `rs`, `md`, `toml`, and `lock` files. The new compatibility rollout policy `docs/roadmap/core_quad/v1_compatibility_rollout.md` naturally mentions this compatibility crate name, which caused the test to fail because this new document path was not listed in the rigid explicit inventory. Because `cargo test --test legacy_guards` runs inside boundary enforcement and `cargo test --all-targets` also picks it up, this caused a repeated failure across CI jobs.

## Resolution
Added the precise path to the inventory in `tests/legacy_guards.rs`:
`"./docs/roadmap/core_quad/v1_compatibility_rollout.md"`

This strictly preserves the nature of the test without widening any wildcard rules or stripping the compatibility terms out of the documentation.

## Explicit Non-Changes
* Did not change the compatibility wording in the documentation.
* Did not weaken the legacy guard scan.
* Did not exclude `docs/roadmap/core_quad/**`.
* Did not modify any production code.
* Unrelated untracked files are perfectly preserved.

## Verification
* `cargo fmt --all --check`: Passed
* `cargo test --test legacy_guards --quiet`: Passed
* `cargo test --all-targets --quiet`: Passed
* `cargo test -p semantic-core-quad --quiet`: Passed
* `git diff --check`: Passed
* `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/harness-check.ps1`: Passed

## Pushed Commit
The resolved fix is pushed as a single commit into the existing `core-quad/v1-compat-mask-delta` branch.
