# Artifact Identity, Compatibility, and Trust Specification (SSF-10)

Status: canonical SSF-10 architecture authority  
Issue: #1581 (SSF-10)  
Owner layer: `sm-format`, `sm-verify`, `smc-cli`

---

## 1. Overview and Objectives

This document establishes the explicit, deterministic, and testable compatibility and artifact-trust contract for the Semantic language platform per milestone SSF-10 (#1581).

Prior to SSF-10, the repository contained header-revision checks and non-cryptographic hash utilities (`fnv1a64`), but lacked:
- A canonical, cryptographic compiled-artifact identity.
- Explicit verifier binding guaranteeing that a verification token corresponds to the exact artifact bytes verified.
- Deterministic stale and mismatched artifact detection.
- A non-destructive migration dry-run path.
- Clear public CLI commands for artifact inspection and hashing.
- Explicit, truthful release asset signing posture (`"unsigned"`).

SSF-10 resolves each of these requirements without creating speculative infrastructure or breaking existing platform boundaries.

---

## 2. Compatibility Contract Dimensions

Semantic defines seven canonical compatibility dimensions:

| Dimension | Canonical Contract Identifier | Baseline / Active Scope | Rejection / Migration Rule |
|---|---|---|---|
| **Source Language** | `semantic-source-v1` | Rust-like executable profile (`fn main`, `let`, control flow, ADTs, tuples, records) | Syntax or type errors report deterministic `sm-diagnostic` codes; no silent grammar mutation. |
| **Manifest / Schema** | `semantic-manifest-v1` | `Semantic.toml` / `Semantic.package` | Structural validator rejects unrecognized fields or root-escape path violations. |
| **Diagnostics Contract** | `semantic-diag-v1` | External JSON schema (`docs/spec/diagnostics_machine_schema_v1.md`) | Byte-deterministic JSON output with stable severity and code strings (`E*`, `W*`, `V*`, `R*`). |
| **Standard Library** | `semantic-stdlib-v0` | `std.core`, `std.quad`, `std.math` (`sqrt`, `abs`), `std.text`, `std.seq`, `std.map`, `std.option`, `std.result` | Language-owned builtins; unrecognized builtins reject at typecheck. |
| **SemCode Format** | `SEMCOD22` | Revision `23`, Epoch `0` with mandatory `ADT0` descriptor table | Header revisions `< 23` are classified `Deprecated`; revisions `> 23` are `Unsupported`; non-SemCode magic is `Incompatible`. |
| **Verifier Gate** | `VerifiedLocal` | Verifier-first mandatory admission token (`VerifiedSemCode`, `VerifiedEntrySemCode`) | Standard execution fails closed on any unadmitted bytecode. No VM execution without prior admission. |
| **Runtime Model** | `DeterministicVM` | Pure compute or capability-gated PROMETHEUS host effects | Effect opcodes require explicit capability bits and manifest grants; unauthorized effects fault. |

Every dimension evaluates to one of four states:
- `Compatible`: matches current toolchain and runtime contract exactly.
- `Deprecated`: older supported contract (e.g. legacy headers without ADT opcodes); migration recommended.
- `Incompatible`: violates contracts, fails verifier admission, or carries corrupt payload; hard rejection.
- `Unsupported`: future or unrecognized version beyond toolchain capabilities; hard rejection.

---

## 3. Canonical Artifact Identity

### A. Cryptographic Hash Construction
Every compiled SemCode artifact (`.smc`) has an exact, deterministic canonical identity computed via zero-dependency, FIPS 180-4 compliant SHA-256:

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

## 4. Verifier Binding Invariant

A successful verification result (`VerifiedSemCode` / `VerifiedEntrySemCode`) is immutably bound to the exact artifact hash verified.

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

It is architecturally impossible for a verification token issued for artifact $A$ to be used to admit or execute artifact $B$. Any modification (even 1 bit) to the artifact invalidates `matches_artifact()`.

---

## 5. Stale and Mismatched Artifact Detection

`smc-cli` provides deterministic detection for out-of-date or mismatched artifacts:
- **Source Modified Detection**: Compares filesystem modification timestamps (`mtime`) between the `.sm` source and the `.smc` artifact. If source `mtime > artifact mtime`, the artifact is flagged `StaleSourceModified`.
- **Content Mismatch Detection**: Re-verifies source-to-binary hash correlation. If the artifact was generated from different source text, verification or staleness checks detect the mismatch fail-closed.
- **Header Mismatch**: Corrupted or unrecognized headers reject immediately at the decoder and verifier boundaries.

---

## 6. Migration Dry-Run (Non-Destructive Guarantee)

Migration preview is provided by the `smc migrate <check|preview>` CLI surface.

### Invariants:
1. **Zero Filesystem Mutation**: Dry-run operations perform strictly zero disk writes or deletions. All input files retain their exact contents, sizes, and hashes before and after execution.
2. **Deterministic Output**: Evaluates the package or source against current compatibility dimensions and emits reproducible human-readable or JSON reports.
3. **Explicit Deprecation Reporting**: Reports legacy constructs (such as pre-revision 23 headers requiring recompilation) without silently rewriting code.

---

## 7. Release Artifact Trust and Signing State

### Explicit Signing Honesty
Semantic binaries (`smc.exe`, `svm.exe`) are currently **unsigned**.
- The repository deliberately does not manufacture fake PKI certificates, simulated signatures, or unevidenced trust claims.
- All CLI surfaces (`smc version`, `smc artifact inspect`) and release verification scripts (`scripts/verify_release_assets.ps1`) explicitly output:
  ```json
  "signing": "unsigned"
  ```
- Any future introduction of code signing will require explicit repository-owner authorization and actual hardware/CI key infrastructure.

### Cryptographic Checksums
All release distribution assets are validated against SHA-256 checksums published alongside the release assets, verified by `scripts/verify_release_assets.ps1`.

---

## 8. CLI Surfaces

### `smc version [--json]`
Displays toolchain version, source commit, enabled cargo features, active SemCode format revision, verifier profile, and release signing state.

```text
Semantic Language Toolchain v0.1.0
Source Commit:      4e6449a323044357
Enabled Features:   DEBUG_SYMBOLS,DEFAULT,PROFILE_LOGOS,PROFILE_RUST,STD
SemCode Format:     SEMCOD22 (epoch=0, rev=23)
Verifier Profile:   VerifiedLocal
Release Signing:    unsigned
```

### `smc artifact hash <path.smc> [--json]`
Computes and outputs the canonical SHA-256 hash of the compiled artifact.

```text
sha256:9d4982fb448e30d60bab66129c855d599b77f5f2386efb344b0dc225c6da1d2a
```

### `smc artifact inspect <path.smc> [--json]`
Decodes and formats comprehensive artifact metadata:
- Canonical artifact hash and payload size in bytes.
- Header magic, epoch, revision, and decoded capabilities.
- Function count, names, code sizes, string counts, and signature status.
- ADT descriptor inventory.
- Verifier admission verdict.
- Toolchain identity and signing state.

### `smc migrate <check|preview> <path> [--json] [--dry-run]`
Performs non-destructive dry-run migration analysis on a source file or project directory.

---

## 9. SSF-11 Entry Conditions

With SSF-10 complete, the following invariants are established for SSF-11 (*Documentation, canonical examples, clean-clone onboarding*):
1. Toolchain versions and format revisions are frozen at `SEMCOD22` (rev 23).
2. All compiled examples can be canonically hashed with `smc artifact hash` and verified with `smc verify`.
3. Release smoke scripts assert `signing: "unsigned"` and verify exact SHA-256 checksums.
4. Clean-clone onboarding documentation can rely on stable `smc version`, `smc artifact`, and `smc migrate` commands.
