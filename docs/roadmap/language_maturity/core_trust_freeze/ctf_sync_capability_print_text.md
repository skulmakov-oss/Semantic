# CTF-SYNC-4 - Capability Wording for `print(text)`

## Status

Issue body / docs-only follow-up.

## Title

`ctf: clarify print text capability boundary`

## Scope

- `print(text)` as practical helper
- no host ABI widening
- no capability bypass
- no debug/logging framework claim

## Purpose

Keep `print(text)` canonical-safe in PCC without turning it into an unrestricted
host-effect claim.

## Acceptance

- `print(text)` remains canonical-safe but capability-aware
- wording does not imply unrestricted host output
- future namespace / capability policy remains open

## Non-Goals

- no host ABI widening
- no capability bypass
- no debug/logging framework claim

