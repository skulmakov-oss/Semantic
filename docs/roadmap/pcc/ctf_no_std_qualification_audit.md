# CTF no_std Qualification Audit

Status:
  DRAFT / QUALIFICATION AUDIT

Core Trust Freeze is **not** declared complete by this document.
This audit records the current `no_std` / `no-default-features` posture and
does not claim full workspace qualification.

Basis:

- [no_std Support Matrix](../../NO_STD.md)
- [Core Trust Freeze Checklist](core_trust_freeze_checklist.md)
- [PCC Practical Core Matrix](practical_core_matrix.md)
- [Public Claim Wording Audit](ctf_public_claim_wording_audit.md)
- [Semantic UI DNA](../../dna/SEMANTIC_UI_DNA.md)

## 1. Executive Summary

The workspace has a real `no_std` and `no-default-features` policy surface, but
current evidence does **not** support a claim of full workspace `no_std`
qualification.

What exists today:

- `docs/NO_STD.md` defines intended build modes and a required check lane.
- CI contains a `cargo check --no-default-features --quiet` job.
- `tools/7hell/run.ps1` does not run a `no-default-features` lane.
- `cargo check --workspace --no-default-features` currently fails.
- `cargo check --workspace --all-features` passes.

Safe overall claim today:

- limited `no_std` posture exists for selected crates;
- `cargo check --no-default-features` is evidence, not proof of full runtime
  qualification;
- full `no_std` qualification remains under separate audit.

Main risk:

- public wording can easily overread the existence of a `no-default-features`
  check as full embedded / release-ready `no_std` support.

## 2. Workspace no_std Evidence

| Evidence source | Command / file | Result | Strength | Notes |
| --- | --- | --- | --- | --- |
| Intent document | `docs/NO_STD.md` | Defines `std`, `no_std`, and `alloc` build modes plus required checks | MEDIUM | Good policy reference, but it is not itself qualification evidence. |
| CI lane | `.github/workflows/ci.yml` | Has `check-no-std` job running `cargo check --no-default-features --quiet` | STRONG | Confirms the workspace expects a `no-default-features` gate. |
| 7hell runner | `tools/7hell/run.ps1` | Does not run a `no-default-features` lane | STRONG | 7hell is useful for PCC gates, but it is not the no_std proof lane. |
| Workspace check | `cargo check --workspace --no-default-features` | Fails today | STRONG | Current failure is the strongest signal that full workspace no_std is not qualified. |
| Workspace check | `cargo check --workspace --all-features` | Passes | STRONG | Confirms the std feature path remains healthy. |

### Current workspace failure summary

The `--no-default-features` workspace lane currently fails in multiple crates,
including:

- `prom-ui` missing `alloc` imports such as `Vec`, `String`, `vec!`, and
  `format!`;
- `sm-verify` using `std::collections::HashSet` under the no-std path;
- `smc-cli` losing `main_entry` when `std` is disabled;
- `prom-state` and `prom-audit` missing `ToString` / `format` imports in their
  no-std path.

That is enough to say the workspace has partial no_std intent, but not full
workspace qualification.

## 3. Crate no_std Posture Matrix

| Crate | Feature posture | no-default-features evidence | std dependency status | Qualification status | Notes |
| --- | --- | --- | --- | --- | --- |
| `ton618-core` | explicit `#![no_std]` + `alloc` gate | `cargo check -p ton618-core --no-default-features` passed | alloc-native, std optional | CHECK-ONLY | This is the cleanest core no_std signal in the workspace. |
| `sm-runtime-core` | explicit `#![no_std]` + `alloc` gate | `cargo check -p sm-runtime-core --no-default-features` passed | alloc-native, std optional | CHECK-ONLY | Runtime vocabulary core compiles in the no-default-features lane. |
| `sm-vm` | explicit `#![no_std]` gate | `cargo check -p sm-vm --no-default-features` passed | core/alloc path works; std is optional | CHECK-ONLY | VM core compiles without default features, but this audit does not claim full runtime qualification. |
| `sm-front` | explicit `#![no_std]` + alloc-capable frontend | `cargo check -p sm-front --no-default-features --features alloc` failed | mixed; alloc path is incomplete | PARTIAL | Missing `alloc` imports and type support under the no-std lane. |
| `sm-sema` | alloc/std split via `sm-front` and `sm-profile` | `cargo check -p sm-sema --no-default-features --features alloc` failed because `sm-front` fails | inherits the frontend alloc gap | PARTIAL | Not fully qualified until the frontend alloc lane is clean. |
| `sm-verify` | explicit `#![no_std]` gate | `cargo check -p sm-verify --no-default-features` failed | std import leak (`std::collections::HashSet`) | PARTIAL | Has a no_std marker, but current source still uses std in the no-default-features lane. |
| `prom-ui` | explicit `#![no_std]` gate | `cargo check -p prom-ui --no-default-features` failed | alloc imports missing in the no-std path | PARTIAL | UI boundary is not no_std-qualified today. |
| `prom-state` | std-first boundary crate | `cargo check -p prom-state --no-default-features` failed | std-dependent formatting / stringification in the no-std lane | PARTIAL | Evidence shows no-default-features gaps rather than qualification. |
| `prom-audit` | std-first boundary crate | `cargo check -p prom-audit --no-default-features` failed | std-dependent formatting / stringification in the no-std lane | PARTIAL | Same pattern as `prom-state`. |
| `smc-cli` | CLI / toolchain surface | `cargo check -p smc-cli --no-default-features` failed | std-only by design for the public binary route | STD-ONLY | The CLI is a host-side tool, not a no_std target. |
| `prom-abi` / `prom-cap` / `prom-gates` / `prom-rules` / `prom-runtime` | host-boundary and policy/runtime orchestration crates | Not separately qualified in this audit | std-first by current manifest layout | STD-ONLY | These crates are not part of the current no_std claim lane. |
| `prom-ui-runtime` / `prom-ui-backend-native` / `prom-ui-demo` | UI orchestration / backend / demo surface | Not separately qualified in this audit | std-first by current manifest layout | STD-ONLY | UI surfaces are not being claimed as no_std-ready here. |
| `semantic-core-backend` / `semantic-core-bench` / `semantic-core-capsule` / `semantic-core-exec` / `semantic-core-quad` / `semantic-core-runtime` | semantic-core support crates | Not separately qualified in this audit | std-first by current manifest layout | STD-ONLY | These crates were not claimed as no_std targets in `docs/NO_STD.md`. |

## 4. Public no_std Claim Audit

### Allowed claims

- `cargo check --no-default-features` exists as a gate or evidence lane.
- Limited `no_std` posture exists for selected crates.
- `no_std` qualification is under audit.
- Some core crates compile with `--no-default-features`.

### Forbidden claims

- full `no_std` qualification;
- embedded-ready;
- complete no_std runtime;
- all workspace crates support `no_std`;
- no_std release-ready;
- no_std readiness as a blanket public promise.

### Current reading

The safe public wording is:

- `Semantic has limited no_std posture in selected core crates, and the
  qualification lane remains under audit.`

That is stronger than “nothing exists” but weaker than “the whole workspace is
no_std-qualified.”

## 5. Freeze Scope Decision

Should full `no_std` qualification block the current Core Trust Freeze?

Current answer:

- No, not by default.
- The current freeze-planning contour is already separated from no_std
  qualification, and the CTF docs do not currently place no_std in the active
  freeze-candidate contour.

However:

- if a future freeze scope explicitly includes no_std readiness, the current
  workspace is not ready yet;
- the failing `cargo check --workspace --no-default-features` lane would be a
  blocker for that broader claim.

## 6. Gaps And Follow-Up Slices

Recommended next slices:

- `CTF-4a`: add or repair an explicit no_std CI lane if the current one is not
  sufficient for the project’s proof standard.
- `CTF-4b`: crate-by-crate no_std cleanup audit for the crates that currently
  fail the lane.
- `CTF-4c`: add no_std compile fixtures for selected core crates if the project
  wants stronger proof than a raw workspace check.
- `CTF-4d`: public no_std wording hardening if README / roadmap / status docs
  start sounding broader than the evidence.

## 7. Final Verdict

The current workspace has a real no_std policy surface and a working CI check
definition, but it does **not** have full workspace no_std qualification yet.

Allowed truth today:

- limited no_std posture exists;
- `--no-default-features` evidence exists;
- no_std qualification remains separate work.

Forbidden truth today:

- full workspace no_std qualification;
- embedded-ready / release-ready no_std claims;
- blanket `all crates support no_std` claims.

Core Trust Freeze remains **not declared complete**.
