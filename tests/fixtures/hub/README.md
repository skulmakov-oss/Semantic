Canonical `smc hub invoke` request fixtures (Issue #1553).

These are strict, checked-in JSON **request** files fed to `smc hub invoke
vector.turbovec <operation> --input <fixture>` by `tests/hub_cli.rs`. Only
request files are checked in as goldens: reply files are not, because every
real reply contains fields that vary per run (`request_id`, timing under
`resource_usage`) and so cannot be byte-compared as a golden. The
integration test instead asserts on the stable, structural parts of a live
reply (`status`, `fault_code`, and specific `payload` fields).

Line endings: this directory is covered by the `text eol=lf` rule in
`.gitattributes` and by `tests/hub_fixture_line_ending_guard.rs`, mirroring
the fix landed in PR #1552 for `tests/fixtures/ui_frame_inspection/`.

## Files and what each exercises

- `valid_index_create.json` -- `vector.index.create`, happy path.
- `valid_index_insert.json` -- `vector.index.insert`, 4 one-hot vectors with
  external ids 101-104 (run after `valid_index_create.json` against the
  same index name).
- `valid_index_describe.json` -- `vector.index.describe`.
- `valid_search.json` -- `vector.search`, top-3 nearest neighbors.
- `valid_search_filtered.json` -- `vector.search.filtered`, restricted to
  external ids `[101, 103]`.
- `valid_index_remove.json` -- `vector.index.remove`, removes id `102`.
- `valid_index_reset.json` -- `vector.index.reset`.
- `reject_capability_denied.json` -- `vector.search` with an empty
  `capabilities` array; admission must deny before dispatch
  (`CapabilityDenied`), and the Hub must remain usable afterward (this
  fixture is also a regression check for a real bug found during CLI
  dogfooding, where a capability-denial audit record once corrupted the
  persisted audit log for every subsequent invocation).
- `reject_unsupported_schema_version.json` -- `schema_version: 99`; must be
  rejected as `SchemaVersionUnsupported` before the payload is even
  interpreted.
- `reject_invalid_vector_dimension.json` -- `vector.index.insert` with a
  3-component vector against an 8-dimensional index; must be a typed
  `VectorDimensionMismatch` tool failure, not a panic.
- `reject_malformed_truncated.json` -- deliberately truncated JSON (an
  unterminated array); must be rejected as `InputRejected` with a JSON
  parse error, not a panic or a partial success.

## Explicitly out of scope for this fixture set (honest limitations)

- **Privacy denial**: `HubFault::PrivacyDenied` exists in the fault
  taxonomy but is never actually constructed anywhere in the v0 admission
  path -- `HubPrivacyClass` is recorded and propagated to the audit record,
  but no policy currently denies a request based on it. A "privacy denial"
  fixture would misrepresent unimplemented behavior as real, so none is
  included.
- **Duplicate tool registration** and **descriptor/API mismatch**: these are
  exercised by unit tests in `crates/semantic-hub/src/registry.rs` and
  `crates/semantic-hub/src/descriptor.rs` directly, since registration
  happens in Rust code at CLI startup, not via a request file -- there is no
  meaningful CLI-level fixture for it.
- **Queue full**: `HubFault::QueueFull` is exercised by a unit test in
  `crates/semantic-hub/src/admission.rs` (`queue_full_is_rejected_when_ambient_depth_meets_budgeted_depth`)
  using a synthetic ambient queue depth; the real CLI dispatches one
  request per process synchronously, so there is no way to observe genuine
  queue contention through the CLI in v0.
- **Cancellation, deadline exceeded, worker crash/quarantine, protocol
  violation**: exercised directly in `crates/semantic-hub/src/runtime.rs`'s
  test module using a narrow test-only fault-injection tool (per the
  project's stated allowance for forcing otherwise-unreachable failure
  paths) rather than as CLI fixtures, since `vector.turbovec` itself has no
  legitimate way to panic or violate its own reply protocol on valid input.
