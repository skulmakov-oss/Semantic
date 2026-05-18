# PCC-8 Stdlib v0 Live Audit

Status: live audit
Owner: language maturity stream
Scope: Stdlib v0 readiness before PCC-8 implementation or fixture work
Non-goal: code changes

## 1. Purpose

This document audits current Stdlib v0 readiness on `main` after PCC-7
closeout.

It is docs-only. It does not add helper behavior.

## 2. Current Known Status

Current `main` already shows a narrow helper-adjacent surface, PCC-8B freezes
the public helper boundary in a separate contract doc, and PCC-8C now adds the
first dedicated positive helper fixtures.

That still does not mean PCC-8 is fully closed:

- `assert` is already used as a runtime-visible failure surface and now has a
  dedicated positive PCC-8 acceptance fixture.
- `print` is exercised by benchmark fixtures for text-only output and now has
  a dedicated positive PCC-8 acceptance fixture for the admitted text path.
- `to_text` is exercised by benchmark fixtures for admitted basic types and
  now has a dedicated positive PCC-8 acceptance fixture for admitted basic
  types.
- `debug_render` remains internal tooling and must not be treated as a public
  `to_text` substitute.
- The stdlib roadmap docs already describe the intended first-wave families
  (`assert`, math helpers, text helpers, `to_text`, sequence helpers, map
  helpers, Option / Result helpers), and PCC-8B now freezes the public helper
  contract boundary without claiming implementation completion.
- Text helper behavior is already backed by earlier PCC fixture suites and now
  has a dedicated positive PCC-8 acceptance fixture as well.
- Sequence / map / Option / Result helper behavior is already backed by
  earlier PCC fixture suites, but PCC-8 still lacks dedicated packaging for the
  broader stdlib boundary itself.
- `std.math` remains a proposed family contract rather than a shipped public
  stdlib module.

## 3. Readiness Matrix

| Layer            | Required for PCC-8                               | Current state | Ready? | Next action |
| ---------------- | ------------------------------------------------ | ------------- | ------ | ----------- |
| surface          | public helper list                               | confirmed-partial | no | keep the public contract frozen but avoid claiming implementation completion |
| surface          | helper naming / canonical call form              | confirmed-partial | no | keep `debug_render` internal and preserve canonical helper spellings |
| assert           | assert behavior                                  | confirmed-working | no | keep helper contract and failure wording stable |
| print            | text-only print behavior                         | confirmed-partial | no | keep text-only boundary documented and stable |
| to_text          | admitted basic types                             | confirmed-working | no | keep admitted types and canonical call sites stable |
| to_text          | rejected unsupported types                       | confirmed-partial | no | keep unsupported-type rejection wording stable |
| text helpers     | text concat / len / equality boundary            | confirmed-partial | no | keep text helper behavior bounded to the current public text contract |
| math helpers     | admitted numeric helpers                         | documented-only | no | freeze first-wave helper list and type scope before implementation |
| sequence helpers | len / is_empty / contains / push / prepend / pop | confirmed-partial | no | keep collection helpers out of public stdlib claims until packaged separately |
| map helpers      | map_empty / map_set / map_get                    | confirmed-partial | no | keep collection helpers out of public stdlib claims until packaged separately |
| Option helpers   | admitted helper surface                          | confirmed-partial | no | preserve narrow standard-form boundary and explicit canonical forms |
| Result helpers   | admitted helper surface                          | confirmed-partial | no | preserve narrow standard-form boundary and explicit canonical forms |
| typecheck        | helper type contracts                            | confirmed-partial | no | freeze helper contracts before any implementation widening |
| diagnostics      | helper misuse diagnostics                        | confirmed-partial | no | keep failure wording stable and separate from debug helpers |
| traps            | runtime helper failures                          | confirmed-partial | no | preserve deterministic trap behavior for false/assert and helper misuse |
| lowering         | helper lowering path                             | confirmed-partial | no | keep helper lowering inspectable and public-contract aligned |
| SemCode          | helper representation                            | confirmed-partial | no | keep helper lowering on the admitted verifier-admissible path |
| verifier         | verifies helper form                             | confirmed-partial | no | keep verifier-first admission intact for helper-like execution paths |
| VM/runtime       | executes helper form                             | confirmed-partial | no | preserve deterministic runtime behavior for helper paths |
| determinism      | deterministic helper behavior                    | confirmed-partial | no | keep helper output / trap behavior stable across runs |
| docs             | public stdlib contract                           | confirmed-partial | no | keep the public contract frozen and separate from implementation completion |
| examples         | canonical examples avoid internal debug helpers  | documented-only | no | keep canonical examples free of `debug_render` and other internal helpers |
| tests            | positive / negative coverage                     | confirmed-partial | no | add dedicated PCC-8 packaging after the contract boundary is frozen |

## 4. PCC-8B Evidence

PCC-8B freezes the public helper contract and debug_render boundary.

Covered:

- public helper family classification;
- debug_render internal-only boundary;
- to_text admitted-types boundary;
- unsupported to_text rejection boundary;
- print text-only boundary;
- helper failure behavior policy;
- capability / host boundary;
- canonical examples rule.

Validation:

- `git diff --check`

PCC-8B does not add tests or implementation.
PCC-8C remains positive fixture packaging.
PCC-8D remains diagnostics / trap fixture packaging.
PCC-8E remains closeout.

## 5. PCC-8C Evidence

PCC-8C adds dedicated positive Stdlib v0 helper acceptance fixtures.

Covered positive cases:

- assert true path;
- print(text) positive path;
- to_text for explicitly admitted basic types;
- admitted text helper surface.

Validation:

- `cargo test --test pcc8_stdlib_acceptance`
- `git diff --check`

PCC-8C does not add new helpers.
PCC-8C does not promote debug_render.
PCC-8C does not expand to_text.
PCC-8C does not cover diagnostics / traps.
PCC-8D remains helper diagnostics and runtime traps.
PCC-8E remains closeout.

## 6. Risk List

Include at least:

- Stdlib v0 can silently become a broad standard-library expansion.
- `debug_render` must not leak into public language semantics.
- `to_text` must not become universal reflection.
- helper failures must be diagnostic / trap stable.
- helpers must not bypass capability or host boundary rules.
- collection helpers must not reopen PCC-7 memory / quota policy.
- Option / Result helpers must not become exception semantics.
- print must remain text-only if current contract says so.
- public helper list must be explicit.
- canonical examples must not rely on internal debug formatting.

## 7. Recommended PCC-8 Split

Default split:

```text
PCC-8A — docs(stdlib): audit Stdlib v0 readiness before implementation
PCC-8B — docs(stdlib): freeze public helper contract and debug_render boundary
PCC-8C — test(stdlib): lock positive basic helper fixtures
PCC-8D — test(stdlib): lock helper diagnostics and runtime traps
PCC-8E — docs(stdlib): close PCC-8 with evidence sync and roadmap status update
```

If the audit finds clear gaps, propose narrow implementation or docs-policy PRs
between B/C/D, for example:

- PCC-8I1 `to_text` admitted-basic-types seam;
- PCC-8I2 helper typecheck contract seam;
- PCC-8I3 helper lowering / verifier seam;
- PCC-8I4 helper diagnostics seam;
- PCC-8I5 public helper contract docs seam.

## 8. Out of Scope

Explicitly list:

- universal reflection;
- debug_render promotion;
- formatting macro system;
- broad stdlib expansion;
- IO capability expansion;
- host ABI widening;
- collection memory / quota policy;
- exception semantics;
- UI / Workbench;
- README promotion.

## 9. Acceptance Checklist

```markdown
- [ ] helper surface inspected
- [ ] assert inspected
- [ ] print inspected
- [ ] to_text inspected
- [ ] text helpers inspected
- [ ] math helpers inspected
- [ ] sequence helpers inspected
- [ ] map helpers inspected
- [ ] Option helpers inspected
- [ ] Result helpers inspected
- [ ] helper type contracts inspected
- [ ] lowering inspected
- [ ] SemCode/verifier inspected
- [ ] VM/runtime inspected
- [ ] diagnostics/traps inspected
- [ ] tests inspected
- [ ] docs inspected
- [ ] canonical examples inspected
- [ ] debug_render boundary inspected
- [ ] risks documented
- [ ] PCC-8 split proposed
- [ ] no code changed
```

## 10. CTF Note

Because this is docs-only:

`CTF touched: none`

Reason:

`docs-only audit; no runtime value, trap, determinism, verifier, SymbolId,
capability, or trace change`
