# Artifact Provenance and Signing Policy v0

Status: current repository-wide governance policy (no runtime behavior)
Issue: #1374
Technical authority: [`../architecture/artifact_identity_and_trust.md`](../architecture/artifact_identity_and_trust.md) (SSF-10)
Companions: [`threat_model_v0.md`](threat_model_v0.md), [`untrusted_project_policy_v0.md`](untrusted_project_policy_v0.md)

## 1. Scope and authority

SSF-10 (`artifact_identity_and_trust.md`) is the canonical technical authority
for artifact SHA-256 identity, producer provenance, verifier ↔ artifact
binding, compatibility policies, migration preview and the explicit unsigned
signing posture. This policy layers governance around it. It does not redefine
any hash, record schema or verifier rule; where the two differ, SSF-10 wins and
this policy must be corrected.

Non-claims: artifacts and release binaries are **not signed**; no package
trust, registry, skill/adapter signing, transparency log or complete
provenance validation exists.

## 2. Vocabulary — these are not synonyms

| Term | Meaning in Semantic | Example |
|---|---|---|
| identity | a deterministic name for exact bytes or inputs | `sha256:` of `.smc` bytes; source / manifest SHA-256 in provenance |
| correlation | linking records that refer to the same bytes, without security strength | Hub FNV-1a/64 `content_digest`; provenance `artifact_hash` link |
| integrity | detection that bytes changed relative to a trusted reference | release asset SHA-256 vs published digest; `CorruptedMismatch` |
| admission | a core layer accepting input for its next stage | `VerifiedSemCode` from `sm-verify` |
| authentication | proof of *who* produced bytes | **not implemented** |
| signature | cryptographic authentication over a digest | **not implemented** |

## 3. Current artifact trust chain

```text
SourceSet identity (source SHA-256)
→ manifest / project identity where applicable (manifest SHA-256, package name/version)
→ producer toolchain identity (compiler name/version, source fingerprint, features)
→ exact SemCode artifact SHA-256
→ verifier token bound to the exact artifact (matches_artifact)
→ runtime profile / capability context (deterministic-v1, admitted capabilities)
→ execution / audit evidence (PROMETHEUS audit archive)
→ release manifest / published digests where applicable
```

Each arrow is either an identity link (recorded in the SSF-10 provenance
sidecar or CLI output) or an admission step. No link is a signature.

## 4. Artifact classes

| Class | Current identity | Status |
|---|---|---|
| Source | SHA-256 in provenance `source.source_hash` | current |
| Manifest / project | SHA-256 `manifest_hash`; package name/version | current |
| Toolchain | `smc version --json` (version, source fingerprint, features, format revision) | current |
| Frontend / IR | not persisted as artifacts; covered transitively by SemCode identity | current (implicit) |
| SemCode | `sha256:` over raw `.smc` bytes | current |
| Verifier admission | `VerifiedSemCode::artifact_hash` binding | current |
| Capability context / runtime profile | provenance `contract.runtime_profile`; admitted capabilities at run time | current |
| Trace / audit | PROMETHEUS canonical audit archive format; audit event identities | current (integrity by canonical format; unsigned) |
| Release | published SHA-256 digests; `signing: "unsigned"` | current |
| UI frame / snapshot | historical inspection tooling; non-authoritative | deferred by explicit non-claim (retired UI) |
| Skill / adapter | none | deferred by explicit non-claim (no ALM) |
| Package | none beyond local package baseline | deferred, blocked before any registry |

## 5. Provenance envelope

The current concrete envelope is the SSF-10 companion record
`<artifact>.provenance.json` (`schema_version`, `artifact_hash`,
`artifact_size_bytes`, `producer`, `source`, `contract`). A future generic
envelope for other artifact classes must keep the same shape principles:

- a schema version;
- the SHA-256 of the exact subject bytes;
- producer identity;
- input identities;
- contract identities (format, verifier, runtime profiles);
- an explicit `signing` state;
- no wall-clock timestamps, host paths or user names that would break
  determinism (private paths must be redacted in any public mode).

Validation rules: subject hash mismatch → reject (`CorruptedMismatch`);
missing record → `MissingProvenance` (fail closed); unknown provenance is
labeled unknown, never inferred; stale inputs are detected by digest.

## 6. Hub distinction

Semantic Hub's FNV-1a/64 `content_digest` (`crates/semantic-hub/src/provenance.rs`)
is **correlation only**: not cryptographic trust, not a signature, not verifier
admission. Hub v0 itself has no cryptographic signing chain. The
repository-wide SHA-256 artifact identity must never be confused with
Hub-local correlation fingerprints.

## 7. Self-hosting provenance chain

The minimum bootstrap evidence record is, conceptually:

```text
BootstrapEvidence {
    c0_identity
    compiler_source_identity
    project_manifest_identity
    c1_artifact_hash
    c1_verifier_binding
    c1_runtime_and_toolchain_profile
    c2_artifact_hash
    c2_verifier_binding
    comparison_rule
    comparison_result
}
```

This is a governance shape, not a required Rust type, and it is not
implemented by this policy. Invariants:

- `C0 + S + admitted deterministic inputs` must identify the evidence that
  produced `C1`.
- `C1 + the same S + admitted deterministic inputs` must identify the evidence
  that produced `C2`.
- The fixed-point proof never depends only on file names or `mtime`.
- Each generation is verifier-admitted before it runs; the comparison result is
  evidence, not admission (threat model §11).

## 8. Future signing prerequisites

Signing may be introduced only after all of these are defined and reviewed:

- trust-root ownership;
- key lifecycle (generation, storage, custody);
- rotation;
- revocation;
- signer identity;
- signed envelope format;
- exact digest coverage (what bytes a signature covers);
- reproducible verification procedure;
- release and distribution policy.

Until then: **Semantic artifacts and release binaries remain explicitly
unsigned**, and every surface keeps reporting `signing: "unsigned"`.

## 9. Display and consumer requirements

Any tool that displays provenance must show unknown as unknown, must not label
an unverified artifact as trusted, and must show `unsigned` explicitly. Display
never creates trust. The retired Workbench/Studio surfaces and the unbuilt ALM
layer carry no provenance claim (§4).

## 10. Privacy and threat-model interaction

Privacy: provenance records must not leak private host paths or user names
(SSF-10 §3 excludes them from artifact bytes); Hub-local data rules are in
`docs/privacy/semantic_hub_data_policy_v0.md`. The broader Studio/ALM privacy
track (#1372) was closed not planned with those surfaces.
Threat model: provenance mitigates threat categories 3, 5 and 7 and the
bootstrap threats in `threat_model_v0.md`.

## 11. Fixtures

Existing evidence: SSF-10 tests in `tests/ssf10_artifact_trust.rs` (artifact
hashing, staleness by digest, verifier binding, migration preview). Fixtures
proposed by #1374 (`tests/provenance/*`: mismatched SemCode hash, stale source
hash, unknown-origin UI frame, skill without eval report, release hash
manifest) are listed here as future fixtures; UI-frame and skill fixtures are
deferred with their surfaces.

## 12. Linked tracks

#1365–#1373 proposed the surrounding layers; their current states are recorded
in `docs/roadmap/pre_bootstrap_governance_closeout.md`.
