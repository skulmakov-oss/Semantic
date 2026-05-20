# CTF Project-Root Trust Policy

Status: active policy
Owner: language maturity / execution contract
Scope: trust policy before PCC-9I project-root implementation
Non-goal: implementation, project-root readiness, CTF closure, or release readiness

## 1. Purpose

This document defines the trust policy for future project-root support before implementation.

It prevents project-root support from bypassing verifier-first execution.

It prevents manifest and project tooling from becoming hidden capability or host-effect widening.

It keeps `semantic.toml`, `src/main.sm`, and `smc new` as future implementation work until separately admitted.

It does not implement behavior.

## 2. Current Baseline vs Target Contract

| Item | Current state | Target / future state | Trust note |
| --- | --- | --- | --- |
| single-file `.sm` | existing baseline | remains supported | must not be broken by project-root work |
| `Semantic.package` | admitted lower-level manifest baseline | may remain internal / transition baseline | current evidence only |
| `semantic.toml` | target contract name | future parser / loader | not implemented by WP6 |
| `src/main.sm` | target default entry convention | future discovery | not implemented by WP6 |
| `smc check <project-root>` | target command | future PCC-9I implementation | must preserve check -> compile -> verify route |
| `smc run <project-root>` | target command | future PCC-9I implementation | must preserve verifier-first route |
| `smc new` | optional follow-up | future skeleton generator | must not create executable trust without check / verify |
| package registry | out of scope | out of scope | no remote fetch |
| dependency resolver | out of scope | out of scope | no version solving |
| workspace | out of scope | out of scope | no multi-package trust model |

## 3. Trust Invariants

```text
Invariant 1:
Project-root execution must not bypass single-file semantic analysis, lowering, SemCode emission, verifier admission, or VM execution contracts.

Invariant 2:
Project-root run must be verifier-first:
project source -> check -> compile -> verify -> run.

Invariant 3:
Manifest parsing must be deterministic and path-normalized.

Invariant 4:
Project-root discovery must not depend on accidental process cwd.

Invariant 5:
Entry paths must not escape project root.

Invariant 6:
Directory traversal, if introduced, must be deterministic and sorted.

Invariant 7:
Project-root support must not introduce host IO capabilities beyond reading explicitly admitted project files.

Invariant 8:
No network, package registry, remote dependency, or telemetry behavior is admitted.

Invariant 9:
Project helper tests are not public execution evidence.

Invariant 10:
Project-root implementation PRs must update CTF status before claiming readiness.
```

## 4. Verifier-First Route

```text
project root
  -> manifest discovery
  -> manifest parse
  -> entry resolution
  -> source load
  -> semantic check
  -> compile / lower
  -> SemCode emit
  -> verifier admission
  -> verified run
```

Any future public project-root run path must go through this route.

`smc check <project-root>` may stop before SemCode or VM, but must not become execution evidence.

`smc run <project-root>` must not run unchecked source.

Any direct helper must be test-only or internal and clearly marked.

## 5. Determinism Requirements

| Surface | Determinism requirement | Evidence required before readiness |
| --- | --- | --- |
| manifest lookup | same project root resolves same manifest | test + trace |
| manifest parse | same file parses to same manifest model | test |
| entry resolution | same manifest resolves same entry path | test |
| path normalization | platform path differences normalized | test |
| directory traversal | sorted deterministic order if used | test |
| import resolution | same imports resolve same modules | replay / golden trace |
| diagnostics | same invalid layout gives stable diagnostic category | diagnostics test |
| project-root run | same project produces same SemCode / verifier / VM result | replay / golden trace |
| smc new | same input creates same skeleton | test before admission |

Explicitly:

- `semantic.toml` determinism is not claimed yet.
- `src/main.sm` discovery determinism is not claimed yet.
- `smc new` determinism is not claimed yet.
- project-level 7hell determinism is not claimed yet.

## 6. Diagnostics Policy

Diagnostics categories, not exact codes:

- missing manifest;
- malformed manifest;
- unsupported manifest version / shape;
- missing package name;
- missing entry;
- entry path escapes project root;
- entry path points to directory;
- entry file missing;
- invalid project root;
- unsupported workspace field;
- unsupported dependency field;
- unresolved local dependency;
- import path escape;
- nondeterministic module root.

Rules:

- Do not invent exact codes unless already stable.
- Diagnostics must classify as project-diagnostic, not VM trap.
- Project diagnostics must not be mislabeled as verifier rejection unless actual SemCode verification is involved.
- Compile/check diagnostics must stay separate from VM traps.

## 7. Capability / Effect Boundary

- Reading project files is a tooling or compiler input operation, not a language-level host capability.
- Project-root support must not introduce network IO.
- Project-root support must not introduce package registry access.
- Project-root support must not introduce remote dependency fetch.
- Project-root support must not introduce telemetry.
- Local audit is not telemetry.
- `smc new` file creation, if implemented later, must be a CLI or tooling effect with explicit user command, not language runtime effect.
- Workbench / UI integration remains out of scope.

## 8. Golden Trace Requirements

Future PCC-9I implementation must add trace and evidence when behavior becomes public.

Suggested future trace set:

- positive minimal project check trace;
- positive minimal project run trace;
- missing manifest diagnostic trace;
- malformed manifest diagnostic trace;
- entry path escape diagnostic trace;
- missing entry diagnostic trace;
- project-root SemCode / verifier trace;
- project-root replay determinism trace.

No trace artifacts are added by CTF-WP6.

Project manifest helper traces are not project-root execution traces.

7hell report traces remain future work.

## 9. Required CTF Note for Future PCC-9I PRs

```text
CTF touched:
  - docs/roadmap/language_maturity/core_trust_freeze/project_root_trust_policy.md
  - docs/roadmap/language_maturity/core_trust_freeze/verifier_first_policy.md
  - docs/roadmap/language_maturity/core_trust_freeze/determinism_matrix.md
  - docs/roadmap/language_maturity/core_trust_freeze/golden_trace_policy.md
  - docs/roadmap/language_maturity/core_trust_freeze/capability_effect_denial_matrix.md

Reason:
  Project-root behavior changes trust surface.

CTF status impact:
  - no status change
  - audit-needed -> freeze-candidate
  - freeze-candidate -> evidence-backed
  - demotion required
```

## 10. PCC-9I Implementation Split

```text
PCC-9I1 — cli(project-model): add project-root check entrypoint
PCC-9I2 — cli(project-model): add project-root run entrypoint
PCC-9I3 — project-model: add minimal semantic.toml parser / loader
PCC-9I4 — cli(project-model): add smc new minimal skeleton
PCC-9I5 — project-model: enforce deterministic module-root and entry policy
PCC-9I6 — diagnostics(project-model): stabilize project layout diagnostics
PCC-9I7 — test(project-model): add project-root positive / negative fixtures
PCC-9I8 — test(core-trust-freeze): add project-root determinism / golden trace coverage
```

Boundary:

- `PCC-9I1` and `PCC-9I2` may depend on `PCC-9I3`.
- No package manager.
- No registry.
- No workspace.
- No remote packages.

## 11. Stop Conditions for Future Implementation PRs

Future implementation PRs must stop if:

1. project-root run would bypass verifier;
2. semantic.toml parser requires broad TOML support beyond minimal shape;
3. file discovery depends on cwd accidentally;
4. path normalization is platform-unstable;
5. directory traversal is unsorted;
6. entry path escape cannot be rejected;
7. import resolution requires workspace / registry semantics;
8. smc new would generate non-deterministic output;
9. diagnostics require new unstable exact codes;
10. implementation would introduce network or host IO.

## 12. Final Verdict

```text
CTF-WP6 defines the trust policy required before PCC-9I project-root implementation.
It does not implement project-root behavior.
It does not close CTF.
It does not claim release readiness.
```

## 13. Acceptance Checklist

```markdown
- [ ] current baseline vs target contract documented
- [ ] verifier-first route defined
- [ ] determinism requirements defined
- [ ] diagnostics policy defined
- [ ] capability/effect boundary defined
- [ ] golden trace requirements defined
- [ ] future PCC-9I split proposed
- [ ] stop conditions defined
- [ ] no project-root implementation added
- [ ] no semantic.toml parser added
- [ ] no smc new added
- [ ] no CTF closure claimed
- [ ] no release readiness claimed
```
