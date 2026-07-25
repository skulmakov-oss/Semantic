# Semantic Linguist Entry Draft

Status: preparation draft for `github-linguist/linguist`

## Purpose

This note prepares the future Linguist language metadata for Semantic without
opening the external PR yet.

It is intentionally conservative. It records the current public identity,
sample plan, and extension decision so the eventual Linguist submission can be
reviewed against stable repository evidence.

## Frozen Identity Inputs

The current frozen public identity is:

- language name: `Semantic Language`
- short form: `Semantic`
- primary public source extension: `.sm`
- generated artifact extension: `.smc`

These values are already aligned across the public naming and status docs.

## Draft Linguist Entry

The draft metadata shape is:

```yml
Semantic:
  type: programming
  aliases:
    - Semantic Language
  extensions:
    - .sm
```

This draft deliberately leaves color and highlighting details open until local
validation confirms what should be claimed externally.

## Sample Directory Plan

The future Linguist sample directory should use a small Semantic-specific set of
representative `.sm` files.

Recommended plan:

- `samples/Semantic/cli_batch_core.sm`
- `samples/Semantic/rule_state_decision.sm`
- `samples/Semantic/data_audit_record_iterable.sm`
- `samples/Semantic/wave2_local_helper_import.sm`
- `samples/Semantic/positive_selected_import.sm`

These should mirror the existing canonical examples pack rather than invent new
syntax or new behavior. The plan intentionally draws only from the Rust-like
executable surface; `examples/canonical/quad_cycle_logos/` (Logos profile) is
a separate, parse/lowering-only qualified example and is not part of this
`.sm` Linguist sample set (see `docs/spec/source_style.md`).

## Extension Review

`.sm` is safe enough to claim for Semantic because:

- it is already the canonical public source extension in the repository docs;
- the current syntax signature is visibly Semantic-specific;
- the canonical examples pack already uses `.sm` as its representative surface;
- the current docs distinguish `.sm` source from `.smc` artifacts.

The extension claim should stay limited to the current public contour and should
not be widened to unpromised syntax.

## Grammar / Highlighting Plan

No grammar implementation is part of this issue.

If the eventual Linguist submission needs a highlighting plan, it should start
with the current executable surface and only later decide whether a custom
grammar or an existing close highlight model is appropriate.

## Readiness Gate

Do not open the external Linguist PR until:

- the sample directory plan is validated;
- local detection behavior is checked;
- extension conflicts are reviewed;
- the public identity freeze remains unchanged.

## Related Documents

- `docs/language/semantic_syntax_signature.md`
- `docs/NAMING.md`
- `examples/canonical/README.md`
- `tests/canonical_examples.rs`
