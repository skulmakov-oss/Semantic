# Semantic Threat Model v0

Status: current repository-wide security authority (governance; no runtime behavior)
Issue: #1371
Companions: [`untrusted_project_policy_v0.md`](untrusted_project_policy_v0.md),
[`artifact_provenance_and_signing_policy_v0.md`](artifact_provenance_and_signing_policy_v0.md),
[`semantic_hub_threat_model_v0.md`](semantic_hub_threat_model_v0.md) (component),
[`../architecture/artifact_identity_and_trust.md`](../architecture/artifact_identity_and_trust.md) (SSF-10 technical authority)

## 1. Purpose

Define, for the whole repository, what Semantic protects, which inputs are
untrusted, which layer admits them, and which threats every current and future
surface must respect. It is the anchor that keeps new layers from becoming soft
bypasses around the existing trust direction:

```text
source → frontend/sema → IR → SemCode → verifier admission → deterministic VM
       → capability boundary → audit
```

This document describes the repository as it exists at the pre-bootstrap
governance closeout. It introduces no runtime behavior, no new capability and
no release claim.

## 2. Non-claims

Semantic does **not** claim:

- production-grade sandboxing or OS-level isolation of any component;
- that Semantic Hub is sandboxed — its trust class is `InProcessUnisolated`;
- broad host I/O, network, process or device support;
- package, registry, extension or plugin trust (none exist);
- signed artifacts or signed release binaries (they are explicitly unsigned);
- that current `main` is production-security audited;
- that retired Workbench / Studio / native UI code is safe for hostile input;
- ALM, skill or adapter safety (no ALM exists).

## 3. Assets to protect

- the verifier-first admission rule;
- deterministic VM execution and replay;
- SemCode artifact identity (SHA-256) and the verifier ↔ artifact binding;
- capability boundaries and the validity of capability denial;
- audit integrity (PROMETHEUS audit archives, Hub audit evidence);
- source and project files, and private user code;
- release artifacts and their published digests;
- public status truth (no claim stronger than evidence).

## 4. Trust zones

| Zone | Contents | Rule |
|---|---|---|
| Z0 Core trusted computing base | frontend / semantic analysis (`sm-front`, `sm-sema`), IR / lowering (`sm-ir`), SemCode format and emission (`sm-format`, `sm-emit`), verifier (`sm-verify`), verified VM / runtime core (`sm-vm`, `sm-runtime-core`), quota / trap enforcement | Only core-owned, audited code. No tooling, UI, Hub, package or bootstrap convenience logic joins Z0. |
| Z1 PROMETHEUS boundary | `prom-abi`, `prom-cap`, `prom-gates`, `prom-runtime`, `prom-state`, `prom-rules`, `prom-audit` | Host effects cross explicit ABI / capability / gate / audit boundaries. No other layer grants effects. |
| Z2 Governed tooling | `smc`, `svm`, release and qualification scripts, Semantic Hub (`InProcessUnisolated`) | Tooling requests core behavior and reports results; it never widens semantics, never admits on the verifier's behalf, and never treats external output as canonical truth. |
| Z3 Untrusted inputs | everything in §6 | Untrusted until parsed, validated, bounded and admitted by the owning Z0/Z1 layer. |

Retired or absent surfaces (Workbench, Studio, native UI, ALM, packages
registry, extensions) are not trust zones; see §10.

## 5. Attacker capabilities

The v0 attacker can:

- supply arbitrary `.sm` sources, project directories, manifests and import graphs;
- supply crafted `.smc` bytes and provenance sidecars;
- supply Hub request payloads and externally produced traces / audit material;
- substitute release assets in transit or in a mirror;
- control file names, directory layout, timestamps (`mtime`) and symlinks
  inside a project they author;
- craft inputs aimed at resource exhaustion (deep nesting, huge literals,
  diagnostic flooding).

Out of scope for v0: an attacker with code execution in the user's process or
OS account, a compromised Rust toolchain, or hardware faults.

## 6. Untrusted inputs

- `.sm` source files;
- project manifests (`semantic.toml`, `Semantic.package`) and project layout;
- imports and dependency declarations;
- `.smc` artifacts received from outside the current local build;
- `<artifact>.provenance.json` sidecars received externally;
- Semantic Hub request payloads;
- traces and audit material received externally;
- release assets before digest verification and verifier admission;
- future packages, adapters and extensions.

## 7. Admission and validation boundaries

| Input | Admitting layer | Current contract |
|---|---|---|
| `.sm` source | frontend / sema | deterministic diagnostics; no execution during check |
| manifests / project root | `smc-cli` project model | data only; unknown directives rejected; declared entry and imports canonicalized and must not escape their declared root (see untrusted project policy) |
| SemCode bytes | `sm-verify` | `VerifiedSemCode` bound to exact artifact SHA-256 (SSF-10 §6, non-bypass law); unsupported revisions rejected |
| provenance sidecar | `smc artifact inspect`, staleness detection | integrity correlation only (`artifact_hash`); `CorruptedMismatch` / `MissingProvenance` fail closed |
| host effects | PROMETHEUS capability / gate layer | denial is a valid, visible outcome |
| Hub requests | `semantic-hub` admission | bounded, `InProcessUnisolated`; FNV `content_digest` is correlation only |
| release assets | published SHA-256 digests + verifier | assets unsigned; trust is digest-bound |

## 8. Threat categories

1. **Malicious project / manifest** — path traversal in entry or import paths,
   symlink escape, misleading package identity, unsupported-but-plausible
   fields, directory-size exhaustion.
2. **Malicious source** — parser stress, deep nesting, huge literals,
   diagnostic flooding, confusing Unicode identity.
3. **Malicious SemCode** — malformed headers, out-of-range operands,
   unverified bytes presented as verified, provenance sidecars that lie.
4. **Malicious host-effect request** — attempts to obtain effects without an
   admitted capability or to hide a denial.
5. **Malicious trace / audit material** — forged or reordered evidence
   presented as canonical.
6. **Malicious external command output** — text from tools presented as
   canonical truth.
7. **Malicious release asset** — substituted binaries or archives.
8. **Bootstrap subversion** — see §11.
9. **Future package / extension** — deferred, see §10.

## 9. Required mitigations

- Verifier rejection is final; no tooling, Hub, UI or bootstrap path overrides it.
- Capability denial is a valid outcome, never an error to bypass.
- Untrusted input never becomes trusted by display alone; display is not admission.
- External output never becomes canonical truth without provenance.
- Artifact freshness is decided by content digest, never by `mtime` (SSF-10 §5).
- Parsing, checking and diagnostics stay bounded and deterministic.
- No hidden telemetry and no silent source upload: the toolchain crates contain
  no network client; the only process spawn in `smc-cli` is the fixed developer
  command `smc snapshots` (`cargo test --test golden_snapshots`), never a
  project-defined command.
- Release assets are verified against published SHA-256 digests and remain
  explicitly unsigned.

## 10. Deferred and future surfaces

Blocked from activation until their own policy is written and reviewed:

| Surface | Status | Blocking rule |
|---|---|---|
| Workbench / Studio / native UI command and snapshot safety | retired (`docs/roadmap/ui_workbench_studio_retirement.md`) | no new UI surface may process hostile projects before a dedicated policy |
| UI frame / snapshot / event scripts | historical inspection tooling only | snapshots are non-authoritative; never live state; never admission |
| ALM traces / skills / adapters | not implemented | suggestions would be hypotheses; no mutation without approval; local-only data |
| Package registry, extensions, plugins | not implemented (#1376 closed not planned pre-self-hosting) | no registry or marketplace before package trust, identity, signature and permission policy |
| Cryptographic signing | not implemented | see provenance and signing policy §6 |

## 11. Self-hosting and bootstrap threats

Self-hosting (#1910) defines the bootstrap sequence:

```text
S  = Semantic compiler source
C0 = current Rust implementation of the compiler
C0(S) → C1.smc
C1(S) → C2.smc
```

Rules:

- `S` is source input. It is not trusted because it is compiler source; it is
  checked like any other `.sm` project.
- `C1` is not trusted because `C0` produced it. `C1` must pass normal verifier
  admission before it executes.
- `C2` is not trusted because `C1` produced it. `C2` must pass normal verifier
  admission.
- `Canonical(C1) == Canonical(C2)` is bootstrap **evidence**, never a verifier
  bypass. No fixed-point comparison may authorize execution of an unverified
  artifact.
- No host-side hidden parser, typechecker, lowering or emitter may be inserted
  to make bootstrap succeed; compiler domain logic in a generation must be the
  admitted SemCode of that generation.
- Bootstrap inputs that can affect output must be identified and recorded:
  compiler / toolchain identity, source-set identity, manifest / project
  identity, SemCode format, verifier profile, runtime profile, explicitly
  admitted capabilities, and deterministic configuration.
- Time, random host state, directory-enumeration order, machine-local paths
  and undocumented environment variables must not silently affect compiler
  output.

Threats: a generation that silently normalizes or migrates the other's input
(see the migration policy bootstrap rule), a comparison keyed on file names or
`mtime`, an artifact executed before admission, and a non-deterministic input
that makes C1 and C2 differ or agree by accident.

## 12. Update triggers

This document must be revisited before:

- any new host capability, I/O family or effect class;
- any network, registry, package fetch or update channel;
- any new process-spawning command;
- any UI, Studio, ALM, plugin or extension surface becoming active;
- SemCode format, verifier profile or runtime profile changes;
- each self-hosting bootstrap stage that changes the trusted computing base;
- any change to release signing posture.

## 13. Relationship to roadmap tracks

#675, #1365, #1366, #1367, #1368, #1369 and #1370 proposed the layers this
model now bounds; layers that were retired or not built are listed in §10
rather than modeled as active surfaces.
