# Conflict Quarantine UX Pattern

Status: language-experience design note

## Purpose

This document records the “conflict quarantine” UX pattern.

## Pattern

Conflict should be visible and quarantined, not erased.

Recommended language-level message:

- this value is in conflict;
- the conflict is not hidden;
- branch control still requires an explicit `bool` predicate;
- the user must choose how to resolve or inspect the conflict.

## Non-Goals

This document does not define a new runtime quarantine subsystem.

