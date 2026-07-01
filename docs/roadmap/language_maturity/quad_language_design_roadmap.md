# Quad Language Design Roadmap

Status: active design roadmap

Related docs:

- [`docs/language/quad_language_design.md`](../../language/quad_language_design.md)
- [`docs/language/semantic_language_experience.md`](../../language/semantic_language_experience.md)
- [`docs/roadmap/language_maturity/semantic_language_experience_roadmap.md`](semantic_language_experience_roadmap.md)
- [`docs/examples/quad_language_examples.md`](../../examples/quad_language_examples.md)
- [`docs/roadmap/language_maturity/quad_surface_lowering_density_audit.md`](quad_surface_lowering_density_audit.md)

## Purpose

This roadmap tracks the documentation sequence for quad surface syntax and the
experience framing around it.

It is a design roadmap, not an implementation promise.

## Sequence

1. [`quad lexical model`](../../language/quad_lexical_model.md)
2. [`quad operation families`](../../language/quad_operation_families.md)
3. [`quad predicate vocabulary`](../../language/quad_predicate_vocabulary.md)
4. [`is` syntax for quad predicates](../../language/quad_is_syntax.md)
5. [`else if` surface syntax](../../language/quad_else_if_surface_syntax.md)
6. [`match` for compact quad selection](../../language/quad_match_surface_syntax.md)
7. [`when` expression for quad rule selection](../../language/quad_when_expression.md)
8. [`expression-bodied functions`](../../language/quad_expression_bodied_functions.md)
9. [`conservative local type inference`](../../language/quad_local_type_inference.md)
10. dense examples
11. canonical lowering
12. [`lowering-density audit`](quad_surface_lowering_density_audit.md)
13. [`migration guidance`](../../language/quad_surface_syntax_migration.md)
14. closeout record

## Guardrails

- verifier-first semantics remain unchanged;
- canonical semantics remain unchanged;
- `bool` remains the branch-control type;
- `quad` remains the semantic-state type;
- the roadmap does not approve hidden coercions;
- the roadmap does not approve SemCode, verifier, or VM changes by itself.
