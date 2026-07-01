# Semantic Action Vocabulary

Status: language-experience design note

## Purpose

This document records the user-facing action vocabulary Semantic should favor.

## Vocabulary

Prefer verbs that explain intent:

- `check` for source validation;
- `compile` for lowering to SemCode;
- `verify` for admission;
- `run` for execution;
- `match` for explicit selection;
- `when` for compact rule selection;
- `is` for readable quad predicates;
- `known`, `unknown`, and `conflict` for explicit quad-state vocabulary.

## Boundary Rule

Action words should not imply hidden authority.

In particular:

- `check` should not mean admission;
- `compile` should not mean execution;
- `run` should not mean verifier approval;
- quad vocabulary should not be treated as branch truthiness.

## Non-Goals

This document does not widen any implementation boundary.

