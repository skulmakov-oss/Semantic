# Semantic Language Experience

Status: language-experience design record

## Purpose

This document records the user-facing experience posture for Semantic as a
language.

It is not grammar, not type theory, and not verifier policy. It is the
human-facing story around how Semantic should feel to use while remaining
honest about its boundaries.

## Emotional Contract

Semantic should feel:

- explicit rather than mysterious;
- deterministic rather than fragile;
- capable of density without becoming opaque;
- strict about admission without sounding punitive;
- expressive about uncertainty and conflict instead of flattening them;
- trustworthy because canonical lowering is visible.

Semantic should not feel like a hidden translation layer that silently rewrites
meaning.

## Onboarding Narrative

The first user-facing story should be:

1. write clear source;
2. let the frontend and typechecker validate it;
3. lower to canonical core semantics;
4. admit the artifact through the verifier;
5. execute only after admission;
6. keep effects behind explicit boundaries.

That narrative should be visible in docs, examples, and roadmap material.

## Action Vocabulary

User-facing language should favor verbs that explain intent:

- `check` for source validation;
- `compile` for lowering to SemCode;
- `verify` for admission;
- `run` for execution;
- `match` for explicit selection;
- `when` for compact rule selection;
- `is` for readable quad predicates;
- `known`, `unknown`, and `conflict` for explicit quad-state vocabulary.

Action words should not imply hidden authority.

## Code Style Principles

Semantic code should prefer:

- explicit public signatures;
- local density where it improves readability;
- `bool` for control flow;
- `quad` for semantic state;
- canonical lowering over decorative syntax;
- named intermediate values only when they improve evidence or reuse;
- concise examples that still show the verifier boundary.

These principles complement, not replace, [`semantic_style.md`](semantic_style.md)
and [`semantic_language_principles.md`](semantic_language_principles.md).

## Freedom With A Spine

Semantic should give users room to express dense quad-heavy logic, but the
language needs a spine:

- canonical lowering stays stable;
- verifier authority stays visible;
- `quad` does not collapse into `bool`;
- uncertainty stays visible;
- conflict stays visible;
- examples stay honest about implementation status.

## Positive Quad Handling

Positive quad handling means the language treats quad state as something users
can work with directly, not something to be avoided.

Good patterns:

- keep `S` visible as conflict;
- use `known(x)` when the user wants intent, not raw bit fiddling;
- prefer `match` when several quad states are being selected;
- use `when` only when the branch selection reads more naturally as rule
  selection;
- keep `bool` conditions explicit.

## Conflict Quarantine UX Pattern

Conflict should be visible and quarantined, not erased.

The recommended language-level message is:

- this value is in conflict;
- the conflict is not hidden;
- branch control still requires an explicit `bool` predicate;
- the user must choose how to resolve or inspect the conflict.

This is a UX principle, not a new runtime state machine.

## Non-Goals

This document does not claim:

- new syntax is implemented just because it is documented;
- any verifier widening;
- any VM widening;
- any SemCode widening;
- any hidden truthiness model;
- any implicit `quad -> bool`;
- any release-status widening;
- any UI authority.

## Related Docs

- [`docs/roadmap/language_maturity/semantic_language_experience_roadmap.md`](../roadmap/language_maturity/semantic_language_experience_roadmap.md)
- [`docs/language/semantic_emotional_contract.md`](semantic_emotional_contract.md)
- [`docs/language/semantic_onboarding_narrative.md`](semantic_onboarding_narrative.md)
- [`docs/language/semantic_action_vocabulary.md`](semantic_action_vocabulary.md)
- [`docs/language/semantic_code_style_principles.md`](semantic_code_style_principles.md)
- [`docs/language/semantic_freedom_with_a_spine.md`](semantic_freedom_with_a_spine.md)
- [`docs/language/semantic_visible_conflict_handling.md`](semantic_visible_conflict_handling.md)
- [`docs/language/semantic_visible_uncertainty_handling.md`](semantic_visible_uncertainty_handling.md)
- [`docs/language/semantic_conflict_visible_semantic_state.md`](semantic_conflict_visible_semantic_state.md)
- [`docs/language/semantic_uncertainty_first_class_material.md`](semantic_uncertainty_first_class_material.md)
- [`docs/language/semantic_positive_quad_handling.md`](semantic_positive_quad_handling.md)
- [`docs/language/semantic_conflict_quarantine_ux_pattern.md`](semantic_conflict_quarantine_ux_pattern.md)
- [`docs/language/semantic_verifier_exoskeleton.md`](semantic_verifier_exoskeleton.md)
- [`docs/language/semantic_language_non_goals.md`](semantic_language_non_goals.md)
- [`docs/language/semantic_documentation_tone.md`](semantic_documentation_tone.md)
- [`docs/examples/semantic_language_experience_examples.md`](../examples/semantic_language_experience_examples.md)
- [`docs/roadmap/language_maturity/semantic_language_experience_closeout.md`](../roadmap/language_maturity/semantic_language_experience_closeout.md)
- [`docs/language/quad_language_design.md`](quad_language_design.md)
- [`docs/language/semantic_style.md`](semantic_style.md)
- [`docs/language/semantic_language_principles.md`](semantic_language_principles.md)
