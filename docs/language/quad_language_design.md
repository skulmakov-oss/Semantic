# Quad Language Design

Status: design record for quad surface syntax

## Purpose

Quad Language Design, or QLD, is the surface-syntax design track for making
quad-heavy Semantic code denser and easier to read without weakening the
verifier-first model.

QLD is a surface layer. It does not redefine canonical semantics.

## Core Rule

```text
bool decides.
quad means.
```

## Design Scope

QLD covers:

- quad lexical meaning;
- quad operation families;
- quad predicate vocabulary;
- `is` surface predicates;
- `else if` surface syntax;
- `when` expression syntax;
- compact `match` syntax;
- expression-bodied functions;
- conservative local type inference for local `let`;
- dense examples;
- canonical lowering documentation;
- migration guidance;
- lowering-density audit notes.

## Canonical Lowering

The verifier sees canonical core semantics, not aesthetic syntax.

Future surface forms lower to the following core forms:

| Surface | Canonical lowering |
| --- | --- |
| `x is S` | `x == S` |
| `known(x)` | `x != N` |
| `unknown(x)` | `x == N` |
| `conflict(x)` | `x == S` |
| `else if` | nested `else { if ... }` |
| `when` | nested expression `if` |
| `match` | deterministic exhaustive decision tree |
| expression-bodied fn | block with final `return` |

## Roadmap

QLD should be read as a staged design record:

1. define the quad lexical model and operation families.
2. define a compact predicate vocabulary that still returns `bool`.
3. define the `is` surface form as a readable predicate alias.
4. define `else if`, `when`, `match`, and expression-bodied functions as
   surface sugar over canonical lowering.
5. define conservative local inference only for local `let`.
6. document dense examples against current core forms.
7. record lowering-density risks before approving implementation work.
8. close the design record once the docs are internally consistent.

## Non-Goals

QLD does not approve:

- SemCode format changes;
- verifier admission changes;
- VM runtime behavior changes;
- implicit `quad -> bool`;
- hidden truthiness;
- hidden coercions;
- parser/typechecker changes by documentation alone;
- optimizer rewrites by documentation alone.

## Related Docs

- [`docs/spec/source_semantics.md`](../spec/source_semantics.md)
- [`docs/spec/types.md`](../spec/types.md)
- [`docs/spec/branch_condition_quad_rule.md`](../spec/branch_condition_quad_rule.md)
- [`docs/examples/quad_language_examples.md`](../examples/quad_language_examples.md)
- [`docs/roadmap/language_maturity/quad_language_design_roadmap.md`](../roadmap/language_maturity/quad_language_design_roadmap.md)
- [`docs/roadmap/language_maturity/quad_language_design_closeout.md`](../roadmap/language_maturity/quad_language_design_closeout.md)
- [`docs/language/semantic_language_experience.md`](semantic_language_experience.md)

