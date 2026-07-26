# Semantic Hub API v0 Specification

Status: draft v0
Owner crate: `semantic-hub`

This document defines the generic Semantic Hub v0 contract: identifiers,
versioning rules, the capability and classification model, the resource
budget, the descriptor shape, the request/reply envelope, the fault
taxonomy, the admission order, and the canonical audit encoding that every
Hub tool adapter is admitted and dispatched against. `semantic-hub` depends
on nothing beyond `std` (its `Cargo.toml` `[dependencies]` table is empty)
and has no path dependency on any other in-repo crate; this contract is the
whole of what it promises.

A Hub reply's payload is untrusted computational evidence, never Semantic
truth by itself.

## 1. Ownership

`semantic-hub` owns tool/operation identity and version syntax; the Hub API
version and envelope schema version; the capability and classification
vocabularies used by requests, replies, descriptors, and audit records; the
resource budget model and its v0 ceiling; the request/reply envelope shape;
the fault taxonomy and reply status model; the admission order; and the
canonical audit text encoding.

`semantic-hub` does not own Semantic language meaning, verifier admission,
or VM execution; any one tool's business logic (that is the adapter's --
see `hub_adapter_contract_v0.md`); or a cryptographic signing/provenance
chain (tracked separately by issue #1374 -- Hub v0 provenance is a bounded
correlation fingerprint, not a security commitment).

## 2. Identifiers

Five newtypes and two version types give the contract stable, validated
names. All five newtypes implement `Display`, `FromStr`, `.as_str()`, and
deterministic `Ord`/`Hash` (`BTreeMap` keys, audit sort/correlation keys).

### 2.1 Dotted identifiers

`HubToolId` and `HubOperationId` use "dotted" syntax, max length
`MAX_DOTTED_ID_LEN = 128` bytes:

- non-empty
- <= 128 bytes
- no leading or trailing `.`
- segments separated by `.`; no segment may be empty
- each segment: ASCII lowercase letters, ASCII digits, `_`, `-` only

Example valid values: `vector.turbovec`, `vector.search.filtered`.

`IdSyntaxError` variants: `Empty`, `TooLong { max, actual }`,
`EmptySegment`, `InvalidCharacter(char)`, `LeadingOrTrailingDot`.

### 2.2 Handle identifiers

`HubRequestId`, `HubSessionId`, `HubCallerIdentity` use a looser "handle"
syntax, max length `MAX_HANDLE_LEN = 128` bytes:

- non-empty
- <= 128 bytes
- ASCII alphanumeric, `_`, `-`, `:`, `.` only (no dot-position or
  dot-segment rules -- `req:local:0001` is valid)

Handle identifiers share `IdSyntaxError` with dotted identifiers.

### 2.3 Version types

`HubToolVersion { major, minor, patch }` (each `u32`): `Display` renders
`"major.minor.patch"`; `FromStr` requires exactly three dot-separated
unsigned-integer parts and rejects any other count (`"1.2"` and `"1.2.3.4"`
are both rejected).

`HubApiVersion { major, minor }` (each `u32`): the version of the Hub
contract itself -- registry, admission, dispatch -- independent of any one
tool's `HubToolVersion`. `CURRENT = HubApiVersion::new(0, 1)`.

## 3. Versioning and compatibility

Two independent version numbers exist and must not be conflated:

- `HubApiVersion` -- the registry/admission/dispatch contract version
- `HUB_ENVELOPE_SCHEMA_VERSION: u32 = 1` -- the `HubRequest`/`HubReply`
  wire-shape version, bumped only when the envelope's own field set
  changes

`HubApiVersion::is_compatible_with(other)` is
`self.major == other.major && other.minor <= self.minor`: majors must match
exactly, and `other`'s minor may be less than or equal to `self`'s -- a
newer minor is rejected outright, never silently downgraded or upgraded.
This one rule governs two independent checks: admission
(`HubApiVersion::CURRENT.is_compatible_with(request.api_version)`, else
`HubFault::ApiVersionUnsupported`) and descriptor validation
(`running_hub_api.is_compatible_with(descriptor.hub_api_version)`, else
`DescriptorError::ApiVersionIncompatible { tool, hub }`).

`request.schema_version` is checked separately and exactly against
`HUB_ENVELOPE_SCHEMA_VERSION`, else `HubFault::SchemaVersionUnsupported` --
there is no minor-version leniency on the envelope schema.

## 4. Capability and classification model

`HubCapability` is a 20-variant enum split into two closed classes.
`as_str()`/`parse()` round-trip for every variant; `parse()` returns `None`
for any unrecognized name rather than guessing.

Non-sensitive (grantable; allowed by default once granted):

```text
VectorIndexCreate      VectorSearch           PrivateStorageRead
VectorIndexRead        VectorFilteredSearch   PrivateStorageWrite
VectorIndexMutate      VectorIndexPersist     ClockMonotonic
CpuCompute             MemoryAllocateBounded
```

Sensitive (`is_sensitive() == true`; never satisfiable via
`HubCapabilitySet::satisfies`, no matter what is granted -- denial is
structural, not a policy toggle):

```text
NetworkAccess              DeviceAccess          ProjectMutation
ArbitraryFilesystemRead    EnvironmentRead       SemanticStateMutation
ArbitraryFilesystemWrite   SecretRead
ProcessSpawn
```

`HubCapabilitySet` is a `BTreeSet<HubCapability>`-backed, schema-versioned
(`HubCapabilitySetVersion::V1`) closed set:

- `.grant(cap)` -- builder; a caller may grant itself a sensitive
  capability (it is recorded and auditable), but this does not make it
  usable
- `.allows(cap)` -- membership test only
- `.satisfies(required: &[HubCapability]) -> bool` -- true only if every
  required capability is present **and** non-sensitive; a required
  capability that is sensitive makes `satisfies` return `false`
  unconditionally, even if it happens to be present in the set

Deny-by-default is the only default: an empty `HubCapabilitySet` allows
nothing.

Four classification enums are attached to descriptors, requests, replies,
and audit records:

- `HubExecutionMode`: `InProcess` (the only mode Hub v0 implements),
  `Subprocess`, `Wasm`, `Remote` (reserved -- exist so the contract does
  not change shape when they are implemented)
- `HubDeterminismClass`: `Deterministic`, `DeterministicWithSeed`,
  `EnvironmentDependent`, `Unknown` -- assigned per operation from measured
  adapter evidence, never inferred from an operation's name
- `HubTrustClass`: `InProcessUnisolated` (the only value Hub v0 uses;
  explicitly does not claim memory-corruption isolation), `ProcessIsolated`,
  `SandboxIsolated` (reserved)
- `HubPrivacyClass`: `PublicSafe`, `ProjectLocal`, `PrivateSource`,
  `OrganizationPrivate`, `SecretSuspected`. `.exportable_by_default()` is
  `true` only for `PublicSafe`. `parse()`/`as_str()` round-trip for all
  five.

## 5. Resource budget and limits

`HubResourceKind` -- 12 budgeted dimensions:

```text
WallTimeMillis   OutputBytes        ResultCount           StorageReadBytes
MemoryBytes      IndexItemCount     QueueDepth            StorageWriteBytes
InputBytes       VectorDimensions   ConcurrentRequests    AuditBytes
```

`.is_hard_enforced_v0()` is `true` for `WallTimeMillis`, `InputBytes`,
`OutputBytes`, `IndexItemCount`, `VectorDimensions`, `ResultCount`,
`QueueDepth`, `ConcurrentRequests`; it is `false` (advisory/observed-only)
for `MemoryBytes`, `StorageReadBytes`, `StorageWriteBytes`, `AuditBytes` --
stated explicitly rather than implied, because in-process Rust cannot
precisely enforce a memory limit without OS-level containment.

`HubResourceBudget` -- one admitted invocation's immutable budget (12
fields; `wall_time_millis`/`memory_bytes`/`input_bytes`/`output_bytes`/
`index_item_count`/`storage_read_bytes`/`storage_write_bytes`/
`audit_bytes` are `u64`, `vector_dimensions`/`result_count`/`queue_depth`/
`concurrent_requests` are `u32`).

`V0_CEILING` (exact values):

```text
wall_time_millis        30000            (30 s)
memory_bytes            536870912        (512 MiB, advisory)
input_bytes             67108864         (64 MiB)
output_bytes            16777216         (16 MiB)
index_item_count        1000000
vector_dimensions       4096
result_count            10000
queue_depth             256
concurrent_requests     32
storage_read_bytes      268435456        (256 MiB, advisory)
storage_write_bytes     268435456        (256 MiB, advisory)
audit_bytes             1048576          (1 MiB, advisory)
```

`.within(ceiling)` / `.first_violation(ceiling) -> Option<HubBudgetExceeded>`
walks the 12 fields in the fixed order above and returns the **first**
dimension on which the requested budget exceeds `ceiling`, carrying the
real `limit` and `attempted` values -- never a placeholder.

`HubResourceUsage` mirrors 9 of the numeric dimensions as `Option<u64>` /
`Option<u32>` (`wall_time_millis`, `peak_memory_bytes`, `input_bytes`,
`output_bytes`, `index_item_count`, `result_count`, `storage_read_bytes`,
`storage_write_bytes`, `audit_bytes` -- no usage field exists for
`vector_dimensions`, `queue_depth`, or `concurrent_requests`, which are
admission-time-only). `None` means "not measured"; never a fabricated zero.
`check_budget(kind, used, delta, limit) -> Result<u64, HubBudgetExceeded>`
uses `checked_add`; an overflowing `used + delta` is itself an excess
(`attempted: u64::MAX`), never wrapped.

Other bounds:

- `MAX_DOTTED_ID_LEN = 128` (bytes; `HubToolId`, `HubOperationId`)
- `MAX_HANDLE_LEN = 128` (bytes; `HubRequestId`, `HubSessionId`,
  `HubCallerIdentity`)
- `MAX_PAYLOAD_BYTES = 33554432` (32 MiB; `HubRequest.payload`, checked
  before registry lookup)

## 6. Descriptors

`HubOperationDescriptor { operation_id, required_capabilities: BTreeSet<HubCapability>, determinism, mutates_tool_state: bool }`

`HubToolDescriptor { tool_id, name, tool_version, hub_api_version, execution_mode, trust_class, operations: Vec<HubOperationDescriptor>, resource_ceiling, adapter_provenance: String }`

`.validate(running_hub_api)` runs, in order, over the whole descriptor:

1. `operations` non-empty, else `DescriptorError::NoOperations`
2. for each operation in declared order: its `operation_id` has not been
   seen before, else `DescriptorError::DuplicateOperation(id)`; then each
   of its `required_capabilities` is non-sensitive, else
   `DescriptorError::OperationRequiresSensitiveCapability { operation, capability }`
3. `running_hub_api.is_compatible_with(descriptor.hub_api_version)`, else
   `DescriptorError::ApiVersionIncompatible { tool, hub }`

A descriptor that asks for a sensitive capability on any operation is
rejected at registration time -- such an operation could never be admitted
anyway (Section 4), so the registry refuses to hold a tool nothing could
ever invoke.

## 7. Envelope: request and reply

`HubRequest` fields (`payload` is checked against `MAX_PAYLOAD_BYTES`,
Section 5, before any registry lookup runs):

```text
schema_version       u32                 must equal HUB_ENVELOPE_SCHEMA_VERSION
api_version          HubApiVersion       checked via is_compatible_with
request_id           HubRequestId        correlation handle
session_id           HubSessionId        correlation handle
caller_identity      HubCallerIdentity   who issued the request
tool_id              HubToolId           target tool
operation_id         HubOperationId      target operation on that tool
capability_context   HubCapabilitySet    capabilities the caller grants itself
privacy_class        HubPrivacyClass     governs export eligibility
resource_budget      HubResourceBudget   requested per-invocation budget
payload              Vec<u8>             opaque to the Hub; interpreted only by the tool
```

`HubReply` fields:

```text
schema_version    u32
request_id        HubRequestId     echoes the request
tool_id           HubToolId
tool_version      HubToolVersion   the real registered tool version, even on rejection
operation_id      HubOperationId
status            HubReplyStatus
payload           Vec<u8>          untrusted computational evidence
resource_usage    HubResourceUsage
```

`tool_version` and `adapter_provenance` (the latter carried into the audit
record, Section 10) come from the actual registered `HubToolDescriptor`
whenever `tool_id` is recognized, even for a pre-dispatch rejection (e.g. a
capability denial) -- only a genuinely unknown `tool_id` falls back to an
internal placeholder descriptor (`tool_version 0.0.0`). An earlier defect
always used the placeholder, so a known tool's capability-denial rejection
misreported `0.0.0`; found via dogfooding and fixed, pinned by
`capability_denial_records_the_real_tool_version_not_a_placeholder`.

`HubReplyStatus`:

- `Success` -- no fault
- `Rejected(HubFault)` -- admission refused the request before dispatch
- `ToolFailed(HubFault)` -- the tool ran and declared failure, or a
  post-dispatch structural check failed
- `Crashed(HubFault)` -- the worker panicked
- `HubFault(HubFault)` -- a fault in the Hub's own logic, not attributable
  to caller input or the tool

`.fault() -> Option<&HubFault>` (`None` only for `Success`), `.is_success()`,
`.as_str()` (`"Success"`, `"Rejected"`, `"ToolFailed"`, `"Crashed"`,
`"HubFault"` -- these five strings are also the only `status_code` values
the canonical audit decoder recognizes; see Section 10).

## 8. Fault taxonomy

`HubFault` -- 21 variants. Every admitted-or-rejected invocation ends in
exactly one `HubReplyStatus` carrying at most one `HubFault`. `.code()`
returns a stable machine-readable string independent of the variant's
carried message text.

Pre-dispatch rejections (`.is_pre_dispatch_rejection() == true`; the
request never reaches the tool):

```text
UnknownTool                tool_id not present in the registry
UnknownOperation           operation_id not declared on that tool's descriptor
ApiVersionUnsupported      request.api_version incompatible with HubApiVersion::CURRENT
SchemaVersionUnsupported   request.schema_version != HUB_ENVELOPE_SCHEMA_VERSION
DescriptorIncompatible     reserved for descriptor-shape rejection at admission time
InputRejected(String)      payload or other input failed a structural/bounds check
CapabilityDenied(String)   capability_context does not satisfy required_capabilities
PrivacyDenied(String)      reserved for privacy-policy admission rejection
ResourceBudgetInvalid(HubBudgetExceeded)   requested budget exceeds a ceiling
QueueFull                  ambient queue depth or concurrency at/over the request's own budget
ToolDisabled               worker state is Disabled or Stopped
ToolQuarantined            worker state is Quarantined
```

Post-dispatch outcomes (the tool was invoked, or dispatch itself failed):

```text
DeadlineExceeded             the deadline had already passed at invoke() entry
Cancelled                    AdmissionAmbient.already_cancelled was true
ResourceExhausted(HubBudgetExceeded)   a live usage measurement exceeded budget
ToolDeclaredFailure(String)  the tool ran and returned Err(HubToolError) itself
WorkerPanicked(String)       the worker's handle() panicked (contained, not propagated)
ProtocolViolation(String)    validate_reply() rejected the tool's own reply shape
OutputRejected(String)       reply payload exceeded resource_budget.output_bytes
AuditProvenanceFailure(String)   recording audit/provenance evidence itself failed
InternalHubFault(String)     a fault in Hub admission/dispatch/registry logic
```

`DescriptorIncompatible` and `PrivacyDenied` are declared and classified as
pre-dispatch, but are not constructed anywhere by the current v0 `admit()`
path -- reserved so the taxonomy need not change shape when descriptor- or
privacy-policy-level admission checks are added.

## 9. Admission order

`admit(registry, request, ambient: AdmissionAmbient) -> Result<AdmittedInvocation, HubFault>`
runs every check in this fixed order and stops at the first failure:

```text
 1. HubApiVersion::CURRENT.is_compatible_with(request.api_version)   else ApiVersionUnsupported
 2. request.schema_version == HUB_ENVELOPE_SCHEMA_VERSION            else SchemaVersionUnsupported
 3. request.payload.len() <= MAX_PAYLOAD_BYTES                       else InputRejected
 4. !ambient.already_cancelled                                       else Cancelled
 5. registry lookup by tool_id                                       else UnknownTool
 6. operation lookup on that tool's descriptor                       else UnknownOperation
 7. worker state not Disabled/Stopped (-> ToolDisabled)
    and not Quarantined (-> ToolQuarantined)
 8. capability_context.satisfies(required_capabilities)              else CapabilityDenied(missing list)
 9. resource_budget.first_violation(tool.resource_ceiling)
    then .first_violation(V0_CEILING)                                else ResourceBudgetInvalid(violation)
10. ambient.current_queue_depth < request.resource_budget.queue_depth
    and ambient.current_concurrent_requests < request.resource_budget.concurrent_requests
                                                                       else QueueFull
```

`AdmissionAmbient { current_queue_depth, current_concurrent_requests, already_cancelled }`
is state admission needs beyond the registry and the request itself. The
in-process `Hub::invoke` always passes `current_queue_depth: 0` and
`already_cancelled: false` -- no queue is implemented in v0 and no live
cancellation signal is wired in; only `current_concurrent_requests` (the
Hub's in-flight counter) is a real measurement. The struct's shape does not
need to change when a real queue or cancellation source is added later.

On success, `AdmittedInvocation { tool, operation }` is the only way
dispatch may proceed; no other path may call an adapter's `handle()`.

## 10. Canonical audit encoding

`HubAuditTrail::to_canonical_text()` / `::from_canonical_text()` produce and
parse a versioned, tab-delimited text format with explicit escaping and
strict round-trip validation -- the same convention `prom-audit` and
`prom-state` use, chosen over serde/JSON because this wire format must stay
byte-stable across releases.

`HUB_AUDIT_FORMAT_VERSION: u32 = 1`. Magic header: `"semantic-hub.audit.v1"`.

Trail-level layout (three header lines, then one line per record):

```text
semantic-hub.audit.v1
<format_version>
<record_count>
<record 0>
<record 1>
...
```

`from_canonical_text` rejects: a first line other than the exact magic
string (`MissingMagicHeader`); a format version other than
`HUB_AUDIT_FORMAT_VERSION` (`UnsupportedFormatVersion`); a declared record
count that does not match the number of record lines actually present
(`FieldCount` -- the same variant used below for a single record's field
count, reused here for the header's count).

Each record line is tab-delimited, exactly `FIELD_COUNT = 20` fields, and
capped at `MAX_AUDIT_RECORD_BYTES = 8192` bytes (checked before any field is
parsed, else `RecordTooLarge`). Field order:

```text
 1. sequence               u64, strictly increasing across the trail
 2. request_id             escaped
 3. session_id             escaped
 4. caller_identity        escaped
 5. tool_id                escaped
 6. tool_version           "major.minor.patch"
 7. adapter_provenance     escaped, free text
 8. operation_id           escaped
 9. execution_mode         Display name (InProcess/Subprocess/Wasm/Remote)
10. determinism            Display name
11. trust_class            Display name
12. privacy_class          Display name
13. capabilities_granted   escaped, ";"-joined capability names
14. input_digest           "fnv1a64:<16 lowercase hex>:<byte_len>"
15. output_digest          same shape as input_digest
16. resource_budget        12 ","-joined values, same field order as Section 5
17. resource_usage         9 ","-joined values ("-" for None), same field order as HubResourceUsage
18. worker_state_after     Display name of one of the 9 HubWorkerState values
19. status_code            one of "Success"/"Rejected"/"ToolFailed"/"Crashed"/"HubFault"
20. fault_code             one of the 21 fault codes in Section 8, or "-" for None
```

Escaping applies only to fields whose content is not a closed/controlled
vocabulary (`request_id`, `session_id`, `caller_identity`, `tool_id`,
`adapter_provenance`, `operation_id`, the joined `capabilities_granted`
string): `\` -> `\\`, tab -> `\t`, newline -> `\n`. Every other field is
rendered from a fixed `Display` implementation that cannot itself contain a
tab, newline, or backslash, so it is written and read back unescaped.
`capabilities_granted` decodes only the eleven non-sensitive capability
names (Section 4); the decoder's name table does not include the nine
sensitive names.

Sequence numbers must strictly increase line-to-line (`sequence <=
previous_sequence` is rejected as `NonMonotonicSequence`; a freshly parsed
trail's first record has no such constraint). `HubAuditTrail::next_sequence()`
returns `0` for an empty trail, `last.sequence + 1` otherwise;
`Hub::seed_next_sequence(n)` lets an embedder (the CLI starts a fresh,
empty in-memory `Hub` every process invocation) resume a persisted log's
numbering instead of restarting at 0 each run.

`status_code` and `fault_code` are two distinct record fields. An earlier
defect set both to the fault's specific code (e.g. both `"CapabilityDenied"`
instead of `status_code = "Rejected"`, `fault_code = "CapabilityDenied"`),
which broke `from_canonical_text` on reload because its decoder recognizes
only the five `HubReplyStatus` names for `status_code`. A regression test
(`rejection_audit_record_status_code_and_fault_code_are_distinct_and_round_trip`)
pins the fix: `status_code` is always the reply-status discriminant,
`fault_code` the specific fault code or `"-"`.

## 11. Unknown-field and unknown-version behavior

An unsupported `api_version` or `schema_version` is a hard rejection
(`ApiVersionUnsupported` / `SchemaVersionUnsupported`), never a silent
ignore, a best-effort parse, or an implicit downgrade. An unsupported audit
`format_version` is likewise a hard `UnsupportedFormatVersion` parse error,
not a lenient fallback to a best-guess layout. There is no partial or
forward-compatible parsing path anywhere in this contract: a request,
descriptor, or audit record either matches the declared shape and version
exactly, or it is rejected with a specific, typed reason.
