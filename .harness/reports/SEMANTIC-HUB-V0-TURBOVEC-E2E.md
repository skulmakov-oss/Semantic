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

## CI remediation (PR #1554, first push)

The initial pushed head failed 4 of 9 CI jobs (`boundary-enforcement`,
`public-api-guard`, `runtime-release-gates`, `test-std`) with an identical
root cause across all four:

```text
error: linking with `cc` failed: exit status: 1
= note: rust-lld: error: unable to find library -lopenblas
```

Diagnosis: `turbovec`'s own `Cargo.toml` unconditionally enables
`ndarray`'s `"blas"` feature via a `[target.'cfg(target_os = "linux")']`
(and `"macos"`) dependency section. This requires a system BLAS library
present on the linker path; the GitHub Actions `ubuntu-latest` runner does
not have one preinstalled by default. This did not surface locally because
this development machine is Windows, where turbovec's Cargo.toml does not
enable that feature at all. It also cannot be fixed from
`semantic-hub-turbovec`'s own `Cargo.toml` -- Cargo feature unification is
additive-only, so a downstream crate cannot un-enable a feature an
upstream crate's own target-conditional dependency turns on.

Fix: added one identical CI step ("Install system BLAS (required by the
turbovec Hub dependency on Linux)": `sudo apt-get install -y
libopenblas-dev`) to the 4 affected `ubuntu-latest` jobs in
`.github/workflows/ci.yml`. No other workflow behavior, trigger, or job
was changed. `.harness/current.task.yaml`'s `workflow_changes` flag was
updated to `true` with a full justification, since this is a genuine,
narrowly-scoped environment remediation directly required by the pinned
dependency this issue instructed adding -- not a scope change.

Jobs that passed without this fix and were left untouched: `pr-ready`
(clippy/fmt do not require a full link step), `check-no-std` (the `smc`
binary has `required-features = ["std"]` and is skipped entirely under
`--no-default-features`), `release-bundle-process` (does not invoke a
fresh `cargo build` of the default-features binaries in its current
form), `pcc-qualification-7hell` (runs on `windows-latest`, where
turbovec's Linux/macOS-only BLAS feature is never enabled).

## Codex review (PR #1554, commit `6045b40585`) -- 5 P1 + 1 P2 findings, all resolved

Codex's automated review found 5 real defects (P1) and one real gap (P2)
via genuine analysis of the diff -- none were stylistic/speculative. Each
is listed with its classification and the fix landed in the same PR.

1. **P1 -- CONFIRMED. `handle_create` never checked the admitted
   `resource_budget.vector_dimensions` ceiling** before calling
   `turbovec::IdMapIndex::new`, so a caller could request up to turbovec's
   own much looser `MAX_DIM` (65536) despite a narrower admitted budget
   (default 4096), risking a `dim^2` rotation-matrix allocation beyond
   what was ever admitted. Fixed: `handle()` now passes
   `context.resource_budget.vector_dimensions` through to `handle_create`,
   which rejects with `DimensionExceedsBudget` before constructing
   anything. Regression tests: `create_rejects_dimension_exceeding_the_admitted_budget`
   / `create_within_the_admitted_dimension_budget_is_accepted` (adapter
   unit tests) and a CLI-level equivalent in `tests/hub_cli.rs`.

2. **P1 -- CONFIRMED, SEVERE. The audit capability parser only recognized
   the 11 non-sensitive `HubCapability` names.** A caller's request can
   *grant* (not just require) a sensitive capability such as
   `NetworkAccess` -- `HubCapabilitySet::grant()` does not filter these
   out, and the audit writer serializes every granted capability
   verbatim. The next `smc hub invoke`/`smc hub audit` call then failed to
   parse that capability name back, corrupting the whole audit log --
   the exact same class of bug as the earlier status_code/fault_code
   conflation, but reachable by any caller's request file, not just an
   internal code path. Fixed: the parser now calls `HubCapability::parse`
   (the same exhaustive, already-tested 20-variant parser used
   elsewhere) instead of a hand-duplicated partial match. Regression
   tests: `a_record_granting_every_sensitive_capability_round_trips`
   (library) and `granting_a_sensitive_capability_survives_audit_round_trip`
   (CLI).

3. **P1 -- CONFIRMED, SEVERE. The persisted audit log was read with
   `MAX_INPUT_BYTES` (8 MiB) -- the bound meant for one caller-supplied
   request file, not for `audit.log`, which grows by one record per
   invocation for the lifetime of a project with no retention policy in
   v0.** Once real accumulated history exceeded 8 MiB, every subsequent
   `smc hub invoke`/`smc hub audit` call would permanently fail until the
   user manually deleted the log -- a realistic, inevitable time bomb
   under normal use, not an edge case. Fixed: introduced a separate,
   dedicated `MAX_AUDIT_LOG_BYTES` (512 MiB, generous rather than tuned to
   any specific real-world size) used only for `audit.log`. Regression
   test: `read_bounded_accepts_a_file_over_the_request_file_limit_under_the_audit_log_limit`.

4. **P1 -- CONFIRMED. A mutating operation's effect (the adapter's `.tvim`
   write) could durably commit before that invocation's audit record was
   durably persisted**, since the two are separate atomic writes to
   separate files with no shared transaction. If the process crashed or
   the audit write failed in that window, the mutation applied with zero
   durable audit trace for it. Genuinely full transactional (single-
   commit) semantics across two independent file formats was judged
   disproportionate to implement in this pass; instead landed a real,
   scoped mitigation: a small pending-marker file
   (`.semantic/hub/pending/<request_id>.json`) is written *before*
   dispatch and cleared only after the audit log write succeeds. If a
   crash/failure happens in between, the marker survives as recoverable
   evidence that an operation was attempted; `smc hub audit --request`
   now reports a distinct `PendingUnresolved` status instead of a bare,
   misleading `UnknownRequest` when it finds a stale marker. This does
   not eliminate the risk window, but converts a previously *silent* gap
   into an *inspectable* one, and the invocation itself is still always
   reported as a failure (never a false success) when the audit write
   fails. Regression test:
   `audit_write_failure_leaves_a_pending_marker_and_the_mutation_still_applies`
   (forces the failure for real by replacing `audit.log` with a directory
   mid-run and confirming both the marker and the applied mutation).

5. **P2 -- CONFIRMED. Admission checked that a requested `resource_budget`
   did not exceed the tool/global ceiling, but never checked that the
   actual payload conformed to the caller's own declared
   `resource_budget.input_bytes`.** A caller narrowing `input_bytes` below
   the 32 MiB global ceiling got no enforcement at all -- any payload
   under the global ceiling was silently dispatched regardless of the
   budget requested for it. Fixed: `admit()` now also checks
   `request.payload.len()` against `request.resource_budget.input_bytes`.
   Regression tests: `payload_exceeding_the_requests_own_input_bytes_budget_is_rejected`
   / `payload_within_the_requests_own_input_bytes_budget_is_admitted`
   (library) and `payload_exceeding_the_requests_own_declared_input_budget_is_rejected`
   (CLI).

6. **"Route index writes through the PROMETHEUS boundary" -- CLASSIFIED
   AS OUT OF SCOPE / CONFLICTS WITH THE EXPLICIT ARCHITECTURE DIRECTIVE**,
   not implemented as a code change. This finding cites `AGENTS.md`'s
   general "no direct external effects outside PROMETHEUS boundaries"
   rule, but issue #1553 (and architecture issue #1526 it implements)
   explicitly directs building Semantic Hub as its *own*, new, parallel
   governance boundary for external tools -- with its own capability
   admission, resource budgets, worker supervision, and audit/provenance
   -- specifically so a second effect boundary does not need to be routed
   through PROMETHEUS's SemCode-host-ABI-specific capability model (a
   different domain, as documented at length in
   `docs/architecture/semantic_hub_v0.md` and the PR body). Routing
   Hub's own filesystem writes through `prom-cap`/`prom-audit` would be
   the "second capability framework" duplication the issue explicitly
   warns against, not a fix for one. Replied on the PR thread with this
   reasoning rather than implementing the suggested change; left for the
   repository owner to overrule if they disagree with this reading of
   the architecture.

All fixes landed in commit `<see git log>`, full local gates re-run and
green (`cargo test --workspace --all-features`: 304 test result blocks,
0 failed; `cargo clippy --workspace --all-targets --all-features -- -D
warnings`: 0 warnings; `cargo fmt --all --check`: clean; harness-check:
ok).

## Remaining before merge

- Push this remediation, confirm the new exact head goes green, resolve
  the review threads (reply to each Codex comment with its
  classification), then stop for explicit repository-owner merge
  approval.
