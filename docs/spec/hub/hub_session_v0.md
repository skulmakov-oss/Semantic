# Hub Session v0 Specification

Status: v0, implemented on branch `feat/semantic-hub-v0-completion`
Track: Hub / execution boundary
Purpose: specify the bounded, ordered, multi-request session contract
(`semantic_hub::HubSession`, `smc hub session`) added in the Semantic Hub
v0 completion pass (issue #1526)

See `docs/architecture/semantic_hub_v0.md` section 19 for the summary and
`docs/spec/hub/hub_api_v0.md`/`docs/spec/hub/hub_adapter_contract_v0.md`
for the base request/reply/adapter contract this document extends, not
replaces.

## 1. Purpose

A single `Hub::invoke` call proves admission, dispatch, and audit for one
request against a freshly-constructed `Hub`. It does not prove that one
registered worker instance can safely process *several* ordered requests
within one process, with a shared, rolling budget across them. `HubSession`
closes that gap, as a thin, deterministic wrapper around the same
admission/dispatch/audit pipeline -- it does not introduce a second one.

## 2. Non-goals

```text
concurrent/parallel request execution within one session (v0 is
  synchronous; see section 4)
mid-dispatch cancellation of an already-admitted request (v0's
  cancellation is pre-admission-only; see section 6)
cross-session ceiling sharing (each HubSession has its own independent
  cumulative counters, even against the same Hub)
a general workflow/orchestration engine (a session is a flat, ordered
  list of independent requests, not a DAG or conditional pipeline)
```

## 3. `HubSessionCeiling`

```rust
pub struct HubSessionCeiling {
    pub max_request_count: u32,
    pub max_cumulative_input_bytes: u64,
    pub max_cumulative_output_bytes: u64,
    pub max_cumulative_wall_time_millis: u64,
    pub max_queue_depth: u32,
    pub max_concurrent_requests: u32,
}
```

`HubSessionCeiling::V0_DEFAULT`: 10,000 requests, 256 MiB cumulative input,
256 MiB cumulative output, 10 minutes cumulative wall time, queue depth 1,
and one concurrent request. `smc hub
session --max-requests <n>` overrides only the request-count dimension;
the other dimensions are not yet independently configurable from the CLI (a
future CLI flag, not a v0 gap in the underlying contract, which already
supports overriding them via `HubSession::new`).

A session ceiling **attenuates**, it never widens, the per-request
`HubResourceBudget` ceiling that already applies to every individual
request (see `hub_adapter_contract_v0.md`). The two are independent
dimensions checked at different points in `admission::admit`:

```text
step 5 (existing): per-request budget vs. tool/global ceiling
step 7 (new):       session cumulative usage vs. session ceiling
```

`HubSession::new` also fixes one caller identity, a capability ceiling,
and a maximum privacy class before the first dispatch. Admission rejects a
request that changes caller, contains a capability outside that ceiling,
or exceeds the privacy ceiling. The CLI fully parses its bounded input
first, requires one caller identity, and fixes the capability union and
maximum privacy class before constructing the session; no request can
widen those values after execution begins.

## 4. Ordering guarantee

```text
input order = admission order = execution order = reply order
             = audit logical sequence order
```

This falls out of the type system, not an independently-enforced
invariant: `HubSession::submit` takes `&mut Hub`, so two overlapping calls
cannot exist; Rust's borrow checker makes this a compile-time guarantee,
not a runtime one. No Tokio, no threads, no async runtime is used or
needed for a v0 session executor -- see `admit()`'s new `WorkerBusy` fault
(`docs/architecture/semantic_hub_v0.md` section 10) for the explicit,
checked (not merely assumed) form of this same invariant at the dispatch
layer.

The caller must submit requests in the exact order it wants them admitted.
`HubSession` does not reorder, batch, or parallelize submissions.

## 5. Session-level cumulative accounting

After each `submit()` call, `HubSession` updates its own cumulative
counters from the resulting `HubReply.resource_usage` -- never a fabricated
value:

```text
resource_usage field is None -> contributes nothing to the cumulative sum
resource_usage field is Some(n) -> cumulative += n (saturating, never
                                    silently wrapping)
```

The internal `HubSessionSummary` (from `HubSession::summary()`) reports:

```text
session_id, caller_identity, capability_ceiling, privacy_ceiling, ceiling
requests_submitted     every submit() call, regardless of outcome
requests_admitted      passed admission (dispatched, whether it then
                        succeeded, failed, or crashed) -- i.e. every
                        HubReplyStatus except Rejected
requests_rejected_pre_dispatch
cumulative_input_bytes, cumulative_output_bytes, cumulative_wall_time_millis
first_logical_sequence, last_logical_sequence (both None if the session
  never admitted any request)
```

## 6. Cancellation model

```rust
pub fn cancel(&mut self, request_id: HubRequestId);
```

Marks a `request_id` as pre-cancelled. The *next* time `submit()` is
called with a matching `request_id`, admission rejects it with
`HubFault::Cancelled` before dispatch (using the same `already_cancelled`
ambient check `admission::admit`'s step 1 already had for a single
`Hub::invoke` call).

There is no mechanism to cancel a request that has already been submitted:
v0 is synchronous, so by the time `submit()` returns for a given request,
that request has already fully completed (succeeded, failed, or crashed).
`cancel()` called for an id that was already submitted is a harmless
no-op -- not an error, since a caller building a cancel list from an
external source (e.g. a user-interrupt signal arriving mid-batch) may not
know submission order in advance.

The CLI parses its full bounded file before execution and applies all
cancel control records before submitting request records. Therefore a
cancel line may appear before or after its target in NDJSON while still
remaining pre-dispatch cancellation; request-to-request execution order is
otherwise unchanged.

## 7. `smc hub session` CLI contract

```text
smc hub session --requests <file> [--out <file>] [--max-requests <n>]
  [--session-id <id>]
```

### 7.1 Input format

Newline-delimited JSON (NDJSON). Blank lines are skipped. Each non-blank
line is exactly one of:

**A request record** (JSON object with a `payload` field):

```json
{
  "request_id": "optional, generated if omitted",
  "tool_id": "vector.turbovec",
  "operation_id": "vector.index.insert",
  "session_id": "optional, overridden by this session's own id regardless",
  "caller_identity": "optional, defaults to cli:local",
  "capabilities": ["VectorIndexMutate", "PrivateStorageRead", "PrivateStorageWrite"],
  "privacy_class": "optional, defaults to ProjectLocal",
  "resource_budget": {"...": "optional partial override of V0_CEILING"},
  "payload": {"index": "docs", "vectors": [[1,0]], "ids": [1]}
}
```

Same shape as `smc hub invoke --input`'s request file, except `tool_id`
and `operation_id` are required JSON fields here (a batch can target
different operations, even different tools once more than one is
registered, line by line) rather than positional CLI arguments.

The complete file is parsed, record-size checked, and duplicate/persisted
identity checked before the first dispatch. A malformed later line,
duplicate ID, mixed caller identity, request count above
`--max-requests`, unresolved pending ID, or audit-capacity failure rejects
the whole batch without applying an earlier mutation. `--out` is likewise
validated before dispatch and may not resolve inside `.semantic/hub/`.

**A cancel record**:

```json
{"cancel": "<request_id>"}
```

### 7.2 Output format

NDJSON on stdout (or `--out <file>`): one reply object per request line,
in submission order (the same shape `smc hub invoke` produces -- see
`hub_api_v0.md`), followed by exactly one final line:

```json
{"session_summary": {
  "capability_ceiling": ["PrivateStorageRead", "VectorSearch"],
  "privacy_ceiling": "ProjectLocal",
  "requests_submitted": 6, "requests_admitted": 6,
  "requests_rejected_pre_dispatch": 0, "cumulative_input_bytes": 310,
  "cumulative_output_bytes": 542, "cumulative_wall_time_millis": 1001,
  "first_logical_sequence": 0, "last_logical_sequence": 5
}}
```

This external presentation object is not the internal `HubSessionSummary`:
it deliberately omits raw `session_id` and `caller_identity`. Those values
remain internal for session validation and persisted audit provenance. Each
per-request reply retains its public `request_id`, which is sufficient for
consumer correlation and for later `smc hub audit --request <request-id>`
lookup. No alternate raw, compatibility, debug, environment-controlled, or
logging output path is defined.

### 7.3 Exit code

`0` only if every request in the batch reached `Success`. A batch-level
preflight failure emits no partial reply stream and returns nonzero. Any other
outcome for any request (including a session-ceiling rejection or a
cancelled request) produces exit code `1` and a
`"SessionCompletedWithFailures: ..."` stderr message -- the per-request
NDJSON replies (already written to stdout/`--out`) are where the specific
fault codes are.

### 7.4 Duplicate `request_id` rejection

A `request_id` colliding with an earlier line in the *same* batch, or with
any record already in the persisted audit log, fails the whole batch with
`DuplicateRequestId`. Persisted correlation uses the bounded file-streaming
audit scanner; the full audit history is never materialized under the
request-file limit. An unresolved pending marker is also a duplicate and
cannot be overwritten by a retry.

### 7.5 Audit durability

Each admitted request's audit record is appended to `.semantic/hub/audit.log`
immediately after that request completes, via `HubAuditTrail::append_records_to_file`
(a true streaming append after the log's first migration to the
`"unbounded"` header form -- see `docs/architecture/semantic_hub_v0.md`
section 11), not batched to the end of the session. A crash partway
through a large batch loses at most the not-yet-appended tail of the
batch; every earlier request in the batch remains durably audited and its
underlying tool-state mutation remains independently recoverable via
`vector.index.recover` (`docs/architecture/semantic_hub_v0.md` section 20).

Every request gets a synced pre-dispatch pending marker, exactly like
`smc hub invoke`. It is removed only after that request's audit record has
been appended and synced. If the process stops after a mutation commits
but before audit completion, `smc hub audit --request <id>` reports
`PendingUnresolved` rather than the misleading `UnknownRequest`.

## 8. Determinism

Real time is not part of any deterministic identity: `logical_sequence`
ordering is purely a function of submission order (an incrementing
counter), never derived from a timestamp. `resource_usage.wall_time_millis`
remains real elapsed time (unchanged from the base contract) and is
excluded from any exact-equality comparison a golden fixture might want to
make against session output, same as for a single `Hub::invoke` reply.

## 9. Related documents

```text
docs/architecture/semantic_hub_v0.md          sections 10, 11, 19, 20
docs/spec/hub/hub_api_v0.md                    base request/reply contract
docs/spec/hub/hub_adapter_contract_v0.md       per-request resource budget
docs/spec/hub/turbovec_adapter_v0.md           vector.index.recover, transaction protocol
docs/security/semantic_hub_threat_model_v0.md  session-specific threat coverage
```
