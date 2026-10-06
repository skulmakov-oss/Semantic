# Deprecation And Migration Policy

Status: current governance companion to SSF-10 (#1378). Technical
compatibility rules live in
[`docs/architecture/artifact_identity_and_trust.md`](../../architecture/artifact_identity_and_trust.md)
§2 and §8; this document governs how changes to those surfaces are made and
announced. It is not a release qualification authority and promotes no surface
to stable.

## Purpose

Make platform evolution explicit instead of forcing users to discover breaking
changes accidentally. Final rules:

```text
No silent compatibility promise.
No silent migration.
No silent removal.
No stable claim without release decision.
Deprecated behavior gets explicit diagnostics.
Migration is dry-run first.
Verifier remains final admission authority after migration.
```

## Compatibility Classes

Classes reuse the public status vocabulary
(`docs/roadmap/public_status_model.md`):

| Class | Meaning | Change rule |
|---|---|---|
| Published stable | inside a published stable release (`v1.2.0`, Windows x64, explicit limits R1–R4) | meaning must not silently change; deprecate before removal; migration note required |
| Qualified limited release | qualified contour not yet a stable promise | changes need a compatibility note; still no silent reinterpretation |
| Landed on `main`, not yet promised | current-main behavior | may change, but must stay honestly labeled |
| Experimental / draft | explicitly labeled draft | may change without deprecation, never presented as stable |
| Internal / out of scope | not a public contract | no compatibility promise |

## Deprecation States

Aligned with SSF-10 §2.A (`Compatible` / `Deprecated` / `Removed`):

| State | Meaning |
|---|---|
| CURRENT (`Compatible`) | supported with its documented meaning |
| DEPRECATED | still supported; emits a deterministic warning naming the replacement; minimum one minor/epoch cycle before removal (SSF-10) |
| REMOVED | rejected with a deterministic diagnostic; never silently reinterpreted |
| EXPERIMENTAL / DRAFT | labeled unstable; outside deprecation guarantees |

Removing or reinterpreting a stable/current contract requires an explicit
deprecation or an explicit compatibility/migration path.

## Surface Policies

| Surface | Policy (authority) | Maturity note |
|---|---|---|
| Source language | `Compatible`/`Deprecated`/`Removed`; removed constructs are compile errors (SSF-10 §2.A) | much of the source contract is still draft v0 spec |
| Project / manifest | schema versioned independently; unknown/future schema fail-closed; schema 0 admitted with warning; no silent manifest rewrite (SSF-10 §2.B) | package baseline is local-only |
| CLI machine contracts | stable JSON fields need deprecation or a versioned schema | only documented machine outputs are contracts |
| Diagnostics | `semantic.diagnostics` v1; renumbering/removing codes, severity flips or JSON shape changes require a schema bump (SSF-10 §2.C) | public codes listed in the diagnostic catalog |
| Standard library | `semantic-stdlib-v1`; additions non-breaking; signature or capability widening is breaking (SSF-10 §2.D) | `std.serde` (G-08) excluded |
| SemCode | `SEMCOD22` rev 23; revisions `< 23` Deprecated, `> 23` Unsupported (SSF-10 §2.F) | no stable binary ABI claimed |
| Verifier profile | `verifier-canonical-v1`; admission is final after any migration | profile changes require version review |
| Runtime deterministic profile | `deterministic-v1`; PRNG or determinism changes are breaking (SSF-10 §2.E) | — |
| Capabilities | adding a capability requirement to an existing API is breaking; capability widening needs threat-model review (`docs/security/threat_model_v0.md` §12) | no broad host I/O claimed |
| Package baseline | local package/manifest baseline only | lockfile and registry migration deferred (below) |

## Migration Behavior

- `smc migrate check|preview` is analysis with guaranteed zero filesystem
  mutation (SSF-10 §8; `tests/ssf10_artifact_trust.rs` test 10).
- No tool silently rewrites source, manifests or artifacts.
- Any future apply-mode migration must be dry-run first, show its plan, and
  leave verifier admission as the final gate.
- No new migration command is introduced by this policy.

## Bootstrap Migration Rule

During self-hosting bootstrap qualification (#1910), the generations `C0`,
`C1` and `C2` must not silently migrate each other's compiler source or
artifacts. If a source, manifest, SemCode, diagnostic or runtime contract
changes between generations, the change must be:

- explicit;
- versioned or otherwise identified;
- represented in the bootstrap inputs;
- reflected in the comparison contract.

A fixed-point test must never succeed because one compiler silently rewrites
or normalizes unsupported input behind the other's back (see
`docs/security/threat_model_v0.md` §11 and
`docs/security/artifact_provenance_and_signing_policy_v0.md` §7).

## Deferred Surfaces (Explicit Non-Claims)

Blocked until they exist and have their own policy:

- Workbench / Studio local data migration — retired contour;
- ALM trace / skill / adapter migration — not implemented;
- package lockfile and registry migration — not implemented (#1376 closed not
  planned before self-hosting);
- UI snapshot schema migration — historical inspection tooling only.

## Release Notes Rule

Every compatibility-relevant release must state:

- Added / Changed / Deprecated / Removed;
- migration notes, linking the migration path when one exists;
- known compatibility limits;
- the status layer of each change (stable vs qualified vs current-main).

Release notes must not promote current-main features silently, and removals
must name their prior deprecation state.

## Fixtures

Existing: `tests/ssf10_artifact_trust.rs` (compatibility classification,
boundary rejection, migrate preview zero mutation). Future fixtures proposed by
#1378 (`tests/compat/*`: deprecated source warning, removed source diagnostic,
manifest old-format plan, diagnostic code mapping) are listed for the first
real deprecation; UI-snapshot and ALM fixtures are deferred with their surfaces.

## Cross-References

- `docs/architecture/artifact_identity_and_trust.md` (SSF-10 technical authority)
- `docs/roadmap/language_maturity/stability_and_compatibility.md`
- `docs/roadmap/language_maturity/compatibility_policy_stack.md`
- `docs/roadmap/public_status_model.md`
