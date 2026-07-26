# Semantic Hub v0 Architecture

Status: Implemented (v0)
Track: Hub / execution boundary
Purpose: document the Semantic Hub v0 architecture -- a governed execution boundary for external computational tools -- closing issue #1553 and consuming architecture issue #1526

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
4. already-cancelled check
5. registry lookup
     unknown tool_id                 -> UnknownTool
     unknown operation on known tool -> UnknownOperation (distinct fault)
6. worker lifecycle gate
     Disabled / Quarantined / Stopped states reject before dispatch
7. capability check
     HubCapabilitySet::satisfies(): every required capability must be
     both present in the grant AND non-sensitive
8. resource budget check
     requested HubResourceBudget must be within BOTH the tool's declared
     resource_ceiling AND the global HubResourceBudget::V0_CEILING,
     checked via first_violation(), which returns the real violating
     dimension/limit/attempted value, never a placeholder
9. queue/concurrency admission
     request's declared queue_depth/concurrent_requests checked
     against ambient counters
```

Steps 3 and 5 are ordered deliberately: payload size is bounded before the
registry is consulted, so an oversized request cannot be used to probe which
tool/operation names exist.

### 6.1 Capability check is structural, not just unfilled

Step 7 is not "nobody has granted these yet." Nine capabilities are marked
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
`resource_ceiling` and this global ceiling (admission step 8); the caller can
only ask for less, never more, than either.

## 10. Fault taxonomy

`HubFault` (`fault.rs`) has 21 variants, each with a stable `.code()` string
independent of its human-readable message:

```text
UnknownTool          UnknownOperation      ApiVersionUnsupported
SchemaVersionUnsupported  DescriptorIncompatible  InputRejected
CapabilityDenied     PrivacyDenied         ResourceBudgetInvalid
QueueFull            ToolDisabled          ToolQuarantined
DeadlineExceeded     Cancelled             ResourceExhausted
ToolDeclaredFailure  WorkerPanicked        ProtocolViolation
OutputRejected       AuditProvenanceFailure  InternalHubFault
```

The stable code string is what appears in the audit record's `fault_code`
field and in the CLI's non-zero-exit error string; the human-readable
message is not parsed by anything and may change freely.

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
`format_version` (`u32`, currently 1), a declared record count (not inferred
from EOF), then one tab-delimited, field-escaped line per record (20 fields
per record). Reload performs strict round-trip validation and enforces that
sequence numbers are monotonically non-decreasing; a corrupt or hand-edited
log fails to load rather than silently accepting a gap or reordering.

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
`vector.index.reset`.

Persistence is implicit -- there is no separate save/load verb. Every
mutating operation loads the index from a scoped `.tvim` file, mutates it,
and atomically rewrites it (temp file + rename); read operations load the
current file fresh on every call. This follows from the process model: the
Hub CLI is one short-lived process per `smc hub invoke` call, with no
long-running Hub daemon holding state in memory across invocations in v0 --
the on-disk file is the source of truth between invocations.

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
smc hub audit --request <request-id>
```

`smc hub tools` lists registered tools in deterministic order (registry
iteration order, section 12).

Persistent state lives under `.semantic/hub/` relative to the current
working directory, matching the project-local storage convention proposed by
issue #1372:

```text
.semantic/hub/
  vector.turbovec/<name>.tvim   one file per persisted index
  audit.log                     whole canonical audit trail,
                                 rewritten atomically each invocation
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

## 19. Related documents

```text
docs/architecture/dependency_boundary_rules.md
docs/architecture/module_ownership_map.md
docs/architecture/svm_verified_execution_core.md
```

Issue #1526 is the architecture-track issue this document is written under.
Issue #1553 is the implementation issue this document closes. Issue #1372
proposed the `.semantic/hub/` project-local storage convention this
implementation follows. Issue #1374 tracks the future cryptographic signing
chain that Hub audit digests do not yet provide.
