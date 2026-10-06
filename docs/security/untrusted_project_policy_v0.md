# Untrusted Project Policy v0

Status: current repository-wide security policy (governance; no runtime behavior)
Issue: #1371
Parent: [`threat_model_v0.md`](threat_model_v0.md)

## 1. Rule

Every Semantic project, file, manifest, import graph and externally received
artifact is untrusted until the owning layer admits it. Opening, displaying or
naming a project grants nothing.

This policy states the current contract and the rules any future tool must
keep. It claims no sandbox: tools run with the invoking user's OS permissions.

## 2. Project root confinement

- A project or package is resolved from an explicit root.
- The declared entry and relative/qualified imports are canonicalized and must
  not escape their declared root; escapes are rejected with deterministic
  diagnostics (`crates/smc-cli/src/package_manifest.rs`: "must not escape its
  declared root", "escapes package module_root", "escapes dependency
  module_root"; qualification fixture
  `examples/qualification/project_manifest_surface/negative_semantic_toml_escape_entry`).
- Symlinks that escape the project root and recursive symlink cycles are
  rejected (`tests/ssf10_artifact_trust.rs` tests 13–14); application and test
  roots that are symlinks or reparse points are rejected
  (`application_host.rs`, `smc test` root handling).
- Confinement is path validation inside the toolchain, not OS isolation.

## 3. Manifests are data

- A manifest is parsed as data and never executed.
- Unknown package-manifest directives are rejected
  (`unknown package manifest directive`); unknown or future manifest schema
  versions are rejected fail-closed (SSF-10 §2.B).
- Manifest fields that are not supported must produce deterministic
  diagnostics where the current contract defines them; a plausible-looking
  field is never silently honored.
- Project or package identity (name, version) is metadata; it grants no trust
  and no capability.

## 4. Imports and dependencies

- Imports resolve only inside the local project/package roots described above.
- Imports never grant capabilities; capability admission is a runtime /
  PROMETHEUS decision.
- Remote fetch is not implemented and must not become implicit: no import,
  manifest field or command downloads content.

## 5. Artifacts

- An externally received `.smc` is untrusted. It executes only through normal
  verifier admission, and the verified token is bound to its exact SHA-256.
- A `.provenance.json` sidecar is unsigned metadata correlated by
  `artifact_hash`; a mismatch is `CorruptedMismatch`, absence is
  `MissingProvenance`.
- Freshness is decided by content digest, never by `mtime`.

## 6. Output and display

- Source display is not trust; showing code says nothing about its safety.
- Diagnostics and command output are data. Consumers must not render them as
  executable markup or treat them as canonical truth without provenance.

## 7. Commands and effects

- No project-defined command is executed automatically. The toolchain has no
  project hook, script or build-step mechanism; the only process spawn in
  `smc-cli` is the fixed developer command `smc snapshots`.
- Capability denial remains a valid, visible result. No tool hides or retries
  around it.

## 8. Data handling

- No hidden telemetry.
- No silent source, trace or artifact upload; the toolchain crates contain no
  network client.
- Local state (for example Semantic Hub's governed storage) stays local; see
  `docs/privacy/semantic_hub_data_policy_v0.md`.

## 9. Future surfaces

Any future package fetcher, registry, extension host, UI shell or ALM layer must
satisfy this policy and the threat model before it is activated (threat model §10).
