# Semantic Hub v0 Architecture

Status: v0, landed on `main` (#1554, #1555) and completed (this document's own completion pass, branch `feat/semantic-hub-v0-completion`) to the full acceptance criteria of architecture issue #1526
Track: Hub / execution boundary
Purpose: document the Semantic Hub v0 architecture -- a governed execution boundary for external computational tools -- closing issues #1553 and #1526

## 1. Purpose

Semantic Hub is a governed execution boundary between the `smc` CLI and
external computational tools. A caller invokes a bounded operation on an
external tool -- for example a vector search over a TurboVec index -- through
a single typed request/reply contract that enforces capability checks,
resource budgets, worker supervision, and audit before and after the call,
instead of each CLI feature reaching into a tool's native API directly with
ad hoc error handling.

v0 implements exactly one adapter (`vector.turbovec`) and exactly one
execution mode (in-process, unisolated). The contract is written so
additional adapters and execution modes (subprocess, WASM, remote) can be
added later without changing the request/reply shape or the admission
pipeline. Nothing beyond in-process execution is implemented; section 15
lists the complete set of non-claims.

Hub governs execution. It does not govern meaning -- see section 4.

## 2. Ownership

```text
crates/semantic-hub            - generic Hub contract + runtime
crates/semantic-hub-turbovec   - reference adapter (vector.turbovec)
crates/smc-cli                 - `smc hub ...` CLI surface (private hub.rs module)
```

`semantic-hub` owns the request/reply envelope, admission pipeline,
capability model, resource budget model, worker lifecycle and supervision,
and audit/provenance format. It defines no adapter and knows nothing about
TurboVec or any concrete tool domain.

`semantic-hub-turbovec` owns the one v0 reference adapter: the
`vector.turbovec` tool against `turbovec::IdMapIndex`, including its own
on-disk persistence format and operation set. It does not own any part of
admission, capability, or audit -- those are consumed from `semantic-hub`
unchanged.

`smc-cli` owns the `smc hub ...` command surface: a new private module
`hub.rs`, not re-exported at the crate root, following the same pattern as
the existing `ui_frame_inspect` module. `smc-cli` is the only crate that
constructs the tool registry and registers `vector.turbovec` at process
startup.

No other crate depends on `semantic-hub` or `semantic-hub-turbovec` in v0.

## 3. Dependency graph

```text
semantic-hub            (depends on nothing but std)

semantic-hub-turbovec
  -> semantic-hub
  -> turbovec (=0.9.0, crates.io, MIT, https://github.com/RyanCodrai/turbovec)
  -> serde, serde_json

smc-cli
  -> semantic-hub
  -> semantic-hub-turbovec
  -> serde, serde_json
  (plus its existing pre-Hub dependencies, unchanged)
```

`semantic-hub` has zero path-dependencies on any other in-repo crate,
verified via `cargo tree`. It does not depend on `prom-cap`, `prom-audit`, or
`sm-runtime-core`. This is a deliberate boundary decision made only after
checking whether those crates' existing types could be reused.

### 3.1 Why not depend on prom-cap, prom-audit, or sm-runtime-core

`prom_cap::CapabilityManifest`/`CapabilityKind` describe SemCode host-ABI
effects (gate reads, pulses, semantic state writes). `sm_runtime_core::RuntimeQuotas`
describes VM execution quotas. `prom_audit`/`prom_state`'s canonical-text
convention serializes SemCode runtime audit and replay events. None of these
describe "an external tool operation's capabilities, resource budget, and
audit trail" -- forcing Hub into `CapabilityKind` would mean adding
vector-search-specific variants to an enum whose job is host-ABI effect
classification, polluting an unrelated ABI-effect enum with a concern it was
never meant to carry.

Instead `semantic-hub` defines its own `HubCapability`,
`HubResourceBudget`/`HubResourceKind`, and `HubAuditRecord`/`HubAuditTrail` --
each deliberately mirroring the established repo pattern rather than
inventing a new one:

- `HubCapabilitySet` mirrors `CapabilityManifest`'s schema-versioned
  `BTreeSet` shape.
- `HubResourceBudget`'s non-panicking `first_violation()`/`check_budget()`
  mirrors `RuntimeQuotas`'s `exceed() -> Option<QuotaExceeded>` pattern.
- `HubAuditTrail`'s canonical-text format (magic header
  `semantic-hub.audit.v1`, `format_version: u32`, tab-delimited escaped
  fields, strict round-trip validation) mirrors
  `prom_audit::AuditReplayArchive`/`prom_state::StateSnapshotArchive`'s exact
  convention.

This is intentional reuse-of-pattern, not a second capability framework. The
repo already has this precedent: `prom-gates::GateId` and `prom-rules::RuleId`
are domain-specific types that live alongside `prom-cap` rather than being
folded into it. `HubCapability`, `HubResourceKind`, and the Hub audit format
follow the same precedent.

## 4. Authority and non-authority

Hub governs execution. Semantic governs meaning. These are not the same
authority and Hub does not claim to be the second one.

A successful Hub reply confirms:

```text
the tool_id and operation_id were known and registered
the request was permitted by capability and privacy policy
the request was within resource budget
the worker executed (or declined, or crashed, or timed out, or was cancelled)
the output passed structural reply validation
```

A successful Hub reply never confirms:

```text
the result is TRUE
the result is relevant to the caller's actual question
the result is safe to commit to Semantic state
the result is logically compatible with anything else the caller believes
```

TurboVec search results returned through Hub are candidates and evidence
only -- never Semantic truth, never a verified-relevance claim, never a
permission-to-act claim. A caller that wants to act on a Hub result must run
it back through Semantic's own admission and verification paths; Hub does
not shortcut that.

## 5. Request lifecycle

```text
CLI caller
  -> typed HubRequest
  -> request admission (semantic-hub/src/admission.rs::admit())
  -> registry lookup
  -> capability + privacy policy check
  -> immutable resource budget check
  -> supervised TurboVec worker dispatch (panic-contained)
  -> validated HubReply
  -> audit + provenance record (canonical text, digest-based)
  -> untrusted computational evidence returned to caller
```

Every request passes through admission before dispatch is attempted, and
every outcome -- successful, rejected, tool-declared-failed, crashed, or
cancelled -- produces exactly one `HubReply` and exactly one audit record.
There is no code path where dispatch silently drops evidence.

## 6. Admission

`crates/semantic-hub/src/admission.rs::admit()` runs the following checks in
this exact order. Each step produces a distinct `HubFault` (section 10);
earlier steps never depend on state a later step establishes.

```text
1. Hub API version compatibility
     HubApiVersion::CURRENT.is_compatible_with(requested)
     same major required; caller minor <= implementation minor
2. envelope schema_version check
     HUB_ENVELOPE_SCHEMA_VERSION = 1
3. payload size bound
     MAX_PAYLOAD_BYTES = 32 MiB, rejected before registry lookup
4. request's own declared input-budget check
     request.payload.len() <= request.resource_budget.input_bytes;
     narrower than step 3 if the caller chose to narrow it, so step 3
     alone does not enforce this
5. already-cancelled check
6. registry lookup
     unknown tool_id                 -> UnknownTool
     unknown operation on known tool -> UnknownOperation (distinct fault)
7. worker lifecycle gate
     Disabled / Quarantined / Stopped states reject before dispatch
8. capability check
     HubCapabilitySet::satisfies(): every required capability must be
     both present in the grant AND non-sensitive
9. resource budget check
     requested HubResourceBudget must be within BOTH the tool's declared
     resource_ceiling AND the global HubResourceBudget::V0_CEILING,
     checked via first_violation(), which returns the real violating
     dimension/limit/attempted value, never a placeholder
10. queue/concurrency admission
     request's declared queue_depth/concurrent_requests checked
     against ambient counters
```

Steps 3 and 6 are ordered deliberately: payload size is bounded before the
registry is consulted, so an oversized request cannot be used to probe which
tool/operation names exist. Step 4 (the request's own narrower input-budget
check) sits between them for the same reason: it is still a payload-shape
rejection, decided before anything registry- or capability-related runs, so
a request that is both over its own input budget and already cancelled
deterministically gets `InputRejected`, not `Cancelled`.

### 6.1 Capability check is structural, not just unfilled

Step 8 is not "nobody has granted these yet." Nine capabilities are marked
`is_sensitive()` on `HubCapability` and can never be satisfied by any grant:

```text
NetworkAccess  ArbitraryFilesystemRead  ArbitraryFilesystemWrite  ProcessSpawn
DeviceAccess   EnvironmentRead          SecretRead
ProjectMutation                        SemanticStateMutation
```

`HubToolDescriptor::validate()` refuses to even *register* a tool whose
operation declares one of these as required -- no v0 tool could ask for one
even if it wanted to; the deny is structural at registration time, not just
a single admission-time check a future bug could bypass.

The remaining eleven capabilities -- `VectorIndexCreate`, `VectorIndexRead`,
`VectorIndexMutate`, `VectorSearch`, `VectorFilteredSearch`,
`VectorIndexPersist`, `CpuCompute`, `MemoryAllocateBounded`,
`PrivateStorageRead`, `PrivateStorageWrite`, `ClockMonotonic` -- are the
bounded, grantable set v0 tools operate within.

## 7. Worker lifecycle and dispatch

`crates/semantic-hub/src/worker.rs` defines nine explicit `HubWorkerState`
values: `Registered`, `Starting`, `Ready`, `Busy`, `Degraded`, `Restarting`,
`Quarantined`, `Disabled`, `Stopped`. Transitions are governed by an explicit
closed table, `HubWorkerState::can_transition_to`, exhaustively unit-tested;
illegal transitions are rejected and leave state unchanged.

`HubSupervisionPolicy::conservative_default()`:

```text
restart_policy                            = OnCrash
max_restarts_before_quarantine            = 3
max_protocol_violations_before_quarantine = 1
```

A single protocol violation quarantines the worker immediately -- stricter
than the crash tolerance of three restarts. A crash can be a transient fault
in the tool's own logic; a reply that does not conform to the expected shape
means the Hub can no longer trust anything else that worker says, including
a future well-formed-looking reply.

`Hub::dispatch` in `crates/semantic-hub/src/runtime.rs`:

```text
health checked -> mark_busy
  -> panic::catch_unwind(AssertUnwindSafe(|| worker.handle(...)))
  -> Ok(bytes): output_bytes budget checked, then worker.validate_reply()
       structural check (adapters may add tool-specific reply-shape
       validation; failure here is ProtocolViolation, distinct from a
       tool-declared failure) -> mark_ready
  -> Ok(Err(HubToolError)): ToolDeclaredFailure -> mark_ready
       (NOT a crash -- worker stays healthy)
  -> panic: WorkerPanicked -> health.report_crash()
       (may transition to Degraded or straight to Quarantined per policy)
  -> protocol violation: health.report_protocol_violation()
       (quarantines at threshold 1, per policy above)
```

Every outcome always produces a `HubReply` and an audit record. A
tool-declared failure does not crash the worker; a panic is contained by
`catch_unwind` and never propagates as a process crash of the CLI itself.

## 8. Reply status taxonomy

`HubReplyStatus` (`envelope.rs`) has five variants:

```text
Success
Rejected(HubFault)     -- pre-dispatch, admission failed
ToolFailed(HubFault)   -- tool declared its own failure, or a protocol violation
Crashed(HubFault)      -- worker panicked
HubFault(HubFault)     -- Hub's own internal fault
```

`Rejected`, `ToolFailed`, `Crashed`, and `HubFault` all carry a `HubFault`
value but are kept as distinct top-level variants, rather than a single
`Failed(HubFault)` with a status field, so a caller or the audit record can
tell which phase of the lifecycle produced the fault without inspecting the
fault code itself.

## 9. Resource governance

`crates/semantic-hub/src/resource.rs` defines twelve budget dimensions
(`HubResourceKind`): `WallTimeMillis`, `MemoryBytes`, `InputBytes`,
`OutputBytes`, `IndexItemCount`, `VectorDimensions`, `ResultCount`,
`QueueDepth`, `ConcurrentRequests`, `StorageReadBytes`, `StorageWriteBytes`,
`AuditBytes`.

`is_hard_enforced_v0()` marks eight as hard-enforced: `WallTimeMillis`,
`InputBytes`, `OutputBytes`, `IndexItemCount`, `VectorDimensions`,
`ResultCount`, `QueueDepth`, `ConcurrentRequests`. The remaining four --
`MemoryBytes`, `StorageReadBytes`, `StorageWriteBytes`, `AuditBytes` -- are
advisory-only in v0. This is stated honestly rather than claimed as
enforced: in-process Rust cannot precisely enforce a memory hard limit
without OS-level containment, which v0 does not have (section 15).

All budget arithmetic is checked (`check_budget()` uses `checked_add` and
rejects overflow rather than silently wrapping).

`HubResourceBudget::V0_CEILING` is the default and maximum budget:

```text
wall_time_millis = 30_000 (30 s)   memory_bytes        = 512 MiB
input_bytes      = 64 MiB          output_bytes        = 16 MiB
index_item_count = 1_000_000       vector_dimensions   = 4096
result_count     = 10_000          queue_depth         = 256
concurrent_requests = 32           storage_read_bytes  = 256 MiB
storage_write_bytes = 256 MiB      audit_bytes         = 1 MiB
```

A caller's requested budget is checked against both the tool's declared
`resource_ceiling` and this global ceiling (admission step 9); the caller can
only ask for less, never more, than either.

## 10. Fault taxonomy

`HubFault` (`fault.rs`) has 28 variants, each with a stable `.code()` string
independent of its human-readable message:

```text
UnknownTool          UnknownOperation      ApiVersionUnsupported
SchemaVersionUnsupported  DescriptorIncompatible  InputRejected
CapabilityDenied     PrivacyDenied         SensitiveCapabilityDenied
WorkerDegraded       SessionLimitExceeded  ResourceBudgetInvalid
QueueFull            ToolDisabled          ToolQuarantined
DeadlineExceeded     Cancelled             ResourceExhausted
WorkerBusy           ToolDeclaredFailure   PersistenceFailed
RecoveryRequired     WorkerPanicked        ProtocolViolation
OutputRejected       AuditProvenanceFailure  InternalHubFault
SequenceExhausted
```

The stable code string is what appears in the audit record's `fault_code`
field and in the CLI's non-zero-exit error string; the human-readable
message is not parsed by anything and may change freely.

The four v0-completion additions, and why each is a distinct top-level
`HubFault` variant rather than folded into an existing one:

- **`SensitiveCapabilityDenied`**: a request whose `capability_context`
  grants any sensitive capability (e.g. `NetworkAccess`) is now rejected
  at admission outright, even when the target operation does not require
  it. Before this pass, such a grant was silently accepted and stripped
  before the adapter ever saw it (`HubCapabilitySet::deny_sensitive()`,
  still applied at dispatch as defense-in-depth) -- distinct from
  `CapabilityDenied`, which means a *required* capability was missing.
- **`WorkerDegraded`**: a mutating operation targeting a `Degraded`
  worker (elevated crash count, not yet quarantined or restarted) is
  rejected; read-only operations still proceed against a degraded worker
  (see `HubWorkerState::accepts_dispatch`). Limits how much further
  damage a flaky worker can do before supervision resolves it, without
  blocking reads unnecessarily.
- **`SessionLimitExceeded`**: the *session-level*, cumulative ceiling
  (request count, cumulative input/output bytes, cumulative wall time --
  see section 19) was exceeded. Distinct from `ResourceBudgetInvalid`/
  `ResourceExhausted`, which are per-request.
- **`WorkerBusy`**: dispatch was attempted while the worker's own health
  state was already `Busy`. Structurally unreachable through the public
  `Hub::invoke`/`invoke_in_session` API under v0's synchronous,
  single-owner (`&mut Hub`) execution model -- checked explicitly anyway,
  so a future concurrent execution mode cannot silently re-enter a
  worker mid-dispatch. (Previously, reaching this state -- impossible
  today, but the code path existed -- was misreported as
  `ToolQuarantined`; fixed as part of adding this variant.)

- **`PersistenceFailed`** / **`RecoveryRequired`**: distinct top-level
  `HubFault` variants for the two adapter-declared failure classes the
  transaction/recovery protocol (section 20) actually produces. An adapter
  never constructs a `HubFault` directly -- it returns a `HubToolError`
  with a stable `code` string, and `runtime.rs`'s `hub_fault_from_tool_error`
  maps the known codes `"PersistenceFailed"` and `"RecoveryRequired"` (and
  `"DeadlineExceeded"`) onto their own `HubFault` variant, falling back to
  `ToolDeclaredFailure` for any other adapter-declared code. This keeps
  adapters ignorant of the full Hub fault taxonomy while still giving these
  two failure classes stable, distinct top-level codes in the audit trail
  and the CLI's exit-code mapping, rather than leaving them indistinguishable
  from an ordinary `ToolDeclaredFailure`. CLI-level infrastructure failures
  that never reach dispatch at all (audit log I/O, scoped-storage
  violations, pending-marker bookkeeping) remain plain `"<Code>: <message>"`
  `Result<(), String>` errors from `smc-cli`'s own command functions, per
  the existing convention -- that distinction (per-request dispatch outcome
  vs. CLI-process-level infrastructure failure) is what actually decides
  whether something is a `HubFault` variant, not the persistence/recovery
  subject matter itself.

## 11. Audit and provenance

`crates/semantic-hub/src/audit.rs` and `provenance.rs` define
`HubAuditRecord`. Each record carries: `sequence` (monotonic, seedable across
process restarts via `Hub::seed_next_sequence()`, since the CLI is one
process per invocation and the audit log is a persisted file reloaded each
time), `request_id`, `session_id`, `caller_identity`, `tool_id`,
`tool_version`, `adapter_provenance`, `operation_id`, `execution_mode`,
`determinism`, `trust_class`, `privacy_class`, `capabilities_granted`,
`input_digest`/`output_digest` (both `HubDigest`), `resource_budget` (full
struct, packed), `resource_usage` (full struct, `Option<T>` fields -- `None`
means "not measured", never a fabricated zero), `worker_state_after`,
`status_code` (the reply-status discriminant: one of
Success/Rejected/ToolFailed/Crashed/HubFault), and `fault_code`
(`Option<one of the 21 HubFault codes>`).

`HubDigest` is FNV-1a-64 plus byte length -- explicitly a non-cryptographic
correlation fingerprint, not a security or integrity guarantee; there is no
signing chain yet. That is tracked as future work under issue #1374, not
claimed as done here.

`status_code` and `fault_code` are kept structurally distinct rather than
folded into one field. An earlier implementation bug conflated them and
broke the canonical-text parser on reload, because `status_code`'s parser
only recognizes the five reply-status names and cannot also parse one of the
21 fault codes. This was caught by dogfooding through the built CLI binary,
not by unit tests in isolation, and fixed with a regression test that
reloads a persisted audit log containing a non-`Success` record.

Canonical serialization: magic header `semantic-hub.audit.v1`,
`format_version` (`u32`, currently 1), a declared record count OR the
literal string `"unbounded"` (see below), then one tab-delimited,
field-escaped line per record (20 fields per record). Reload performs
strict round-trip validation and enforces that sequence numbers are
monotonically non-decreasing; a corrupt or hand-edited log fails to load
rather than silently accepting a gap or reordering.

**Streaming reads and appends (v0-completion pass).** The pre-completion
implementation re-read, fully parsed into a `Vec<HubAuditRecord>`, and fully
re-wrote the entire audit history on every single `smc hub invoke` call and
every `smc hub audit` lookup -- for one new/looked-up record. Three
additions avoid this without changing the on-disk format for a reader that
does not care:

- `HubAuditTrail::find_by_request_streaming(text, id)` scans line-by-line
  and returns as soon as a match is found, instead of parsing every record
  into a `Vec` first. Used by `smc hub audit`.
- `HubAuditTrail::next_sequence_streaming(text)` parses only the final
  record line, not the whole history. Used by `smc hub invoke` and
  `smc hub session` to seed `Hub::seed_next_sequence`.
- `HubAuditTrail::append_records_to_file(path, records)` appends new
  records directly (`OpenOptions::append`) instead of rewriting the whole
  file, once the file's header already carries the `"unbounded"` sentinel
  in place of an exact count (written by
  `to_canonical_text_streaming()`). A file still in the older exact-count
  form is migrated once -- a full read/rewrite, the same cost the
  pre-completion path always paid, but only the *first* time a
  v0-completion build touches that file; every append after that is
  cheap. `smc hub invoke`'s `save_audit_trail` now always writes the
  streaming form too, so a project stays migrated rather than
  flip-flopping between the two header shapes.
- `smc hub session` appends each admitted request's audit record
  immediately after that request completes, not once at the end of the
  batch -- so a crash partway through a large session batch loses at most
  the not-yet-appended tail of the batch, never an already-reported
  request's evidence. See section 19.

`from_canonical_text`'s strict record-count check is unchanged (still an
error) for any file using the older exact-count header; it is simply not
applied when the header is `"unbounded"`, since an append-friendly file's
true count is not knowable without a full scan and re-deriving it on every
append would defeat the purpose.

**Extended `HubReply`/`HubProvenance` (v0-completion pass).** `HubReply`
(`envelope.rs`, `HUB_ENVELOPE_SCHEMA_VERSION` bumped 1 -> 2) gained
`logical_sequence: u64` (equal to the audit record's own `sequence`),
`provenance: HubProvenance`, and `warnings: Vec<String>` (always empty in
v0 -- no warning-producing path exists yet, never backfilled just to be
non-empty). `HubProvenance` (`provenance.rs`) grew from a TurboVec-shaped
struct into a generic, Hub-owned envelope satisfying the "generic Hub
provenance envelope" requirement of #1526's acceptance criteria: it now
carries `schema_version`, `request_id`, `session_id`, `logical_sequence`,
`caller_identity`, the same tool/adapter/execution/determinism/trust/
privacy fields as before, `input_digest`/`output_digest`,
`capability_context_digest`/`resource_budget_digest` (both derived from
the same canonical-text encodings the audit trail's own packed columns
use, via `HubCapabilitySet::canonical_text()` and
`HubResourceBudget::canonical_text()` -- one shared encoding, not two that
could drift), `worker_state_after`, an optional `artifact:
HubArtifactProvenance` (kind/id/digest of a mutating operation's committed
durable artifact -- `None` for a non-mutating operation), and `warnings`.
`smc-cli`'s `build_cli_reply_json` surfaces all of this in the CLI's JSON
output, not only the raw payload bytes.

## 12. Tool registry

`crates/semantic-hub/src/registry.rs` stores tools in a
`BTreeMap<HubToolId, ...>`, specifically for deterministic ascending-order
iteration -- `smc hub tools` output is stable across runs and machines.

Registration rejects three cases: an invalid descriptor
(`HubToolDescriptor::validate()` fails for no operations declared, duplicate
operation ids within the tool, incompatible Hub API version, or any
operation requiring a sensitive capability); an exact duplicate (same
`tool_id`, identical descriptor -> `DuplicateToolId`); and a conflicting
duplicate (same `tool_id`, a different descriptor -> `ConflictingDescriptor`).
v0 has no supported tool-replace-in-place lifecycle -- re-registering the
same `tool_id` with a changed descriptor is rejected outright rather than
silently overwriting the previous registration.

There is no dynamic loading and no filesystem discovery of tools. Tools are
registered by explicit Rust code in the Hub CLI binary at process startup.
In v0 that is exactly one entry: `vector.turbovec`.

## 13. TurboVec adapter

`crates/semantic-hub-turbovec` wraps `turbovec::IdMapIndex`, which layers
stable `u64` external IDs over the positional `TurboQuantIndex`. Operations:
`vector.index.create`, `vector.index.describe`, `vector.index.insert`,
`vector.index.remove`, `vector.search`, `vector.search.filtered`,
`vector.index.reset`, `vector.index.recover` (v0-completion pass -- see
section 20).

Persistence is implicit -- there is no separate save/load verb. Every
mutating operation loads the index from a scoped `.tvim` file, runs it
through the write-ahead transaction protocol in section 20, and produces
artifact provenance for the reply; read operations load the current file
fresh on every call. This follows from the process model: the Hub CLI is
one short-lived process per `smc hub invoke` call (or one short-lived
process handling many requests for one `smc hub session` batch), with no
long-running Hub daemon holding state in memory across separate CLI
invocations in v0 -- the on-disk file is the source of truth between them.

`vector.index.reset` has no native TurboVec "clear" operation. It is
implemented as constructing a fresh empty index with the same dimension and
bit width and overwriting the file -- documented as the actual
implementation, not hidden behind a name that implies a native clear exists.

Index identity is a validated `IndexName`: lowercase ASCII alphanumeric plus
`_`/`-` only, maximum 64 characters. No `.`, `/`, or `\` characters are
permitted at all, so path traversal through an index name is structurally
impossible by construction, not merely checked for and rejected at runtime.
`MAX_INDEX_COUNT = 256` persisted indexes per scoped directory.

The full workflow was verified live through the built `smc` binary, not just
unit tests: create an index, insert four vectors, describe it, search
(correct nearest-neighbor ranking), filtered search (correctly excludes ids
outside the allowlist), remove one id, search again (confirmed gone), and
look the operation up in the audit log by request_id -- all through real
TurboVec quantization, with no stub or mock path.

## 14. Determinism

Determinism is empirically measured, not assumed. See
`crates/semantic-hub-turbovec/tests/determinism_qualification.rs`, three
passing tests run on this machine. TurboVec's rotation-matrix construction
uses a fixed internal seed constant (42, via ChaCha8Rng + QR decomposition),
not OS randomness. Verified: (a) repeated identical search on the same
loaded index produces byte-identical replies across 10 repetitions; (b)
reloading the index fresh from its persisted file, simulating separate CLI
process invocations, produces byte-identical replies across 3 independent
fresh loads; (c) exact-duplicate-vector ties produce a stable, repeatable
order (the specific tie-break rule is TurboVec's own internal behavior, not
something this adapter defines). Not verified: cross-CPU-backend
byte-identity -- TurboVec has runtime SIMD dispatch (AVX-512/AVX2/scalar on
x86, NEON on ARM); a different CPU could take a different code path, and
this has only been tested on one machine.

Operation classification: `vector.index.insert`, `vector.search`, and
`vector.search.filtered` are `DeterministicWithSeed` (fixed internal seed,
not caller-supplied); `vector.index.create`, `vector.index.describe`,
`vector.index.remove`, and `vector.index.reset` are `Deterministic` (no
floating-point kernel involved).

## 15. In-process limitations and non-claims

The TurboVec adapter's trust class is `InProcessUnisolated` -- chosen
deliberately, it does not claim memory-corruption isolation. A panic inside
the worker is contained by `catch_unwind` at the dispatch boundary, but that
is crash containment within one process, not sandboxing.

Execution mode `InProcess` is the only mode implemented in v0. `Subprocess`,
`Wasm`, and `Remote` exist as enum variants on the execution mode type so the
API contract will not need to change shape when they are implemented later,
but nothing beyond `InProcess` exists today.

This document, and the v0 implementation it describes, make none of the
following claims:

```text
no subprocess execution is implemented
no WASM execution is implemented
no remote execution is implemented
no process-level isolation exists
no memory isolation exists
no cryptographic signing chain exists (digests are correlation
  fingerprints only, see section 11)
no cross-CPU determinism guarantee exists
no dynamic tool loading exists
no plugin marketplace exists
no Workbench/Studio/ALM integration exists
TurboVec search results are candidates/evidence only -- never Semantic
  truth, never a verified-relevance claim, never a permission-to-act claim
```

## 16. CLI surface

```text
smc hub tools
smc hub describe <tool-id>
smc hub invoke <tool-id> <operation-id> --input <file> [--out <file>]
smc hub session --requests <file> [--out <file>] [--max-requests <n>]
smc hub audit --request <request-id>
```

There is no dedicated `smc hub recover` subcommand: recovery
(`vector.index.recover`) is a bounded operation like any other, reachable
through `invoke` or `session` -- see section 20.

`smc hub tools` lists registered tools in deterministic order (registry
iteration order, section 12). `smc hub session` is detailed in section 19.

Persistent state lives under `.semantic/hub/` relative to the current
working directory, matching the project-local storage convention proposed by
issue #1372:

```text
.semantic/hub/
  vector.turbovec/
    <name>.tvim        one file per persisted index
    <name>.tvim.txn    one write-ahead transaction record per index
                        (section 20), always overwritten in place
  audit.log             canonical audit trail; appended to directly once
                         in streaming form (section 11), not rewritten
                         whole on every invocation
```

The request file passed via `--input` is JSON and requires an explicit
`capabilities` array (deny-by-default, no auto-grant shortcut) and a
`payload` object whose shape is tool-specific. An optional
`resource_budget` object may partially override `V0_CEILING`; unspecified
fields fall back to the ceiling default.

Exit code 0 is returned only for a `Success` status. Any other status
produces a non-zero exit with an error string of the form
`"{fault_code}: {message}"`, matching the existing single `Ok -> 0` /
`Err -> 1` CLI convention -- there are no finer-grained OS exit codes
anywhere in this repository's CLI today. The fault code is carried in the
leading token of the error string, the same pattern the pre-existing
`smc look` command uses for `LookStatus`.

## 17. Relation to Semantic Core, PROMETHEUS, and the rest of the CLI

Semantic Hub has no coupling to Semantic Core or PROMETHEUS. It does not
depend on `sm-front`, `sm-sema`, `sm-ir`, `sm-verify`, `sm-vm`,
`sm-runtime-core`, `prom-abi`, `prom-cap`, `prom-state`, `prom-rules`,
`prom-runtime`, or `prom-audit`, and none of those crates depend on
`semantic-hub` or `semantic-hub-turbovec`. The only new coupling is
`smc-cli`, which now also depends on both Hub crates in addition to its
existing dependencies, and owns the `smc hub ...` command surface alongside
its other pre-existing commands.

Hub does not compile, verify, or execute SemCode. It does not read or write
Semantic state. It does not participate in the verifier-first execution path
described in `docs/architecture/svm_verified_execution_core.md`. A Hub tool
result is external computational evidence a caller may choose to feed into
Semantic's own admission and verification paths later -- Hub itself does not
perform that step and does not shortcut it.

**Relation to issue #1373 (Semantic extension and plugin boundary).** #1373
defines what *extensions* (Workbench/Studio-facing UI extensions) may
display, request, transform, and propose -- a caller-facing, UI-adjacent
surface. Hub defines how *external computational tools* (TurboVec, a future
SMT solver, ...) are admitted, isolated, invoked, supervised, budgeted, and
audited -- a callee-facing execution boundary. The two are non-duplicative:
an extension could, in principle, be one of the callers that eventually
issues a `HubRequest` (through whatever caller-identity/session model the
host application built on top of Hub uses), but Hub itself has no concept
of an "extension," does not implement #1373's display/request/propose
model, and #1373 does not implement admission, capability enforcement,
resource budgets, or audit for external tool execution. Hub consumes
neither #1373's rules nor is consumed by them; they govern adjacent,
non-overlapping surfaces.

**Tool-to-tool calls are forbidden in v0**, by construction, not only by
policy: `HubTool::handle` receives a `&RestrictedHubContext` (resource
budget, sanitized capabilities, deadline) and its own payload -- it has no
reference to the `Hub` that dispatched it, no way to construct a new
`HubRequest`, and no route back into admission. A tool cannot call another
tool even if its implementation tried to; there is no API surface for it to
call.

## 18. Forbidden shortcuts

Future Hub work must not:

```text
grant a sensitive capability to any tool, by request or by registry entry
treat a tool-declared failure as a worker crash, or vice versa
treat a HubReply Success status as a claim about truth, relevance,
  or safety of the underlying result
implement a second capability, budget, or audit framework elsewhere
  in the repo instead of extending semantic-hub's
claim cross-CPU byte-identical determinism without new measured evidence
claim process or memory isolation without a new execution mode
  actually implementing it
skip an audit record for any dispatch outcome
conflate status_code and fault_code in the audit format
```

## 19. Session model

`crates/semantic-hub/src/session.rs` (`HubSession`) proves the property a
single `Hub::invoke` call cannot: that one registered worker instance can
safely process several ordered requests, with a rolling whole-session
ceiling attenuating every individual request's own per-request budget.
Full contract: `docs/spec/hub/hub_session_v0.md`.

Key properties:

```text
synchronous, single-owner (&mut Hub) -- no Tokio, no threads, no async
input order = admission order = execution order = reply order
             = audit logical sequence order
every session-submitted request still goes through Hub::invoke_in_session,
  the SAME admission/dispatch/audit pipeline as a direct Hub::invoke call
session ceiling (HubSessionCeiling): max_request_count,
  max_cumulative_input_bytes, max_cumulative_output_bytes,
  max_cumulative_wall_time_millis -- checked in admission::admit's own
  step 7, using the SAME AdmissionAmbient mechanism the existing
  queue/concurrency checks (step 6) already used, not a parallel path
cancellation is cooperative and pre-admission-only: a request id marked
  cancelled before HubSession::submit processes it is rejected with
  Cancelled before dispatch; there is no mechanism to cancel a request
  already admitted (v0 is synchronous -- by the time submit() returns,
  the request has already fully completed or failed)
```

`smc hub session --requests <file> [--out <file>] [--max-requests <n>]`
reads newline-delimited JSON: each line is either a request record (the
same shape `smc hub invoke`'s `--input` file uses, but with `tool_id`/
`operation_id` as required fields rather than positional CLI arguments,
since a batch can target different operations line by line) or a control
record `{"cancel": "<request_id>"}`. Output is NDJSON: one structured reply
per admitted or rejected request, in submission order, followed by one
final `{"session_summary": {...}}` line (requests submitted/admitted/
rejected, cumulative input/output bytes, cumulative wall time, first/last
logical sequence). Exit code is non-zero if any request in the batch did
not reach `Success` (`SessionCompletedWithFailures`), matching `smc hub
invoke`'s existing `Ok -> 0` / `Err -> 1` convention.

Each admitted request's audit record is durably appended immediately after
that request completes (see section 11's streaming-append description),
not batched to the end -- a crash partway through a large batch loses at
most the not-yet-appended tail. There is no per-request pending marker the
way `smc hub invoke` has (see `write_pending_marker`'s doc comment):
`smc hub session`'s immediate post-request audit append already gives the
same "durable evidence before moving to the next request" property for the
audit trail, and the underlying tool's own durable state (a TurboVec index)
has its own independent recovery path (section 20) for the narrower window
between a mutation's commit and this function's audit append. This is a
deliberate v0 scope decision, not an oversight.

## 20. Transaction and recovery protocol

`crates/semantic-hub-turbovec/src/transaction.rs` adds a recoverable
write-ahead protocol around every durable `.tvim` mutation, replacing
`save_atomic`'s previous "write temp file, rename" with no durable trace of
*intent*. Sequence, per mutation:

```text
1. begin(): durably write <name>.tvim.txn with phase=Intent, BEFORE the
   candidate artifact write begins. The record's candidate_file_name is
   what save_atomic then writes to.
2. Candidate written to that scoped temp path.
3. fs::rename to the final <name>.tvim path (atomic on both POSIX and
   Windows via std::fs::rename's MoveFileEx/ReplaceFile semantics).
4. The final path is read back (never the in-memory write buffer) and
   digested -- proof of what is actually durable, not what was intended.
5. commit(): durably rewrite the SAME <name>.tvim.txn with phase=Committed
   and the verified digest. Success is reported to the caller only after
   this write completes.
```

`<name>.tvim.txn` is always overwritten in place, one record per index --
not an unbounded transaction log, matching the "at most one in-flight
mutation per stateful tool instance" concurrency model Hub v0 already has.

**Recovery** (`transaction::recover`, reachable via the
`vector.index.recover` operation on any `smc hub invoke`/`session` call --
no dedicated CLI subcommand) inspects a `phase=Intent` record left by an
interrupted mutation and resolves it without guessing:

```text
candidate file still present   -> the rename never happened. Remove the
                                   abandoned candidate; finalize the
                                   record against whatever the final
                                   artifact already durably holds (its
                                   pre-transaction state).
candidate gone, final present  -> fs::rename's atomicity means this state
                                   is only reachable if the rename
                                   completed; the crash window was between
                                   that success and commit()'s own write.
                                   Finalize as committed against the
                                   final artifact's real digest.
neither exists                 -> cannot prove either outcome (e.g. a
                                   create() that crashed before the
                                   candidate write even began). Marked
                                   Indeterminate; the caller must not
                                   treat the index as usable until a
                                   human resolves it. Never silently
                                   reported as success.
```

`TurboVecAdapter::load()` -- the entry point every read and every further
mutation goes through -- refuses to proceed if an index's transaction
record is still `phase=Intent`, returning a `RecoveryRequired`-coded
`HubToolError`. `runtime.rs`'s `hub_fault_from_tool_error` recognizes that
code and surfaces it as the distinct `HubFault::RecoveryRequired` variant
in the reply (see section 10), not the generic `ToolDeclaredFailure`.
`handle_recover` is the only code path that inspects a transaction record
without going through `load()`'s gate -- it would otherwise be gated by
the exact condition it exists to resolve.

Artifact provenance (`HubToolOutcome::artifact`, `HubProvenance::artifact`
-- section 11) is the direct byproduct of step 4 above: every mutating
operation's reply carries the verified digest of what it actually
committed, not a value derived from the request payload or the in-memory
write buffer.

## 21. Related documents

```text
docs/architecture/dependency_boundary_rules.md
docs/architecture/module_ownership_map.md
docs/architecture/svm_verified_execution_core.md
docs/spec/hub/hub_session_v0.md
```

Issue #1526 is the architecture-track issue this document is written under
and, as of this v0-completion pass, closes against its own full acceptance
criteria. Issue #1553 is the original implementation issue (#1554, #1555)
this document also closes. Issue #1373 is the extension/plugin boundary
issue, related but non-duplicative -- see section 17. Issue #1372 proposed
the `.semantic/hub/` project-local storage convention this implementation
follows. Issue #1374 tracks the future cryptographic signing chain that Hub
audit digests do not yet provide.
