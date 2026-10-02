# Artifact Identity, Provenance, Compatibility, and Trust Specification (SSF-10)

Status: canonical SSF-10 architecture authority  
Issue: #1581 (SSF-10)  
Owner layer: `sm-format`, `sm-verify`, `smc-cli`

---

## 1. Overview and Objectives

This document establishes the explicit, deterministic, and testable compatibility, provenance, and artifact-trust contract for the Semantic language platform per milestone SSF-10 (#1581).

Prior to SSF-10, the repository contained header-revision checks and non-cryptographic hash utilities (`fnv1a64`), but lacked:
- A canonical, cryptographic compiled-artifact identity.
- Explicit verifier binding guaranteeing that a verification token corresponds to the exact artifact bytes verified.
- Producer provenance tracking separating the producing compiler from the inspecting toolchain.
- Digest-based stale and mismatched artifact detection independent of filesystem timestamps (`mtime`).
- Explicit, formal compatibility policies across language, manifest, diagnostics, stdlib, runtime, and verifier dimensions.
- A non-destructive migration dry-run path.
- Public CLI commands for artifact inspection and hashing.
- Explicit, truthful release asset signing posture (`"unsigned"`).

SSF-10 resolves each of these requirements without creating speculative infrastructure or breaking existing platform boundaries.

---

## 2. Canonical Compatibility Policies (Section 5 Authority)

Semantic defines six explicit compatibility policies across its subsystems:

### A. Source Language Compatibility Policy
- **States**: Every language construct is classified as `Compatible`, `Deprecated`, or `Removed`.
- **Minimum Deprecation Window**: Any construct marked deprecated (via `// @deprecated` or `#[deprecated]`) must remain supported with compiler diagnostic warnings for a minimum of 1 minor/epoch cycle before removal.
- **Removed Constructs**: Transition to `Removed` results in a deterministic compile-time error (`E0005`/`E0201`), never silent behavior alteration.

### B. Manifest and Project Compatibility Policy
- **Independent Versioning**: Manifest schemas (`semantic.toml`) are versioned independently from the source language compiler version.
- **Supported Schemas**: Currently schema version `1`. Schema version `0` is admitted with deprecation warnings.
- **Fail-Closed Rejection**: Unknown or future schema versions (e.g. `schema_version = 99`) are rejected fail-closed.
- **Zero Silent Migration**: The compiler and CLI never modify manifests or project files automatically.

### C. Diagnostics Contract Compatibility Policy
- **Contract Identifier**: `semantic.diagnostics`, current schema version `1`.
- **Breaking Changes**: Any change to diagnostic output that:
  1. Renames, renumbers, or removes an established diagnostic error code (`E*`, `W*`, `V*`, `R*`),
  2. Modifies diagnostic severity from warning to error (or vice-versa),
  3. Modifies structural JSON keys or types,
  is classified as a breaking contract change and requires bumping the machine schema version (`semantic.diagnostics/v2`).

### D. Standard Library Compatibility Policy
- **Contract Baseline**: Version `semantic-stdlib-v1` (`0.1.0`) covering `std.core`, `std.quad`, `std.math` (`sqrt`, `abs`), `std.text`, `std.seq`, `std.map`, `std.option`, `std.result`.
- **Behavior-Preserving Additions**: Introducing new pure, non-conflicting builtins or library modules is non-breaking.
- **Breaking Changes**: Changing argument counts, parameter types, return types, or widening capability requirements is breaking.

### E. Runtime and Total Determinism Compatibility Policy
- **Canonical Profile**: `deterministic-v1` executed by `sm-vm`.
- **Total Determinism PRNG Rule**: Any supported runtime PRNG contract must produce bit-for-bit identical pseudo-random sequences across repeated executions with the identical seed. PRNG algorithm alterations constitute a breaking runtime contract change.

### F. SemCode and Verifier Gate Policy
- **Wire Format**: SemCode format `SEMCOD22` (Revision 23, Epoch 0) with mandatory `ADT0` descriptor table.
- **Verifier Profile**: `verifier-canonical-v1` enforcing loop CFG acyclicity, operand stack limits, register bounds, and capability gating.
- **Verifier Admission Gate**: Canonical execution consumes verifier-admitted SemCode. Revisions `< 23` are classified `Deprecated`; revisions `> 23` are `Unsupported`.

---

## 3. Canonical Artifact Identity & Digest Authority

### A. Cryptographic Hash Construction
Every compiled SemCode artifact (`.smc`) has an exact, deterministic canonical identity computed via a zero-dependency SHA-256 implementation following the algorithm defined by FIPS PUB 180-4:

$$\text{ArtifactHash} = \text{SHA-256}(\text{RawArtifactBytes})$$

Formatted as lowercase hex prefixed by `sha256:`:
```text
sha256:73a23b307f08154c9f3ef35fda96bad341466bbf15df43e79253d012e7ff4024
```

### B. What the Canonical Hash Covers
Because the hash is computed directly over the complete wire serialization of the `.smc` binary, it strictly and transitively binds:
1. **Header Spec**: 8-byte magic (`SEMCOD22`), epoch (`0`), revision (`23`), and capability bitfield.
2. **ADT Descriptors (`ADT0`)**: Complete canonical descriptor table (type names, variant names, payload arities).
3. **Callable Signatures (`SIG0`)**: Parameter counts and parameter types for all defined functions.
4. **Instruction Bytecode**: Exact opcode sequence, operands, and jumps.
5. **Constant String Tables**: All deduplicated string literals.
6. **Debug Information (`DBG0`)**: If compiled with `--debug-symbols`, the instruction-boundary debug map.
7. **Ownership Tracks (`OWN0`)**: All borrow and write event paths and execution anchors.

The hash excludes non-deterministic metadata: no wall-clock timestamps, build paths, or host usernames are present in the artifact.

---

## 4. Companion Artifact Provenance & Producer Binding

To identify the toolchain that originally produced an arbitrary `.smc` without format churn on the wire bytecode, Semantic defines a canonical companion provenance record (`<artifact>.provenance.json`):

```json
{
  "schema_version": 1,
  "artifact_hash": "sha256:73a23b307f08154c9f3ef35fda96bad341466bbf15df43e79253d012e7ff4024",
  "artifact_size_bytes": 1024,
  "producer": {
    "compiler_name": "smc",
    "compiler_version": "0.1.0",
    "build_target": "x86_64-pc-windows-msvc",
    "source_fingerprint": "443569c32db5fc8a",
    "profile": "release",
    "enabled_features": ["std", "profile-rust", "debug-symbols"]
  },
  "source": {
    "package_name": "demo",
    "package_version": "0.1.0",
    "entry_file": "main.sm",
    "source_hash": "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    "manifest_hash": "sha256:2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae"
  },
  "contract": {
    "semcode_format": "SEMCOD22",
    "semcode_epoch": 0,
    "semcode_revision": 23,
    "verifier_profile": "verifier-canonical-v1",
    "runtime_profile": "deterministic-v1",
    "stdlib_version": "semantic-stdlib-v1",
    "diagnostic_contract": "semantic.diagnostics"
  }
}
```

### Trust Model (Integrity-Only Companion Record)
Companion provenance (`<artifact>.provenance.json`) is an unsigned metadata sidecar cryptographically correlated via `artifact_hash`. It proves correlation to the artifact SHA-256 digest, but sidecar contents are not independently authenticated signatures; release trust requires external manifest/hash authority.

If the artifact bytes change or do not match `artifact_hash`, inspection reports `CorruptedMismatch`.

---

## 5. Digest-Based Stale and Mismatch Detection Authority

Filesystem timestamps (`mtime`) are strictly treated as non-authoritative hints. The canonical authority for artifact freshness and compatibility is content digest comparison:

1. **Immunity to mtime Spoofing**: If source code is replaced with new content, but its `mtime` is backdated or preserved, `detect_artifact_staleness` computes the actual source SHA-256 and compares it against provenance `source_hash`. The mismatch is detected immediately as `StaleSourceChanged`.
2. **Explicit Detection Statuses**:
   - `Fresh`: Source, manifest, and artifact hashes match provenance exactly.
   - `StaleSourceChanged`: Source content hash has changed.
   - `ProjectMismatch`: Artifact provenance belongs to a different package/project name.
   - `ManifestMismatch`: Manifest content hash has changed.
   - `ToolchainMismatch`: Producing compiler major version differs from the current compiler.
   - `MissingProvenance`: Companion provenance is absent (fail-closed; cannot prove correlation).
   - `UnsupportedProvenance`: Provenance record is corrupt or invalid JSON.

---

## 6. Verifier Binding Invariant

A successful verification result (`VerifiedSemCode` / `VerifiedEntrySemCode`) is immutably bound to the exact artifact hash verified:

```rust
pub struct VerifiedSemCode {
    header: SemcodeHeaderSpec,
    functions: Vec<VerifiedFunction>,
    artifact_hash: [u8; 32],
}

impl VerifiedSemCode {
    pub fn artifact_hash(&self) -> [u8; 32];
    pub fn artifact_hash_hex(&self) -> String;
    pub fn matches_artifact(&self, bytes: &[u8]) -> bool;
}
```

### Non-Bypass Law
$$\forall A, B : A \neq B \implies \neg \text{matches\_artifact}(\text{Verify}(A), B)$$

It is architecturally impossible for a verification token issued for artifact $A$ to be used to admit or execute artifact $B$.

---

## 7. Release Artifact Trust and Signing State

### Explicit Signing Honesty
Semantic binaries (`smc.exe`, `svm.exe`) are currently **unsigned**.
- The repository deliberately does not manufacture fake PKI certificates, simulated signatures, or unevidenced trust claims.
- All CLI surfaces (`smc version`, `smc artifact inspect`) and release verification scripts (`scripts/verify_release_assets.ps1`) explicitly output:
  ```json
  "signing": "unsigned"
  ```
- Release verification (`scripts/verify_release_assets.ps1`) executes the extracted release binary `smc version --json` to capture and record the actual release toolchain evidence (compiler version, source fingerprint, features, format revision) alongside the exact SHA-256 asset checksums.

---

## 8. CLI Surfaces

### `smc version [--json]`
Displays toolchain version, source fingerprint (FNV-1a hash of compiler source closure), enabled cargo features, active SemCode format revision, verifier profile (`verifier-canonical-v1`), and release signing state (`unsigned`).

### `smc artifact hash <path.smc> [--json]`
Computes and outputs the canonical SHA-256 hash of the compiled artifact.

### `smc artifact inspect <path.smc> [--json]`
Decodes and formats comprehensive artifact metadata:
- Canonical artifact hash and payload size in bytes.
- Header magic, epoch, revision, and decoded capabilities.
- Function count, names, code sizes, string counts, and signature status.
- ADT descriptor inventory.
- Verifier admission verdict.
- **Producer Toolchain Identity** (from provenance, or explicit `UNRECORDED - NO PROVENANCE ATTACHED`).
- **Inspecting Toolchain Identity** (current process).
- Signing state.

### `smc migrate <check|preview> <path> [--json] [--dry-run]`
Performs non-destructive dry-run migration analysis on a source file or project directory with guaranteed zero filesystem mutation.

---

## 9. SSF-11 Entry Conditions

With SSF-10 complete, the following invariants are established for SSF-11 (*Documentation, canonical examples, clean-clone onboarding*):
1. Toolchain versions and format revisions are frozen at `SEMCOD22` (rev 23).
2. All compiled examples can be canonically hashed with `smc artifact hash` and verified with `smc verify`.
3. Companion provenance records can be created alongside `.smc` binaries.
4. Release smoke scripts assert `signing: "unsigned"` and verify exact SHA-256 checksums bound to real release toolchain evidence.
5. Clean-clone onboarding documentation can rely on stable `smc version`, `smc artifact`, and `smc migrate` commands.
