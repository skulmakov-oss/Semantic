# Private Custody Mode

Status: repository custody and access discipline note.

This document records the operational posture used when the Semantic repository
is private. It does not change the language, SemCode, verifier, VM, runtime,
capability, audit, or UI contracts.

## Purpose

Private custody mode exists to protect active R&D while preserving the same
engineering discipline used for public-facing work.

The privacy boundary is an access-control boundary, not a shortcut around the
project's normal evidence rules.

## Invariants

- `main` remains protected by pull-request flow.
- Squash merge remains the preferred merge path for a clean `main` history.
- Code, spec, and docs changes still move through reviewable branches.
- Claims about implemented behavior still require matching tests or explicit
  inspection evidence.
- Private visibility must not be used to hide unstable or contradictory public
  contract language.
- External publication remains a separate release decision, not an automatic
  consequence of landing changes on `main`.

## Access Discipline

Private repository access should be treated as capability-bearing access.

Minimum rules:

- grant access only to accounts that need the repository;
- avoid using alternate accounts for real project operations unless there is a
  clear ownership or recovery reason;
- keep branch and PR history reviewable even for private-only work;
- prefer issues, PR bodies, and docs over private memory for architectural
  decisions;
- record sensitive exclusions as scope boundaries without exposing unnecessary
  implementation detail.

## PR Discipline While Private

Docs-only, code, and mixed PRs should keep their intent explicit.

| PR class | Expected evidence |
|---|---|
| Docs-only | changed document list, scope boundary, no contract widening unless stated |
| Spec | affected contract surface, precedence note, downstream implementation impact |
| Code | tests, negative cases where relevant, verifier/runtime boundary statement |
| Release-facing | checklist alignment, asset or CI evidence, known limits preserved |

A private PR may be smaller and more iterative, but it should not become
untraceable. The repository being private reduces outside exposure; it does not
reduce the need for internal auditability.

## Relationship to Public Documents

Existing documents that use public-facing language still matter. They define
what can safely be said if a release, snapshot, README, paper, or external
package is later published.

Private custody mode therefore separates two questions:

```text
What is safe to work on privately?
What is safe to claim publicly?
```

The first question is governed by access control and project custody. The
second is governed by specs, tests, release notes, and explicit non-claims.

## Non-Goals

Private custody mode does not imply:

- weakening branch protection;
- bypassing CI when CI is relevant;
- hiding known limitations;
- treating planned architecture as implemented behavior;
- publishing private details by accident;
- converting Semantic into an undocumented private prototype.

## Operational Checklist

After changing repository visibility or access policy, verify:

- repository visibility matches the intended custody state;
- `main` is still the default branch;
- pull request protection remains active for `main`;
- merge commits and rebase merges remain disabled if squash-only history is the
  intended policy;
- CI still runs on pull requests;
- external-facing documents still distinguish implemented, stable, and planned
  behavior;
- release-facing documents do not claim broader behavior than the current tests
  and specs support.

## Blocking Rule

Do not use private visibility as a substitute for architectural clarity.

If a change would be misleading in public language, either keep it private and
clearly marked as experimental, or update the relevant status, roadmap, and
non-claim documents in the same change series.
