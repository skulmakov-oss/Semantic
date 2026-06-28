# Raw Execution Compatibility Inventory

Status:
  CANONICAL INVENTORY / CTF PLANNING SUPPORT

Core Trust Freeze:
  NOT DECLARED COMPLETE

This document classifies execution routes by trust class.
It does not change API behavior.
It does not remove compatibility helpers.
It does not declare freeze complete.

## 1. Trust Class Definitions

### Canonical Trusted

Verifier-first route.
Requires admission through `sm-verify`.
Produces or consumes `VerifiedSemCode` / `VerifiedEntrySemCode` or an
equivalent verified token.
May be described as the trusted execution route.

### Verified Compatibility

Compatibility or wrapper helper that still performs verifier admission or
accepts already verified artifacts.
May be safe, but must not be described as the main canonical route unless the
docs explicitly say so.

### Raw Lower-Level

Accepts raw SemCode bytes or bypasses the explicit public verifier-first route.
Useful for tests, internal mechanics, compatibility, or lower-level VM access.
Must not be described as trusted canonical execution.

### Tooling-Only

Inspects artifacts but does not execute trusted code.
Example: disassembly.

### Test/Internal

Used by tests or fixtures.
Must not be presented as a public trusted execution surface.

### Unknown

Evidence is insufficient.
Must not be promoted until classified.

## 2. Canonical Route Summary

The canonical trusted execution shape is:

`source / project root -> check / compile -> SemCode artifact -> verify -> VerifiedSemCode / VerifiedEntrySemCode -> VM execution`

Core API names supporting that route:

- `verify_semcode_token`
- `VerifiedSemCode`
- `VerifiedEntrySemCode`
- `run_verified_entry_semcode*`

The canonical public route remains verifier-first. Raw helper families exist,
but they do not redefine the public trust boundary.

## 3. Central Execution Inventory

| Crate / surface | API / command | Accepts raw bytes? | Requires verified token? | Runs verifier internally? | Trust class | Allowed claim | Forbidden claim | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `sm-verify` | `verify_semcode_token` | Yes | No at the API boundary; returns the token | Yes | Canonical Trusted | Canonical admission gate | Trusted execution without admission | Produces `VerifiedSemCode`. |
| `sm-verify` | `verify_semcode` | Yes | No | Yes | Verified Compatibility | Legacy admission API | Main canonical boundary | Returns `VerifiedProgram`; kept for compatibility. |
| `sm-verify` | `VerifiedSemCode` | No | N/A | N/A | Canonical Trusted | Verified admission artifact | Raw bytes are trusted by default | Canonical token type. |
| `sm-verify` | `VerifiedEntrySemCode` | No | N/A | N/A | Canonical Trusted | Verified entry artifact | Raw bytes are trusted by default | Canonical token type for entry execution. |
| `sm-vm` | `run_verified_entry_semcode*` | No at the token boundary | Yes | No | Canonical Trusted | Trusted token execution route | Raw byte execution without admission | Canonical VM entry path. |
| `sm-vm` | `run_verified_semcode*` | Yes | No at the API boundary; acquires a token internally | Yes | Verified Compatibility | Verified byte shim | Primary canonical route | Byte-based compatibility wrapper. |
| `sm-vm` | `run_semcode` | Yes | No | No | Raw Lower-Level | Low-level raw VM helper | Trusted canonical execution | Bypasses verifier admission by design. |
| `sm-vm` | `run_semcode_with_entry` | Yes | No | No | Raw Lower-Level | Low-level raw VM helper | Trusted canonical execution | Raw entry helper. |
| `sm-vm` | `run_semcode_with_config` | Yes | No | No | Raw Lower-Level | Low-level raw VM helper | Trusted canonical execution | Raw configured helper. |
| `sm-vm` | `run_semcode_collecting_hello_observations` | Yes | No | No | Raw Lower-Level | Diagnostic / observation helper | Trusted canonical execution | Raw observation helper. |
| `prom-runtime` | `run_verified_semcode` | Yes | No at the API boundary; acquires a token internally | Yes | Verified Compatibility | Public compatibility wrapper | Primary canonical route | Verified wrapper around admitted execution. |
| `prom-runtime` | `run_verified_semcode_entry` | Yes | No at the API boundary; acquires a token internally | Yes | Verified Compatibility | Public compatibility wrapper | Primary canonical route | Entry-targeted compatibility wrapper. |
| `smc-cli` | `smc check` | Source / project-root input | No | Yes, via source admission / analysis | Canonical Trusted | Canonical source validation route | Trusted execution claim for raw bytes | Public verifier-first source path. |
| `smc-cli` | `smc compile` | Source / project-root input | No | Yes, via compiler pipeline | Canonical Trusted | Canonical artifact producer | Execution claim | Produces SemCode artifact. |
| `smc-cli` | `smc verify` | Yes (`.smc` bytes) | No | Yes | Canonical Trusted | Canonical artifact admission | Raw execution trusted by default | Artifact verification command. |
| `smc-cli` | `smc run` | Source / project-root input | No | Yes, through verify-before-run helpers | Canonical Trusted | Canonical public source execution route | Byte-first canonical route | Public route remains verifier-first. |
| `smc-cli` | `smc run-smc` | Yes (`.smc` bytes) | No | Yes, through verify-before-run helpers | Canonical Trusted | Canonical public artifact execution route | Raw byte execution without admission | Public route remains verifier-first. |
| `smc-cli` | `smc disasm` | Yes (`.smc` bytes) | No | No | Tooling-Only | Artifact inspection | Trusted execution | Disassembler / inspection route only. |

## 4. Allowed And Forbidden Claims By Trust Class

| Trust class | Allowed claim | Forbidden claim |
| --- | --- | --- |
| Canonical Trusted | verifier-first trusted route | raw bytes are trusted without admission |
| Verified Compatibility | compatibility helper around a verified/admitted route | primary canonical route if not documented as such |
| Raw Lower-Level | low-level helper / compatibility surface | trusted execution route |
| Tooling-Only | artifact inspection | execution |
| Test/Internal | test fixture helper | public trust route |
| Unknown | requires audit | ready / trusted |

## 5. Current Weak Or Ambiguous Areas

The following wording remains easy to misread if docs drift:

- `run_semcode*` names can be mistaken for trusted canonical execution.
- `run_verified_semcode*` names are compatibility helpers and can sound more
  canonical than they are.
- `smc run` / `smc run-smc` are public verifier-first routes, but they may call
  lower-level helpers internally.
- `smc disasm` is tooling-only inspection, not execution.
- `verify_semcode` is a legacy admission API and should not be presented as the
  preferred token-first boundary for new production code.

## 6. Wording Policy

Use the following wording consistently across specs and roadmap docs:

- canonical trusted route: verifier-first and token-first execution through
  `verify_semcode_token` and `Verified*` token paths;
- verified compatibility helper: a wrapper around admitted execution that
  remains compatible, but is not the preferred new canonical boundary;
- raw lower-level helper: a byte-oriented or internal surface that must not be
  described as trusted canonical execution;
- tooling-only route: inspection or disassembly only, not execution.

## 7. Follow-Up Recommendations

Recommended next slices:

- `CTF-2b`:
  wording hardening for raw and verified compatibility helpers.
- `CTF-2c`:
  optional tests or notes proving the public CLI run path remains verifier-first
  if wording becomes ambiguous again.
- `CTF-3`:
  public claim wording audit across roadmap and surface docs.

Do not remove raw APIs in this doc. The purpose here is perimeter clarity, not
API deletion.

## 8. Final Verdict

The execution perimeter now has a canonical inventory:

- canonical trusted route: verifier-first token route;
- verified compatibility route: retained wrappers around admitted execution;
- raw lower-level route: explicit byte helpers;
- tooling-only route: artifact inspection.

Core Trust Freeze remains **not declared complete**.
