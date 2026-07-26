# Semantic Hub v0 Threat Model

Status: Implemented (v0)
Track: Hub / execution boundary security
Purpose: document the threat model for Semantic Hub v0 -- the governed
execution boundary between the `smc` CLI and the `vector.turbovec` tool --
closing part of issue #1553, consuming issue #1371 (Semantic threat model and
untrusted project policy) and issue #1372 (Studio and ALM local data
governance) as design constraints.

This document describes one component's threat model. It is not the
repository-wide threat model. The repository-wide track is `#1371`, and its
own planned document is `docs/security/threat_model_v0.md` (not yet written
as of this document). Nothing here should be read as satisfying that broader
issue; this document only covers Semantic Hub v0.

## 1. Non-claims

State plainly, before anything else, what this document and the system it
describes do not claim:

```text
this is not an independent third-party security audit
this is not a sandbox
this does not provide memory-corruption isolation
this does not independently vet the turbovec dependency's internals
  line-by-line; the pinned crate is trusted as a supply-chain dependency,
  not re-audited as part of this work
this does not provide cryptographic tamper-evidence for the audit log
this does not implement subprocess, WASM, or remote execution
this does not implement encryption-at-rest or file permission hardening
  for `.semantic/hub/`
```

Every mitigation described below is a real, tested control in the shipped
code. None of them should be read as broader than stated.

## 2. Trust zones

Semantic Hub v0 has exactly one execution trust class:
`HubTrustClass::InProcessUnisolated`.

```text
+-----------------------------------------------------+
| OS process: one `smc hub invoke` invocation          |
|                                                       |
|   +-----------------+     +------------------------+ |
|   | Hub runtime      | -> | vector.turbovec adapter | |
|   | (admission,      |    | (in-process Rust code,  | |
|   |  registry,       |    |  same address space)    | |
|   |  worker          |    |                          | |
|   |  supervision,    |    +------------------------+ |
|   |  audit)          |                                |
|   +-----------------+                                 |
+-----------------------------------------------------+
```

There is no sub-boundary between the Hub runtime and the TurboVec adapter in
v0. Both run as ordinary Rust code in the same OS process, the same address
space, and the same privilege level as the CLI itself. `catch_unwind` at the
dispatch boundary (section 7.8) contains Rust panics; it does not create a
security boundary between the two halves of this process.

`HubExecutionMode::Subprocess`, `::Wasm`, and `::Remote` exist as enum
variants so the request/reply contract will not need to change shape when a
real isolation boundary is added later. No code path for any of them exists
today. A future execution mode that introduces a real trust boundary is the
trigger for revising this document (section 11).

The Hub CLI process itself -- the OS process boundary around the whole
`smc hub invoke` invocation -- is the only trust boundary that exists in v0.
Everything inside it (Hub runtime and TurboVec adapter alike) is one
undifferentiated trust zone.

## 3. Assets to protect

```text
- the tool registry's integrity
    (which tools/operations are registered, and their declared
    capabilities/budgets, must not be corrupted or silently overwritten)
- the audit log's integrity
    (every admitted-or-rejected invocation must produce exactly one
    record, and a persisted log must not silently accept corruption)
- the resource budget invariants
    (a caller must never be able to obtain a budget wider than a tool's
    declared ceiling or the global V0_CEILING)
- the capability deny-by-default guarantee
    (sensitive capabilities must never become satisfiable, regardless of
    what a request or a future tool registration asks for)
- the scoped storage directory's confinement
    (`.semantic/hub/vector.turbovec/<name>.tvim` must never resolve
    outside its scoped root)
```

## 4. Attacker capabilities assumed

The assumed attacker is a local user who can:

```text
- construct arbitrary `smc hub invoke <tool-id> <operation-id>
  --input <file>` JSON request files, including malformed, oversized,
  or adversarially crafted ones
- choose arbitrary index names, vector coordinates, external ids, and
  allowed-id filter lists within whatever the payload schema accepts
- invoke the CLI repeatedly, including concurrently, and observe exit
  codes, stdout/stderr, and the audit log
```

The attacker is NOT assumed to have direct filesystem access to
`.semantic/hub/` that bypasses the CLI (for example, hand-editing
`audit.log` or a `.tvim` file directly). If they did, nothing in v0
currently stops them: there is no encryption-at-rest and no file
permission hardening applied to `.semantic/hub/` beyond whatever the
OS default permissions on the project directory already provide. The
audit trail's `from_canonical_text` round-trip validation (section 7.10)
would detect and reject a hand-edited `audit.log` that breaks its
format invariants, but it cannot detect a forged record that is
internally well-formed.

The attacker is not assumed to have network access to the process (there is
no network-facing surface to attack) and is not assumed to be able to modify
the Hub or TurboVec source code or the compiled binary.

## 5. Untrusted inputs

```text
- the request file passed via `--input <file>`
    (tool_id, operation_id, api_version, schema_version, privacy_class,
    capabilities, resource_budget, and the tool-specific payload object)
- vector and id payloads within that request
    (vectors: [[f32]], ids: [u64], for insert/search operations)
- allowed_ids filter lists for `vector.search.filtered`
- index names supplied as part of the payload
```

All of the above are treated as fully attacker-controlled. None of them are
trusted by construction; each is validated at a specific admission or
adapter-level step before it can reach code that assumes well-formed input.

## 6. Admission and validation boundaries

The exact order of admission checks, and the exact fault produced by each
step, is owned by `crates/semantic-hub/src/admission.rs::admit()` and is
documented in full in `docs/architecture/semantic_hub_v0.md`, section 6. This
document does not repeat that ordering; it references it as the canonical
source. In summary, admission runs (in order) API-version compatibility,
envelope schema-version check, the 32 MiB payload size bound, an
already-cancelled check, registry lookup, worker lifecycle gate, capability
check, resource budget check, and queue/concurrency admission -- each
producing a distinct, typed `HubFault` rather than a generic rejection.

Below that admission layer, the TurboVec adapter performs its own
domain-specific validation (vector coordinate checks, dimension checks,
index name charset checks) before calling into `turbovec`. Those checks are
described per-threat in section 7.

## 7. Threat categories, and their real mitigations

Each subsection below names one considered threat and the actual code-level
mitigation for it. All of these were verified live through the built `smc`
binary and/or dedicated tests, not assumed from design intent alone.

### 7.1 Malicious or oversized request payload

Mitigation: the request payload is bounded to `MAX_PAYLOAD_BYTES = 32 MiB`
and this bound is checked before any parsing is attempted, and before the
registry is even consulted (admission step 3, ahead of step 5). An oversized
payload cannot be used to probe which tool/operation names exist, because
the size check runs first regardless of what the payload contains.

### 7.2 Malformed, truncated, or trailing-garbage JSON

Mitigation: parsing goes through `serde_json`, which rejects malformed JSON,
truncated JSON, and trailing garbage after a valid JSON value by
construction. There is no hand-rolled parser in this path.

### 7.3 Invalid vector values (NaN, Infinity, huge magnitude)

Mitigation: NaN, positive/negative infinity, and huge-magnitude coordinates
(magnitude >= 1e16) are pre-validated by the TurboVec adapter using
turbovec's own exported `first_invalid_coord` check -- reused directly from
the dependency rather than reimplemented -- before any turbovec function
that would otherwise panic on such input is called. This has been verified
live and by dedicated tests to produce a typed rejection, not a crash.

### 7.4 Dimension mismatch

Mitigation: a vector whose length does not match an index's declared
dimension is rejected with a typed error before any turbovec call that
assumes matching dimensions.

### 7.5 Path traversal via a crafted index name

Mitigation: index names are restricted to a fixed charset -- lowercase ASCII
alphanumeric plus `_`/`-` only, 1-64 bytes -- that structurally excludes
`.`, `/`, and `\`. This makes path traversal impossible by construction,
not merely checked for and rejected: there is no character sequence an
attacker can supply in an index name that resolves outside the scoped
storage root, because the derived file path is always
`<fixed_scoped_root>/<name>.tvim` and `<name>` cannot contain a path
separator or a `..` sequence.

### 7.6 Unbounded index proliferation

Mitigation: `MAX_INDEX_COUNT = 256` persisted indexes per scoped directory.
An attacker who repeatedly creates new indexes cannot grow the persisted
index count without bound.

### 7.7 Sensitive capability escalation

Mitigation: this is enforced at two independent layers, not one.

```text
- request-admission layer: HubCapabilitySet::satisfies() treats the nine
  sensitive capabilities (NetworkAccess, ArbitraryFilesystemRead,
  ArbitraryFilesystemWrite, ProcessSpawn, DeviceAccess, EnvironmentRead,
  SecretRead, ProjectMutation, SemanticStateMutation) as NEVER
  satisfiable, regardless of what a request's capabilities array grants.
  is_sensitive() short-circuits satisfies() to false for these -- this is
  checked in code, not merely "nobody happens to have granted these yet."
- tool-registration layer: HubToolDescriptor::validate() refuses to even
  register a tool descriptor whose declared operations require one of the
  nine sensitive capabilities. A future tool cannot be admitted into the
  registry at all while asking for one of these, independent of whether a
  caller ever requests it.
```

A request granting zero capabilities is denied everything it needs, not
granted anything by default. This was verified live: an empty
`capabilities: []` search request was rejected with
`CapabilityDenied: missing or denied capabilities: VectorSearch,
PrivateStorageRead`.

The eleven non-sensitive capabilities (`VectorIndexCreate/Read/Mutate`,
`VectorSearch`, `VectorFilteredSearch`, `VectorIndexPersist`, `CpuCompute`,
`MemoryAllocateBounded`, `PrivateStorageRead/Write`, `ClockMonotonic`) still
require an explicit grant per request -- the CLI request-file schema
requires a `capabilities` array and has no auto-grant-everything default.

### 7.8 Worker panic

Mitigation: every adapter call is wrapped in
`std::panic::catch_unwind(std::panic::AssertUnwindSafe(...))`. A caught
panic is converted into a `HubFault::WorkerPanicked` outcome and a
worker-health "crash" transition, escalating to `Quarantined` after three
crashes (or immediately, under a "never restart" policy). One bad request
cannot take down the whole CLI process or corrupt the tool registry.

This is explicitly not memory-safety isolation (see section 1). It does
nothing against undefined behavior from `unsafe` code -- `semantic-hub`
itself has `#![forbid(unsafe_code)]` and contains no unsafe blocks, but the
`turbovec` dependency is third-party code with its own SIMD kernels and
runtime CPU-feature dispatch that has not been independently
line-by-line audited for memory safety as part of this work. It also does
nothing against a stack overflow (which aborts the process and is
unrecoverable by `catch_unwind`), or against memory corruption that occurs
before a panic is ever raised.

### 7.9 Malformed or non-conforming adapter reply (protocol violation)

Mitigation: a reply that fails `worker.validate_reply()`'s structural check
is classified as `ProtocolViolation`, a fault class distinct from a
tool-declared operation failure. The supervision policy quarantines the
worker at threshold 1 for a protocol violation, stricter than the
threshold of 3 crashes tolerated before quarantine -- a tool that violates
the wire contract is treated as untrustworthy in a way a transient crash is
not, because a non-conforming reply means the Hub can no longer trust
anything else that worker says, including a future well-formed-looking
reply.

### 7.10 Tampered or corrupted audit log on disk

Mitigation: `HubAuditTrail::from_canonical_text` performs strict
round-trip validation on load -- magic header check (`semantic-hub.audit.v1`),
format version check, monotonic sequence check, and a declared-vs-actual
record count check -- and fails loudly with `AuditProvenanceFailure` rather
than silently accepting a corrupted, truncated, reordered, or hand-edited
log as valid evidence.

## 8. Residual in-process risks

These are real, named gaps in v0, not merely theoretical future work:

```text
- memory-corruption isolation does not exist: the TurboVec adapter runs
  in-process and unisolated (HubTrustClass::InProcessUnisolated); a
  memory-safety bug in the dependency chain has the same blast radius as
  a bug in the Hub or CLI's own code
- precise memory-usage enforcement does not exist: memory_bytes is an
  advisory-only budget dimension in v0, and it is not even measured --
  only declared. A malicious or buggy tool could allocate beyond the
  declared "budget" and this would not be hard-stopped; at best it would
  show up as an advisory observation after the fact, and in v0 it would
  not show up at all, because memory usage is not observed
- the turbovec crate itself is a supply-chain trust assumption: it is
  pinned at =0.9.0 and used as published; it has not been independently
  audited byte-by-byte as part of this task
- concurrent multi-process access to the same `.semantic/hub/` directory
  is now serialized by a single whole-project exclusive advisory lock
  (`std::fs::File::lock`, `hub_data_root/hub.lock`), acquired by
  `cmd_hub_invoke` before the first read of `audit.log` and held through
  the final write; this closes the audit-log-record-loss and
  index-update-loss races previously named here. It does not eliminate
  every possible interleaving: it blocks indefinitely (no timeout), so
  heavy concurrent use queues rather than runs in parallel -- acceptable
  for v0's single-user CLI usage pattern, not for a future
  multi-tenant/server deployment. `smc hub audit` (read-only) does not
  take the lock, since atomic rename already guarantees a reader never
  observes a torn file
- cryptographic tamper-evidence of the audit log does not exist: digests
  (HubDigest = FNV-1a-64 + byte length) are non-cryptographic correlation
  fingerprints only, chosen to confirm "this record refers to exactly
  these bytes," not to detect deliberate forgery. A signing/provenance
  chain is tracked separately under issue #1374 and is not implemented
  here
```

## 9. Out-of-scope threats for v0

```text
- subprocess execution threats: HubExecutionMode::Subprocess is an enum
  variant only; no code path exists, so the threat surface (e.g. a
  compromised child process, argument injection into a spawned tool)
  does not exist yet either. It is absent, not mitigated
- WASM execution threats: same reasoning, for HubExecutionMode::Wasm
- remote execution threats: same reasoning, for HubExecutionMode::Remote
- network-based threats: no networking crate or API is referenced
  anywhere in crates/semantic-hub or crates/semantic-hub-turbovec, so no
  network attack surface exists on this component. This is a weaker
  guarantee than an enforced sandbox or firewall -- it holds only because
  there is no networking code, not because network access was attempted
  and blocked
- multi-tenant/server-style concurrent Hub usage: the project-level lock
  described in section 8 makes concurrent CLI invocations safe (no lost
  records/updates) but serializes them into a queue with no fairness,
  priority, or timeout guarantees -- adequate for v0's single-user CLI,
  not a substitute for a real multi-tenant scheduler
```

## 10. Relationship to #1371 / #1372

This document consumes, and does not re-derive or duplicate, the
vocabulary and design constraints proposed by issue #1371 (Semantic threat
model and untrusted project policy) and issue #1372 (Studio and ALM local
data governance). Concretely: Hub v0 uses `HubPrivacyClass`
(`PublicSafe`/`ProjectLocal`/`PrivateSource`/`OrganizationPrivate`/
`SecretSuspected`) as a required field on every request and audit record,
and keeps all Hub state project-local under `.semantic/hub/`, matching the
`<project>/.semantic/local/` storage convention #1372 proposes. Hub v0 does
not implement, override, or attempt to satisfy those issues' broader policy
scope (Studio/ALM/skill-export/consent-level machinery); it only needed the
privacy-classification vocabulary, because it has no learning, export, or
skill features of its own. The repository-wide threat model document that
issue #1371 will eventually produce (`docs/security/threat_model_v0.md`,
referenced from `SECURITY.md`, not yet written as of this document) is the
correct place for cross-component untrusted-project policy; this document
stays scoped to Semantic Hub v0 alone.

## 11. Update policy

This document should be revised when either of the following happens:

```text
- an execution mode beyond InProcess (Subprocess, Wasm, or Remote) is
  actually implemented, since that introduces a real trust boundary
  that section 2 currently describes as nonexistent
- the tool registry gains a second tool, since every threat category in
  section 7 was verified against exactly one adapter (vector.turbovec)
  and a second tool may have different input shapes, different adapter
  invariants, or a different determinism/panic profile worth naming
  explicitly
```

Until either trigger occurs, this document describes the complete v0
threat model: one tool, one trust class, one process, no isolation beyond
panic containment.
