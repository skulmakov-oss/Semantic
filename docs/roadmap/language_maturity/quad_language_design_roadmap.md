# Quad Language Design Roadmap

Status: active design roadmap

Related docs:

- [`docs/language/quad_language_design.md`](../../language/quad_language_design.md)
- [`docs/language/semantic_language_experience.md`](../../language/semantic_language_experience.md)
- [`docs/examples/quad_language_examples.md`](../../examples/quad_language_examples.md)

## Purpose

This roadmap tracks the documentation sequence for quad surface syntax and the
experience framing around it.

It is a design roadmap, not an implementation promise.

## Sequence

1. quad lexical model
2. quad operation families
3. quad predicate vocabulary
4. `is` syntax for quad predicates
5. `else if` surface syntax
6. `match` for compact quad selection
7. `when` expression for quad rule selection
8. expression-bodied functions
9. conservative local type inference
10. dense examples
11. canonical lowering
12. lowering-density audit
13. migration guidance
14. closeout record

## Guardrails

- verifier-first semantics remain unchanged;
- canonical semantics remain unchanged;
- `bool` remains the branch-control type;
- `quad` remains the semantic-state type;
- the roadmap does not approve hidden coercions;
- the roadmap does not approve SemCode, verifier, or VM changes by itself.

