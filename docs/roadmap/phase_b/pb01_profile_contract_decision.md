# PB-01 — ParserProfile Contract Decision

Status: decided (owner decision 2026-10-04)
Scope: Phase-B PB-01, Module 01 `sm-profile`, issues #1618–#1632
Historical audit umbrella: #1617
Base: `main` `8ebd32945df2a8df80d89dcc8b1dd3f49b41fbf6`
Reference oracle (unchanged): C1 `89641da8237f4fcefb50cf1958a50e4d4003aea7` = `v1.2.0`

## Forensic findings that shaped the decision

1. The only production readers of profile policy are `sm-front`
   (`features.allow_f64_math`, `features.allow_logos_surface`,
   `features.allow_schema_surface`, `compatibility`). Every canonical
   production entry (`smc-cli`, `sm-ir`, `sm-front` no-profile wrappers)
   uses `ParserProfile::foundation_default()`.
2. `allow_debug_symbols`, `allow_gate_surface`, every
   `CapabilityExpectations` field, `abi` and `aliases` had no production
   consumer.
3. The alias vocabulary (`! & | ^ N F T S`) belongs to the TON618 quad-logic
   line language (`z = T & ! a`). In canonical Semantic a lone `&` is a lex
   error, `^` is not a token and `|` is the or-pattern separator. The only
   callers of `normalize`, `train_profile*` and `add_alias` were
   the TON618 compatibility binary (`src/bin/`), the retained, non-owning TON618 compatibility
   perimeter.

## Decisions

| # | Question | Decision |
|---|---|---|
| 1 | Canonical public baseline constructor? | `ParserProfile::foundation_default()`. |
| 2 | What does `Default` mean? | Exactly `foundation_default()`. |
| 3 | May `Default` differ from `foundation_default()`? | No. The former strict meaning is the explicit `ParserProfile::core()`. |
| 4 | What does training start from? | An explicit caller-supplied baseline (`train_profile(base, samples)`). TON618 passes `ParserProfile::core()`. |
| 5 | Is training strict or best-effort? | Strict and atomic: any rejected evidence returns `TrainingError` listing every rejection and leaves the profile unchanged. There is no best-effort mode (no caller needs one). |
| 6 | Canonical alias conflict policy? | One rule (`ParserProfile::add_alias`): exact duplicate → idempotent `Ok`; same raw, different target → `AliasError::Conflict`; never first-wins or last-wins. Duplicate JSON alias keys fail decoding. |
| 7 | Are aliases part of canonical source admission? | No. They are RESERVED for canonical Semantic. A non-empty alias map is rejected at canonical admission (`ProfileError::AliasesNotCanonical`). |
| 8 | Which layer applies them? | None in the canonical contour. The TON618 perimeter (the TON618 binary's `alias_compat` module) applies them to TON618 lines only. |
| 9 | How are aliases kept from rewriting literals/comments? | Canonical Semantic never applies aliases, so no Semantic literal or comment can be rewritten. `ParserProfile::normalize` (a second, incomplete lexer over Semantic text) is removed from `sm-profile`; the line tokenizer lives only in TON618, whose line language has no text literals. |
| 10 | Profile JSON version compatibility rule? | Exactly `ProfileVersion::SUPPORTED` = 1.0 is admitted. Any other major is incompatible; a higher minor may carry unknown semantics and is also rejected (fail closed). |
| 11 | Unknown JSON fields? | Rejected at every level (`deny_unknown_fields` on `ParserProfile`, `ProfileVersion`, `FeaturePolicy`, `CapabilityExpectations`). |
| 12 | ACTIVE `FeaturePolicy` fields | `allow_f64_math`, `allow_logos_surface`, `allow_schema_surface` (enforced by `sm-front`), plus `CompatibilityMode`. |
| 13 | RESERVED fields | `allow_debug_symbols`, `allow_gate_surface`, all `CapabilityExpectations`, `abi`, `aliases`. Canonical admission requires the baseline values (`true`, `true`, all `false`, `GateSurface`, empty), so no profile can appear to deny or require what the compiler would ignore. |
| 14 | ACTIVE `CapabilityExpectations` | None. They are declarative metadata and never a capability grant; SemCode capabilities stay derived from actual usage. |
| 15 | What is `AbiProfile` authoritative over? | Nothing. It is RESERVED/non-authoritative; no ABI boundary consumes it. |
| 16 | Schema/version bump? | No (option A: a repair that preserves the 1.0 contract). The serialized shape is unchanged. The repair makes 1.0 mean what `docs/spec/profile.md` already claimed, so `semantic.foundation/1.0` (the Foundation Source contract's profile) stays valid and byte-identical in JSON. Documents now rejected were already outside the documented contract (unknown fields, unsupported versions, invalid aliases). |
| 17 | Behaviors identical to C1 | Every canonical compile/parse/check with `foundation_default()` (it is accepted unchanged by the new gate); `core()` admission behavior equals the former `Default` for every ACTIVE field; TON618 line normalization output; `foundation_default()` JSON. |
| 18 | Intentional changes after v1.2.0 | See the table below. |

## Intentional behavior changes (each maps to an FA issue)

| Change | Issue |
|---|---|
| `ParserProfile::default()` now equals `foundation_default()`; former strict default is `core()` | #1618 |
| Training rejects token-count mismatch instead of skipping it | #1619 |
| Conflicting training evidence is an explicit error | #1620 |
| `normalize` removed from `sm-profile` (moved to TON618 line perimeter) | #1621, #1622 |
| `from_json`/`load_from_file` reject unsupported versions | #1623 |
| Canonical admission rejects non-baseline RESERVED values | #1624, #1625, #1626, #1627 |
| Canonical admission rejects non-empty aliases | #1628 |
| `add_alias`/JSON/training share `validate_alias` | #1629 |
| Rejected training candidates are reported | #1630 |
| Unknown JSON fields fail closed at every level | #1631 |
| `add_alias` returns `Result` and never overwrites | #1632 |

`core()` pins its RESERVED fields to baseline (`allow_debug_symbols`,
`allow_gate_surface` = `true`, `abi` = `GateSurface`), unlike the former
`Default`. This is the only value difference and has no production effect
because those fields had no consumer (#1624, #1626, #1627). TON618 profiles
saved before PB-01 (`abi: Core`, reserved `false`) still load; they are only
refused by canonical Semantic admission, which never consumed them.

## Ownership after PB-01

- `sm-profile`: schema, `validate`, `validate_for_canonical_source`,
  `validate_alias`, conflict rule, version policy. It contains no lexer.
- `sm-front`: the canonical lexer, plus one private
  `canonical_profile_admission` gate run by all four `parse_*`/`admit_*`
  profile entries.
- TON618 (TON618 binary, `alias_compat` module): line tokenizer, legacy
  normalization, strict training. It is a non-owning compatibility perimeter.

The dependency direction is unchanged (`sm-front → sm-profile`; no cycle).

Known limit: `ParserProfile` fields stay `pub` (struct-update construction is
used across the workspace). A direct `aliases` edit therefore bypasses
`add_alias`, but `validate()` re-checks it at every serialization boundary,
and canonical admission rejects any alias at all.
