# Semantic Code Style Principles

Status: language-experience design note

## Purpose

This document records the code-style stance Semantic should communicate.

## Principles

Semantic code should prefer:

- explicit public signatures;
- local density where it improves readability;
- `bool` for control flow;
- `quad` for semantic state;
- canonical lowering over decorative syntax;
- named intermediate values only when they improve evidence or reuse;
- concise examples that still show the verifier boundary.

## Relationship To Style Docs

This document complements:

- [`docs/language/semantic_style.md`](semantic_style.md)
- [`docs/language/semantic_language_principles.md`](semantic_language_principles.md)
- [`docs/spec/source_style.md`](../spec/source_style.md) — the frozen
  canonical presentation contract for currently executable Semantic source

It does not replace them.

## Non-Goals

This document does not define a formatter or rewrite fixtures.

