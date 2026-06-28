# CTF Trust-Boundary Guard Audit

Status:
  DRAFT / AUDIT ONLY

Core Trust Freeze is **not** declared complete by this document.
This audit checks whether the current freeze-candidate contour is mechanically
guarded, documented only, or still needs follow-up.

Basis:

- [Core Trust Freeze Checklist](core_trust_freeze_checklist.md)
- [PCC Practical Core Matrix](practical_core_matrix.md)
- [Runtime Ownership Conservative Contour Closeout](runtime_ownership_conservative_contour_closeout.md)
- [Sequence Conservative Ownership Contour Closeout](sequence_conservative_ownership_contour_closeout.md)
- [ADT Payload Ownership Paths](../../architecture/adt_payload_ownership_paths.md)
- [Semantic UI DNA](../../dna/SEMANTIC_UI_DNA.md)

## 1. Executive Summary

The repository already has meaningful mechanical guard coverage for the core
SemCode / VM / capability / dependency boundaries.

What is already guarded:

- `sm-vm` does not show `prom-ui` or `sm-ir` in its normal dependency graph,
  and the repo has explicit guard tests for that boundary.
- `prom-cap` does not show `prom-ui` in its normal dependency graph, and the
  repo has an explicit guard test for that boundary.
- `sm-format`, `sm-verify`, and `sm-vm` remain on the verified / format-owner
  side of the SemCode boundary in the current graph and spec docs.
- The canonical public execution route is documented as verifier-first and is
  exercised by the current CLI / 7hell paths.

What is documented but not fully mechanically guarded:

- raw / compatibility byte-entry helpers in `prom-runtime` and `sm-vm`;
- `sm-format` and `sm-verify` staying off `sm-ir` by explicit normal-graph
  tests rather than a dedicated crate-local dependency guard;
- the public claim boundary, which remains doc-driven rather than test-driven;
- the UI / Workbench authority boundary, which is defined in DNA/spec docs but
  not by a repo-wide mechanical guard in this audit.

This audit does **not** promote Core Trust Freeze. It identifies where a
follow-up guard PR is still warranted.

## 2. A. VM Trust Boundary

### Current graph result

`cargo tree --edges normal -p sm-vm` shows direct normal dependencies on:

- `prom-abi`
- `prom-cap`
- `sm-format`
- `sm-runtime-core`
- `sm-verify`

It does **not** show:

- `prom-ui`
- `sm-ir`
- `sm-emit`
- `sm-front`
- `sm-sema`

### Existing guard coverage

- [tests/trust_boundary_guards.rs](../../../tests/trust_boundary_guards.rs)
  checks that `sm-vm` does not depend on `prom-ui` or `sm-ir`.
- [tests/dependency_boundaries.rs](../../../tests/dependency_boundaries.rs)
  checks that execution crates do not depend on frontend / sema crates.
- [tests/vm_token_first_policy_guard.rs](../../../tests/vm_token_first_policy_guard.rs)
  checks production files for byte-shim usage of `run_verified_semcode*`.
- [docs/roadmap/language_maturity/runtime_boundary_hardening.md](../../language_maturity/runtime_boundary_hardening.md)
  documents the verified-only execution route and the intended runtime
  boundary.

### Missing guard coverage

- There is no dedicated explicit assertion that `sm-vm` normal graph excludes
  `sm-emit`.
- The dev graph still includes `sm-emit` for tests, so the normal-graph /
  dev-graph distinction should remain obvious in future audits.

### Risk level

MEDIUM

### Recommended follow-up

- `CTF-1a`: add or strengthen a dependency guard for the `sm-vm` normal graph
  excluding `sm-emit`, and keep the dev-dependency explanation explicit.

## 3. B. Capability Boundary

### Current graph result

`cargo tree --edges normal -p prom-cap` shows a single normal dependency on
`prom-abi`.

It does **not** show `prom-ui`.

### Existing guard coverage

- [tests/trust_boundary_guards.rs](../../../tests/trust_boundary_guards.rs)
  checks that `prom-cap` does not depend on `prom-ui` and only depends on
  `prom-abi` among `prom-*` crates.
- [docs/spec/ui_abi_capability_admission.md](../../spec/ui_abi_capability_admission.md)
  and [docs/spec/ui_contract_map.md](../../spec/ui_contract_map.md) place UI
  capability identity above `prom-cap`, not inside it.

### Missing guard coverage

- No additional mechanical guard was found in this audit beyond the
  dependency-boundary test.

### Risk level

LOW

### Recommended follow-up

- Keep the current `prom-cap` dependency guard test as the canonical boundary
  check.
- If UI capability mapping widens, refresh the UI capability docs before any
  freeze claim changes.

## 4. C. SemCode Format Authority Boundary

### Current graph result

- `cargo tree --edges normal -p sm-format` is empty beyond the crate itself.
- `cargo tree --edges normal -p sm-verify` depends on `sm-format` and
  `sm-runtime-core`, not `sm-ir`.
- `cargo tree --edges normal -p sm-vm` depends on `sm-format` and
  `sm-verify`, not `sm-ir`.

### Existing guard coverage

- [tests/frontend_boundaries.rs](../../../tests/frontend_boundaries.rs)
  checks that `sm-emit` re-exports the canonical SemCode contract from
  `sm-format` and does not create a second local owner.
- [docs/spec/vm.md](../../spec/vm.md) and [docs/spec/verifier.md](../../spec/verifier.md)
  describe the verifier-first route over verified SemCode.
- [docs/roadmap/language_maturity/runtime_boundary_hardening.md](../../language_maturity/runtime_boundary_hardening.md)
  states that `sm-vm` is execution only and `prom-runtime` is orchestration only.

### Missing guard coverage

- There is no dedicated crate-local dependency guard that explicitly asserts
  `sm-format !-> sm-ir`.
- There is no dedicated crate-local dependency guard that explicitly asserts
  `sm-verify !-> sm-ir` or `sm-verify !-> sm-emit` in the normal graph.

### Risk level

MEDIUM

### Recommended follow-up

- `CTF-1a`: add explicit normal-graph dependency guard tests for
  `sm-format !-> sm-ir` and `sm-verify !-> sm-ir`.

## 5. D. Verifier-First Execution Boundary

### Canonical verified path evidence

Current public execution route is documented as:

`verify -> run_verified_semcode* -> execute`

Evidence:

- [docs/spec/vm.md](../../spec/vm.md)
- [docs/spec/verifier.md](../../spec/verifier.md)
- [docs/roadmap/language_maturity/runtime_boundary_hardening.md](../../language_maturity/runtime_boundary_hardening.md)
- [tests/vm_token_first_policy_guard.rs](../../../tests/vm_token_first_policy_guard.rs)
- [tests/canonical_examples.rs](../../../tests/canonical_examples.rs)
- [tests/smc_run_smc_cli.rs](../../../tests/smc_run_smc_cli.rs)

### Raw / compatibility API inventory

Public or narrow-compatibility helpers that still exist:

- `crates/sm-vm/src/semcode_vm.rs`
  - `run_semcode`
  - `run_semcode_with_entry`
  - `run_semcode_with_config`
  - `run_semcode_collecting_hello_observations`
  - `run_verified_semcode`
  - `run_verified_semcode_entry`
  - `run_verified_semcode_with_entry`
  - `run_verified_semcode_with_config`
  - `run_verified_semcode_with_host_and_capabilities[_and_config]`
  - token-based `run_verified_entry_semcode*`
- `crates/prom-runtime/src/lib.rs`
  - public compatibility APIs `run_verified_semcode`
  - public compatibility APIs `run_verified_semcode_entry`

### Documentation status

- The compatibility helpers are explicitly documented as compatibility or
  lower-level routes, not as the canonical public contract.
- The verifier-first route is the canonical route in docs/spec and in CLI
  qualification docs.

### Missing guard coverage

- No dedicated audit was found that inventories every raw / compatibility byte
  helper and classifies each one as canonical, compatibility-only, or
  test-only.
- The boundary is therefore documented and partially enforced, but not fully
  perimeter-audited.

### Risk level

MEDIUM

### Recommended follow-up

- `CTF-2`: raw execution compatibility perimeter audit.

## 6. E. PROMETHEUS Boundary

### Current dependency / source evidence

`cargo tree --edges normal -p prom-runtime` shows orchestration over:

- `prom-abi`
- `prom-audit`
- `prom-cap`
- `prom-gates`
- `prom-rules`
- `prom-state`
- `sm-runtime-core`
- `sm-verify`
- `sm-vm`

It does **not** show `prom-ui`.

### Existing documentation coverage

- [docs/roadmap/language_maturity/runtime_boundary_hardening.md](../../language_maturity/runtime_boundary_hardening.md)
  states that `prom-runtime` orchestrates verified execution sessions only and
  must not become a second execution authority.
- [crates/prom-runtime/src/lib.rs](../../../crates/prom-runtime/src/lib.rs)
  documents the byte-based verified compatibility APIs as compatibility-only
  helpers.

### Missing guard coverage

- No dedicated dependency-boundary test was found specifically for
  `prom-runtime` vs `prom-ui`.
- The orchestration boundary is mostly documented, with the general dependency
  shape visible in `cargo tree`.

### Risk level

MEDIUM

### Recommended follow-up

- `CTF-5`: PROMETHEUS orchestrator boundary audit.

## 7. F. Public Claim Boundary

### Allowed claims

- Practical core has a qualified baseline.
- Runtime ownership has a conservative qualified contour.
- SemCode format authority is split into `sm-format`.
- Dynamic sequence ownership is safe but conservative.
- Some areas are `READY`, some `CONSERVATIVE`, some `PARTIAL`, some
  `DEFERRED`.

### Forbidden claims

- Core Trust Freeze complete.
- stable release readiness.
- full language completion.
- full symbolic alias precision.
- iterator / range ownership complete.
- full no_std qualification beyond evidence.

### Suspicious wording

No new suspicious claim wording was introduced by this audit. The main risk is
claim drift if later docs flatten `READY` into `freeze complete` or treat
`DEFERRED` as if it were qualified.

### Existing guard coverage

- [docs/roadmap/pcc/core_trust_freeze_checklist.md](core_trust_freeze_checklist.md)
  explicitly separates freeze candidates, blockers, deferred areas, and
  forbidden claims.
- [docs/roadmap/pcc/practical_core_matrix.md](practical_core_matrix.md)
  keeps `READY` / `CONSERVATIVE` / `PARTIAL` / `DEFERRED` / `UNKNOWN` /
  `OUT OF SCOPE` distinct.
- [docs/roadmap/pcc/runtime_ownership_conservative_contour_closeout.md](runtime_ownership_conservative_contour_closeout.md)
  and [docs/roadmap/pcc/sequence_conservative_ownership_contour_closeout.md](sequence_conservative_ownership_contour_closeout.md)
  explicitly defer symbolic precision.

### Risk level

LOW

### Recommended follow-up

- `CTF-3`: public claim wording audit, if later docs begin to compress
  qualified / conservative / deferred distinctions.

## 8. Guard Matrix

| Boundary | Expected invariant | Current evidence | Existing guard | Gap | Risk | Follow-up |
| --- | --- | --- | --- | --- | --- | --- |
| `sm-vm !-> prom-ui` | VM normal graph must not include UI crates | `cargo tree --edges normal -p sm-vm` omits `prom-ui` | `tests/trust_boundary_guards.rs::sm_vm_dependency_boundaries` | none in normal graph; keep explicit source guard | LOW | keep current guard |
| `sm-vm !-> sm-ir` | VM normal graph must not include compiler IR crate | `cargo tree --edges normal -p sm-vm` omits `sm-ir` | `tests/trust_boundary_guards.rs::sm_vm_dependency_boundaries` | none in normal graph | LOW | keep current guard |
| `sm-vm !-> sm-emit` in normal graph | VM normal graph must not include SemCode producer facade | `cargo tree --edges normal -p sm-vm` omits `sm-emit`; dev graph includes `sm-emit` | none dedicated | explicit normal-graph guard missing | MEDIUM | `CTF-1a` dependency guard |
| `prom-cap !-> prom-ui` | capability core must not depend on UI | `cargo tree --edges normal -p prom-cap` shows only `prom-abi` | `tests/trust_boundary_guards.rs::prom_cap_dependency_boundaries` | none | LOW | keep current guard |
| `sm-format !-> sm-ir` | SemCode format owner must not depend on IR owner | `cargo tree --edges normal -p sm-format` is self-contained | none dedicated | explicit dependency guard missing | MEDIUM | `CTF-1a` dependency guard |
| `sm-verify !-> sm-ir` in normal graph | verifier must consume SemCode, not IR internals | `cargo tree --edges normal -p sm-verify` shows `sm-format` + `sm-runtime-core` only | none dedicated | explicit dependency guard missing | MEDIUM | `CTF-1a` dependency guard |
| `sm-verify !-> sm-emit` in normal graph | verifier must not depend on producer facade | `cargo tree --edges normal -p sm-verify` omits `sm-emit` | none dedicated | explicit dependency guard missing | MEDIUM | `CTF-1a` dependency guard |
| verified execution path remains canonical | public route must remain verifier-first | `docs/spec/vm.md`, `docs/spec/verifier.md`, `runtime_boundary_hardening.md`, `cli_public_sample_qualification_matrix.md` | `tests/vm_token_first_policy_guard.rs`, `tests/canonical_examples.rs`, `tests/smc_run_smc_cli.rs` | raw / compatibility helpers remain documented perimeter | MEDIUM | `CTF-2` raw compatibility perimeter audit |
| raw execution compatibility perimeter explicitly documented | compatibility helpers are not canonical trust claims | `crates/prom-runtime/src/lib.rs`, `crates/sm-vm/src/semcode_vm.rs` comments | docs/spec + token-first guard | helper inventory not fully classified in one place | MEDIUM | `CTF-2` raw compatibility perimeter audit |
| PCC matrix does not declare Core Trust Freeze complete | docs must not widen into freeze claim | `core_trust_freeze_checklist.md`, `practical_core_matrix.md` | docs-only policy | none; claim drift is the only risk | LOW | `CTF-3` claim wording audit if needed |
| runtime ownership contour does not claim symbolic alias precision | conservative contour stays conservative | runtime ownership closeout docs | docs-only evidence + runtime tests | no dedicated anti-overclaim test | LOW | `CTF-3` claim wording audit if wording drifts |
| UI / Workbench not authority | UI remains projection / operator surface | `docs/dna/SEMANTIC_UI_DNA.md`, UI spec docs | docs/spec + DNA doctrine | not mechanically enforced by one repo-wide test | MEDIUM | `CTF-5` if UI boundary becomes freeze-critical |

## 9. Recommended Next Steps

Recommended follow-up slices are small guard or audit passes only:

- `CTF-1a`:
  add or strengthen dependency guard tests where the current invariant is only
  evidenced by graph inspection.
- `CTF-2`:
  raw execution compatibility perimeter audit.
- `CTF-3`:
  public claim wording audit.
- `CTF-4`:
  no_std qualification audit.
- `CTF-5`:
  PROMETHEUS orchestrator boundary audit.

Do not recommend language expansion as a Core Trust Freeze blocker unless the
freeze scope explicitly pulls that surface in.

## 10. Final Verdict

The freeze-candidate contour is already partially mechanically guarded and
partially policy-documented.

The repository is **not** yet at Core Trust Freeze complete.

The next useful work is to strengthen the few normal-graph and compatibility
perimeter gaps without widening claims.
