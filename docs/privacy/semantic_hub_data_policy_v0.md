# Semantic Hub v0 Data Policy

Status: draft v0 -- reference implementation on branch `feat/semantic-hub-v0-turbovec-e2e` (PR #1554), not yet landed on `main`
Track: Hub / execution boundary privacy
Purpose: document what Semantic Hub v0 stores, where, and under what privacy
classification -- closing part of issue #1553, consuming issue #1372
(Studio and ALM local data governance) and issue #1371 (Semantic threat
model and untrusted project policy) as design constraints.

This document describes the storage and classification behavior of one
component, Semantic Hub v0. It is not the repository-wide data governance
policy that issue #1372 will eventually define. It consumes that issue's
privacy-class vocabulary as a design constraint and does not re-implement or
claim to satisfy that issue's broader scope.

## 1. Non-claims

```text
no export or redaction pipeline exists in Hub v0 -- there is no "export"
  feature at all in this component
no encryption-at-rest exists for .semantic/hub/
no consent-level UI or consent-gating workflow exists
this document is a classification-and-storage-location policy only, not
  a governance system
no automatic PII/secret detection or scrubbing is performed on payload
  content before it is stored or digested
```

## 2. Privacy classes

Every `HubRequest` carries a required `HubPrivacyClass` field (CLI default:
`"ProjectLocal"`), and it is copied verbatim onto the resulting
`HubAuditRecord`. The Hub never silently upgrades or downgrades this value;
whatever the caller declares is what gets recorded.

```text
PublicSafe            content the caller asserts is safe to treat as
                       public. The only class marked
                       exportable_by_default() -- but nothing in Hub v0
                       actually exports anything (see section 3).

ProjectLocal           the CLI default. Content scoped to the current
                       project, not asserted safe for wider sharing.

PrivateSource          content the caller asserts derives from private
                       source material.

OrganizationPrivate    content the caller asserts is scoped to an
                       organization, narrower than ProjectLocal but not
                       yet SecretSuspected.

SecretSuspected        content the caller flags as possibly containing
                       secret material (credentials, tokens, keys, or
                       similar).
```

Only `PublicSafe.exportable_by_default()` returns `true`; every other class
returns `false`. This distinction is currently inert: Hub v0 has no export
feature, so `exportable_by_default()` is recorded as inspectable metadata
for a future policy layer to consume, not enforced by any export gate here,
because there is nothing to export yet. A future export or ALM-facing
feature that reads Hub state must consult this field rather than assume
all Hub data is safe to move.

Privacy class is caller-declared, not inferred from payload content. The
Hub does not inspect vector coordinates, ids, or index names to decide
whether they look sensitive; it trusts the caller's classification exactly
as given.

## 3. Request payload handling

A `smc hub invoke` request file (JSON, passed via `--input <file>`) contains:

```text
tool_id, operation_id            which tool/operation to invoke
api_version, schema_version      envelope compatibility fields
privacy_class                    HubPrivacyClass (default "ProjectLocal")
capabilities                     explicit array, no default grant
resource_budget                  optional partial override of V0_CEILING
payload                          tool-specific object
                                  (for vector.turbovec: vectors, ids,
                                  index names, filter lists, etc.)
caller_identity, session_id,     free-text strings supplied by the CLI
  request_id                     caller (defaults: "cli:local",
                                  "cli-session", an auto-generated id)
```

`capabilities` has no auto-grant-everything default: a request that omits
capabilities, or supplies an empty array, is denied every capability its
requested operation needs. This was verified live: an empty
`capabilities: []` search request produced `CapabilityDenied: missing or
denied capabilities: VectorSearch, PrivateStorageRead`. There is no implicit
trust extended to a request merely because it was issued locally.

## 4. Audit payload policy

Every admitted-or-rejected invocation produces exactly one
`HubAuditRecord` -- never silently dropped. This holds even for a
capability-denial rejection or a worker panic; both produce a full audit
record, not a partial one.

Audit records store DIGESTS of the input and output payloads
(`HubDigest` = FNV-1a-64 hash + byte length), never the raw payload bytes.
This is structural, not a policy choice that a future code change could
silently bypass: the `HubAuditRecord` type itself has no field capable of
holding raw payload content. There is no configuration flag that would
cause a raw payload to be written to `audit.log`; the type shape makes it
impossible.

Concretely, this means the audit log never contains: raw vector
coordinates, raw external ids, raw index names beyond what appears in
`tool_id`/`operation_id` metadata, or any other content-bearing field from
the request or reply payload. What it does contain is the digest and byte
length of each, plus the structural metadata listed in section 6.

## 5. Digest-versus-content policy

`HubDigest`'s FNV-1a-64 hash is explicitly NOT a cryptographic integrity
guarantee and NOT a tamper-evidence mechanism. It is a fast,
non-cryptographic hash chosen only as a bounded, deterministic correlation
fingerprint -- its job is to let an operator confirm "this audit record
refers to exactly these bytes," not to detect deliberate forgery. An
attacker who can construct arbitrary payloads can also construct an
FNV-1a-64 collision without meaningful difficulty; nothing about this
digest resists deliberate tampering.

No signing or provenance chain exists in Hub v0. That is explicitly the
separate, not-yet-implemented, docs-only issue #1374 ("Semantic artifact
provenance and signing chain"). Semantic Hub v0 does not claim to satisfy
issue #1374; it only records a correlation fingerprint, and this document
should not be read as asserting anything stronger than that.

## 6. TurboVec metadata privacy

`.tvim` index files store exactly what the caller supplies in the insert
payload: `{index, vectors: [[f32]], ids: [u64]}`. Whatever the caller
chooses to put into `vectors` and `ids` is what gets persisted, in
quantized (lossy-compressed, 2-4 bits per coordinate) form for the vector
data and verbatim for the external `u64` ids and their positional mapping.

The Hub does not inject file paths, source code snippets, or any other
contextual metadata into an index automatically. If a caller never puts a
file path or source snippet into `vectors`/`ids`, none appears in the index.
If a caller chooses to encode a file path as part of an id or as a vector
component, that is the caller's choice and outside the Hub's visibility --
the Hub has no schema-level awareness of what an id or vector "means."

## 7. Storage behavior

Persistent Hub state lives under `.semantic/hub/`, relative to the CLI's
current working directory:

```text
.semantic/hub/
  vector.turbovec/
    <name>.tvim        one file per persisted index
  audit.log             whole canonical audit trail, rewritten
                         atomically on every invocation
```

This location matches the `<project>/.semantic/local/` project-local
storage convention proposed by issue #1372. All Hub-owned state stays
inside the project directory; nothing is written outside it, and nothing is
written to a user-global or system-global location.

## 8. Persistence behavior

Persistence is implicit and automatic, not opt-in. Every mutating
TurboVec operation loads the index file, mutates it, and atomically
rewrites it (temp file + rename); every invocation appends to and rewrites
`audit.log`. There is no separate save/load verb and no flag to skip
persistence for a single invocation.

This is a real limitation, stated plainly: a user cannot currently disable
persistence or audit recording for a single `smc hub invoke` call. Every
successful mutation is written to disk, and every invocation -- successful,
rejected, or crashed -- is recorded in the audit log, whether or not the
caller wanted that particular call remembered.

## 9. Retention

There is no automatic expiry, no time-based rotation, and no size cap on
`audit.log` in v0. The only size-related cap in this subsystem is
`MAX_INDEX_COUNT = 256` persisted `.tvim` files per scoped directory, which
bounds index proliferation, not audit log growth or index file age.

Data persists indefinitely until a user manually deletes it. There is no
mechanism in v0 that ages out old audit records or old index files on its
own.

## 10. Deletion

Deleting the `.semantic/hub/` directory manually removes all Hub-owned
state: every `.tvim` index file and the audit log. This is a real,
sufficient way to clear all Hub data today.

There is no CLI command that does this for the user. `smc hub` has no
"clear" or "reset-all" subcommand in v0 (per-index `vector.index.reset`
exists as a TurboVec operation and clears one index's contents, but it does
not delete the audit log or remove the index count from `MAX_INDEX_COUNT`
bookkeeping, and there is no equivalent for the audit log at all). This is
named here as a real gap: manual filesystem deletion is currently the only
way to fully clear Hub state.

## 11. Export

None exists. There is no export feature, no redaction pipeline, and no
code path in Hub v0 that reads Hub state and produces an external-facing
copy of it. `HubPrivacyClass::exportable_by_default()` is recorded as
metadata for a future policy layer (see section 2); it is not consumed by
anything in this component today because there is nothing to export.

## 12. No hidden telemetry

Verified: no networking crate or API is referenced anywhere in
`crates/semantic-hub` or `crates/semantic-hub-turbovec`. There is no
telemetry client, no analytics call, and no background reporting of any
kind in either crate.

## 13. No silent upload

Same basis as section 12: the absence of any networking code in these two
crates means there is no code path capable of uploading request payloads,
vector data, ids, or audit records anywhere. This guarantee is only as
strong as "no networking code exists" -- it is not an enforced sandbox or a
firewall rule, it is the simpler fact that nothing in this component ever
attempts a network call. See the corresponding threat-model discussion in
`docs/security/semantic_hub_threat_model_v0.md`, section 10, for why this is
stated as a weaker guarantee than an enforced block.

## 14. Relationship to #1371 / #1372

This document consumes, and does not re-implement or duplicate, the
privacy-class and storage-location vocabulary proposed by issue #1372
(Studio and ALM local data governance) and the untrusted-project posture
proposed by issue #1371 (Semantic threat model and untrusted project
policy). Concretely: Hub v0 uses `HubPrivacyClass` on every request and
audit record as its only privacy-governance mechanism, and stores all state
under `.semantic/hub/`, matching the `<project>/.semantic/local/`
convention #1372 proposes.

Hub v0 does not implement, override, or claim to satisfy those issues'
broader governance scope -- Studio/ALM consent-level workflows, skill
export policy, or organization-wide data handling rules. It has no
learning, export, or skill-authoring feature of its own in v0, so it only
needed the classification vocabulary those issues define, not their full
governance machinery. When #1371 and #1372 are published as full documents,
this document should be reviewed for consistency with them, but it is not
gated on their publication and does not attempt to preempt their content.
