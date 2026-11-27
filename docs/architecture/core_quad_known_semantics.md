# Core quad: "known", raw planes and legacy delta semantics

Status: compatibility documentation for `semantic-core-quad` and the legacy TON618 compatibility crate at `main` `70df85e4` (identical to tag `v1.3.0-rc.2` for these crates). It records existing behaviour; it does not change any API or semantics. Tracking: #2103.

## Two representations
- **Raw planes**: per lane a truth bit and a falsity bit (encoding `N=00 F=01 T=10 S=11`, truth = high bit). `QuadroReg32::mask_true` / `mask_false` and `QuadTile128::true_mask` / `false_mask` are raw planes: `mask_true` contains `T` **and** `S`.
- **Exact-state masks**: one-hot per lane. `QuadroReg32::masks()` returns `QuadMasks32 { n, f, t, s }`; there `t` is exactly `T`.

The same expression means different things on the two representations. For example, `t | f` is "any evidence" (`F`, `T`, `S`) on raw planes, but "exactly `F` or `T`" on exact-state masks.

## Meaning of the legacy names (frozen)
| Item | Meaning | N | F | T | S |
|---|---|---|---|---|---|
| `QuadState::is_known`, `QuadroReg32::mask_known`, `QuadMasks32::known`, `QuadTile128::known_mask` | exact `F` or `T` (determinate) | 0 | 1 | 1 | 0 |
| `is_null` / `mask_null` / `null_mask` | exact `N` | 1 | 0 | 0 | 0 |
| `is_conflict` / `mask_conflict` / `conflict_mask` | exact `S` | 0 | 0 | 0 | 1 |

The core executor's `QIsKnown` instruction uses `QuadState::is_known` and therefore returns `false` for `S`.

The Quad Logic Frame specification uses "Known / Non-null" for "any state other than `N`". That is a different notion ("has evidence"), and it includes `S`. The two must not be conflated.

## Legacy delta events
`StateDelta32` / `StateDelta128` (unchanged):
- `entered_true` / `left_true` / `entered_false` / `left_false`: **raw-plane** membership. Equal to `PlaneDelta32`.
- `entered_super` / `left_super` / `became_conflicted` / `resolved_conflict`: exact `S`.
- `became_known` / `became_unknown`: exact `F` or `T`. So `N -> S` and `S -> N` produce neither.
- `changed`: any plane change.

Exact `N`/`F`/`T`/`S` entry and exit for 32 lanes: `ExactStateDelta32`.

## Cross-crate divergence
The legacy TON618 compatibility crate (the compatibility-only owner in the v1 policy) has its own `QuadroReg::calc_delta` (its `quadro.rs`, L178-189 at `70df85e4`). Its `StateDelta.entered_true` / `left_true` / `entered_false` / `left_false` are **exact-state** (`strict_true` / `strict_false`).

So the same field name means:
- exactly `T` in the TON618 compatibility crate;
- truth plane (`T` or `S`) in `semantic-core-quad`.

For `F -> S`:
- `semantic-core-quad` reports `entered_true`;
- the TON618 compatibility crate reports `left_false` and `entered_super`, but not `entered_true`.

Neither crate is changed by this note. Code moving between the two must translate explicitly.

## History (explanatory only)
- The EXOcode root commit (`5f4a64fa`, 2026-02-12) computed `entered_true` from exact-state masks and had no "known" predicate.
- "Known" as `F | T` first appears in the squashed semantic-core import `9bbc7ff3` (2026-05-01), together with raw-plane deltas.
- The Quad Logic Frame freeze (`d618aac3`, 2026-07-08) defined "known" as "not `N`".

That some operators and formulas are equal at the two endpoints does **not** prove the semantics were continuous in between. The history explains the current divergence; the current code and normative specifications remain authoritative.

## Compatibility guidance for future additions
- Do not add fields to the existing public delta structs. They have public fields and no `#[non_exhaustive]`, so downstream exhaustive patterns and struct literals would break. Add new types instead.
- Adding inherent methods can silently take precedence over a same-named downstream trait method. Audit names before adding, and record any addition in the changelog.
