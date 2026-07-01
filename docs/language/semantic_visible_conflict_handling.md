# Visible Conflict Handling

Status: language-experience design note

## Purpose

This document records the expectation that conflict should remain visible in the
language experience.

## Meaning

Conflict is not something Semantic should hide behind vague errors or silent
fallbacks.

Recommended stance:

- show that the value is in conflict;
- keep the conflict state explicit;
- do not let branch control silently consume it;
- require an explicit `bool` predicate when control flow is needed.

## Non-Goals

This document does not define a new runtime state machine.

