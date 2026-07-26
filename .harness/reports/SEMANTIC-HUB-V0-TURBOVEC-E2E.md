# SEMANTIC-HUB-V0-TURBOVEC-E2E closeout evidence

Issue: #1553 (owner implementation issue for architecture #1526; consumes
policy issues #1371-#1374). Branch: `feat/semantic-hub-v0-turbovec-e2e`.
Base SHA: `8eaae2c54bee166ba71b6b6dd6b367f981c0f7fa` (`origin/main` tip at
task start, matching the issue's recorded baseline exactly).

## Stash / working-tree state at task start

`stash@{0}`: "On tooling/smc-look-ui-frame-e2e: pre-existing uncommitted
Source Style v0 example migration, unrelated to #1365 - stashed to keep
this branch's diff clean" -- pre-existing, unrelated to this task, left
untouched throughout.

## What was built

- `crates/semantic-hub` -- generic Hub contract + in-process runtime.
  Zero path-dependencies on any other in-repo crate (`cargo tree -p
  semantic-hub --no-default-features` is empty besides the package
  itself).
- `crates/semantic-hub-turbovec` -- reference adapter over the real
  `turbovec = "=0.9.0"` crate (crates.io, MIT,
  https://github.com/RyanCodrai/turbovec).
- `crates/smc-cli/src/hub.rs` -- new private CLI module wiring `smc hub
  tools|describe|invoke|audit`, dispatched from `app.rs`.
- Six new docs (`docs/architecture/semantic_hub_v0.md`,
  `docs/spec/hub/{hub_api_v0,hub_adapter_contract_v0,turbovec_adapter_v0}.md`,
  `docs/security/semantic_hub_threat_model_v0.md`,
  `docs/privacy/semantic_hub_data_policy_v0.md`) plus a `docs/spec/cli.md`
  update.
- Canonical request fixtures under `tests/fixtures/hub/` with a
  `.gitattributes` `text eol=lf` rule and `tests/hub_fixture_line_ending_guard.rs`
  guard (mirrors the PR #1552 fix pattern exactly).
- `tests/hub_cli.rs` -- 11 real subprocess integration tests against the
  built `smc` binary.
- `crates/semantic-hub-turbovec/tests/determinism_qualification.rs` -- 3
  empirical determinism tests.
- `crates/semantic-hub-turbovec/benches/hub_turbovec_bench.rs` -- manual
  `Instant`-based benchmark evidence (no new harness dependency), including
  a diagnostic that isolates and quantifies the cold-cache reload cost.
- `tests/public_api_contracts.rs` + two new golden snapshots, registering
  both new crates' public surfaces with the existing guard mechanism.

## Real bugs found via dogfooding the built binary (not just unit tests)

1. **Pre-dispatch rejections showed `tool_version: 0.0.0`** even for a
   known, registered tool (e.g. a capability-denial rejection), because
   `finish_pre_dispatch` always used an internal placeholder descriptor.
   Fixed in `crates/semantic-hub/src/runtime.rs`: it now looks up the real
   registered descriptor first, falling back to the placeholder only for a
   genuinely unknown tool_id. Regression test:
   `capability_denial_records_the_real_tool_version_not_a_placeholder`.
2. **The persisted audit log became unparseable after any rejection.** A
   capability-denial (or any non-success) audit record set both
   `status_code` and `fault_code` to the same specific fault code (e.g.
   `"CapabilityDenied"`), but `HubAuditRecord::from_canonical_line`'s
   parser only recognizes the 5 reply-status names for `status_code` --
   so the very next `smc hub invoke` call failed with
   `AuditProvenanceFailure: corrupt audit log`. Fixed: `status_code` is now
   always the reply-status discriminant, `fault_code` the specific fault,
   kept structurally distinct. Regression test:
   `rejection_audit_record_status_code_and_fault_code_are_distinct_and_round_trip`,
   plus CLI-level coverage in `capability_denial_is_rejected_and_hub_remains_usable_afterward`.
3. A benchmark-only bug (overlapping external ids across batch-size runs
   in `bench_insertion_throughput`) was also found and fixed while
   gathering performance evidence; not a product bug.
4. **Public API golden snapshots drifted after `cargo fmt`.** Several
   `pub use` re-export lines in `crates/semantic-hub/src/lib.rs` and one
   `pub fn new(...)` in `crates/semantic-hub-turbovec/src/lib.rs` exceeded
   rustfmt's line-width limit and were wrapped onto multiple physical
   lines; the golden-snapshot normalizer's continuation-handling only
   covers `pub fn`, not `pub use`, so the wrapped re-exports produced a
   truncated snapshot line. Fixed by splitting the long `pub use`
   statements into several shorter ones (all under the width limit) and
   shortening the `new()` signature via a direct `HubResourceBudget`
   import instead of a fully-qualified path -- both fixes keep every
   tracked `pub` line on one physical line permanently, rather than
   pinning the golden snapshot to one incidental rustfmt-wrap outcome.

## Test evidence (exact counts, this pass)

- `cargo test -p semantic-hub`: **89 passed**, 0 failed.
- `cargo test -p semantic-hub-turbovec` (unit + `tests/` dir): **25 + 3 =
  28 passed**, 0 failed (25 in `src/lib.rs`'s `#[cfg(test)]` modules, 3 in
  `tests/determinism_qualification.rs`).
- `cargo test --test hub_cli` (root package, real subprocess against the
  built `smc` binary): **11 passed**, 0 failed.
- `cargo test --test hub_fixture_line_ending_guard`: **2 passed**.
- `cargo test --test public_api_contracts public_api_inventory_matches_checked_in_contract_snapshots`:
  **1 passed** (both new crates registered).
- `cargo clippy -p semantic-hub -p semantic-hub-turbovec --all-targets -- -D warnings`:
  clean, zero warnings.
- `cargo fmt --all --check`: clean after `cargo fmt --all`.
- `pwsh -File scripts/harness-check.ps1`: `[harness] ok`.
- `git diff --check --cached`: clean.

## Determinism qualification (measured, not assumed)

Read turbovec's actual vendored source
(`~/.cargo/registry/src/.../turbovec-0.9.0/src/rotation.rs`): rotation
matrix construction uses a **fixed** internal constant
`ROTATION_SEED: u64 = 42` (ChaCha8Rng + QR decomposition), not OS
randomness. Empirically verified on this machine (x86_64, Windows):
repeated identical search on one loaded index is byte-identical across 10
repetitions; reloading fresh from the persisted `.tvim` file (simulating
separate CLI process invocations) is byte-identical across 3 independent
loads; exact-duplicate-vector ties produce a stable, repeatable order.
NOT verified: cross-CPU-backend byte identity (turbovec dispatches
AVX-512/AVX2/scalar or NEON at runtime; only tested on one machine).
Classified `DeterministicWithSeed` for insert/search/filtered-search,
`Deterministic` for create/describe/remove/reset.

## Benchmark evidence (this machine, dev + bench profile; see file header
for exact command)

Real, quantified finding: because each Hub CLI invocation is one OS
process and the adapter reloads the index fresh from disk (rather than
keeping an in-memory cache across calls), every operation pays a
~250-300ms cold rotation-matrix/codebook/SIMD-blocked-layout construction
cost, roughly **3176x** the cost of a search against an already-warm
in-memory instance (0.085ms) on this machine/dataset. Hub's own dispatch
overhead (admission + catch_unwind + audit, measured via a direct-adapter
baseline) is ~125-150 microseconds -- negligible next to the reload cost.
This is a real, honestly-documented architectural characteristic of the
v0 process-per-invocation design, not a Hub governance overhead problem.

## Dependency boundary evidence

```text
cargo tree -p semantic-hub --no-default-features
  -> semantic-hub only (no in-repo crate dependencies)

cargo tree -i semantic-hub
  -> semantic-hub-turbovec -> smc-cli -> semantic_language (root)
  -> smc-cli -> semantic_language (root)
  (no circular dependency; Semantic Core / sm-* / prom-* crates do not
   depend on Hub in either direction)
```

## Honest limitations / non-claims recorded in the docs

- No subprocess/WASM/remote execution mode implemented (API is
  mode-neutral; only `InProcess` exists).
- `HubTrustClass::InProcessUnisolated` -- explicitly no memory-corruption
  isolation; `turbovec` itself was not independently line-by-line audited.
- `HubFault::PrivacyDenied` exists in the taxonomy but is never actually
  constructed in v0 -- no privacy-based denial policy is implemented yet
  (recorded and propagated only). No "privacy denial" fixture was created
  to avoid misrepresenting this.
- Memory/storage/audit-byte budget dimensions are advisory-only (not hard
  enforceable in-process without OS containment) -- stated honestly, not
  claimed as enforced.
- No file locking across concurrent `smc hub invoke` processes writing the
  same index -- a real, named gap.
- Digests (`HubDigest`, FNV-1a-64 + length) are non-cryptographic
  correlation fingerprints only, not a signing/tamper-evidence chain
  (tracked separately by issue #1374, not implemented here).
- `index_version` in the search reply is a static placeholder (`1`), not
  real content-based versioning.

## Final local gate results (all green)

- `cargo fmt --all --check`: clean.
- `cargo check --workspace --all-targets --all-features --keep-going`:
  clean, no errors.
- `cargo test --workspace --all-features`: all targets passed, 0 failed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`:
  clean, zero warnings across the entire workspace.
- `cargo test --test public_api_contracts`: all 4 tests passed (including
  both new crates' snapshots).
- `pwsh -File scripts/harness-check.ps1`: `[harness] ok`.
- `git diff --check --cached`: clean.
- Windows `core.autocrlf=true` qualification: this development machine
  already has `core.autocrlf=true` set globally, so the passing
  `tests/hub_fixture_line_ending_guard.rs` run *is* the real qualification
  (not a simulation) -- independently reconfirmed with a direct byte scan
  (`file` command) over every fixture in `tests/fixtures/hub/`, finding no
  CRLF bytes.
- Working tree confirmed clean of scratch/temp state (`git status --short`
  shows only the intended changes; all CLI dogfooding used isolated temp
  directories outside the repo, never the repo root).

## Remaining before merge

- Push branch, open PR, exact-head CI, review remediation -- pending
  explicit next steps per the execution contract, followed by a stop for
  explicit repository-owner merge approval.
