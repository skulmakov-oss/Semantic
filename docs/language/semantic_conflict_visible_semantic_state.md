# Conflict As Visible Semantic State

Status: language-experience design note

## Purpose

This document states that conflict should be treated as visible semantic state.

## Meaning

Conflict is not a bug-shaped absence. It is part of the semantic story and
should remain inspectable.

Recommended stance:

- keep `S` visible as a first-class state;
- avoid source and docs that flatten conflict into hidden failure;
- require explicit `bool` predicates when control flow needs to branch.

## Non-Goals

This document does not add a new conflict runtime.

