# ParserProfile Specification

Status: draft v0
Owner crate: `sm-profile`
Primary consumers: `sm-front`, `sm-sema`, `sm-ir`, `smc`

## Purpose

`ParserProfile` is the canonical policy contract for language-surface
acceptance and producer policy.

Contract rule:

- profile defines what is allowed to be parsed and produced
- SemCode header defines what was actually produced
- verifier proves the produced artifact is consistent with that contract

`ParserProfile` is not an embedded runtime contract and is not a second SemCode
metadata layer.

## Canonical Ownership

`ParserProfile` lives in `sm-profile`.

The following are architectural violations:

- duplicate profile schema outside `sm-profile`
- hidden parser defaults deep inside frontend or sema entrypoints
- support or legacy modules acting as a second profile owner

## Current Schema

Current public fields:

- `identity`
- `version`
- `abi`
- `compatibility`
- `features`
- `capabilities`
- `aliases`

Current policy subdomains:

- `AbiProfile`
- `CompatibilityMode`
- `FeaturePolicy`
- `CapabilityExpectations`

## Current Version Policy

Current default public profile version:

- major `1`
- minor `0`

Contract rule:

- incompatible schema or meaning changes require a major version bump
- backward-compatible additions or clarifications require a minor version bump

## Current Default Profile

The current public baseline profile is `ParserProfile::foundation_default()`.

Important rule:

- defaults must be chosen at the public entry boundary
- deeper parser and semantic stages must not silently invent their own hidden
  default profile

Contract rule (PB-01): `ParserProfile::default()` is identical to
`foundation_default()`. The explicit strict profile is `ParserProfile::core()`.

## Policy Scope

ACTIVE policy (enforced by `sm-front` canonical admission):

- `features.allow_f64_math`: whether `f64` surface is allowed
- `features.allow_logos_surface`: whether Logos surface is allowed
- `features.allow_schema_surface`: whether schema surface is allowed
- `compatibility`: whether legacy-compatible surface branches are accepted

RESERVED / NON-AUTHORITATIVE (no production consumer):

- `features.allow_debug_symbols`
- `features.allow_gate_surface`
- `capabilities.*`
- `abi`
- `aliases`

Canonical Semantic source admission (`validate_for_canonical_source`, run by
every `sm-front` `parse_*_with_profile` / `admit_*_with_profile` entry)
rejects a profile whose RESERVED fields differ from the baseline
(`true`, `true`, all `false`, `GateSurface`, empty aliases). A profile can
therefore never appear to deny or require something the compiler ignores.

This is compile-time policy, not runtime authority.

## Capability Expectations

`CapabilityExpectations` expresses profile-level expectations and restrictions.

Important rule:

- `CapabilityExpectations` is RESERVED declarative metadata; no field is
  enforced and none ever grants a capability
- profile expectations do not replace the actual SemCode capability contract
- capability bits in the produced artifact must still be derived from actual
  usage

That means:

- profile may allow more than a specific program uses
- producer must not emit extra capability claims just because the profile
  allows them

## Compatibility Mode Rule

`CompatibilityMode` affects surface acceptance only.

It may permit legacy syntax. It does not enable aliases.

## Alias Rule

`aliases` is the TON618 legacy alias vocabulary (`! & | ^ N F T S`). It is
not part of canonical Semantic source admission; a non-empty alias map is
rejected there. Legacy alias training and line normalization live in the
TON618 compatibility perimeter (the TON618 compatibility binary (`src/bin/`)).

One alias rule (`sm_profile::validate_alias`) and one conflict rule
(`ParserProfile::add_alias`) apply to every entry path (direct, training,
JSON): exact duplicates are idempotent, a different target for an existing
raw token is a conflict, and nothing is ever overwritten.

It must not weaken:

- verifier admission
- runtime isolation
- capability enforcement
- SemCode safety guarantees

## Serialization Rule

The canonical serialized form is the JSON schema used by `sm-profile`.

`from_json` / `load_from_file` are semantic admission points:

- only version `1.0` (`ProfileVersion::SUPPORTED`) is admitted; any other
  major or a higher minor is rejected
- unknown fields are rejected at every nesting level
- duplicate alias keys are rejected
- every alias must pass `validate_alias`
- identity must be non-empty

Serialization requirements:

- deterministic roundtrip
- no silent semantic field loss
- version fields preserved
- alias map preserved

Any schema change requires:

1. update this specification
2. update `sm-profile` roundtrip tests
3. update user-facing validation behavior if public semantics changed

## No Silent Mutation Rule

The following are forbidden without version review:

- changing the meaning of an existing feature flag
- changing the meaning of an existing compatibility mode
- changing ABI policy interpretation
- changing serialized schema semantics while pretending the version is unchanged
