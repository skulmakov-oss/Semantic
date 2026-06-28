# CTF Raw Execution Compatibility Perimeter Audit

Status:
  DRAFT / AUDIT ONLY

Core Trust Freeze is **not** declared complete by this document.
This audit inventories the execution perimeter so canonical, compatibility,
raw, and tooling-only routes are not conflated.

Basis:

- [Core Trust Freeze Checklist](core_trust_freeze_checklist.md)
- [CTF Trust-Boundary Guard Audit](ctf_trust_boundary_guard_audit.md)
- [PCC Practical Core Matrix](practical_core_matrix.md)
- [Semantic VM Specification](../../spec/vm.md)
- [Semantic Verifier Specification](../../spec/verifier.md)
- [VM Token-First Policy Guard](../../../tests/vm_token_first_policy_guard.rs)
- [Semantic UI DNA](../../dna/SEMANTIC_UI_DNA.md)

## 1. Executive Summary

The repository has a clear verifier-first public route, but the raw / byte /
compatibility perimeter is spread across multiple crates and CLI commands.

What is canonical and trusted today:

- canonical admission is token-based through `verify_semcode_token`;
- canonical execution is token-based through `VerifiedEntrySemCode` and
  `run_verified_entry_semcode*`;
- the CLI public smoke path (`check`, `compile`, `verify`, `run`, `run-smc`)
  is verifier-first in intent and exercised by the current public CLI matrix.

What is intentionally retained as compatibility or raw lower-level surface:

- `sm-vm` raw helpers that accept bytes directly and bypass verifier
  admission;
- `run_verified_semcode*` wrappers in `sm-vm` and `prom-runtime`, which accept
  bytes but internally verify and then delegate to token execution;
- `smc disasm`, which is artifact inspection only and not trusted execution.

The current risk is not hidden runtime behavior. The risk is wording drift:
raw helper names can be read as canonical unless the compatibility perimeter is
kept explicit in docs.

## 2. Canonical Trusted Route

Current canonical trusted shape:

`source / project root -> check / compile -> SemCode artifact -> verifier ->
VerifiedSemCode / VerifiedEntrySemCode token -> VM execution`

### Canonical entrypoints

- `sm-verify::verify_semcode_token`
- `sm-verify::VerifiedSemCode`
- `sm-verify::VerifiedEntrySemCode`
- `sm-vm::run_verified_entry_semcode*`

### Canonical route evidence

- [docs/spec/vm.md](../../spec/vm.md) states the standard execution rule is
  verifier-first.
- [docs/spec/verifier.md](../../spec/verifier.md) defines
  `verify_semcode_token` as the canonical admission gate.
- [tests/vm_token_first_policy_guard.rs](../../../tests/vm_token_first_policy_guard.rs)
  enforces the token-first policy for new production code.
- [tests/canonical_examples.rs](../../../tests/canonical_examples.rs) and
  [tests/smc_run_smc_cli.rs](../../../tests/smc_run_smc_cli.rs) keep the public
  CLI smoke route exercised.

### Route interpretation

- `smc check` and `smc compile` are canonical source-to-artifact steps.
- `smc verify` is canonical artifact admission.
- `smc run` and `smc run-smc` are canonical public execution routes that remain
  verifier-first even when they delegate to lower-level helpers internally.

## 3. Execution API Inventory

| Crate | API / command | Accepts raw bytes? | Requires Verified token? | Runs verifier internally? | Classification | Risk | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `sm-verify` | `verify_semcode_token` | Yes | No at the API boundary; returns the token | Yes | canonical trusted | LOW | Canonical admission gate; produces `VerifiedSemCode`. |
| `sm-verify` | `verify_semcode` | Yes | No | Yes | verified compatibility helper | MEDIUM | Legacy admission API that returns `VerifiedProgram`; kept for compatibility, not the preferred token-first boundary. |
| `sm-vm` | `run_verified_entry_semcode*` | No at the token boundary; token required | Yes | No | canonical trusted | LOW | Canonical token execution path; does not accept raw bytes. |
| `sm-vm` | `run_verified_semcode*` | Yes | No at the API boundary; internally acquires a token | Yes | verified compatibility helper | MEDIUM | Byte shim family retained for compatibility. |
| `sm-vm` | `run_semcode*` | Yes | No | No | raw lower-level helper | HIGH | Raw SemCode execution path; bypasses verifier admission by design. |
| `sm-vm` | `run_semcode_collecting_hello_observations` | Yes | No | No | raw lower-level helper | HIGH | Diagnostic / observation helper built on the raw execution path. |
| `prom-runtime` | `run_verified_semcode*` | Yes | No at the API boundary; internally acquires a token | Yes | verified compatibility helper | MEDIUM | Public compatibility API; explicitly documented as noncanonical. |
| `smc-cli` | `smc check` | Source / project-root input, not raw bytes | No | Yes, via source admission / analysis | canonical trusted | LOW | Verifier-first source analysis gate. |
| `smc-cli` | `smc compile` | Source / project-root input, not raw bytes | No | Yes, via compiler pipeline | canonical trusted | LOW | Artifact production route. |
| `smc-cli` | `smc verify` | Yes (`.smc` bytes) | No | Yes | canonical trusted | LOW | Artifact admission command; implemented via verification, not execution. |
| `smc-cli` | `smc run` | Source / project-root input, not raw bytes | No | Yes, through verify-before-run helpers | canonical trusted | MEDIUM | Public execution route; internally verifies before the observation/runtime helper runs. |
| `smc-cli` | `smc run-smc` | Yes (`.smc` bytes) | No | Yes, through verify-before-run helpers | canonical trusted | MEDIUM | Artifact execution route; internally verifies before the observation/runtime helper runs. |
| `smc-cli` | `smc disasm` | Yes (`.smc` bytes) | No | No | tooling-only helper | LOW | Artifact inspection only; not a trusted execution route. |

## 4. Token-First Policy

The current token-first contract is:

1. `verify_semcode_token(bytes)`
2. `token.require_entry("main")?`
3. `run_verified_entry_semcode(&entry_token)`

The policy guard in
[tests/vm_token_first_policy_guard.rs](../../../tests/vm_token_first_policy_guard.rs)
exists to prevent new production code from drifting back to the byte-shim
surface when a token path is available.

This means:

- `VerifiedSemCode` and `VerifiedEntrySemCode` are the canonical trust
  boundaries for verified execution;
- byte-based `run_verified_semcode*` wrappers are compatibility helpers, not
  the preferred canonical route for new code;
- raw `run_semcode*` helpers are lower-level and remain outside the canonical
  public contract.

## 5. CLI Public Route

| Route | Steps | Trusted? | Allowed claim | Forbidden claim | Evidence | Follow-up |
| --- | --- | --- | --- | --- | --- | --- |
| `smc check` | source / project-root -> source admission | Yes | Canonical source validation gate | "trusted execution" | `docs/roadmap/pcc/cli_public_sample_qualification_matrix.md`, `tests/canonical_examples.rs` | none |
| `smc compile` | source / project-root -> SemCode artifact | Yes | Canonical artifact producer | "execution" | `docs/roadmap/pcc/cli_public_sample_qualification_matrix.md`, `tests/canonical_examples.rs` | none |
| `smc verify` | `.smc` bytes -> verifier -> admission result | Yes | Canonical artifact admission | "raw execution trusted by default" | `tests/canonical_examples.rs`, `tests/pcc6_option_result_diagnostics.rs`, `tests/smc_run_smc_cli.rs` | keep wording aligned with token-first docs |
| `smc run` | source / project-root -> compile -> verify -> runtime helper | Yes | Canonical public source execution route | "byte-first canonical route" | `tests/canonical_examples.rs`, `tests/cli_public_smoke_matrix.rs` | keep helper inventory explicit |
| `smc run-smc` | `.smc` bytes -> verify -> runtime helper | Yes | Canonical public artifact execution route | "raw byte execution without admission" | `tests/smc_run_smc_cli.rs`, `tests/cli_public_smoke_matrix.rs` | keep helper inventory explicit |
| `smc disasm` | `.smc` bytes -> disassembler | No | Tooling-only inspection | "trusted execution" | `tests/pcc9_project_model_acceptance.rs` | keep tooling wording explicit |

## 6. Documentation Alignment

Current documentation is mostly aligned, but not uniform in wording:

- `docs/spec/verifier.md` is explicit that `verify_semcode_token` is the
  canonical admission gate.
- `docs/spec/vm.md` describes the standard route as
  `verify -> run_verified_semcode* -> execute`, which is correct as a family
  description but can be read too broadly because it includes byte-shim
  helpers.
- `tests/vm_token_first_policy_guard.rs` closes that ambiguity for new
  production code by requiring the token path.
- `smc-cli` command docs and current CLI qualification docs describe the public
  route as verifier-first, but the raw helper inventory is still spread across
  crates.

The audit conclusion is that the route is sound, but the perimeter vocabulary
still benefits from a single inventory page.

## 7. Risk Assessment

| Risk | Level | Why |
| --- | --- | --- |
| Raw helper misread as trusted route | MEDIUM | `run_semcode*` and `run_verified_semcode*` are public-looking names and can blur canonical vs compatibility intent. |
| Compatibility helper used as canonical wording | MEDIUM | `run_verified_semcode*` is a byte shim, but the helper family name can be mistaken for the preferred route. |
| CLI `run` / `run-smc` read as byte-first execution | LOW | The helpers verify first, but the internal raw helper names deserve explicit perimeter wording. |
| `smc disasm` read as execution | LOW | It is inspection only and does not execute admitted code. |
| Docs drift from token-first policy | MEDIUM | The policy is already guarded; the remaining issue is keeping docs and helper names aligned. |

## 8. Deferred Follow-Up Slices

Recommended follow-up work:

- `CTF-2a`:
  centralize the raw / compatibility execution inventory in a single docs
  page or spec appendix.
- `CTF-2b`:
  add wording or annotation hardening where raw helpers could be misread as
  canonical trusted routes.
- `CTF-2c`:
  add or refresh a CLI route note proving the public `run` and `run-smc`
  paths are verifier-first if the wording becomes ambiguous again.
- `CTF-3`:
  public claim wording audit.

Do **not** remove raw APIs in this audit. The current goal is perimeter
clarification, not API deletion.

## 9. Final Verdict

The current execution perimeter is safe but mixed:

- canonical trusted route: qualified
- verified compatibility helpers: retained and documented
- raw lower-level helpers: retained and explicitly noncanonical
- tooling-only helpers: retained and non-execution

Core Trust Freeze remains **not declared complete**.

This audit makes the perimeter readable so later freeze planning can keep the
canonical token-first route distinct from compatibility and raw helper paths.
