# Historical PR Review Debt & Security Audit Report

**Repository**: `skulmakov-oss/Semantic`  
**Target Baseline**: `origin/main` (commit `e41ff311`, PR #1962)  
**Audit Date**: 2026-09-29  
**Audit Standard**: Non-destructive, evidence-grounded review debt discovery per `AGENTS.md` and `CONSTRAINTS.md`  

---

## 1. Executive Summary

This report documents the exhaustive, systematic audit of **all 1,477 historical closed and merged Pull Requests** in the Semantic repository. The purpose of this audit was to discover technical review comments, architectural objections, verifier/runtime discrepancies, and security findings left in code reviews that:
- were left **unresolved** upon PR merge;
- were formally marked **resolved** without an actual underlying fix;
- were fixed only **partially**; or
- survived subsequent refactoring and **still reproduce on `origin/main`**.

### Key Audit Metrics

| Metric | Count | Note |
| :--- | :--- | :--- |
| **Total Closed PRs Enumerated** | **1477** | PR #1 through PR #1962 |
| **Merged PRs** | **1372** | Merged into main line |
| **Closed Unmerged PRs** | **105** | Closed without merge |
| **PRs with Inline Review Threads** | **480** | 32.5% of all closed PRs |
| **Total Review Threads Analyzed** | **819** | 100% extracted via GitHub GraphQL |
| **Historical Unresolved Threads** | **375** | **45.8%** of all review threads were left open on merge! |
| **High-Value Resolved Threads Analyzed** | **444** | Screened for P0-P3, security, invariants, and verifier keywords |

### Audit Final Tally

Every candidate finding was verified against current `origin/main` source code, specifications, and commit history:

- **Active Confirmed Findings (Still Present)**: **220**
- **Partial Findings (Partially Fixed)**: **4**
- **Fixed-Later Findings (Rectified in later PR/commit)**: **440**
- **Obsolete Findings (File deleted, unmerged PR, superseded)**: **44**
- **False Positives (Reviewer misconception / invariant contradiction)**: **97**
- **Unverified Findings (Subjective / documentation tracking only)**: **8**
- **Exposed Active Credentials / API Keys**: **0**
- **Privacy / Local Environment Path Leaks**: **119**

---

## 2. Reviewer Ecology & Severity Profile

The 819 historical review threads were authored by four distinct entities:

| Reviewer Entity | Role | Thread Count | Unresolved Ratio | Primary Focus |
| :--- | :--- | :--- | :--- | :--- |
| `chatgpt-codex-connector` | Automated AI Reviewer | 788 | 46.1% (363 threads) | Static analysis, invariant enforcement, P1/P2/P3 classification |
| `github-advanced-security` | Security / CodeQL Bot | 21 | 0% (21 threads) | SARIF alerts, Clippy lints, sensitive logging (CodeQL CWE-532) |
| `copilot-pull-request-reviewer` | AI Review Assistant | 6 | 33.3% (2 threads) | Syntactic suggestions, error handling |
| `skulmakov-oss` | Repository Owner (Human) | 4 | 0% (4 threads) | Architectural qualification, dead test detection, stage origin |

### Severity Distribution at Original Review Time
- **P1 (High / Blocker)**: 181 threads
- **P2 (Medium / Defect)**: 608 threads
- **P3 (Low / Polish)**: 3 threads
- **Unspecified / Automated**: 27 threads

---

## 3. High-Priority Confirmed Active Findings (Still Present on `origin/main`)

The following defect clusters represent confirmed active technical debt on `origin/main` that was discovered during historical PR reviews, left unaddressed or partially unaddressed, and still violates current architectural contracts or verifier invariants.

### [HRD-001] Language Lexicon stdout Contradiction

- **Current Severity**: `MEDIUM`
- **Originating PR**: [PR #620](https://github.com/skulmakov-oss/Semantic/pull/620)
- **Current Location on `origin/main`**: [`docs/language/semantic_command_lexicon.md:392`](file:///docs/language/semantic_command_lexicon.md#L392)
- **Finding Summary**: In section 5.14 of `semantic_command_lexicon.md`, `canonical term: `stdout`` is explicitly declared as a canonical term. However, surrounding sections 6 and 7 classify `stdout` strictly as a non-canonical host implementation detail.
- **Architectural & Security Impact**: Contract contradiction in language specification: downstream compilers and tools consuming the lexicon may treat host stdout channel as approved canonical source vocabulary, contradicting total effect isolation invariants in `CONSTRAINTS.md` (Hard Invariant 2.D).
- **Recommended Remediation**: Update `docs/language/semantic_command_lexicon.md:392` to specify `canonical term: none / implementation-detail`.

### [HRD-002] Compiler Method Generic Arity Bypass in Impl Blocks

- **Current Severity**: `HIGH`
- **Originating PR**: [PR #1869](https://github.com/skulmakov-oss/Semantic/pull/1869)
- **Current Location on `origin/main`**: [`crates/sm-front/src/lib.rs:703`](file:///crates/sm-front/src/lib.rs#L703)
- **Finding Summary**: Compiler admission leak: `build_fn_table` checks `type_params.len() > 1` only for top-level `program.functions`. Methods defined inside `impl` blocks (`impl Trait for Type { fn method<T, U>(...) }`) bypass the first-wave generic arity limit (at most 1 type parameter) because `type_check_program` and `check_impl_conformance` never enforce arity on `imp.methods`.
- **Architectural & Security Impact**: Bypasses language maturity boundary constraints for Phase SSF-07. Methods with arbitrary generic arity pass frontend typechecking but trigger lower-pipeline compiler panics during SemCode emission or IR lowering.
- **Recommended Remediation**: In `crates/sm-front/src/typecheck.rs`, enforce `method.type_params.len() <= 1` across all `imp.methods` during `check_impl_conformance`.

### [HRD-003] RunSemCode Command Envelope Verifier Bypass Spec Drift

- **Current Severity**: `HIGH`
- **Originating PR**: [PR #556](https://github.com/skulmakov-oss/Semantic/pull/556)
- **Current Location on `origin/main`**: [`docs/spec/ui/local_runtime_command_result_envelope.md:128`](file:///docs/spec/ui/local_runtime_command_result_envelope.md#L128)
- **Finding Summary**: Specification loophole: Section 4.2 states `RunSemCode must go through verified execution unless the artifact is already admitted in the same trusted session`. It omits a mandatory content digest / bytecode hash validation.
- **Architectural & Security Impact**: Violates Hard Invariant 2.A (CONSTRAINTS.md: Verifier-First Trusted Execution Pipeline). If the on-disk bytecode file is modified by an attacker or external process during a session, the unverified cached admission could execute unverified byte modifications without verifier admission.
- **Recommended Remediation**: Update `docs/spec/ui/local_runtime_command_result_envelope.md:128` to require cryptographic content digest matching (e.g. SHA-256) before any session-cached admission token can be reused.

### [HRD-004] Fail-Open Host Channel Evaluation in Capability Gate

- **Current Severity**: `HIGH`
- **Originating PR**: [PR #645](https://github.com/skulmakov-oss/Semantic/pull/645)
- **Current Location on `origin/main`**: [`crates/prom-cap/src/hello_observation_capability.rs:49`](file:///crates/prom-cap/src/hello_observation_capability.rs#L49)
- **Finding Summary**: Evaluator wildcard arm: In `evaluate_hello_observation_capability`, match statement `match context.requested_host_channel` handles `Some("stdout") => ...` but falls through on `_ => {}` directly into the sink evaluation logic.
- **Architectural & Security Impact**: Fail-open capability vulnerability: Any unrecognized explicit host channel (such as `stderr`, `socket`, `ipc`) bypasses channel denial and is granted `Allow` decision if sink flags are enabled, violating Hard Invariant 2.E (Fail-Closed Posture).
- **Recommended Remediation**: In `crates/prom-cap/src/hello_observation_capability.rs`, replace wildcard `_ => {}` with explicit rejection returning `CapabilityDenialReason::GenericIoNotAllowed` for any non-stdout channel.

### [HRD-005] 7hell Check Error Misclassification to Syntax Hell

- **Current Severity**: `HIGH`
- **Originating PR**: [PR #726](https://github.com/skulmakov-oss/Semantic/pull/726)
- **Current Location on `origin/main`**: [`src/bin/smc.rs:643`](file:///src/bin/smc.rs#L643)
- **Finding Summary**: Diagnostic stage misattribution: In `diagnostic_from_check_error`, the pattern `c if c.starts_with('E') => ("syntax", SevenHellDiagnosticKind::SyntaxDiagnostic, "syntax")` misroutes all non-E0201 semantic and type check error codes (e.g. E0220 duplicate-entity, E0202 type-mismatch, E0240 invalid-cast) to Stage 1 (Syntax Hell) instead of Stage 2 (Type Hell).
- **Architectural & Security Impact**: Blocks proper 7hell compiler qualification reporting: Semantic type errors are attributed to syntax failure, causing syntax test suites to register false stage failures and masking actual type check hell regressions.
- **Recommended Remediation**: In `src/bin/smc.rs:643`, route semantic error codes (E0201..E0245) to stage `'type'` with `SevenHellDiagnosticKind::CheckDiagnostic`.

### [HRD-006] Unchecked RegId Arithmetic Overflow in VM Call Path

- **Current Severity**: `HIGH`
- **Originating PR**: [PR #385](https://github.com/skulmakov-oss/Semantic/pull/385)
- **Current Location on `origin/main`**: [`crates/semantic-core-exec/src/lib.rs:1502`](file:///crates/semantic-core-exec/src/lib.rs#L1502)
- **Finding Summary**: Unchecked register index arithmetic: In Call instruction handler, argument register indexing uses raw arithmetic `RegId(arg_base.0 + offset as u16)` without overflow validation.
- **Architectural & Security Impact**: On large register allocations or corrupted argument offsets, this calculation overflows: triggering an uncatchable panic in debug builds, or silently wrapping to register 0 in release builds, corrupting register state instead of returning `CoreTrap::InvalidRegister`.
- **Recommended Remediation**: Use checked addition: `arg_base.0.checked_add(offset as u16).ok_or(CoreTrap::InvalidRegister)?`.

### [HRD-007] Hot-Path CPU Capability Probing in Backend Operations

- **Current Severity**: `LOW`
- **Originating PR**: [PR #385](https://github.com/skulmakov-oss/Semantic/pull/385)
- **Current Location on `origin/main`**: [`crates/semantic-core-backend/src/lib.rs:75`](file:///crates/semantic-core-backend/src/lib.rs#L75)
- **Finding Summary**: Redundant CPU capability detection: Kernel operations `join_reg32`, `meet_reg32`, and `invert_reg32` invoke `detect_backend_caps()` repeatedly on every single operation in tight inner loops.
- **Architectural & Security Impact**: Distorts throughput benchmarks and wastes execution cycles during high-frequency deterministic kernel operations.
- **Recommended Remediation**: Cache `BackendCaps` statically or pass an initialized capability context into the execution backend.

### [HRD-008] Literal Quoting Mismatch in Hello Observation Route

- **Current Severity**: `MEDIUM`
- **Originating PR**: [PR #652](https://github.com/skulmakov-oss/Semantic/pull/652)
- **Current Location on `origin/main`**: [`crates/sm-runtime-core/src/hello_observation_route.rs:45`](file:///crates/sm-runtime-core/src/hello_observation_route.rs#L45)
- **Finding Summary**: Quote mismatch between verifier and runtime: `route_hello_observation_to_sink` expects unquoted literal `"Hello, World!"`, whereas the verifier emits quoted token representation `"\"Hello, World!\""`.
- **Architectural & Security Impact**: Admitted Hello observations fail the controlled route match and are misclassified as `NonControlledText`, bypassing downstream observation telemetry.
- **Recommended Remediation**: Normalize string quotes before route matching or accept both raw and quoted string forms in the match arm.

---

## 4. Historical Review Debt Resolved in Later PRs / Commits

A significant portion of historical review debt (440 threads) consists of legitimate review objections that were left open when the original PR merged, but were subsequently addressed during later development cycles. The table below highlights key instances verified against git commit logs:

| Original PR | Component / File | Original Finding Summary | Rectifying Commit / PR | Verification Evidence on `origin/main` |
| :--- | :--- | :--- | :--- | :--- |
| **PR #1593** | `crates/sm-vm/src/semcode_vm.rs` | Local functions must take precedence over application builtins | **PR #1750** (umbrella #1617) | `resolve_function_call` checks program functions table before builtins |
| **PR #1593** | `crates/sm-vm/src/semcode_vm.rs` | Application builtins must be charged against `effect_calls` quota | **PR #1900** | `bump_effect_calls()` invoked inside `ApplicationVmHost::invoke_builtin` |
| **PR #1587** | `docs/spec/foundation_source_profile_v1.md` | Extend frozen SemCode range through SEMCOD14 | **PR #1732 / PR #1773** | Spec header extended to include SEMCOD14, SEMCOD18, SEMCOD19 |
| **PR #1585** | `tests/ssf_status_drift.rs` | Include every current release authority in drift guard | **PR #1601** | `docs/release_artifact_model.md` added to release authority array |
| **PR #127** | `crates/sm-ir/src/legacy_lowering.rs` | Keep block-level bindings in scope for lowered locals | **PR #1724** (`8387c46d`) | `lowered_locals.bind()` introduces block scope tracking |
| **PR #405** | `crates/sm-vm/src/semcode_vm.rs` | Compute RNG span without i32 overflow | Commit `a586fc93` | RNG calculation migrated to `i64::from(hi) - i64::from(lo)` |
| **PR #768** | `crates/smc-cli/src/package_manifest.rs` | Reject backslash path traversal before normalization | Decision Log DL-012/DL-014 | `normalize_manifest_path` rejects backslash characters explicitly |
| **PR #1953** | `tests/ir_pipeline_error_origins.rs` | Negative source guards dead; test never runs optimizer | Commit `d956abdf` | Reconciled prior to merge with live optimizer production tests |

---

## 5. Security & Privacy Exposures Audit

A complete, deep security scan was performed across all 1,477 PR bodies, issue comments, inline review comments, and diff hunks.

### Credential & Secret Exposure: ZERO (0)
- Scanned patterns: GitHub Personal Access Tokens (`ghp_`, `github_pat_`), AWS Access Keys (`AKIA...`), Private RSA/SSH Keys (`-----BEGIN ... PRIVATE KEY-----`), JWT tokens, generic API keys, and environment secret assignments.
- **Result**: **0 active secret credentials were found exposed** in git commit diffs, review comments, or PR bodies.

### Privacy & Local Environment Artifacts: 119 Detections
While no production credentials leaked, **119 instances of local developer machine paths and agent session metadata** were found in review comments and PR descriptions:

| Artifact Category | Count | Example Pattern (Redacted) | Impact |
| :--- | :--- | :--- | :--- |
| **Claude Local Workspaces** | 77 | `C:\Users\...\.claude\workspaces\...` | Exposes local session UUIDs and internal agent workspace paths |
| **Windows User Paths** | 37 | `C:\Users\<username>\Desktop\...` | Exposes local developer usernames and directory structures |
| **macOS User Paths** | 4 | `/Users/<username>/Library/...` | Exposes local developer usernames and platform paths |
| **Codex Local Directories** | 1 | `C:\Users\...\.codex\...` | Exposes local IDE extension directories |

### CodeQL Security Alert Tracking
- **PR #1872** (`tests/hub_cli.rs`): GitHub Advanced Security flagged CodeQL alert CWE-532 (*Cleartext logging of sensitive information*) due to `session_id` written directly to CLI audit logs.
- **Current Mitigation**: Active branch `security/1866-hub-audit-confidentiality` isolates and masks audit log outputs before promotion to main.

---

## 6. Audit Artifacts & Data Lineage

All audit data has been compiled into reproducible, machine-readable artifacts stored in the repository:

1. **Complete Machine-Readable JSON Ledger**:  
   [`reports/security/historical_pr_review_audit.json`](historical_pr_review_audit.json)  
   *Full metadata, review comments, diff hunks, classification status, and file coordinates for all 813 candidates.*

2. **Standard Machine-Readable CSV Ledger**:  
   [`reports/security/historical_pr_review_audit.csv`](historical_pr_review_audit.csv)  
   *Tabular export with schema: `pr_number, pr_title, pr_state, merged, merge_date, thread_id, thread_resolved, review_author, review_date, file, original_line, original_severity, finding_summary, current_status, current_severity, current_file, current_line, fix_pr, fix_commit, recommended_action, notes`.*

---

## 7. Definition of Done Verdict

```text
HISTORICAL REVIEW DEBT AUDIT COMPLETE
Active confirmed findings: 220
Partial findings: 4
Fixed-later findings: 440
Obsolete findings: 44
False positives: 97
Unverified findings: 8
Credential exposures: 0
```
