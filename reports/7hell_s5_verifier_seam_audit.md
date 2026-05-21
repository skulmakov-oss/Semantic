# 7HELL-S5 Verifier Seam Audit

Status: audit-only
Scope: locate verifier-stage seam before execution
Non-goal: implementation, VM execution, project-root, readiness, or CTF closure

## Findings

### Compile-to-SemCode seam

Exact source-to-SemCode APIs already exist in `sm-ir` and are re-exported by `sm-emit`:

- `sm_emit::compile_program_to_semcode(input: &str) -> Result<Vec<u8>, FrontendError>`
- `sm_emit::compile_program_to_semcode_with_options_debug(input: &str, profile: CompileProfile, opt: OptLevel, debug_symbols: bool) -> Result<Vec<u8>, FrontendError>`

What the seam does:

- accepts source text directly; no path, package, or project-root context is required at the API boundary
- performs frontend compilation internally (`compile_program_to_immutable_ir`), then IR validation, then SemCode emission
- emits deterministic bytes for the same source/profile/options; this is already covered by golden-byte and compat tests
- does not include timing, host-path data, or debug symbols by default when `debug_symbols = false`

Safest options for a future S5 route:

- use a fixed profile and optimization level
- keep `debug_symbols = false`
- avoid any command path that reads via package admission or cache plumbing

### Verifier seam

Exact verifier API:

- `sm_verify::verify_semcode(bytes: &[u8]) -> Result<VerifiedProgram, RejectReport>`

What the seam does:

- accepts SemCode bytes only; no source, path, package, or project-root context
- does not run VM execution
- does not require runtime host capability context
- returns stable success data: `header` and `functions` (`name`, `code_len`, `string_count`, `debug_symbol_count`)
- returns stable failure data in `RejectReport`: `VerificationCode`, `function`, `offset`, `message`

### CLI command routes

- `cmd_compile` compiles source to SemCode bytes and writes a `.smc` file; it also reads source through package-admission plumbing and can print metrics/debug output
- `cmd_hash_smc` compiles source to SemCode bytes through cache/graph plumbing; unsafe for S5 because cache and graph state are part of the route
- `cmd_verify` is verifier-only on existing `.smc` bytes; it does not call VM
- `cmd_run` compiles source, then verifies, then runs VM through `render_controlled_observation_envelope`
- `cmd_run_smc` verifies existing `.smc` bytes and then runs VM through `render_controlled_observation_envelope`

Unsafe routes to reject for S5:

- `smc_cli::run(["compile", ...])`
- shelling out to `smc`
- temporary `.smc` files
- cache-pack driven report generation
- `cmd_run` / `cmd_run_smc`
- `run_semcode_collecting_hello_observations`
- project-root or `semantic.toml`
- timing or metrics in stable report output
- absolute paths in stable report output

### Safe candidate route

Proposed future S5 route only:

`read source as UTF-8 -> semantic_check_source -> compile source to SemCode bytes with fixed profile/options -> verify_semcode(bytes) -> populate Verifier Hell only -> keep VM Hell blocked -> keep Practical Hell blocked -> result remains incomplete unless verifier fails`

This preserves verifier-first discipline and keeps VM execution out of the route.

### Required S5 guardrails

- compile-to-SemCode only after Syntax/Type pass
- verifier only on emitted SemCode bytes
- no VM
- no `.smc` temp files
- no cache
- deterministic output
- no final PASS while VM/Practical remain blocked
- keep `--project` rejected
- keep `target.kind = "single-file"`
- add or adjust snapshots for the new verifier stage

## Verdict

S5 verdict: GO

Reason:
Safe seam exists.

- compile API: `sm_emit::compile_program_to_semcode_with_options_debug(...)` or `sm_emit::compile_program_to_semcode(...)`
- verifier API: `sm_verify::verify_semcode(bytes)`
- no VM in verifier seam
- no project-root in either seam
- future PR shape: `7HELL-S5 — cli(7hell): add verifier stage execution for selected single-file fixtures`
