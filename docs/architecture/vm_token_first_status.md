# VM Token-First Execution Status

## Executive Verdict
VM verified execution is policy-level token-first, with byte-based verified shims retained strictly as compatibility coverage.

## Canonical Verified Execution Path
The canonical VM execution pipeline requires a cryptographically or structurally verified token prior to execution. The sequence is strictly:
`VerifiedEntrySemCode -> run_verified_entry_semcode* -> prepare_verified_execution -> VmProgramView -> exec_loop`

## Production Guard Status
A strict call-site guard (`require_entry`) is enforced in production paths. Byte-shim execution drift is structurally blocked from entering the production runtime pipeline.

## Compatibility Byte-Shim Policy
Byte-based `run_verified_semcode*` APIs remain as intentionally supported compatibility shims. They are fully tested but are considered secondary to the canonical token-first execution path.
Public `prom-runtime` byte APIs remain as compatibility wrappers, and do not constitute evidence of byte-first VM execution.

## Raw / Diagnostic Path Policy
Raw execution methods such as `run_semcode*` and `disasm_semcode` remain intentionally raw and diagnostic. They are necessary for analyzing compiler output, internal logic, and raw binary formats, and are preserved specifically for these use cases without requiring verified tokens.

## Test Coverage Classification
- **Representative Tests:** Migrated to token-first execution.
- **B-Category / End-to-End Tests:** Selectively migrated to canonical token-first execution where appropriate.
- **A/C Compatibility Tests:** Intentionally documented and preserved to cover byte-shim compatibility.
- **Raw/Diagnostic Paths:** Maintained as unverified to serve diagnostic, legacy, or specific internal testing needs. 

*Not all tests are token-first. Tests specifically targeting raw diagnostics or public byte-shim API compatibility remain on byte-based execution.*

## Explicit No-Go List
- Do not describe `run_token_first_main` as a production canonical entrypoint. It is a testing convenience helper.
- Do not migrate public API byte-shim tests to token-first execution.
- Do not deprecate raw execution (`run_semcode*`) or diagnostic analysis.
- Do not bypass the production guard or attempt to re-introduce raw byte-based execution in the production pipeline.
- Do not consider byte-shim wrappers as evidence against the token-first VM policy.
