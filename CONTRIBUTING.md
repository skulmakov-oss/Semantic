# Contributing to Semantic

Thank you for contributing to Semantic.

Semantic is a deterministic, verifier-first language and execution platform. Contributions are welcome, but changes must preserve the repository's architectural boundaries, deterministic behavior, verification guarantees, and task governance.

This document is a contributor entry point. It does not override repository authority.

## 1. Read the repository authorities first

Before proposing or implementing a change, read these sources in order:

1. [`AGENTS.md`](AGENTS.md) — canonical repository bootstrap, routing, and contributor/agent authority.
2. [`CONSTRAINTS.md`](CONSTRAINTS.md) — non-negotiable architectural, semantic, determinism, and verification invariants.
3. [`.harness/current.task.yaml`](.harness/current.task.yaml) — the active task authorization envelope, including allowed and forbidden paths.
4. [`docs/spec/`](docs/spec/) — normative language, SemCode, verifier, runtime, and compatibility contracts.
5. [`docs/agents/WORKFLOW.md`](docs/agents/WORKFLOW.md) — execution lifecycle and repository workflow.
6. [`docs/agents/VERIFICATION.md`](docs/agents/VERIFICATION.md) — verification tiers, admission gates, and CI parity requirements.

A lower-level document may make a rule stricter, but it must not weaken a higher-level rule.

If the requested change conflicts with a repository constraint or the active Harness envelope, stop and report the conflict. Do not bypass the restriction or broaden scope to make the change fit.

## 2. Keep changes small and bounded

Semantic uses checkpoint-style development.

Please follow these rules:

- Make one logical change per pull request.
- Keep the implementation strictly inside the authorized task scope.
- Do not perform opportunistic refactors or unrelated cleanup.
- Do not widen language, verifier, VM, format, capability, or runtime semantics without explicit architectural authority.
- Treat a green CI run as evidence, not as permission to widen scope or merge.
- Do not silently change stable or release claims simply because code exists on `main`.

For substantial work, start from an issue or another explicit repository-owner authorization so that scope, non-goals, and acceptance criteria are clear before implementation begins.

## 3. Respect subsystem ownership

Changes must remain within the repository's ownership boundaries:

- `sm-front` — parser, lexer, AST, source syntax surface.
- `sm-sema` — semantic analysis, type checking, compile-time diagnostics.
- `sm-ir` — IR, lowering, and optimizer logic.
- `sm-format` — SemCode binary format, opcode vocabulary, decoding structures.
- `sm-emit` — producer-facing SemCode emission facade.
- `sm-verify` — verifier admission rules.
- `sm-runtime-core` — shared runtime vocabulary, execution types, and quotas.
- `sm-vm` — deterministic VM execution.
- `smc-cli` — public CLI and authorized host I/O; it must not redefine language or verifier semantics.
- `prom-*` — PROMETHEUS host ABI, capabilities, gates, runtime sessions, rules, and audit boundaries.
- `prom-ui*` — UI orchestration and presentation boundaries (retired contour: preserved, not remediated or extended; see `docs/roadmap/ui_workbench_studio_retirement.md`).

Do not duplicate authority across layers. For example, the CLI must not become a second parser or verifier, and the VM must not silently reinterpret a verifier contract.

## 4. Development workflow

A normal contribution should follow this sequence:

1. **Ingest the task** — read the active authorities, issue, specs, and Harness scope.
2. **Plan** — define the smallest implementation that satisfies the contract and identify explicit non-goals.
3. **Implement** — use small, auditable patches and stay within authorized paths.
4. **Verify** — run the required risk-tier checks and collect fresh evidence.
5. **Prepare the PR** — inspect scope, diff hygiene, verification results, public API impact, compatibility impact, and remaining risks.

If an unexpected architecture or contract question appears during implementation, stop and resolve it before continuing. Do not improvise a new architecture inside a bug fix.

## 5. Harness and scope checks

The Harness is the repository's per-task authorization envelope.

Before committing:

```powershell
git ls-files --others --exclude-standard
pwsh -File scripts/harness-check.ps1
```

New files must be visible to the staged-path check when Harness verification is performed.

Before PR handoff, verify the committed scope against `main`:

```powershell
git diff --name-only origin/main...HEAD
git diff --check
git status
```

Every changed path must be authorized by the task envelope or by an explicit controlled governance transition.

## 6. Risk classification and verification

Verification effort must scale with risk.

### R0 — informational/docs

Examples: typos, comments, non-normative documentation.

Minimum checks:

```powershell
pwsh -File scripts/workspace_fmt_check.ps1
pwsh -File scripts/harness-check.ps1
git diff --check origin/main
```

### R1 — private/internal

Examples: internal refactors or isolated crate fixes.

Run the R0 checks plus:

```powershell
pwsh -File scripts/admission_guard.ps1 -PRReady
```

and focused component tests.

### R2 — boundary/contract

Examples: parser behavior, public APIs, type rules, IR, serialization, or cross-crate contracts.

Run the R1 checks plus:

```powershell
pwsh -File scripts/admission_guard.ps1 -CIParity
```

and the relevant boundary/public API/golden tests.

### R3 — critical/systemic

Examples: verifier admission, SemCode binary format, VM execution, determinism, capability gates, PROMETHEUS runtime, security, or release compatibility.

Run the R2 checks plus the repository's full preflight and relevant 7hell qualification, with fresh adversarial review where required by the workflow.

See [`docs/agents/VERIFICATION.md`](docs/agents/VERIFICATION.md) for the current canonical commands and known CI-parity differences.

## 7. Tests and evidence

Behavior changes require tests.

Contributors must:

- add positive admission and negative rejection tests where applicable;
- preserve deterministic outputs;
- keep verifier/runtime and format tests aligned with normative contracts;
- use focused regression tests for every fixed defect;
- update golden artifacts only when the contract explicitly authorizes a format or behavior change;
- never delete, disable, weaken, or rewrite assertions merely to make CI pass;
- report exact verification commands and results before claiming completion.

If an unexpected check fails, investigate the root cause. Do not add shims or unrelated fixes without explicit scope authority.

## 8. Formatting and code quality

Rust changes should remain warning-clean and formatted according to the repository tooling.

Use the repository's Windows-safe formatting helper when working locally on Windows:

```powershell
pwsh -File scripts/workspace_fmt_check.ps1
```

For normal PR-ready validation:

```powershell
pwsh -File scripts/admission_guard.ps1 -PRReady
```

Do not introduce new dependencies without explicit architectural justification and scope authorization.

## 9. Determinism and trusted execution

Semantic treats determinism and verifier admission as architectural properties, not implementation details.

Contributions must preserve:

- deterministic diagnostics and execution;
- Quad Logic semantics (`N`, `F`, `T`, `S`) as defined by the normative spec;
- verifier-first trusted execution;
- fail-closed admission and validation;
- capability and host-effect boundaries;
- the separation between raw/diagnostic VM APIs and canonical trusted execution.

Do not add direct filesystem, network, or OS effects to deterministic core crates.

## 10. Public APIs, SemCode, verifier, and runtime changes

Changes to public APIs, binary formats, verifier rules, VM semantics, runtime traps/errors, compatibility claims, or release behavior require explicit contract review.

When touching these areas:

- identify the normative spec owner;
- state compatibility impact;
- update public API/golden/spec evidence only within the authorized checkpoint;
- preserve fail-closed behavior;
- do not assign new opcodes, codes, format revisions, or runtime identities without explicit architectural authority.

## 11. Pull request requirements

A pull request should be narrow, reviewable, and evidence-backed.

Include at least:

- the task/checkpoint or issue reference;
- scope and explicit non-goals;
- files or subsystems changed;
- risk tier (`R0`–`R3`);
- tests and verification commands run;
- public API / SemCode / verifier / runtime / compatibility impact, when applicable;
- any remaining findings or known limitations.

Use `Refs #<issue>` for ordinary linkage unless issue closure has been explicitly authorized. Do not use `Closes`, `Fixes`, or `Resolves` as a shortcut for a separate issue-close decision.

Do not auto-merge. A completed implementation and green CI are review inputs, not merge authorization.

## 12. Security reports

Do not disclose security vulnerabilities through public issues or pull requests when they require private handling.

Follow [`SECURITY.md`](SECURITY.md) for supported versions and the repository's vulnerability-reporting process.

## 13. AI-assisted contributions

AI-assisted contributions are welcome, but agents do not receive broader authority than human contributors.

Agents must follow the same repository constraints, Harness scope, specs, tests, and review gates. `AGENTS.md` and the repository-native skills define additional required routing for agent-driven work.

An agent must never self-authorize a wider scope, silently bypass a missing required capability, or continue after a contract blocker merely because an implementation path appears obvious.

## 14. Licensing

By contributing, you agree that your contribution will be provided under the repository's existing license and applicable project notices. See [`LICENSE`](LICENSE) and [`NOTICE`](NOTICE).

---

Thank you for helping keep Semantic deterministic, auditable, verifier-first, and architecturally coherent.
