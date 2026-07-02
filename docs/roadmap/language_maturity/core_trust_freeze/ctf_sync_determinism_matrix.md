# CTF-SYNC-3 - Determinism Matrix Additions

## Status

Issue body / docs-only follow-up.

## Title

`ctf: update determinism matrix for PCC surfaces`

## Scope

- text concatenation
- `to_text(...)`
- sequence iteration
- map helper behavior
- `assert`
- `print(text)`

## Purpose

Record the deterministic PCC surfaces that now matter to the trust lane.

## Acceptance

- deterministic surfaces are listed
- map iteration remains out of scope
- output / capability effects are not treated as unrestricted host ABI

## Non-Goals

- no deterministic behavior change
- no map iteration promotion
- no host ABI widening

