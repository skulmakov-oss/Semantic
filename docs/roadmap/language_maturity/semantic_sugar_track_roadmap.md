# Semantic Sugar Track Roadmap

Status: active documentation roadmap
Scope type: documentation only

## Purpose

This roadmap groups the sugar-track proposals that improve readability while
keeping the Semantic core deterministic and verifier-friendly.

It is a documentation roadmap, not an implementation promise.

## Design Goals

- keep canonical lowering obvious
- reduce repetition in record-heavy code
- reduce return noise in value-producing blocks
- make quad predicates read like domain language
- keep diagnostics and parsing simple

## Included Topics

- field punning
- tail expression return
- quad predicate vocabulary aliases

## Deferred Topics

- `when let`
- `unless`
- pipeline operator `|>`

## Related Docs

- [`docs/language/semantic_sugar_track.md`](../../language/semantic_sugar_track.md)
- [`docs/language/semantic_language_experience.md`](../../language/semantic_language_experience.md)
- [`docs/language/semantic_code_style_principles.md`](../../language/semantic_code_style_principles.md)
- [`docs/language/semantic_quad_surface.md`](../../language/semantic_quad_surface.md)

## Sequence

1. field punning
2. tail expression return
3. quad predicate vocabulary aliases
4. examples
5. closeout record

## Guardrails

- do not claim grammar support before parser work lands
- do not add hidden semantics
- do not widen runtime behavior
- do not introduce a second canonical spelling for the same core meaning
- keep examples explicitly labeled as proposed until implementation exists
