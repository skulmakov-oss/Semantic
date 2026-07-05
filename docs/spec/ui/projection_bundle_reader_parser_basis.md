# ProjectionBundle Reader/Parser Basis v0

Status: basis
Track: POST-UI / Intent-Driven Projection
Scope type: reader/parser claim basis
Current achieved level: Level 3 baseline
Reader evidence status: narrow reader-facing fixture evidence only
General Level 4 status: not claimed
Loader status: not claimed
Runtime status: not claimed
Production UI status: not claimed
Authority status: non-authorizing
Levels 4–7 are not claimed.

This document defines the basis required before any general ProjectionBundle reader/parser claim may be made.

It does not implement a reader.
It does not implement a parser.
It does not define final serialization.
It does not define a loader.
It does not define runtime behavior.
It does not authorize production UI wiring.

## 1. Purpose

This document defines the basis required before any general ProjectionBundle reader/parser claim may be made.

It does not implement a reader.
It does not implement a parser.
It does not define final serialization.
It does not define a loader.
It does not define runtime behavior.
It does not authorize production UI wiring.

The purpose of this basis is to prevent wishful thinking from becoming architecture.

## 2. Current Evidence Boundary

Current evidence:

- positive inert sketch accepted;
- positive normalized reader output golden fixture;
- negative fixture pack rejected;
- negative rejection report golden fixture;
- exact golden comparison guard;
- claim-boundary guard.

This evidence is narrow reader-facing fixture evidence only.
It does not satisfy a general Level 4 claim.

## 3. Reader vs Parser Meaning

Reader:

- consumes one approved fixture-facing input form;
- extracts expected fields;
- emits deterministic observable output;
- rejects known invalid fixture cases;
- has no loader authority;
- has no runtime authority;
- has no verification authority.

Parser:

- consumes a specified input grammar or serialization;
- produces a deterministic representation;
- has explicit error semantics;
- handles unknown, duplicate, missing, malformed, and out-of-order fields according to a documented policy;
- has no loader authority;
- has no runtime authority;
- has no verification authority.

A reader may be narrower than a parser.
A reader/parser basis does not by itself claim Level 4.

## 4. Minimum Evidence Required Before General Level 4

Before any general Level 4 claim, the following must exist:

- approved reader/parser input boundary;
- approved output representation boundary;
- deterministic output golden tests;
- deterministic rejection golden tests;
- missing-field rejection tests;
- malformed-field rejection tests;
- unknown-field policy tests;
- duplicate-field policy tests;
- field-ordering policy tests;
- placeholder trust rejection tests;
- no activation path;
- no loader path;
- no runtime path;
- no production UI wiring;
- claim-boundary guard update.

## 5. Required Input Boundary

Future reader/parser work must define these input boundary categories without selecting final serialization:

- accepted input form;
- rejected input form;
- malformed input form;
- unknown field behavior;
- duplicate field behavior;
- field ordering behavior;
- placeholder trust behavior;
- line ending behavior;
- path normalization behavior.

This basis does not select JSON, YAML, TOML, binary, or any final serialization.

## 6. Required Output Boundary

Future reader/parser output must have:

- stable field order;
- stable section order;
- stable path form;
- stable boolean spelling;
- stable rejection reason strings;
- no timestamps;
- no absolute paths;
- no host-specific data;
- no OS-specific data;
- no rustc-version data;
- no nondeterministic ordering.

## 7. Error Model

The error model must include:

- missing_required_field;
- malformed_field;
- unknown_field;
- duplicate_field;
- invalid_policy_value;
- placeholder_trust_rejected;
- unsupported_input_form;
- internal_guard_error.

Every rejected input must produce one stable primary error.
Secondary errors may exist later, but primary error ordering must be deterministic.

## 8. Trust Placeholder Policy

Placeholder hash and signature values are fixture evidence only.
A future general reader/parser claim must prove placeholder trust rejection.
A reader/parser must not convert placeholder trust into activation authority.
Verification status remains not_verified unless a separate verification implementation exists.

## 9. Authority Boundary

Reader/parser output is representation only.

It does not admit semantic content.
It does not grant capability authority.
It does not create audit authority.
It does not load bundles.
It does not activate bundles.
It does not render UI.
It does not authorize production UI behavior.

Semantic owns meaning.
Projection owns presentation intent.
UI IR owns structure.
Action IR owns affordance routing.
Binding Graph owns deterministic dependency mapping.
Patch streams own projection updates.
Shell owns rendering behavior.
Renderer owns pixels.
Verifier / admission owns semantic admission decisions.
Capability / audit authority owns capability checks, host-effect permission, critical action authorization, and audit evidence boundaries.
Runtime owns execution / scheduling only where explicitly specified.

This basis does not move authority between layers.

## 10. Promotion Rule

General Level 4 may not be claimed until the Level-4 evidence matrix is complete.

Level 5+ may not be claimed from reader/parser evidence.

Any PR that claims Level 4 must update the evidence matrix and must be reviewed against this basis.
