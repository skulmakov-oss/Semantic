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
- A single whole-project exclusive advisory lock (`std::fs::File::lock`)
  now serializes concurrent `smc hub invoke` processes against the same
  project, closing the audit-log/index-update-loss races previously
  named here (see the round-5 remediation section below) -- it blocks
  indefinitely with no timeout/fairness guarantee, adequate for v0's
  single-user CLI, not a multi-tenant scheduler.
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

## Codex review, round 2 (PR #1554, commit `54bb441c`) -- 3 P1 + 1 P2 findings, all resolved

A second automated review pass ran against the round-1 remediation commit
itself and found 4 further real defects introduced or left open by that
commit (none stylistic/speculative). All four are fixed in this same PR.

1. **P1 -- CONFIRMED, SEVERE. `dispatch()` handed the adapter the caller's
   raw, unsanitized `request.capability_context`.** Admission's
   `check_capabilities` only verifies an operation's *required*
   capabilities are present and non-sensitive -- it never strips a
   sensitive capability the caller happened to grant *alongside* the
   required ones. An admitted request could therefore still carry e.g.
   `NetworkAccess`, and an adapter naively calling
   `context.capability_context.allows(NetworkAccess)` would observe
   `true` for something Hub structurally denies by design, defeating the
   deny-by-default sensitive-capability model documented in
   `docs/security/semantic_hub_threat_model_v0.md`. Fixed: added
   `HubCapabilitySet::deny_sensitive(&self) -> Self` and `dispatch()` now
   builds `RestrictedHubContext` from `request.capability_context.deny_sensitive()`
   instead of the raw set. The unsanitized set is still exactly what the
   audit record captures -- sanitization is a dispatch-boundary concern,
   not an audit one. Regression test:
   `a_sensitive_capability_granted_by_the_caller_is_not_visible_to_the_adapter`
   (`crates/semantic-hub/src/runtime.rs`), using a new test-only
   `test.report-network-access-visibility` operation on the fault-injection
   tool that reports back what it observed through the restricted context.

2. **P1 -- CONFIRMED. `resource_budget.wall_time_millis` was documented as
   one of the 8 hard-enforced v0 resource dimensions but was never read
   anywhere in `invoke`/`dispatch` -- declaring even a 1ms budget had zero
   effect.** Fixed: `invoke()` now derives a deadline from
   `started + Duration::from_millis(request.resource_budget.wall_time_millis)`,
   combines it with any caller-supplied deadline (`.min()`, tighter wins),
   and rejects with `DeadlineExceeded` before admission runs if already
   past it. Honestly scoped, and documented as such in
   `docs/spec/hub/hub_adapter_contract_v0.md` Section 9: this is checked
   at dispatch entry only, not preemptive of a native adapter call already
   in flight (no detached/interruptible execution in the v0 synchronous,
   in-process architecture). One classification note: `HubFault::DeadlineExceeded`
   is listed in the taxonomy as a post-dispatch fault
   (`is_pre_dispatch_rejection() == false`), so this particular
   entry-check produces a `HubReplyStatus::ToolFailed`, not `Rejected`,
   even though it runs before `admit()` -- pre-existing classification
   behavior, left unchanged since only the enforcement gap itself was the
   confirmed finding. Regression test:
   `a_tiny_wall_time_budget_produces_deadline_exceeded_before_dispatch`.

3. **P1 -- CONFIRMED. Caller-supplied `request_id`s were accepted without
   checking the persisted audit trail or the pending-marker directory for
   reuse.** `HubAuditTrail::find_by_request` always returns the *first*
   matching record, so replaying an id would append a second, permanently
   unreachable-by-lookup record; and reusing an id that still had an
   unresolved pending marker (from a prior failed audit write) would let
   the retry's own success clear that marker, erasing the only durable
   evidence that the earlier, still-unresolved invocation may already have
   applied its effect. Fixed: `cmd_hub_invoke` now checks
   `persisted_trail.find_by_request(&request_id)` and
   `pending_marker_path(&request_id).is_file()` immediately after loading
   the persisted trail and before writing any new pending marker or
   dispatching, rejecting with a new `DuplicateRequestId` error if either
   is true. Regression tests:
   `a_reused_request_id_is_rejected_and_does_not_append_a_second_audit_record`
   and `reusing_a_request_id_with_an_unresolved_pending_marker_is_rejected`
   (`tests/hub_cli.rs`, the latter a direct continuation of the round-1
   pending-marker test's failure scenario).

4. **P2 -- CONFIRMED. `MAX_AUDIT_RECORD_BYTES` (8 KiB) was enforced only
   on parse (`HubAuditRecord::from_canonical_line`), never on serialize
   (`to_canonical_line`)**, so a sufficiently long unbounded input field
   (tool `name` or `adapter_provenance`, both embedded verbatim/escaped
   into every audit line for that tool) could produce a serialized record
   the trail's own parser would then reject on the very next read --
   corrupting the audit log with a self-inflicted, unrecoverable record.
   Changing `to_canonical_line`'s signature to a fallible one was judged a
   larger, more invasive change than the actual root cause warranted,
   since the real problem is that these fields were unbounded at their
   source. Fixed instead at that source: added
   `descriptor::MAX_TOOL_NAME_LEN` (256) and
   `MAX_ADAPTER_PROVENANCE_LEN` (2048) with a new
   `DescriptorError::FieldTooLong` variant, checked in
   `HubToolDescriptor::validate()` (which every tool registration already
   goes through) -- these two ceilings keep every field that reaches
   `to_canonical_line` well under `MAX_AUDIT_RECORD_BYTES` by construction.
   Regression tests: `descriptor_with_oversized_name_is_rejected` /
   `descriptor_with_oversized_adapter_provenance_is_rejected`.

All fixes landed in one remediation commit on top of `54bb441c`, full
local gates re-run and green (`cargo test --workspace --all-features`:
zero failures across every crate; `cargo clippy --workspace --all-targets
--all-features -- -D warnings`: 0 warnings after one lint fix
[`needless_option_as_deref`]; `cargo fmt --all`: clean; harness-check:
ok; `git diff --check`: clean).

## Codex review, round 3 (PR #1554, commit `dd7b5805`) -- 5 P1 + 1 P2 findings, all resolved

A third automated review pass ran against the round-2 remediation commit.
Before fixing anything, each of the 6 findings was independently
re-verified against the actual current source by a dedicated investigator
(not taken on the reviewer's word) -- 5 CONFIRMED, 1 a repeat of round 1's
already-adjudicated out-of-scope PROMETHEUS-routing finding.

0. **Repeat -- OUT OF SCOPE, same disposition as round 1 item 6.**
   "Route TurboVec writes through PROMETHEUS." Independently re-verified:
   the described mechanism (`save_atomic`'s `index.write`/`std::fs::rename`
   reached via Hub-local capability checks only) is accurate but is the
   deliberate, documented architecture of Hub v0 per issue #1553/#1526 and
   `docs/architecture/semantic_hub_v0.md` sections 3.1/4/17 -- unchanged
   since round 1's identical disposition. No code change.

1. **P1 -- CONFIRMED, SEVERE. Reject symlinked scoped-storage roots.**
   `ScopedStorage::index_path`/`ensure_root` joined/created the root
   verbatim with no symlink check. If `.semantic/hub/vector.turbovec`
   were already a symlink when the CLI starts (e.g. shipped inside a
   malicious checked-out project), every read (`load`, feeding
   describe/insert/remove/search/reset) and write (`save_atomic`, feeding
   create/insert/remove/reset) would silently resolve through the OS to
   wherever the symlink pointed -- outside the directory the adapter is
   scoped to, despite the index name itself being charset-validated.
   Fixed: `ScopedStorage` gained `ensure_root_checked()` and
   `checked_index_path()`, both rejecting with `ScopedStorageError::RootIsSymlink`
   via `std::fs::symlink_metadata(...).file_type().is_symlink()` (a root
   that does not exist yet is not a violation); `load`/`save_atomic` now
   use these instead of the unchecked originals, surfacing a new
   `ScopedStorageViolation` `HubToolError`. Regression tests (Unix-gated,
   since symlink creation needs no special privilege on the ubuntu/macos
   CI runners but does on Windows):
   `checked_index_path_rejects_a_symlinked_root` (storage.rs, mechanism
   level) and `create_rejects_a_symlinked_scoped_root_end_to_end` (lib.rs,
   through the real `handle()` path), both confirming the symlink target
   is never touched.

2. **P1 -- CONFIRMED. Enforce the deadline during adapter execution.**
   `Hub::invoke`'s only deadline check ran once, before dispatch; a
   dispatch that overran its `wall_time_millis` budget (nothing polls
   `deadline_exceeded()` mid-call in v0) still returned a plain `Success`,
   despite `WallTimeMillis` being classified `is_hard_enforced_v0() ==
   true`. Fixed with the reviewer's own "at minimum" option: `invoke()`
   now rechecks the same `effective_deadline` immediately after `dispatch`
   returns, downgrading an `Ok` result to `HubFault::DeadlineExceeded` if
   the deadline has since passed -- consistent with the existing
   `OutputRejected` precedent that an adapter's already-applied side
   effects are not undone by a post-hoc rejection. True in-flight
   preemption (polling inside `handle_insert`'s/`handle_search`'s loops)
   was scoped out as materially larger and already-documented as deferred
   in `docs/spec/hub/hub_adapter_contract_v0.md` Section 9. Regression
   test: `a_dispatch_that_overruns_its_wall_time_budget_is_rejected_after_the_fact`,
   using a new test-only `test.slow` fault-injection operation that
   sleeps past a 1ms budget.

3. **P1 -- CONFIRMED. Apply vector-dimension budgets to existing
   indexes.** Only `vector.index.create` ever consulted
   `resource_budget.vector_dimensions`; `describe`/`insert`/`remove`/
   `search`/`reset` all load an existing index via the shared `load()`
   helper with no dimension check at all, so an index created under a
   wide budget in one CLI invocation could later be operated on by a
   separate invocation admitted with a much narrower dimension budget,
   with zero enforcement -- the round-2 fix (commit 54bb441c) explicitly
   scoped itself to `handle_create` only, per its own writeup above,
   leaving this open by design of that pass's scope. Fixed at the one
   shared choke point: `load(name, max_dim)` now rejects with
   `DimensionExceedsBudget` if the loaded index's `dim()` exceeds the
   caller's admitted ceiling, and every one of the five callers (plus
   `create`'s pre-existing check) now threads `max_dim` through. Regression
   test: `describe_rejects_an_existing_index_whose_dimension_exceeds_the_current_budget`.

4. **P1 -- CONFIRMED. Reject unknown request fields before mutating
   state.** None of the six `payload.rs` request structs set
   `#[serde(deny_unknown_fields)]`, so a typo such as `"bit_wdith": 2` was
   silently dropped, `bit_width` fell back to its default, and the index
   was permanently created with parameters the caller never asked for --
   unrecoverable, since there is no delete-index operation in v0. (The
   investigator also flagged that the finding's citations -- `hub_api_v0.md`'s
   exact-shape contract and `AGENTS.md:L9` -- don't actually mandate this
   at the adapter-payload layer, which is explicitly documented as opaque
   to the Hub; the underlying defect is real regardless of whether the
   cited text technically requires the fix.) Fixed: added
   `#[serde(deny_unknown_fields)]` to all six request structs (none use
   `#[serde(flatten)]`, so this is conflict-free). Regression test:
   `create_index_request_rejects_an_unknown_field_instead_of_silently_ignoring_it`.

5. **P2 -- CONFIRMED. Prevent the audit log from exceeding its read
   limit.** `save_audit_trail` had no preflight against
   `MAX_AUDIT_LOG_BYTES` before writing -- the same defect class the
   round-1 fix already addressed once at a smaller threshold (raising
   `MAX_INPUT_BYTES` to a dedicated, larger `MAX_AUDIT_LOG_BYTES`), which
   only delayed rather than closed the contradiction: once normal
   accumulated history crossed the cap, every subsequent `invoke`/`audit`
   call (mutating or read-only) would fail permanently at
   `load_audit_trail`'s own read bound, with no rotation/trim mechanism to
   recover except manual deletion. (The investigator also corrected the
   finding's citation -- the actual "no size cap" promise lives in
   `docs/privacy/semantic_hub_data_policy_v0.md`'s Retention section, not
   `AGENTS.md:L9` as cited -- without that changing the underlying defect's
   reality.) Fixed: `cmd_hub_invoke` now preflights via a new pure
   `would_exceed_audit_log_cap(current_text_len)` helper (current trail
   length + one worst-case `MAX_AUDIT_RECORD_BYTES` record vs. the cap),
   rejecting with `AuditLogFull` before writing any pending marker or
   dispatching -- so `audit.log` itself never crosses the cap, and
   read-only `smc hub audit` keeps working indefinitely on the
   still-under-cap log. Regression test:
   `would_exceed_audit_log_cap_rejects_only_once_the_projected_size_passes_the_cap`,
   exercised as pure logic rather than via a real multi-hundred-megabyte
   fixture.

All fixes landed in one remediation commit on top of `dd7b5805`, full
local gates re-run and green (`cargo test --workspace --all-features`:
zero failures across every crate, including the new Unix-gated symlink
tests compiled out on this Windows dev machine but exercised on the
ubuntu-latest/macos CI runners; `cargo clippy --workspace --all-targets
--all-features -- -D warnings`: 0 warnings; `cargo fmt --all`: clean;
harness-check: ok; `git diff --check`: clean).

## Codex review, round 4 (PR #1554, commit `27cb0daf`) -- 2 confirmed, 2 out of scope

A fourth pass reviewed the round-3 remediation commit and surfaced 4 new
findings (the other 12 inline comments GitHub showed against this head
were round 1-3 comments GitHub re-anchors to the latest commit when their
line position is still valid, not new findings). Each was independently
re-verified before deciding; 2 confirmed and fixed, 2 classified out of
scope against real, pre-existing documentation (not a post-hoc excuse
invented for this review -- verified by reading the actual doc text
before accepting the classification).

1. **OUT OF SCOPE -- "Serialize audit-log updates across CLI processes"
   (P1).** Two overlapping `smc hub invoke` processes against the same
   project directory can genuinely race on `audit.log` via last-writer-
   wins atomic rename, silently discarding one invocation's audit record
   even though it already reported `Success` and cleared its own pending
   marker. Independently confirmed the mechanics are real (no lock, mutex,
   flock, or advisory-lock file exists anywhere in `crates/smc-cli` or
   `crates/semantic-hub-turbovec`) -- but this exact gap was already
   named, in almost these words, in `docs/security/semantic_hub_threat_model_v0.md`
   sections 8 and 9 ("concurrent multi-process access to the same
   `.semantic/hub/` directory is not guarded... named here explicitly as
   out-of-scope for v0, not something silently assumed safe") and in this
   report's own "Honest limitations" section below, written during v0's
   original implementation pass -- before any Codex review ran, not
   invented now to dodge the finding. Section 4 of the same doc already
   assumes the attacker "can invoke the CLI repeatedly, including
   concurrently." No code change.

2. **OUT OF SCOPE -- "Serialize index read-modify-write transactions"
   (P1).** The identical race one layer down: concurrent inserts/removes/
   resets on the same `.tvim` index can lose an update the same way.
   Same disposition and same pre-existing documentation as item 1 above
   -- both races share the same root cause (no file locking under
   `.semantic/hub/`) and the same prior adjudication. No code change.

3. **P1 -- CONFIRMED, SEVERE. Reject symlinks in every scoped-root
   component, not only the final one.** The round-3 fix's
   `check_root_is_not_a_symlink` called `std::fs::symlink_metadata` on the
   *whole* joined root path, which only inspects the final path
   component's own link status -- the OS resolves every component before
   it (following symlinks) just to locate that final component's parent.
   So an ancestor such as `.semantic` or `.semantic/hub` being a symlink,
   with `vector.turbovec` itself a real directory reached through it,
   passed the round-3 check completely -- the identical bypass class one
   directory level higher, left open by an incomplete first fix. Verified
   empirically on this Windows dev machine by compiling the literal check
   logic against a live NTFS junction one level above the scoped root:
   the check reported `Ok` and a real read through that path returned
   data from outside the intended tree. Fixed: `check_root_is_not_a_symlink`
   now walks every successively-longer prefix of the root path
   (`self.root.components()`), calling `symlink_metadata` on each and
   rejecting as soon as any component reports `is_symlink()` --
   `canonicalize` is deliberately not used, since it would silently
   resolve through the very link this is hunting for. No signature or
   call-site changes needed; `ensure_root_checked()`/`checked_index_path()`
   already funnel every real read/write through this one function.
   Regression test (Unix-gated, same rationale as round 3's leaf-symlink
   test):
   `checked_index_path_rejects_a_symlink_in_an_ancestor_component_not_just_the_root_itself`.

4. **P2 -- CONFIRMED. Reject unknown resource-budget override fields.**
   `CliResourceBudgetOverride` (the CLI request file's optional
   `resource_budget` override) had no `#[serde(deny_unknown_fields)]`, so
   a misspelled key such as `"output_byte"` (missing the trailing `s`)
   was silently dropped by serde, and `merge_budget`'s
   `unwrap_or(ceiling.field)` fallback then substituted the full,
   generous `V0_CEILING` value for that dimension instead of erroring --
   backwards from what a caller narrowing their own budget intends. This
   is the identical bug class already fixed once for the six adapter
   payload structs in round 3 (item 4 above), just in a sibling struct at
   the CLI-envelope layer that fix never reached. Fixed: added
   `#[serde(deny_unknown_fields)]` to `CliResourceBudgetOverride` (no
   `#[serde(flatten)]` used, so conflict-free); the existing
   `InputRejected: malformed request file` error path already surfaces
   the resulting deserialize error with no new error-handling code
   needed. (The outer `CliRequestFile` envelope struct was deliberately
   left unchanged: a typo in its top-level fields, e.g. `capabilities` ->
   `capabilites`, fails *closed* today via `#[serde(default)]` producing
   an empty capability set that admission then denies -- the asymmetry
   that makes the budget-override case a real security concern (fails
   *open* to a permissive default) does not apply there.) Regression
   test: `resource_budget_override_rejects_a_misspelled_field_instead_of_silently_defaulting`.

All fixes landed in one remediation commit on top of `27cb0daf`, full
local gates re-run and green (`cargo test --workspace --all-features`:
zero failures; `cargo clippy --workspace --all-targets --all-features --
-D warnings`: 0 warnings; `cargo fmt --all`: clean; harness-check: ok;
`git diff --check`: clean).

## Codex review, round 5 (PR #1554, commit `681bdeb5`) -- clean pass

Round 5 (submitted against `681bdeb5`) came back with no new inline
findings -- the automated review's own convention is to react instead of
commenting when it has no suggestions. All 21 findings across rounds 1-4
are now either fixed with regression tests or classified with documented
reasoning (out of scope / repeat of an already-adjudicated decision).

## Post-review-loop hardening: close the two round-4 out-of-scope findings

After round 5 came back clean, the repository owner reviewed the two
round-4 findings that had been classified OUT_OF_SCOPE (concurrent
multi-process races on `audit.log` and on a `.tvim` index file,
disposed of by citing this report's and the threat model's own
pre-existing "no file locking implemented, named here explicitly as
out-of-scope for v0" documentation) and asked for them to be addressed
rather than left as a documented limitation.

Fixed: `crates/smc-cli/src/hub.rs` gained `acquire_project_lock()`,
which opens (creating if absent) `.semantic/hub/hub.lock` and calls
`std::fs::File::lock()` -- a blocking exclusive advisory lock, stable in
std as of this project's pinned toolchain (1.97.1), wrapping `flock(2)`
on Unix / `LockFileEx` on Windows with automatic release on `Drop`
(including on an ungraceful process exit, which is why an OS-level lock
was used here instead of a hand-rolled marker file that would need its
own stale-lock recovery logic). `cmd_hub_invoke` now acquires this lock
immediately before `load_audit_trail` and holds it (a local binding,
dropped -- and so unlocked -- on every return path) through the final
`save_audit_trail`/`clear_pending_marker`, serializing the entire
load -> dispatch -> save sequence against any other `smc hub invoke`
process for the same project. This single project-level lock closes
both races at once, since `Hub::invoke` internally reaches the
`TurboVecAdapter`'s `load`/`save_atomic` calls within the same critical
section -- no separate per-index lock was needed in
`crates/semantic-hub-turbovec`, keeping that crate's dependency-minimal
design intact.

A `fs4` crate dependency was added and then removed once compilation
revealed `std::fs::File::lock` already provides this natively on the
pinned toolchain -- ladder-checked (stdlib covers it) before settling on
the final zero-new-dependency implementation.

Regression tests (real subprocesses, actually launched concurrently via
`spawn`, not simulated), added to `tests/hub_cli.rs`:
- `concurrent_invocations_against_the_same_project_do_not_lose_audit_records`
  -- 8 concurrent `smc hub invoke` processes against the same project;
  confirms all 8 request_ids survive in `audit.log`.
- `concurrent_inserts_against_the_same_index_do_not_lose_updates` -- 6
  concurrent inserts against the same index; confirms the final index
  length is 6, not fewer.

Both tests pass, and per-invocation timing in the first test's output
(~220-280ms each, summing to roughly the total wall-clock of the whole
run) confirms the invocations genuinely serialized rather than
coincidentally not colliding.

`docs/security/semantic_hub_threat_model_v0.md` sections 8 and 9 and
this report's "Honest limitations" section above were updated to
describe the lock instead of the gap; the residual limitation (no
timeout/fairness, adequate for a single-user CLI, not a future
multi-tenant server mode) is now stated explicitly in its place.

Full local gates re-run and green (`cargo test --workspace
--all-features`: zero failures, including both new concurrency tests;
`cargo clippy --workspace --all-targets --all-features -- -D warnings`:
0 warnings after one lint fix [`suspicious_open_options`, requiring an
explicit `.truncate(false)`]; `cargo fmt --all`: clean; harness-check:
ok; `git diff --check`: clean).

## Remaining before merge

- Push this remediation, confirm CI goes green on the new head, post a
  brief note on the PR describing the owner-directed hardening (not a
  Codex-finding reply, since Codex did not raise this in round 5), and
  request one final review pass.
- If that pass is clean, stop for explicit repository-owner merge
  approval (already granted, conditional on a clean review pass) before
  squash-merging.
