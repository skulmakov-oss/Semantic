# Positive Quad Handling Pattern

Status: language-experience design note

## Purpose

This document records the positive pattern for working with quad state in
Semantic.

## Pattern

Positive quad handling means the language treats quad state as something users
can work with directly, not something to avoid.

Good patterns:

- keep `S` visible as conflict;
- use `known(x)` when the user wants intent, not raw bit fiddling;
- prefer `match` when several quad states are being selected;
- use `when` only when the branch selection reads more naturally as rule
  selection;
- keep `bool` conditions explicit.

## Non-Goals

This document does not turn quad state into implicit truthiness.

