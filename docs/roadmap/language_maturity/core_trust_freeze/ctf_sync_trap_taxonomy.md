# CTF-SYNC-2 - Trap Taxonomy Edge Cases

## Status

Issue body / docs-only follow-up.

## Title

`ctf: record PCC trap taxonomy edge cases`

## Scope

- missing map key
- sequence out-of-bounds
- empty `pop`
- unsupported `to_text`
- invalid `print`
- invalid collection key/value
- invalid control-flow misuse

## Purpose

Keep unresolved PCC failure edges visible without forcing them into a frozen
trap class too early.

## Acceptance

- each edge is classified as diagnostic / trap candidate / unresolved policy
- no runtime behavior is changed
- unresolved semantics are explicitly marked

## Non-Goals

- no runtime behavior change
- no trap-class promotion by wording alone
- no CTF closure claim

