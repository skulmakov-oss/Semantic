# PCC-8 Stdlib v0 Public Helper Contract

Status: contract freeze
Owner: language maturity stream
Scope: public helper boundary before PCC-8 fixture packaging
Non-goal: implementation changes

## 1. Purpose

This document freezes the first-wave public helper boundary before PCC-8C and
PCC-8D tests.

It does not add helper behavior. It only states which helpers are public,
which are internal, and which remain deferred.

## 2. Public Helper Families For Stdlib v0

The first-wave public families are contract categories, not implementation
promises:

- assert family
- print family
- to_text family for admitted basic types
- text helpers already admitted by prior PCC phases
- sequence helpers already admitted by PCC-7 fixture-backed surface
- map helpers already admitted by PCC-7 fixture-backed surface
- Option / Result helper surface only where already admitted by PCC-6
- math helpers remain proposed and not yet fixture-backed unless evidence
  exists

## 3. Public / Internal / Deferred Classification

| Helper / family               | Public in Stdlib v0?                | Status                                       | Boundary |
| ----------------------------- | ----------------------------------- | -------------------------------------------- | -------- |
| assert                        | yes                                 | partial / evidence-backed                    | deterministic runtime failure surface |
| print(text)                   | yes                                 | partial / evidence-backed                    | text-only, no arbitrary debug rendering |
| print(non-text)               | no                                  | rejected                                     | must remain diagnostic-stable |
| to_text(admitted basic types) | yes                                 | partial / evidence-backed                    | explicit admitted set only |
| to_text(record)               | no                                  | rejected                                     | no universal reflection |
| to_text(ADT)                  | no unless already admitted          | deferred                                     | no implicit debug formatting |
| to_text(collection)           | no unless already admitted          | deferred                                     | no generic structural rendering |
| debug_render                   | no                                  | internal-only                                | not language semantics, not public stdlib |
| text concat / equality / len   | yes if already admitted              | fixture-backed by PCC-3 / benchmark evidence | keep bounded |
| sequence helpers               | yes if already admitted              | fixture-backed by PCC-7                      | no memory / quota expansion |
| map helpers                    | yes if already admitted              | fixture-backed by PCC-7                      | no missing-key policy expansion |
| Option / Result helpers        | only admitted standard-form surface  | fixture-backed by PCC-6                      | no exception semantics |
| math helpers                   | proposed                             | not closed                                   | require later fixture evidence |

## 4. debug_render Boundary

`debug_render` is internal tooling.

It is not part of Semantic language semantics.
It is not a public stdlib helper.
It must not appear in canonical examples as a substitute for `to_text`.
It must not be used to justify public formatting or reflection behavior.
It may be used only by internal diagnostics and tooling where already
admitted.

## 5. to_text Boundary

`to_text` is not universal reflection.

It only applies to explicitly admitted types.
Unsupported types must reject with stable diagnostics.
Record, ADT, and collection `to_text` behavior must not be inferred from debug
rendering.
Any expansion requires explicit PCC-8 or post-PCC work.

## 6. Failure Behavior

Freeze policy shape:

- helper misuse must be diagnostic-stable or trap-stable;
- assert false must remain deterministic;
- print non-text must reject if current contract says text-only;
- unsupported to_text must reject;
- helper failures must not depend on host state.

## 7. Capability / Host Boundary

Stdlib helpers must not bypass capability gates.

print / output behavior must remain within the admitted runtime boundary.

No host ABI widening is introduced by this contract.
No IO expansion is introduced by this contract.

## 8. Canonical Examples Rule

Canonical examples may use public helpers only.

Canonical examples must not rely on debug_render.
README or public docs must not present internal tooling helpers as language
helpers.

## 9. Follow-up Split

```text
PCC-8C — test(stdlib): lock positive basic helper fixtures
PCC-8D — test(stdlib): lock helper diagnostics and runtime traps
PCC-8E — docs(stdlib): close PCC-8 with evidence sync and roadmap status update
```

PCC-8E closes the current admitted helper surface; it does not expand the
public contract.
