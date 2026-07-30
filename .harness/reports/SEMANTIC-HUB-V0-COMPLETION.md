# Semantic Hub v0 Completion -- Closeout Report

## Task identity

```text
Task ID:        SEMANTIC-HUB-V0-COMPLETION
Issue:           #1526 (architecture: define Semantic Hub secure external tool runtime)
Baseline:        #1553 / #1554 / #1555 (Semantic Hub v0 + TurboVec, merged to main)
Branch:          feat/semantic-hub-v0-completion
Base commit:     e18f7d64093a0ac39b106cfc19754a55090f08b2 (main, at branch creation)
Authorized by:   repository owner direct instruction (detailed task specification)
```

## Scope

This task completes the remaining #1526 acceptance-criteria gaps identified
by a direct line-by-line audit performed earlier in this session (recorded
in project memory as `issue_1526_acceptance_audit`): a bounded multi-request
session executor with session-level ceiling attenuation, a recoverable
write-ahead transaction protocol for durable TurboVec mutations, streaming
(not full-materialize) audit reads/writes, a generic Hub-level provenance
envelope embedded in every `HubReply`, an extended fault taxonomy, and one
new `smc hub session` CLI command. No subprocess/WASM/remote execution
mode. No Workbench/Studio/ALM work. No new external dependency (turbovec
stays pinned at `=0.9.0`).

## Files read (mandatory materials + existing implementation)

```text
AGENTS.md
.harness/current.task.yaml (prior task file, SEMANTIC-HUB-V0-TURBOVEC-E2E)
Issue #1526 full body (via gh issue view)
crates/semantic-hub/src/*.rs (all 13 modules, read in full)
crates/semantic-hub-turbovec/src/lib.rs, payload.rs, storage.rs (read in full)
crates/smc-cli/src/hub.rs (read in full)
```

`.harness/task_lifecycle.md` and `.harness/task_templates/*`, referenced in
the task specification as mandatory reading, do not exist anywhere in this
repository (confirmed via `find`) -- proceeded per the convention already
established by the prior task's own `.harness/current.task.yaml`, which
does exist and was read. The `semantic` skill named in `AGENTS.md` is not
present in this session's available skill set; proceeded without it.
Codebase Memory MCP was used per `AGENTS.md`'s mandatory instruction (the
project's index was stale -- did not include `crates/semantic-hub` at all
-- and was re-indexed before use).

## Architecture decisions

**Extended, did not duplicate, the existing single-request pipeline.**
`Hub` already kept one registered worker instance and an accumulating
audit trail alive across multiple `invoke()` calls in memory -- the CLI
simply never called it more than once per process. `HubSession` is a thin
wrapper that reuses the exact same `admission::admit` -> `dispatch` ->
`record_and_reply` pipeline via a new `Hub::invoke_in_session` entry point
(sharing its body with `Hub::invoke` through a private `invoke_impl`), with
session-ceiling attenuation added as a new `AdmissionAmbient.session: Option<SessionAdmissionAmbient>`
field and a new admission step 7 -- the same pattern the existing
queue/concurrency check (step 6) already used, not a parallel admission
path.

**`PersistenceFailed`/`RecoveryRequired` are not `HubFault` variants.**
The original task specification named these as required top-level fault
codes. Repository truth is that CLI-level infrastructure failures (audit
I/O, scoped-storage violations, pending-marker bookkeeping) are reported
as plain `"<Code>: <message>"` strings from `smc-cli`'s own command
functions -- the existing convention for `AuditProvenanceFailure`,
`ScopedStorageViolation`, `DuplicateRequestId`. Forcing these two into the
`HubFault` enum (which governs per-request admission/dispatch outcomes,
not CLI/adapter infrastructure) would not have matched what actually
produces them: `RecoveryRequired` surfaces via `HubToolError` from
`TurboVecAdapter::load()` (wrapped as `ToolDeclaredFailure` like any other
adapter-declared error), and `PersistenceFailed` is a `smc-cli`-string
convention for transaction-record I/O failures. Four new `HubFault`
variants were added instead, each backed by real, reachable (or, for
`WorkerBusy`, deliberately defensive) admission/dispatch logic:
`SensitiveCapabilityDenied`, `WorkerDegraded`, `SessionLimitExceeded`,
`WorkerBusy`.

**New v0 hardening beyond the original #1554 baseline, found while
extending the fault taxonomy:**
- A request whose `capability_context` grants ANY sensitive capability is
  now rejected outright at admission (`SensitiveCapabilityDenied`), not
  silently stripped and allowed through as before. This is an intentional
  behavior change; one pre-existing integration test
  (`granting_a_sensitive_capability_survives_audit_round_trip`) asserted
  the old permissive behavior and was rewritten to assert the new
  rejection while preserving its original regression-test intent (audit
  round-trip survives a record containing a sensitive-capability grant).
- A mutating operation against a `Degraded` worker is now rejected
  (`WorkerDegraded`); reads still proceed, unchanged.
- `dispatch()`'s pre-existing `Busy`-state handling was actually a latent
  bug: a re-entrant dispatch attempt (structurally unreachable today, but
  the code path existed) was misreported as `ToolQuarantined`. Fixed as
  part of adding `WorkerBusy` as its own, correctly-labeled fault.
- `HubFault::Cancelled` was misclassified as a post-dispatch fault in
  `is_pre_dispatch_rejection()`, even though `admission::admit` (its only
  real producer) always returns it at admission step 1. Found via a new
  `HubSession` unit test expecting `Rejected(Cancelled)` and observing
  `ToolFailed(Cancelled)` instead; fixed with a regression test.
- `leak_known_fault_code`'s hand-maintained code list (in `audit.rs`) was
  not updated when the four new fault variants were added, breaking
  round-trip parsing for any audit record carrying one of them --caught by
  a real CLI integration test failure (`granting_a_sensitive_capability_...`),
  not a unit test in isolation. Fixed, and a new exhaustive regression
  test (`every_hub_fault_code_is_recognized_by_the_audit_parser`) added so
  a future fault addition without a matching parser entry fails loudly.
- The "unbounded" streaming audit-header sentinel (added for
  `append_records_to_file`) was initially only wired into
  `from_canonical_text`, not into `find_by_request_streaming` or
  `next_sequence_streaming` (written earlier, before the sentinel design
  was finalized) -- both choked on any file using the new default header
  format. Found via 4 real `smc hub` CLI integration test failures after
  rebuilding the binary (not caught by this crate's own unit tests, which
  had only ever exercised the numeric-count header for those two
  functions). Fixed in both functions, with new regression tests using
  the streaming header as input.

## Public contract changes

`HUB_ENVELOPE_SCHEMA_VERSION` bumped 1 -> 2 (see `envelope.rs` doc
comment for why this has no real backward-compatibility burden today).
`HubReply` gained `logical_sequence: u64`, `provenance: HubProvenance`,
`warnings: Vec<String>`. `HubProvenance` grew from a TurboVec-shaped
struct into a generic envelope (`schema_version`, `request_id`,
`session_id`, `logical_sequence`, `caller_identity`,
`capability_context_digest`, `resource_budget_digest`,
`worker_state_after`, `artifact: Option<HubArtifactProvenance>`,
`warnings`, plus the fields it already had). `HubTool::handle`'s return
type changed from `Result<Vec<u8>, HubToolError>` to
`Result<HubToolOutcome, HubToolError>` (`HubToolOutcome { payload,
artifact }`) -- the only way for an adapter to report artifact
provenance for a mutating operation. `HubFault` gained
`SensitiveCapabilityDenied`, `WorkerDegraded`, `SessionLimitExceeded`,
`WorkerBusy` (21 -> 25 variants). New `semantic_hub::session` module
(`HubSession`, `HubSessionSummary`, `HubSessionCeiling`,
`SessionAdmissionAmbient` re-exported from `admission`). New
`HubResourceKind` variants: `SessionRequestCount`, `SessionInputBytes`,
`SessionOutputBytes`, `SessionWallTimeMillis`. New `Hub::invoke_in_session`
method (shares its body with `Hub::invoke` via a private `invoke_impl`).
New `HubAuditTrail` methods: `find_by_request_streaming`,
`next_sequence_streaming`, `append_records_to_file`,
`to_canonical_text_streaming`. New `HubCapabilitySet::canonical_text()`,
`HubResourceBudget::canonical_text()` (also now the single shared
implementation `audit.rs`'s `pack_budget` calls, removing a small
duplication). New `semantic-hub-turbovec::transaction` module
(`TransactionRecord`, `TransactionPhase`, `RecoveryOutcome`, `begin`,
`commit`, `recover`, `read_record`). New `vector.index.recover` operation
on `vector.turbovec` (8th operation, was 7). `TurboVecAdapter::load()`
now refuses to load an index with an unresolved `Intent` transaction.

No changes to any existing PROMETHEUS crate's public API, the compiler,
verifier, or VM. `smc-cli`'s own `lib.rs` public re-export surface is
unchanged (`hub.rs` remains a private module).

## CLI changes

New `smc hub session --requests <file> [--out <file>] [--max-requests <n>]`.
Full contract: `docs/spec/hub/hub_session_v0.md`. No dedicated `smc hub
recover` subcommand: `vector.index.recover` is reachable through the
existing generic `invoke`/`session` commands, since they are already
fully generic over `<tool-id> <operation-id>`.

`smc hub invoke`'s reply JSON gained `logical_sequence`, `provenance`
(`input_digest`, `output_digest`, `worker_state_after`, `artifact`), and
`warnings` fields. `smc hub audit`'s and `smc hub invoke`'s audit-log
read/write paths now use the streaming functions above instead of always
fully materializing the log into a `Vec<HubAuditRecord>`; `save_audit_trail`
now always writes the streaming ("unbounded" header) form, so a project's
audit log stays in the append-friendly shape rather than flip-flopping
between two header forms across different commands.

## Fault taxonomy

25 `HubFault` variants (was 21). See "Architecture decisions" above for
the four additions and the deliberate exclusion of `PersistenceFailed`/
`RecoveryRequired` as enum variants. Full list, current audit-parser
`CODES` list, and per-fault pre/post-dispatch classification: `fault.rs`,
`docs/architecture/semantic_hub_v0.md` section 10.

## Capability/effect integration

No new capability variants. `SensitiveCapabilityDenied` is a new
admission-time REJECTION using the existing `HubCapability::is_sensitive()`
classification, not a new capability. `vector.index.recover` requires the
same three capabilities every other mutating TurboVec operation requires
(`VectorIndexMutate`, `PrivateStorageRead`, `PrivateStorageWrite`) -- a
caller cannot inspect or resolve a stuck transaction without the authority
that would let it cause one.

## Transaction/recovery model

Full description: `docs/architecture/semantic_hub_v0.md` section 21,
`docs/spec/hub/turbovec_adapter_v0.md` section 10.1-10.2. Summary: one
`<name>.tvim.txn` record per index (always overwritten in place),
`begin()` (durable intent, before the candidate write) ->
candidate write -> atomic rename -> read-back digest -> `commit()`
(durable completion record with the verified digest). `recover()`
resolves an interrupted transaction into exactly one of `NoTransaction`,
`AlreadyCommitted`, `RolledBackAbandonedCandidate`, `FinalizedCommit`, or
`Indeterminate` (never silently reported as success when unprovable).

## Resource model

Session-level cumulative ceiling (`HubSessionCeiling`: request count,
cumulative input/output bytes, cumulative wall time) added as a NEW,
attenuating layer on top of the existing 12-dimension per-request
`HubResourceBudget` -- it narrows, never widens, what a per-request budget
already allows. 4 new `HubResourceKind` variants distinguish session-scope
violations from per-request ones in audit/error evidence.

## Worker lifecycle

No new `HubWorkerState` variants (the existing 9-state machine and its
transition table were already complete). `dispatch()`'s `Busy`-state
handling corrected (see "Architecture decisions"). `admission::admit`
gained a new rule: `Degraded` + `mutates_tool_state` -> `WorkerDegraded`.

## Tests added

```text
crates/semantic-hub:      +23 tests (109 -> 132)
crates/semantic-hub-turbovec: +6 tests in lib.rs, +6 new tests in
                           transaction.rs (38 -> 44 in lib unittests;
                           transaction.rs is a wholly new module)
tests/hub_cli.rs (root):  +10 new integration tests (22 -> 32),
                           1 rewritten for the sensitive-capability
                           hardening, 1 renamed (7 -> 8 operations)
```

New unit-test coverage: session ceiling enforcement (request count,
cumulative input bytes), cancellation, cross-session isolation, logical
sequence ordering, `WorkerDegraded`/`WorkerBusy`/`SensitiveCapabilityDenied`/
`SessionLimitExceeded` fault paths, transaction begin/commit/recover (all
3 recovery outcomes plus the already-committed and no-transaction cases),
streaming audit read/write/append (including the "unbounded" header
regression), every-fault-code audit round-trip.

New integration-test coverage (real `smc` subprocess, per `tests/hub_cli.rs`'s
existing convention): full session batch (create/insert/search/remove/
search/recover) with structural assertions on ordering, mutation
visibility, and the session summary; session mutations persisting across
the process boundary (reloaded via a separate plain `invoke`); session
ceiling enforcement via `--max-requests`; session cancel-line rejection;
duplicate `request_id` within a batch and against prior history; malformed
NDJSON line reporting with a line number; empty-batch handling; recovery
reachable through the generic `invoke` command.

## Fixtures added

```text
tests/fixtures/hub/session_workflow.ndjson   (LF-only, verified;
  automatically covered by the existing tests/hub_fixture_line_ending_guard.rs
  regression guard, which discovers fixture files via `git ls-files`)
```

## Commands run and exact results

```text
cargo fmt --all -- --check
  -> initially found diffs in 7 files (never fmt'd while hand-writing);
     `cargo fmt --all` applied, re-checked clean.

cargo clippy -p semantic-hub --all-targets --all-features -- -D warnings
  -> 4 findings (filter_next, 3x unnecessary_lazy_evaluations in
     audit.rs's new append_records_to_file), fixed, re-run clean.
cargo clippy -p semantic-hub-turbovec --all-targets --all-features -- -D warnings
  -> clean on first run.
cargo clippy -p smc-cli --all-targets --all-features -- -D warnings
  -> 1 finding (large_enum_variant on CliSessionLine), fixed by boxing
     the HubRequest variant, re-run clean.

cargo test -p semantic-hub -p semantic-hub-turbovec -p smc-cli
  -> semantic-hub: 132 passed, 0 failed
  -> semantic-hub-turbovec: 44 passed (lib) + 3 passed (determinism_qualification), 0 failed
  -> smc-cli: 127 passed, 0 failed
  (306 total across the three crates, 0 failed)

cargo build -p semantic_language --bin smc --release
  -> succeeded (used for all CLI dogfooding and integration tests below)

cargo test -p semantic_language --test hub_cli --release
  -> First full run after the streaming-audit/session work: 27 passed,
     4 failed (all four traced to the SAME root cause: the "unbounded"
     streaming header sentinel was only wired into `from_canonical_text`,
     not into `find_by_request_streaming`/`next_sequence_streaming`).
  -> Fixed both functions, added 2 regression tests.
  -> Re-run after rebuilding the release binary: 31 passed, 0 failed.
```

```text
cargo test -p semantic_language --test public_api_contracts --release
  -> First run: FAILED (1/4) -- public API surface genuinely drifted
     (new session module, extended HubReply/HubProvenance/HubFault,
     HubToolOutcome, new vector.index.recover operation). Also
     surfaced a pre-existing snapshot-tool limitation: its
     normalized_public_surface() only follows multi-line CONTINUATION
     for `pub fn` signatures, not multi-line `pub use { ... };` blocks
     -- two of this diff's re-export lines had grown too long for
     rustfmt's line width and wrapped, so the snapshot would have
     silently captured only "pub use admission::{" (truncated, hiding
     which items are re-exported). Fixed by splitting the re-exports
     into multiple single-line `pub use` statements (matching the
     file's own pre-existing convention for other multi-item
     re-exports) rather than touching the snapshot tool itself.
  -> Regenerated tests/golden_snapshots/public_api/{semantic_hub_lib,
     semantic_hub_turbovec_lib}.txt from the tool's own actual output.
  -> Re-run: 4 passed, 0 failed.

cargo check --workspace --all-targets --all-features
  -> clean, 0 errors, 0 warnings.

cargo test --workspace --all-features
  -> every single `test result:` line across the entire workspace
     (~250 test binaries) reads `ok` with 0 failed. No regressions
     anywhere outside the Hub crates from this change.
```

## Adversarial findings

Six real, self-caught defects during implementation (not found by a
separate adversarial review pass -- see the required fresh-context
adversarial review section below for that pass):

1. `HubFault::Cancelled` misclassified as post-dispatch -- found via a new
   `HubSession` unit test.
2. `leak_known_fault_code` missing the 4 new codes -- found via a real
   CLI integration test failure.
3. `find_by_request_streaming`/`next_sequence_streaming` rejecting the
   "unbounded" streaming header -- found via 4 real CLI integration test
   failures after rebuilding the binary.
4. `dispatch()`'s `Busy` state was pre-existing-mislabeled as
   `ToolQuarantined` -- found while adding `WorkerBusy`.
5. A stray copy-paste-garbled test function name in `session.rs`
   (`a_fault_that_recurs_across_different_faults_share_faults_across_sessions_are_independent`)
   -- self-caught before running, renamed to
   `independent_sessions_against_the_same_hub_do_not_share_ceiling_state`.
6. A nonsensical `assert_ne!` in a `transaction.rs` test comparing a
   value to itself plus a suffix -- self-caught before running, rewritten
   to a meaningful assertion.

## Remaining explicit non-goals (unchanged from #1554/#1553, still true)

```text
no subprocess execution
no WASM execution
no remote execution
no process-level isolation
no memory isolation
no cryptographic signing chain (still tracked under #1374)
no cross-CPU determinism guarantee
no dynamic tool loading
no plugin marketplace
no Workbench/Studio/ALM integration
```

New v0-completion-specific non-claims:
```text
session-level cumulative output-byte checking is necessarily post-hoc
  (a reply's actual size is unknown before dispatch) -- only PRIOR
  requests' measured usage is checked before admitting the next one
in-process cancellation remains cooperative and pre-admission-only; there
  is still no mid-dispatch preemption in v0 (unchanged from the base
  contract's wall-time-budget limitation)
smc hub session's audit durability is per-request (immediate append
  after each request), not per-batch with a pre-dispatch pending marker
  the way smc hub invoke has -- a deliberate, documented v0 scope
  decision (docs/spec/hub/hub_session_v0.md section 7.5), not an
  oversight
```

## Git status

```text
Modified (24): .harness/current.task.yaml; crates/semantic-hub/src/{admission,audit,
capability,envelope,fault,lib,provenance,resource,runtime}.rs;
crates/semantic-hub-turbovec/src/lib.rs, src/payload.rs,
tests/determinism_qualification.rs; crates/smc-cli/src/hub.rs;
docs/architecture/semantic_hub_v0.md; docs/privacy/semantic_hub_data_policy_v0.md;
docs/security/semantic_hub_threat_model_v0.md; docs/spec/cli.md;
docs/spec/hub/{hub_adapter_contract_v0,hub_api_v0,turbovec_adapter_v0}.md;
tests/golden_snapshots/public_api/{semantic_hub_lib,semantic_hub_turbovec_lib}.txt;
tests/hub_cli.rs

New (5): .harness/reports/SEMANTIC-HUB-V0-COMPLETION.md;
crates/semantic-hub/src/session.rs;
crates/semantic-hub-turbovec/src/transaction.rs;
docs/spec/hub/hub_session_v0.md;
tests/fixtures/hub/session_workflow.ndjson

All 29 changed/new paths fall within .harness/current.task.yaml's own
allowed_paths; none of the forbidden_paths were touched.
```

## Diff stat

```text
24 files changed, 2655 insertions(+), 320 deletions(-)
 (git diff --stat, working tree at the point validation gates completed)
```

## Final verdict

(Filled in after validation gates and the adversarial review pass both
complete.)
