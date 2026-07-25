# Canonical Examples Pack

Status: finalized canonical examples pack for `PR-D1`

## Purpose

This directory publishes the curated examples pack used by the current
readiness contour.

It replaces the earlier planning-only pack in:

- `examples/readiness_draft_canonical/`

This pack is intentionally split into:

- twelve positive examples inside the current `qualified limited release` contour
- one boundary example that shows a still-real limit honestly
- one Logos declarative-profile example, qualified through its own
  parse/lowering path rather than `check`/`compile`/`verify`/`run`

`match_control_flow` and `rule_state_decision` also serve as the canonical
demonstrations of `docs/spec/source_style.md` (Semantic Canonical Source Style
v0): compact guard returns, compact `match` arms, and a data/domain/validation/
orchestration top-level order.

This pack is also the canonical `.sm` sample surface intended for future GitHub
Linguist review. The twelve positive examples are small, readable, and stable
enough to serve as representative language samples:

- `cli_batch_core`
- `rule_state_decision`
- `data_audit_record_iterable`
- `text_collections_toolbox`
- `stdlib_v0_helpers`
- `collections_core`
- `text_core`
- `match_control_flow`
- `option_result_control_flow`
- `loop_control_flow`
- `wave2_local_helper_import`
- `positive_selected_import`

The boundary example remains included as an honest exclusion marker and should
not be treated as a positive sample for Linguist detection.

## Canonical Examples — Authoritative Inventory

This table is the **single authoritative inventory** of the canonical
examples pack. Every other current-facing index (`docs/examples_index.md`,
`docs/spec/source_style.md`, `docs/language/semantic_linguist_entry_draft.md`)
links to this table rather than maintaining its own copy of the same status
fields, so there is exactly one place to update when an example's status
changes.

| # | Example | Profile | Purpose | Qualification level | Expected result | Style v0 status |
|---|---|---|---|---|---|---|
| 1 | `cli_batch_core` | Rust-like | small CLI-style computation core over `Sequence(i32)` and `text` | executable: `check`/`compile`/`verify`/`run` | pass | migrated |
| 2 | `rule_state_decision` | Rust-like | record-oriented rule/state decision logic with explicit `Result(T, E)` handling | executable: `check`/`compile`/`verify`/`run` | pass | migrated |
| 3 | `data_audit_record_iterable` | Rust-like | data-heavy audit pass over direct-record `Iterable` dispatch | executable: `check`/`compile`/`verify`/`run` | pass | migrated |
| 4 | `text_collections_toolbox` | Rust-like | practical toolbox example for control flow, text, collections, and stdlib | executable: `check`/`compile`/`verify`/`run` | pass | migrated |
| 5 | `stdlib_v0_helpers` | Rust-like | practical helper-surface example for current PCC stdlib v0 | executable: `check`/`compile`/`verify`/`run` | pass | already compliant |
| 6 | `collections_core` | Rust-like | standalone practical collections surface example | executable: `check`/`compile`/`verify`/`run` | pass | migrated |
| 7 | `text_core` | Rust-like | standalone practical `text` surface example | executable: `check`/`compile`/`verify`/`run` | pass | migrated |
| 8 | `match_control_flow` | Rust-like | compact quad decision program; demonstrates Source Style v0 | executable: `check`/`compile`/`verify`/`run` | pass | migrated |
| 9 | `option_result_control_flow` | Rust-like | practical `Option` / `Result` control flow over the admitted public surface | executable: `check`/`compile`/`verify`/`run` | pass | migrated |
| 10 | `loop_control_flow` | Rust-like | practical loop-driven control flow over admitted `while`, `loop`, `break`, and `continue` | executable: `check`/`compile`/`verify`/`run` | pass | migrated |
| 11 | `wave2_local_helper_import` | Rust-like | admitted helper-module executable authoring with direct local-path bare import | executable: `check`/`compile`/`verify`/`run` | pass | already compliant |
| 12 | `positive_selected_import` | Rust-like | admitted helper-module executable authoring with direct local-path selected import | executable: `check`/`compile`/`verify`/`run` | pass | migrated |
| 13 | `boundary_alias_import` | Rust-like | intentional boundary: top-level alias import on the executable path is still rejected | intentional rejection: `check` | expected diagnostic failure preserved | already compliant |
| 14 | `quad_cycle_logos` | Logos | canonical Logos declarative-profile example (`System` / `Entity` / `Law`); demonstrates Source Style v0 | parse + lowering only: `dump-ast`, `dump-ir --profile logos`; `check`/`run` honestly rejected | pass (parse/lowering); honest rejection (`check`/`run`) | migrated |

Style v0 status meanings:

- **migrated** — the file was rewritten in this or a prior migration pass to
  apply `docs/spec/source_style.md` (compact guard returns where the
  condition and returned expression are simple, compact match arms, blank
  lines between semantic phases).
- **already compliant** — the file already matched Source Style v0 before any
  migration pass; no rewrite was needed or performed.

`stdlib_v0_helpers` and `wave2_local_helper_import` (and its helper module)
contain no single-condition/single-`return` guards, so B.5 compaction does
not apply to them; their existing multi-line `if` blocks (assignment bodies,
not `return`) are already the canonical B.8 shape. `boundary_alias_import`'s
files contain no guard-return or match-arm forms to migrate; they are
preserved byte-for-byte to protect the exact rejection diagnostic asserted by
`tests/canonical_examples.rs`.

## Validation

Canonical examples are validated by:

```text
cargo test -q --test canonical_examples
cargo test -q --test canonical_source_style
```

Positive examples are checked, compiled, verified, and run through the public
`smc` command surface.

The boundary example is checked to ensure the current diagnostic remains
explicit and deterministic.

The Logos example is validated through `smc dump-ast` and
`smc dump-ir --profile logos`, and is checked to confirm that `smc check`
still fails on it with a Rust-like frontend diagnostic (the documented
honesty boundary between the two source surfaces — see
`docs/spec/source_style.md`).
