# Quad Language Design Roadmap

Status: active documentation roadmap
Track: Semantic language / quad surface design
Scope type: documentation only

## Purpose

This roadmap groups the quad-surface language work into one visible design
path.

It exists to keep quad lexical meaning, predicate vocabulary, compact selection
syntax, and canonical lowering aligned with the current verified source
contract.

## Design Goals

- keep `quad` first-class and visible
- keep `bool` as the branch-control type
- keep selection explicit rather than truthy
- keep lowering deterministic
- keep the compact syntax readable in dense code

## Included Topics

- quad lexical model
- quad operation families
- quad predicate vocabulary
- `if` / `else if` quad selection
- `match` as compact quad selection
- expression-bodied functions as source sugar
- conservative local inference in quad-heavy code
- migration guidance for quad surface syntax
- dense quad examples

## Related Contract Docs

- `docs/spec/types.md`
- `docs/spec/source_semantics.md`
- `docs/spec/syntax.md`
- `docs/spec/branch_condition_quad_rule.md`
- `docs/language/semantic_style.md`
- `docs/roadmap/language_maturity/quad_surface_lowering_density_audit.md`

## Roadmap Reading

The canonical reading order for this design slice is:

1. lexical model
2. operation families
3. predicate vocabulary
4. surface syntax
5. lowering
6. migration guidance
7. examples

## Non-Goals

This roadmap does not:

- admit hidden truthiness
- widen runtime behavior
- define a new verifier class
- redefine `quad` as a boolean alias
- change PROMETHEUS boundary behavior
- claim unsupported `when` or `is` surface syntax as current admitted grammar

## Linked Experience Docs

The quad roadmap should be read with the language-experience docs:

- `docs/language/semantic_language_experience.md`
- `docs/language/semantic_style.md`
- `docs/language/semantic_language_principles.md`
- `docs/roadmap/language_maturity/quad_surface_lowering_density_audit.md`

## Closeout Plan

This roadmap closes when:

- the surface contract is documented in the spec bundle
- dense examples are recorded
- migration advice exists
- the design record closeout is written
