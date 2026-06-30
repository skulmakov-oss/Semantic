# sm-vm VM-M14 VM Test Helper Result Surfaces Audit

## Status

VM-M14 is an inspection-only audit.

It does not approve VM behavior changes, verifier changes, SemCode changes, fixture rewrites, public API widening, or optimization selection.

## Context

- VM-M12 audited helper-boundary lowering shape.
- VM-M13 defined the helper-boundary result-equivalence evidence boundary.
- VM-M14 now inspects whether existing VM/test helpers already expose enough observable execution result to compare helper and inline fixture pairs at harness level.

## Problem

Fixture-local assertions are not enough to prove helper-vs-inline pair equivalence at harness level.

Before adding new harness code, we need to inspect whether existing VM/test APIs already expose suitable observable results.

## Existing Result Surfaces

| Surface | Existing API / location | What it proves | What it does not prove | Public or test-only | Suitable for pair equivalence? |
|---|---|---|---|---|---|
| Successful run / exit status | `sm_vm::run_verified_entry_semcode`, `sm_vm::run_verified_semcode`, `sm_vm::run_verified_entry_semcode_with_config`, `sm_vm::run_semcode` in `crates/sm-vm/src/semcode_vm.rs` | The artifact admitted and executed successfully, or failed with a `RuntimeError`. | It does not prove helper and inline variants produced the same observable result. | Public runtime API. | Not by itself. It is only a success/failure surface. |
| Returned value | No general value-returning VM execution API is exposed today for ordinary verified execution. The nearest returned artifact is `run_verified_entry_semcode_with_profile(...) -> Result<VmOpcodeProfile, RuntimeError>`. | The profile path proves opcode-count output for measurement. | It does not prove semantic returned value equality between helper and inline fixtures. | Public feature-gated profile API. | No. The current returned artifact is a profile summary, not the semantic return value. |
| RuntimeError / trap kind | `RuntimeError` in `crates/sm-vm/src/semcode_vm.rs` | The execution failed, and the failure class is observable. | It does not prove successful runs are equivalent, only that failures are classified. | Public runtime API. | Only for negative-path comparison, not positive pair equivalence. |
| Verifier rejection | `sm_verify::verify_semcode_token(...) -> Result<VerifiedSemCode<'_>, RejectReport>` in `crates/sm-verify/src/lib.rs` | Admission succeeded or rejected, and rejection diagnostics are available. | It does not prove runtime equivalence between helper and inline fixtures. | Public verifier API. | No, not for helper-vs-inline equivalence. |
| Disasm text | `sm_vm::disasm_semcode(...)` in `crates/sm-vm/src/semcode_vm.rs` | Bytecode / opcode shape and function names are visible. | It does not prove runtime behavior or result equivalence. | Public diagnostic API behind `disasm`. | No, this is shape comparison only. |
| Host/effect call recording | `sm_vm::run_verified_semcode_with_host_and_capabilities(...)` and the `RecordingHostAbi` test fixtures in `tests/bytecode_compat.rs` / `tests/prometheus_runtime.rs` | Host calls, state query/update, event posts, and clock reads can be recorded. | It does not prove equivalence unless the fixtures intentionally exercise host/effect behavior. | Public runtime API with test harnesses. | Only when a fixture is designed around host/effect observation. Not suitable for the current helper fixtures by default. |
| Trace / observation output | `sm_vm::run_semcode_collecting_hello_observations(...)` in `crates/sm-vm/src/semcode_vm.rs`; test-only execution summaries such as `tests/g1_execution_integrity.rs::execution_summary` | Controlled observations or a test-only pipeline summary can be collected. | It does not prove helper-vs-inline result equality unless the fixture emits a comparable observable. | Mixed: public diagnostic helper plus test-only summary code. | Potentially, but only if the fixture actually produces a shared observable. |

## Candidate Comparison Strategies

| Strategy | What it proves | Exists today? | Requires code changes? | Requires public API widening? | Acceptable for VM-M15? |
|---|---|---|---|---|---|
| A. Returned-value comparison | Helper and inline variants produce the same observable value at the harness boundary. | No general helper-vs-inline value-return surface is exposed today; verified execution APIs are unit-returning. | Yes, unless a private test helper can already extract the needed observable. | Not necessarily, if a private test-only helper is added; yes if a public API is needed. | Yes, if implemented as a narrow test-only helper. Not available today as a generic surface. |
| B. Execution-summary comparison | The two variants converge on the same test-only summary or digest of execution. | Partially. `tests/g1_execution_integrity.rs::execution_summary` shows a test-only pipeline summary pattern, and `VmOpcodeProfile` exists for opcode counts. | Likely yes, unless an existing helper already returns a suitable summary. | No, if kept test-only. | Yes, if the summary is private/test-only and clearly bounded. |
| C. Trap/error-kind comparison | Both variants fail the same way. | Yes. `RuntimeError` and `RejectReport` exist today. | No. | No. | Only as a negative-path guard. Not enough for positive equivalence. |
| D. Final VM state digest | The two variants end in the same summarized VM state. | Not exposed today as a general helper/result surface. | Yes. | Likely yes if the digest must be exposed outside tests. | Conditional. Only acceptable if a private test-only state digest already exists or can be added without public API widening. |
| E. Disasm / bytecode shape comparison | The compiled shape is the same or intentionally different. | Yes, via `disasm_semcode`. | No. | No. | Acceptable only as a compile-shape check, not as pair equivalence. |
| F. Fixture-local assertions only | Each fixture independently reaches its own expected state. | Yes, this is what VM-M12/VM-M13 currently rely on. | No. | No. | Yes as a smoke guard, but too weak as the only evidence boundary for harness-level pair equivalence. |

## Recommended VM-M15 Path

Preferred recommendation:

VM-M15 should first audit whether existing VM test helpers can compare returned values or execution summaries without adding public result-inspection APIs.

Current inspection suggests that `sm-vm` does not expose a general semantic return-value surface for ordinary verified execution today, so the likely next step is a private test-only result observation boundary rather than a public VM API.

Fallback:

If no existing helper path is sufficient, VM-M15 should specify a private test-only result observation boundary before any production API is introduced.

## Non-claims

This audit does not claim:

- helper and inline fixtures are already harness-equivalent;
- VM optimization is approved;
- helper inlining is approved;
- public VM result APIs should be widened;
- SemCode format should change;
- verifier admission should change;
- fixtures should be rewritten in this PR.

## Validation

- `git diff --check`
- `cargo fmt --check`
- `git status --short`

Untracked local artifacts remain outside VM-M14 scope and were not staged.

If `cargo fmt --check` fails, the blocker must be recorded honestly and unrelated files must not be modified.
