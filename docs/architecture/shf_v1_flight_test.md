# SHF-V1 compiler-oriented flight test

## Identity and scope

Base: `3e4802326ef1121090d3b9c90a497df30fd882e8`, tree `22d53b844a6f3353f5d778f8681dbebdddccc627`. Toolchain: Rust 1.97.1. This test corpus is a new reduced compiler workload, with no claim of user-provided MiniLang provenance. All algorithms execute in ordinary Semantic; Rust transports inputs, invokes the compiler/verifier/VM, and compares expected results.

The enclosing Git commit identifies the candidate. No production code, dependency, Harness, workflow, bootstrap pin or published contract changes are part of this deliverable. Refs #2033; parent #1910 remains open.

## Executable programs

* `tests/fixtures/shf_v1/lexer.sm` and `frontend.sm`: UTF-8 byte inspection, ASCII identifiers and decimal lexemes, whitespace/comments, full token stream, precedence parser, append-only indexed AST and declaration-before-use validation. `GRAMMAR.md` freezes the fixture language. It supports let declarations and arithmetic, not assignment, scopes or evaluation. Literals remain source spans. Diagnostics carry byte ranges and deterministic error codes.
* `evidence.sm`: explicit knowledge-lattice join, independently retained positive/negative source provenance, per-observation authorization and rejected-observation quarantine history. All 16 quad pairs are tested; T joined with F yields S. Invalid encoded states return a separate error. T does not authenticate a source; quarantine history does not revoke a later independently authorized observation.
* `structured.sm`: indexed record arena with nested parent links and enum tags, numeric-key maps, persistent sequences/record updates, Option/Result and in-memory Bytes/little-endian round trips. This is a bounded linear-chain model, not a general graph or complete symbol table. Option/Result are function results rather than deeply nested fields; tuple coverage is not established by this fixture.

## Pipeline and golden cases

All successful execution uses `compile_program_to_semcode_with_options` at O0/O1, `verify_semcode_token`, entry selection and verified VM APIs. No Rust guest parser/evaluator or host-side symbol-table algorithm is present. Compilation repeats compare complete bytes within an optimization level. O0/O1 must preserve semantic outcomes; their bytes and counters need not match.

| Program | Positive goldens | Negative/boundary goldens |
| --- | --- | --- |
| Frontend | precedence AST: 6 nodes for `let x=1+2*3;`; 30-term expression: 60 nodes; medium declaration/reference program | duplicate/reserved name 3; unknown/self-reference 4; syntax/EOF 2; invalid UTF-8 byte 1; depth/input limit 6 |
| Evidence | all 16 joins; eight provenance/quarantine scenarios; contradictory sources retain S | unauthorized observations excluded; duplicate provenance deduplicated; encoded states 4/u32::MAX rejected |
| Structured | sizes 0,1,2,8,32,64 return 0,0,3,84,1488,6048; u32 byte round trips including maximum | negative/count65/maximum rejected; short/overflow byte reads None; invalid byte256 and u16 value65536 rejected |

Length above 4096 bytes is rejected before lexical analysis with limit code 6 and an empty span at input length. An early invalid character does not override this admission check. Source offsets refer to bytes, not graphemes.

## Measured resource behavior (OBSERVED / REPRODUCED)

The existing application-host metered API reports counters for pure main wrappers; the test host panics if any host method is reached. Repeated executions require identical counters and zero EffectCalls. These wrappers include assertions and entry preparation; the counters are not whole-process RSS or elapsed-time measurements.

| Workload | Steps O0 / O1 | Calls | Peak Frames |
| --- | ---: | ---: | ---: |
| scanner 64 bytes | 5356 / 5353 | 261 | 3 |
| frontend 30 terms / 60 nodes | 22142 / 21666 | 797 | 6 |
| evidence scenario | 406 / 387 | 13 | 4 |
| structured 1 node | 405 / 395 | 5 | 3 |
| structured 8 nodes | 1476 / 1445 | 12 | 3 |
| structured 32 nodes | 5148 / 5045 | 36 | 3 |
| structured 64 nodes | 10044 / 9845 | 68 | 3 |

The metrics test prints SHA-256 for each complete injected source and emitted SemCode. Windows and WSL Ubuntu-24.04 executions produced matching printed artifact hashes and counters for these cases. Source fixture hashes can be reproduced with `sha256sum tests/fixtures/shf_v1/*.sm`; injected source hashes intentionally differ from individual fixture hashes.

Default VerifiedLocal is 100000 Steps and 16384 Calls. The 4096-space input, proposed 508-node input and token-boundary input reproducibly stop at Steps=100001 with limit100000. These are expected quota failures, not qualified structural boundary successes. The 511-token guard is statically present but its boundary is not dynamically established under that envelope. Increasing quotas to hide this observation is not part of this test. No production defect is established by ordinary bounded exhaustion.

ARG tests use existing explicit policy entrypoints for fixed god and custom adaptive, compare results/events, and reject insufficient authorization. They do not establish ARG support for every CLI/application-host path. Physical-memory bounds, global effect rollback and absence of copying amplification are not proven.

## Qualification

Orchestrator independently inspected the code and ran the tests. Separate Terra reviewers identified and corrected malformed-state admission, length-priority documentation and adaptive-max test coverage. This is local independent-agent review, not a formal GitHub approval.

Windows commands (native exit0 unless noted):

```text
cargo test --test shf_v1_flight_test --test shf_v1_frontend --test shf_v1_evidence --test shf_v1_structured --test shf_v1_metrics -- --nocapture
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
pwsh -NoProfile -File scripts/workspace_fmt_check.ps1
pwsh -NoProfile -File scripts/harness-check.ps1
git diff --check
```

The targeted command has 8 tests across five binaries. Workspace pass precedes the final test-only assertions; impacted targeted tests are rerun after corrections. The workspace run includes sm-vm tests. WSL Ubuntu-24.04 ran the same targeted command under Rust1.97.1, exit0, and reran the final adaptive-max assertions successfully. Full Linux workspace qualification is NOT RUN. Windows cargo fmt has the documented command-line-length restriction; repository workspace formatter is used. Full `cargo +1.97.1 fmt --all -- --check` also passed in the Linux copy. Hosted checks remain pending publication and cannot be inferred from local checks.

## Limitations and next step

REPRODUCED: standard Steps exhaustion precedes several intended frontend structural stress boundaries. INFERRED: repeated immutable-state transport and helper calls may contribute to cost; attribution requires a separate measured investigation. NOT RUN: physical heap accounting, full Linux workspace, other architectures, generic self-coverage and bootstrap fixed point.

The next bounded step is to profile the larger frontend workloads within an explicitly qualified resource envelope, retain byte spans and semantic results, and decide whether measured copy/call costs justify a production correction. Broadening MiniLang grammar is separate from this frozen corpus.

This delivers compiler-oriented execution evidence. It establishes no C1/C2 equivalence, Bootstrap Seal, complete SHF stage, unrestricted language completeness, release promotion or self-hosting completion.
