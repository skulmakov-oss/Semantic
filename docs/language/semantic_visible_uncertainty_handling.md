# Visible Uncertainty Handling

Status: language-experience design note

## Purpose

This document records the expectation that uncertainty should remain visible in
Semantic.

## Meaning

Uncertainty should not be flattened into false certainty.

Recommended stance:

- represent unknown state explicitly;
- keep `N` visible when it matters;
- use predicates such as `known(x)` and `unknown(x)` when the intent is
  inspection rather than raw state manipulation;
- avoid implicit coercions that erase the distinction.

## Non-Goals

This document does not add new uncertainty semantics.

