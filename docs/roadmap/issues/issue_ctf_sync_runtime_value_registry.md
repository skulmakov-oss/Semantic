Title: ctf: align RuntimeValue registry wording with PCC practical surface

## Description

Align the Core Trust Freeze runtime value registry wording with the PCC practical surface that is now closed.

The goal is to keep the trust registry honest about which value families are actually exercised by the qualified PCC contours, without changing implementation behavior.

## Source Doc

- `docs/roadmap/language_maturity/core_trust_freeze/ctf_sync_runtime_value_registry.md`

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

## Acceptance Criteria

- runtime value families are listed in wording that reflects the PCC-qualified surface;
- no new RuntimeValue implementation is introduced;
- collection carrier wording remains conservative;
- ABI / final representation is not overclaimed;
- unresolved carrier details remain explicitly marked as follow-up.

## Out of Scope

- do not change RuntimeValue implementation;
- do not change VM behavior;
- do not change SemCode;
- do not introduce new carriers;
- do not claim a final ABI for collections.
