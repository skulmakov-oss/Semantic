# ProjectionBundle Reader Vocabulary Split Audit

Status: PASS

## Scope

This audit records the reader vocabulary split after the completed post-ui cleanup chain.

No reader code, probe fixture, snapshot, docs, Cargo, loader, runtime, or production UI behavior is changed by this slice.

## Cleanup chain

- #1402 centralized `EXPECTED_SECTIONS`.
- #1403 centralized `EXPECTED_SCALAR_KEYS`.
- #1418 centralized `KNOWN_SECTIONS` / `KNOWN_FIELDS`.

## Vocabulary layers

### `EXPECTED_SECTIONS`

Baseline expected section list used by reader validation.

This is not the complete known section vocabulary.

### `EXPECTED_SCALAR_KEYS`

Expected scalar key list used by duplicate-scalar validation.

This does not include array keys.

### `KNOWN_SECTIONS` / `KNOWN_FIELDS`

Allowed vocabulary used by:

- unknown-item detection;
- probe snapshot unknown item classification.

This vocabulary includes scalar fields and array fields.

## Important distinction

`EXPECTED_*` and `KNOWN_*` are intentionally separate.

Examples:

- `projection_bundle` root is known vocabulary but is not part of `EXPECTED_SECTIONS`.
- `source_refs` is a known field but is an array field, not a scalar key.
- `required_capabilities` is a known field but is an array field, not a scalar key.
- `diagnostics.expected` is known vocabulary for optional diagnostics handling.

## Non-claims

This audit does not claim:

- loader contract readiness;
- runtime integration;
- production UI activation;
- Level 4 or Level 5 readiness;
- cryptographic trust verification.

## Verification

- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/harness-check.ps1`
- No reader source changes.
- No snapshot changes.
