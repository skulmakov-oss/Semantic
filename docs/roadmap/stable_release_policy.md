# Semantic Stable Release Policy

Status: release-governance policy for the published stable line

Read this document using the canonical status vocabulary in:

- `docs/roadmap/public_status_model.md`

Read this policy together with:

- `docs/release_artifact_model.md`

This policy governs:

- the published stable line

It does not automatically promote:

- landed-on-`main` behavior
- or the current qualified limited-release contour

## Current Stable Reading

The current published stable line is:

- `v1.1.1`

Current practical-programming qualification is separate and remains:

- `qualified limited release`

Those are distinct decisions.

## Scope Freeze Rule

While maintaining or validating the stable line:

- do not silently widen the stable promise
- do not treat landed-on-`main` work as stable by default
- do not reopen broader feature scope through release-maintenance PRs

Allowed stable-line work:

- release-facing docs sync
- release asset validation
- packaging fixes
- narrow correctness fixes that are rerun through the full validation contour

## Stable Tag Preconditions

A stable tag or stable-line refresh is allowed only if all relevant release
validation remains green, including:

- workspace tests
- boundary and ownership guards
- public API compatibility checks
- release bundle verification
- release asset smoke verification
- release-facing docs matching actual repository behavior

## FR-9 Release Qualification Path

This path defines the order for release qualification. It does not produce a
release, publish a stable version, or mark the project production-ready.

| Step | Gate / check | Owner | Required before | Produces release artifact? | Notes |
|---:|---|---|---|---|---|
| 1 | Status freeze / scope check | release-facing docs | any release-candidate gate | No | Confirm the release target is explicit, no widened feature surface is smuggled in, current-main behavior is not silently promoted, and the public status vocabulary remains honest. |
| 2 | `PRReady` local gate | `scripts/admission_guard.ps1` | readiness validation | No | Useful for PR admission, but not sufficient for release qualification. This is a local gate, not a GitHub CI dependency. |
| 3 | `Readiness` local gate | `scripts/admission_guard.ps1` | full release-gate validation | No | Stronger readiness validation, but still not publication and not release-ready by itself. |
| 4 | `FullPreflight` local gate candidate | `scripts/admission_guard.ps1` | release bundle or tag decision when explicitly scoped | No | Heavy local gate candidate. Do not run casually; use only under an explicit release-gate scope, not for ordinary docs PR validation. |
| 5 | Release bundle verification | `scripts/verify_release_bundle.ps1` | asset smoke or release-candidate decision | No | Separate from ordinary PR readiness. Requires an explicit release-bundle candidate; documentation-only PRs do not create that bundle. |
| 6 | Release asset smoke verification | `scripts/verify_release_assets.ps1` | publication decision | No | Separate from ordinary PR readiness. Requires explicit candidate artifacts; documentation-only PRs do not create artifacts. |
| 7 | Release-facing docs / status review | public status and release-policy docs | final release decision | No | Public status must match actual behavior. Release notes, if used, must preserve the status disclaimer: published stable remains distinct from qualified limited release, and landed-on-main or current-main-only behavior must not be described as stable. |
| 8 | Final human release decision | project owner / release owner | tag or publication | Yes, only if explicitly approved | Passing gates does not automatically publish. Stable publication requires an explicit later human decision. |

FR-9 release qualification must not imply:

- public stable release before an explicit publication decision;
- production-ready status;
- stable runtime ABI;
- stable binary ISA;
- package registry support;
- dependency solving;
- `smc new` support;
- broad host IO;
- GitHub CI authority;
- release artifact creation by a documentation-only PR.

## Promotion Rule

Behavior should be described as `published stable` only when:

- the stable line explicitly promises it
- supporting release assets and validation cover it

Landed behavior on current `main` remains unpromoted until an explicit later
decision promotes it.

## Publish Rule

Stable release notes should state:

- the exact released commit
- the artifact model for the published asset set
- the validated asset set
- the stable-ready surfaces
- the known limits that remain outside the stable promise

## Non-Commitments

The following remain outside the stable-release critical path unless explicitly
promoted later:

- broader practical-programming widening beyond the current stable promise
- broader executable-module authoring
- UI
- broader runtime and ecosystem work already landed on `main` but not yet
  promoted
