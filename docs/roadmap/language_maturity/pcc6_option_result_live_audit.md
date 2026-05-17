# PCC-6 Option / Result Live Audit

Status: live audit
Owner: language maturity stream
Scope: Option / Result readiness before PCC-6 implementation or fixture work
Non-goal: code changes

## 1. Purpose

This document audits current Option / Result readiness on `main` after PCC-5
closeout.

It is docs-only. It does not add Option / Result behavior.

## 2. Current Known Status

Current `main` already contains the narrow first-wave Option / Result
standard-form baseline across the practical stack:

- Parser support exists for `Option(T)` / `Result(T, E)` declared type
  positions and the canonical `Option::Some`, `Option::None`, `Result::Ok`,
  `Result::Err` constructor surface.
- The frontend AST/type model already carries the standard-form families as
  first-class nominal source types rather than as a general generics surface.
- Type checking validates contextual constructor typing, payload validation,
  and explicit match payload binding for the standard forms.
- Lowering already emits the canonical ADT-style carrier path for both
  standard forms.
- SemCode, verifier, and VM already execute the emitted standard-form paths.
- Diagnostics already exist for contextual constructor typing and match policy
  boundaries.
- Canonical examples and qualification fixtures already exercise the
  first-wave surface through the full practical pipeline.
- The current docs already freeze the narrow first-wave boundary in
  `option_result_standard_forms_scope.md`.
- Dedicated PCC-6 positive Option fixture packaging now exists.
- Dedicated Result and negative PCC-6 fixture packaging is still missing.

Audit verdict:

```text
The narrow first-wave Option / Result baseline is already present in main.
PCC-6 does not need a new architecture seam before fixture packaging.
PCC-6B now covers the Option side; Result and negative PCC-6 fixture packaging
remain to be added.
```

CTF touched: none
Reason: docs-only audit; no runtime value, trap, determinism, verifier,
SymbolId, capability, or trace change.

## 3. Readiness Matrix

| Layer | Required for PCC-6 | Current state | Ready? | Next action |
| --- | --- | --- | --- | --- |
| parser | `Option(T)` type syntax | confirmed-working | yes | keep as fixture-backed evidence |
| parser | `Result(T, E)` type syntax | confirmed-working | yes | keep as fixture-backed evidence |
| parser | standard constructors | confirmed-working | yes | keep as fixture-backed evidence |
| parser | Option / Result match syntax | confirmed-working | yes | keep as fixture-backed evidence |
| frontend model | standard-form type representation | confirmed-working | yes | keep as fixture-backed evidence |
| typecheck | Option payload validation | confirmed-working | yes | keep as fixture-backed evidence |
| typecheck | Result payload validation | confirmed-working | yes | keep as fixture-backed evidence |
| typecheck | match arm payload binding | confirmed-working | yes | keep as fixture-backed evidence |
| exhaustiveness | Option / Result match policy | confirmed-partial | partial | dedicated Result-specific qualification / negative packaging still missing |
| lowering | Option / Result constructor / match lowering | confirmed-working | yes | keep as fixture-backed evidence |
| SemCode | stable representation | confirmed-working | yes | keep opcode mapping and verifier coverage stable |
| verifier | validates emitted form | confirmed-working | yes | keep bytecode checks stable |
| VM | executes Option / Result path | confirmed-working | yes | keep runtime carrier behavior stable |
| diagnostics | clear Option / Result errors | confirmed-working | yes | keep diagnostic needles stable |
| tests | positive / negative coverage | confirmed-partial | partial | dedicated Result and negative PCC-6 fixture packaging is still missing |
| docs | standard-form boundary | documented-only | partial | keep scope boundary synced with live evidence |

## 4. Risk List

Observed risks are narrow, not architectural:

- Option / Result may accidentally reopen general generics.
- `Option(T)` / `Result(T, E)` must not become angle-bracket generic syntax.
- Hidden prelude injection is out of scope.
- Host ABI widening is out of scope.
- Result must not become exception or call-boundary semantics.
- Match ergonomics must remain canonical unless separately scoped.
- Option / Result should reuse the existing ADT-style carrier path where the
  current design says so.
- Dedicated Result and negative PCC-6 fixture packaging may still be missing
  even if the baseline implementation already exists.

## 5. Recommended PCC-6 Split

No new architecture seam is required for the current baseline.

Recommended split:

```text
PCC-6B — test(option): lock positive Option standard-form fixtures
PCC-6C — test(result): lock positive Result standard-form fixtures
PCC-6D — test(option-result): lock negative diagnostics fixtures
PCC-6E — docs(option-result): close PCC-6 with evidence sync and roadmap status update
```

If a missing seam appears during fixture packaging, split it narrowly and do
not widen the PR into general generics or host ABI work.

## 6. Out of Scope

- general generics
- angle-bracket generics
- user-defined parameterized ADTs
- hidden prelude injection
- exception semantics
- host ABI widening
- collections
- records
- stdlib ergonomic helpers
- advanced match syntax
- UI / Workbench

## 7. Acceptance Checklist

- [x] parser surface inspected
- [x] AST/frontend model inspected
- [x] typecheck inspected
- [x] Option constructor support inspected
- [x] Result constructor support inspected
- [x] match/payload binding inspected
- [x] exhaustiveness policy inspected
- [x] lowering inspected
- [x] SemCode/verifier inspected
- [x] VM runtime inspected
- [x] diagnostics inspected
- [x] tests inspected
- [x] docs inspected
- [x] risks documented
- [x] PCC-6 split proposed
- [x] PCC-6B positive Option fixtures added
- [x] no code changed

## 8. PCC-6B Evidence

PCC-6B adds dedicated positive Option standard-form acceptance fixtures.

Covered positive cases:

- `Option(T)` declared type position;
- `Option::Some(value)` constructor;
- `Option::None` constructor;
- payload binding through explicit match;
- Option value across a function boundary.

Validation:

- `cargo test --test pcc6_option_acceptance`
- `git diff --check`

PCC-6B does not cover Result. Result positive fixtures remain PCC-6C.
Negative diagnostics remain PCC-6D.
PCC-6 closeout remains PCC-6E.
