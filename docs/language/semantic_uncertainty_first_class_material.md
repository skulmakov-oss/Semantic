# Uncertainty As First-Class Material

Status: language-experience design note

## Purpose

This document states that uncertainty should be treated as first-class material
in Semantic.

## Meaning

Uncertainty should remain explicit rather than being erased into false
certainty.

Recommended stance:

- represent unknown state explicitly;
- keep `N` visible when it matters;
- use `known(x)` and `unknown(x)` as readable predicates where appropriate;
- do not hide uncertainty behind implicit coercions.

## Non-Goals

This document does not add new uncertainty semantics.

