# CTF-SYNC-1 - RuntimeValue Registry Alignment

## Status

Issue body / docs-only follow-up.

## Title

`ctf: align RuntimeValue registry wording with PCC practical surface`

## Scope

- `quad`
- `bool`
- `i32`
- `u32`
- `text`
- `Sequence(T)`
- `Map(K, V)`
- `Option(T)`
- `Result(T, E)`

## Purpose

Record that PCC now uses the following value families as practical surfaces:

- `quad`
- `bool`
- `i32`
- `u32`
- `text`
- `Sequence(T)`
- `Map(K, V)`
- `Option(T)`
- `Result(T, E)`

## Acceptance

- registry wording reflects PCC-qualified value families
- no new RuntimeValue implementation is introduced
- collection carrier wording remains conservative
- ABI / final representation is not overclaimed

## Non-Goals

- no RuntimeValue implementation change
- no VM change
- no SemCode change
- no new carriers
- no final ABI promise for collections

