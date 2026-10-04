# PB-03 — Semantic Core Integrity Decision

Status: decided
Scope: Phase-B PB-03, `sm-sema` Logos semantic core: #1671–#1680, #1693–#1696, #1705
Historical audit umbrella: #1617 · Previous slices: PB-01 (#1985), PB-02 (#1986) · Residual: #1987 (untouched)
Base: `main` `8305a8a1fe8fda66a3fa7648076ea0e5e73f167d`
Reference oracle (unchanged): C1 `89641da8237f4fcefb50cf1958a50e4d4003aea7` = `v1.2.0`

Invariant: unknown, unresolved, malformed, ambiguous, overflowed or
contradictory semantic state stays explicit and fails closed.

## Base reproductions (all confirmed at `8305a8a1`)

| Issue | Base behavior |
|---|---|
| #1671 | `Type::Quad → QVec(1)`; the canonical spec example `When Sensor.val == T` is **rejected** (E0201) |
| #1672 | `i32, u32 → Int`; `f64, fx → Fx` |
| #1673 | `is_compatible_cmp(Unknown, Unknown) = true`; `When MissingA == MissingB` admitted |
| #1674 | `"nope && junk"`, `"x \| y"` → `Quad` |
| #1675 | `"Present("`, `"fooPresent(x"` → `Bool` |
| #1676 | `"."`, `"1..2"`, `"1.2.3"` → `Fx` |
| #1677 | `When N` emits W0240 "condition is always false" |
| #1678 | W0241 folds `fx.add(0.1, 0.2)` with host `f64` → `0.30000000000000004` |
| #1679 | `Log.emit("Sensor.val")` counts as a use of `Sensor.val` |
| #1680 | W0253 fires on `Sensor2` / `"v42"` |
| #1693 | unqualified `v` resolves via a Law owner guessed from the first `When` |
| #1694 | two `Law "Check"` in one module admitted (different guessed owners) |
| #1695 | hand-built Entity with duplicate field (`bool`, `quad`) admitted; the insert error was discarded |
| #1696 | 65,537 distinct types: `Bool` gets an ID that resolves to `QVec(1)` |
| #1705 | `equals_fast(..)` computed and assigned to `_` in `analyze_logos_program` |

## Decisions

| # | Question | Decision |
|---|---|---|
| 1 | Authoritative `SemanticType` model | `I32, U32, F64, Fx, QVec(N), Mask, Str, Bool, Quad, Unit, Unknown`. `From<sm_front::Type>` is total; non-Logos composites map to `Unknown`. |
| 2 | `quad` vs `QVec(1)` | Distinct. Source `quad` → `Quad`. |
| 3 | `i32` vs `u32` | Distinct (`docs/spec/types.md`: no implicit cross-family coercion). |
| 4 | `f64` vs `fx` | Distinct. The former `Int ↔ Fx` compatibility is removed. |
| 5 | Is `Unknown` a valid comparison operand? | Never. `is_compatible_cmp` is false whenever either side is `Unknown`; unresolved names/fields are explicit `ConditionInferError::Unresolved`. |
| 6 | Owner of Logos condition structure | `sm-front`. The parser projects each `When` into `LogosWhen::structure: LogosCondition` and `condition_atoms` / `effect_atoms: Vec<LogosAtom>` from canonical tokens. `sm-sema` interprets only that projection. |
| 7 | May `sm-sema` infer meaning from substrings? | No. The text heuristics (`infer_atom_type_core`, `infer_when_condition_type_core`, `track_entity_field_usage_core`, `has_magic_number_core`, `infer_law_entity_core`, `fold_fx_const_call_core`) are removed. |
| 8 | `Present(...)` contract | `Present(x)` with exactly one name or `Entity.field` atom, matching parentheses, nothing else in the condition. The atom must resolve; result `Bool`. |
| 9 | Numeric literal lexical form | The canonical `sm-front` lexer `Num` token: `Integer` (no `.`) or `Decimal`. `.`, `1..2`, `1.2.3` are not a single numeric token, so the condition is `Unsupported` and rejects. |
| 10 | Quad `N` in dead-condition analysis | `N` is unknown and never "always false". W0240 fires only for a condition proven `bool`-false: literal `false`, or a comparison of two literals of one family whose result is false (e.g. `T == F`). A quad-valued condition (`N`, `F`, `T`, `S`, evidence ops) is never claimed dead, because no Logos contract defines quad-valued `When` firing. |
| 11 | Authoritative `fx` evaluator for diagnostics? | Fixed-point arithmetic exists only in `sm-vm` and `sm-ir` (above `sm-sema`, and duplicated), with no shared literal encoding. W0241 fx constant folding is therefore removed rather than approximated with host `f64`. The W0241 catalog entry is unchanged. |
| 12 | Module-level Law namespace | Law names are module-level. Duplicates are keyed on module + public name (E0221). |
| 13 | Implicit owning Entity for Laws? | None. Logos syntax declares no owner. The `_global` fallback is removed. |
| 14 | Unqualified Entity fields inside Laws? | Not valid: no contract declares shorthand. Entity ownership comes only from `Entity.field`. An unqualified name that is not a module symbol is unresolved. |
| 15 | Symbol insertion collision | Rejects. Entity names already map collisions to E0220. The Law-scope field-injection path (the only discarded `insert`) is deleted. Duplicate field names inside one Entity are rejected (E0220). |
| 16 | Max TypeRegistry size | `TypeId(u16)`: 65,536 distinct types (IDs 0..=65535). `intern` returns `Result<TypeId, TypeRegistryError::CapacityExhausted>`; a failed intern leaves the registry unchanged. |
| 17 | Does TypeRegistry ID equality participate in admission? | No (option B). It is canonical storage/debug identity only; `analyze_logos_program` no longer uses it. |
| 18 | Intentional changes from C1 | Every change maps to the issue set: `quad` fields compare with quad literals (#1671); cross-family numeric comparisons reject (#1672); unresolved operands reject (#1673); malformed or unmodeled conditions reject (#1674–#1676); no W0240 for quad-valued conditions (#1677); no W0241 fx folding (#1678); W0252/W0253 only on parsed atoms (#1679, #1680); module-level duplicate Laws reject and unqualified field shorthand is no longer resolved (#1693, #1694); duplicate Entity fields reject (#1695); `TypeRegistry::intern` is fallible (#1696); no TypeId gate (#1705). |
| 19 | Behaviors preserved | Logos parse acceptance (the projection is additional data); `When true`/`Sensor.val == T`/`S` corpus; E0220/E0222/E0223/W0250/W0251; all Rust-like/Foundation behavior (`SemanticType` is Logos-only); Model B (no Logos compile/VM/SemCode). |

## Modeled condition surface (`LogosCondition`)

- `Atom(a)`
- `Compare { lhs, op: == | !=, rhs }`
- `Present(a)`
- `Not(a)` (`!`)
- `Evidence { op: && | ||, operands }` (same-operator chain of two or more atoms)
- `Unsupported`: well-lexed tokens forming none of the above. `sm-sema` rejects it.

Atoms: `true`/`false`, `N`/`F`/`T`/`S`, integer/decimal `Num`, string literal, name, `Entity.field`.

Typing:
- Numeric atoms carry the canonical `sm-front` literal family (`NumericLiteral`, the same rules as RustLike numeric literals): unsuffixed integer is `i32`, unsuffixed decimal is `f64`, and a suffix selects `i32`/`u32`/`f64`/`fx`. There is no contextual literal typing and no family coercion, so `u32_field == 5` rejects and `u32_field == 5u32` is admitted.
- Evidence and `!` operators require uniform `Quad` (→ `Quad`) or uniform `Bool` (→ `Bool`) operands.

## Cross-crate changes

- `sm-front`: the `LogosWhen` projection (`structure`, `condition_atoms`, `effect_atoms`) and the `LogosAtom`/`LogosCondition`/`LogosCompareOp`/`LogosEvidenceOp`/`NumericLiteral` re-exports. Built once in the parser from canonical tokens, it is the single syntax authority.
- `ton618-core` catalog and `docs/ERROR_CODES.md` mirror: E0221 text now says "Duplicate Law name within one module." (#1694); the W0241 entry is kept and marked not emitted (#1678).
- `smc-cli` and `tests/sema_diagnostic_catalog_coherence.rs`: warning fixtures that used `When N` as a W0240 generator now use `When false` (#1677); the W0241 rendered-help case is removed (#1678).

## Model-B contract preservation exposed by FA-03-002

On base, unformatted `smc check` rejected the canonical Logos example
`quad_cycle_logos` only because of the #1671 `quad → QVec(1)` defect. Once that
was repaired, Logos semantic analysis would have let `smc check` succeed,
silently widening the sealed SSF-02 Model-B contract. Owner decision: the
legacy, unformatted `smc check` is executable-source admission and rejects
Logos-owned source with the stable `SOURCE SURFACE BOUNDARY` diagnostic
(`smc-cli` `cmd_check`, intercepting only a would-be success, both cached and
fresh). Existing failure output is unchanged.

Deliberately unchanged:
- `smc check --format human|json` and `smc lsp` (`canonical_check`, the sealed
  SSF-09 diagnostics authority) still analyze Logos for diagnostics;
- `dump-ast` and `dump-ir --profile logos` still allow inspection;
- `compile`, `verify`, `run` and `dump-bytecode` still reject.

`tests/pb03_logos_check_boundary.rs` pins that the two `check` surfaces
intentionally differ. This is not a new Logos feature.

## SSF-09 fixture correction required by FA-03-008 semantic repair; diagnostics authority unchanged

Three SSF-09 editor-baseline fixtures used `When N` purely to trigger W0240:
`rootless/logos_warnings.sm` and the `src/util/helper.sm` files of
`project_ok` and `project_broken`. After #1677, `N` is unknown and is not
"always false", so those fixtures now use the genuinely false `When false`. The
derived goldens were regenerated through `SM_UPDATE_SSF09_GOLDENS=1`; only the
quoted source-snippet line changes. `canonical_check`, LSP transport, the schema
format, W0240's code, severity and rendering are unchanged.
`quad_n_is_not_always_false` pins `When N` → no W0240 and `When false` → W0240.
