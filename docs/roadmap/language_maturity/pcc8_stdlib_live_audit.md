# PCC-8 Stdlib v0 Live Audit

Status: PCC-8 closed / evidence synced for current admitted helper surface
Owner: language maturity stream
Scope: Stdlib v0 readiness before PCC-8 implementation or fixture work
Non-goal: code changes

## 1. Purpose

This document audits current Stdlib v0 readiness on `main` after PCC-7
closeout.

It is docs-only. It does not add helper behavior.

## 2. Current Known Status

Current `main` already shows a narrow helper-adjacent surface, PCC-8B freezes
the public helper boundary in a separate contract doc, PCC-8C adds the first
dedicated positive helper fixtures, and PCC-8D adds the first dedicated
negative helper fixtures.

That still does not mean PCC-8 is a broad stdlib completion:

- `assert` is already used as a runtime-visible failure surface and now has
  both positive and negative PCC-8 acceptance coverage.
- `print` is exercised by benchmark fixtures for text-only output and now has
  both positive and negative PCC-8 acceptance coverage for the admitted text
  path and non-text rejection.
- `to_text` is exercised by benchmark fixtures for admitted basic types and
  now has both positive and negative PCC-8 acceptance coverage for admitted
  types and rejected record input.
- `debug_render` remains internal tooling and must not be treated as a public
  `to_text` substitute.
- The stdlib roadmap docs already describe the intended first-wave families
  (`assert`, math helpers, text helpers, `to_text`, sequence helpers, map
  helpers, Option / Result helpers), and PCC-8B now freezes the public helper
  contract boundary without claiming implementation completion.
- Text helper behavior is already backed by earlier PCC fixture suites and now
  has dedicated positive PCC-8 coverage.
- Sequence / map / Option / Result helper behavior is already backed by
  earlier PCC fixture suites, but PCC-8 does not claim them as new stdlib
  expansion.
- `std.math` remains a proposed family contract rather than a shipped public
  stdlib module.
- Helper behavior is deterministic within the current admitted surface.

## 3. Readiness Matrix

| Layer            | Required for PCC-8                               | Current state | Ready? | Next action |
| ---------------- | ------------------------------------------------ | ------------- | ------ | ----------- |
| surface          | public helper list                               | confirmed-partial | no | keep the public contract frozen but avoid claiming implementation completion |
| surface          | helper naming / canonical call form              | confirmed-partial | no | keep `debug_render` internal and preserve canonical helper spellings |
| assert           | assert behavior                                  | confirmed-working | yes | keep helper contract and deterministic trap wording stable |
| print            | text-only print behavior                         | confirmed-working | yes | keep text-only boundary documented and stable |
| to_text          | admitted basic types                             | confirmed-working | yes | keep admitted types and canonical call sites stable |
| to_text          | rejected unsupported types                       | confirmed-working | yes | keep unsupported-type rejection wording stable |
| text helpers     | text concat / len / equality boundary            | confirmed-working | yes | keep text helper behavior bounded to the current public text contract |
| math helpers     | admitted numeric helpers                         | documented-only | no | freeze first-wave helper list and type scope before implementation |
| sequence helpers | len / is_empty / contains / push / prepend / pop | confirmed-partial | no | keep collection helpers out of public stdlib claims until packaged separately |
| map helpers      | map_empty / map_set / map_get                    | confirmed-partial | no | keep collection helpers out of public stdlib claims until packaged separately |
| Option helpers   | admitted helper surface                          | confirmed-partial | no | preserve narrow standard-form boundary and explicit canonical forms |
| Result helpers   | admitted helper surface                          | confirmed-partial | no | preserve narrow standard-form boundary and explicit canonical forms |
| typecheck        | helper type contracts                            | confirmed-working | yes | keep helper contracts and failure wording stable |
| diagnostics      | helper misuse diagnostics                        | confirmed-working | yes | keep failure wording stable and separate from debug helpers |
| traps            | runtime helper failures                          | confirmed-working | yes | preserve deterministic trap behavior for false/assert and helper misuse |
| lowering         | helper lowering path                             | confirmed-partial | no | keep helper lowering inspectable and public-contract aligned |
| SemCode          | helper representation                            | confirmed-partial | no | keep helper lowering on the admitted verifier-admissible path |
| verifier         | verifies helper form                             | confirmed-partial | no | keep verifier-first admission intact for helper-like execution paths |
| VM/runtime       | executes helper form                             | confirmed-partial | no | preserve deterministic runtime behavior for helper paths |
| determinism      | deterministic helper behavior                    | confirmed-working | yes | keep helper output / trap behavior stable across runs |
| docs             | public stdlib contract                           | confirmed-working | yes | keep the public contract frozen and separate from implementation completion |
| examples         | canonical examples avoid internal debug helpers  | documented-only | no | keep canonical examples free of `debug_render` and other internal helpers |
| tests            | positive / negative coverage                     | confirmed-working | yes | dedicated PCC-8 packaging now covers positive and negative admitted helper cases |

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

## 6. PCC-8D Evidence

PCC-8D adds dedicated Stdlib v0 helper diagnostics and runtime trap fixtures.

Covered negative / trap cases:

- assert(false) deterministic runtime trap;
- print(non-text) text-only diagnostic;
- unsupported to_text(record) rejection;
- assert arity mismatch rejection;
- assert argument type mismatch rejection;
- to_text arity mismatch rejection.

Validation:

- `cargo test --test pcc8_stdlib_diagnostics`
- `cargo test --test pcc8_stdlib_acceptance`
- `git diff --check`

PCC-8D does not add helpers.
PCC-8D does not promote debug_render.
PCC-8D does not expand to_text.
PCC-8D does not introduce universal reflection.
PCC-8D does not change host / capability boundaries.
PCC-8E remains closeout.

## 7. PCC-8E Closeout

PCC-8A — docs audit / scope correction.
PCC-8B — public helper contract and debug_render boundary freeze.
PCC-8C — positive basic helper fixtures.
PCC-8D — helper diagnostics and runtime trap fixtures.
PCC-8E — bounded closeout / roadmap sync.

PCC-8 Stdlib v0 is closed for the current admitted helper surface.

Explicit evidence-backed statements:

- assert positive and false / trap paths are evidence-backed;
- print(text) is evidence-backed;
- print(non-text) rejection is evidence-backed;
- to_text for admitted basic types is evidence-backed;
- unsupported to_text(record) rejection is evidence-backed;
- helper arity / argument diagnostics for admitted helpers are evidence-backed;
- admitted text helper surface is evidence-backed;
- helper behavior is deterministic within the current admitted surface;
- no new helper semantics were introduced;
- no host ABI widening was introduced;
- no IO / capability expansion was introduced.

Bounded-open note:

```text
The following are not claimed complete by PCC-8E:
- broad stdlib completion;
- std.math implementation;
- universal to_text / reflection;
- debug_render as public helper;
- formatting macro system;
- IO/capability expansion;
- collection memory/quota policy;
- exception-like Option / Result helper semantics.
```

## 8. Risk List

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

## 9. Recommended PCC-8 Split

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

## 10. Out of Scope

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

## 11. Acceptance Checklist

```markdown
- [x] helper surface inspected
- [x] assert inspected
- [x] print inspected
- [x] to_text inspected
- [x] text helpers inspected
- [x] helper type contracts inspected
- [x] diagnostics/traps inspected
- [x] tests inspected
- [x] docs inspected
- [x] debug_render boundary inspected
- [x] risks documented
- [x] PCC-8A/B/C/D/E evidence chain synced
- [x] no code changed by closeout
```

## 12. CTF Note

Because this is docs-only:

`CTF touched: none`

Reason:

`docs-only bounded closeout; no runtime value, trap, determinism, verifier,
SymbolId, capability, or trace change`
