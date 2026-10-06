# Pre-Bootstrap Governance Closeout

Status: closeout evidence map for #1371, #1374, #1378 (governance only)
Base SHA: `242c684dcf0702f1affdf3a2bfa5f05eabeaae41`
Remains open: #1910 Semantic Self-Hosting Foundation — **not started**

This record maps every acceptance item of the three governance issues to its
current authority. `DEFERRED-BY-EXPLICIT-NONCLAIM` is used only for surfaces
that do not exist (retired or never built) and are blocked from activation
until their own policy exists; it never hides a missing current contract.

Related-track states used below (GitHub, at base SHA): #675 closed completed;
#1365 closed completed; #1366, #1367, #1368, #1369, #1370, #1372, #1373, #1375,
#1376, #1377, #1909 closed not planned. #1376 (package ecosystem) and #1909
(Full Sigma / Native Reasoning) are not required for first self-hosting and are
not reopened.

## #1371 — Semantic threat model and untrusted project policy

Original goal: an explicit threat model and untrusted-project policy before new
layers expand the attack surface.
Authorities: `docs/security/threat_model_v0.md`,
`docs/security/untrusted_project_policy_v0.md`, `SECURITY.md`.

| Acceptance item | Evidence | Status |
|---|---|---|
| `threat_model_v0.md` exists | `docs/security/threat_model_v0.md` | SATISFIED |
| Trust zones defined | threat model §4 (Z0 core, Z1 PROMETHEUS, Z2 governed tooling incl. Hub `InProcessUnisolated`, Z3 untrusted) | SATISFIED |
| Untrusted project policy defined | `untrusted_project_policy_v0.md` | SATISFIED |
| Untrusted manifest/source/import behavior | project policy §2–§4; threat model §6–§8 | SATISFIED |
| SemCode validation/admission policy | threat model §7; project policy §5; SSF-10 §6 | SATISFIED |
| Workbench/Studio command-output safety | threat model §10 (retired contour, blocked) + project policy §6–§7 for current CLI output | DEFERRED-BY-EXPLICIT-NONCLAIM (retired Workbench/Studio) |
| UI snapshot/event security | threat model §10 (historical inspection tooling, non-authoritative) | DEFERRED-BY-EXPLICIT-NONCLAIM (retired UI) |
| ALM trace/skill/adapter risks | threat model §10 | DEFERRED-BY-EXPLICIT-NONCLAIM (no ALM) |
| Package/extension marked deferred but blocked | threat model §10 | SATISFIED (blocking rule stated) |
| UI/ALM cannot override verifier rejection | threat model §9 ("Verifier rejection is final; no tooling, Hub, UI or bootstrap path overrides it") | SATISFIED |
| No hidden telemetry / silent upload | threat model §9; project policy §8 | SATISFIED |
| No new runtime behavior or release claim | docs only; one comment-only Rust edit | SATISFIED |
| Links #675, #1365–#1370 | threat model §13; this record | SATISFIED |

Self-hosting additions: threat model §11 (S/C0/C1/C2 trust rules, recorded
deterministic inputs, forbidden hidden host logic).

## #1374 — Semantic artifact provenance and signing chain

Original goal: a trust-chain anchor for artifacts, releases and future
packages/skills.
Authorities: `docs/architecture/artifact_identity_and_trust.md` (SSF-10,
technical), `docs/security/artifact_provenance_and_signing_policy_v0.md`
(governance).

| Acceptance item | Evidence | Status |
|---|---|---|
| Provenance chain document exists | provenance policy §3 + SSF-10 | SATISFIED |
| Artifact classes defined | provenance policy §4 | SATISFIED |
| Source/manifest/toolchain/SemCode/verifier/runtime identities | provenance policy §4; SSF-10 §3–§6 | SATISFIED |
| UI frame/snapshot provenance | provenance policy §4 (non-authoritative, retired UI) | DEFERRED-BY-EXPLICIT-NONCLAIM |
| Trace/audit provenance | provenance policy §4 (PROMETHEUS canonical audit archive; unsigned) | SATISFIED |
| Skill/adapter provenance | provenance policy §4 | DEFERRED-BY-EXPLICIT-NONCLAIM (no ALM) |
| Package provenance deferred but blocked before registry | provenance policy §4; threat model §10 | SATISFIED (blocking rule stated) |
| Release artifact provenance | provenance policy §3–§4; SSF-10 §7 | SATISFIED |
| Generic provenance envelope specified | provenance policy §5 (SSF-10 sidecar + generic shape and validation rules) | SATISFIED |
| Signing defined as deferred, not claimed | provenance policy §8 (prerequisites; explicitly unsigned) | SATISFIED |
| Workbench/Studio display requirements | provenance policy §9 (general display rules; retired surfaces carry no claim) | DEFERRED-BY-EXPLICIT-NONCLAIM (retired Workbench/Studio) |
| ALM provenance requirements | provenance policy §4, §9 | DEFERRED-BY-EXPLICIT-NONCLAIM (no ALM) |
| Privacy interaction with #1372 | provenance policy §10 (#1372 closed not planned; Hub data policy) | SATISFIED |
| Threat-model interaction with #1371 | provenance policy §10 | SATISFIED |
| Required fixtures listed | provenance policy §11 | SATISFIED |
| No implementation or release claim | docs only | SATISFIED |
| Links #1365–#1373 | provenance policy §12; this record | SATISFIED |

Hub distinction: provenance policy §6; Hub threat model and data policy now
point to it and keep "Hub v0 itself has no cryptographic signing chain";
`crates/semantic-hub/src/provenance.rs` comment reconciled (comment only).
Self-hosting additions: provenance policy §7 (`BootstrapEvidence` shape and
invariants).

## #1378 — Semantic migration and deprecation policy

Original goal: a compatibility anchor that lets Semantic evolve without silent
breakage.
Authorities: `docs/roadmap/language_maturity/deprecation_and_migration.md`
(governance companion to SSF-10), SSF-10 §2 and §8,
`stability_and_compatibility.md`, `compatibility_policy_stack.md`.

| Acceptance item | Evidence | Status |
|---|---|---|
| Migration/deprecation policy doc exists | `deprecation_and_migration.md` (current governance companion to SSF-10) | SATISFIED |
| Compatibility classes defined | policy "Compatibility Classes" | SATISFIED |
| Deprecation states defined | policy "Deprecation States" | SATISFIED |
| Source migration policy | "Surface Policies" + SSF-10 §2.A | SATISFIED |
| SemCode compatibility policy | "Surface Policies" + SSF-10 §2.F | SATISFIED |
| Verifier/runtime migration policy | "Surface Policies" + SSF-10 §2.E–§2.F | SATISFIED |
| Capability migration policy | "Surface Policies" (capability row) | SATISFIED |
| Manifest migration policy | "Surface Policies" + SSF-10 §2.B | SATISFIED |
| CLI migration policy | "Surface Policies" (CLI machine contracts) | SATISFIED |
| Diagnostics migration policy | "Surface Policies" + SSF-10 §2.C | SATISFIED |
| Workbench/Studio local data migration | "Deferred Surfaces" | DEFERRED-BY-EXPLICIT-NONCLAIM (retired Workbench/Studio) |
| ALM trace/skill/adapter migration | "Deferred Surfaces" | DEFERRED-BY-EXPLICIT-NONCLAIM (no ALM) |
| Package/lockfile migration staged and deferred | "Deferred Surfaces" | SATISFIED (staged, blocked) |
| Dry-run migration command model | "Migration Behavior" (`smc migrate check` / `preview`, zero mutation) | SATISFIED |
| Fixtures listed | policy "Fixtures" | SATISFIED |
| Release-note requirements | policy "Release Notes Rule" | SATISFIED |
| No implementation or release claim | docs only | SATISFIED |
| Aligns with public status vocabulary | classes reuse `public_status_model.md` | SATISFIED |
| Links #1370–#1377 | policy cross-references; this record | SATISFIED |

Self-hosting additions: policy "Bootstrap Migration Rule".

## Verdict

All acceptance items that matter to current or self-hosting architecture are
SATISFIED. Every DEFERRED-BY-EXPLICIT-NONCLAIM item concerns a retired
(Workbench, Studio, native UI) or never-built (ALM, registry, extensions)
surface, and each is blocked from activation until its own policy exists.

**#1371, #1374 and #1378 can honestly close.** #1910 stays open and
self-hosting implementation has not started.

Governance boundary for SHF-0 under #1910:

```text
repository-wide threat model: landed
untrusted-project policy: landed
artifact provenance authority: landed/reconciled with SSF-10
signing posture: explicitly unsigned
future signing prerequisites: documented
migration/deprecation governance: landed/reconciled
bootstrap trust/provenance/migration rules: explicit
self-hosting implementation: NOT STARTED
```
