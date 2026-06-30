# Semantic Language Principles

Status: docs-only principles record for Semantic language framing

## Purpose

This document records the core principles that should remain visible when
describing Semantic, its verified execution model, and its public surface.

It is a documentation record, not a grammar change, not a verifier change, and
not a runtime change.

## Principles

- `bool` decides.
- `quad` means.
- Semantic keeps native quad semantics, including uncertainty and conflict.
- Deterministic execution protects meaning; it does not replace expressive
  freedom.
- Verifier-first admission protects meaning by checking what is allowed before
  execution.
- Uncertainty and conflict stay visible instead of being silently collapsed.
- Every surface syntax form must lower to canonical deterministic semantics.
- Future syntax should remain tied to canonical lowering, verifier admission,
  and SemCode authority boundaries.
- Local profiling and evidence paths are diagnostic surfaces, not production
  telemetry.
- Public claims about acceleration, P5-A, GPU/Vulkan, or performance should
  stay narrow and evidence-based.

## Assistant Framing

When writing about Semantic, keep these statements clear:

- `bool` is branch control; `quad` is semantic state.
- There is no implicit `quad -> bool` collapse.
- Determinism is the condition that makes expressive programs trustworthy.
- Verifier-first is a protection boundary for meaning, not a cage around it.
- Canonical lowering owns the deterministic meaning of accepted surface forms.
- Uncertainty and conflict must remain visible through the toolchain.
- Do not overstate runtime readiness or promotion status from local evidence.

## Non-Claims

This document does not claim:

- new runtime behavior;
- new verifier behavior;
- new SemCode behavior;
- any public API widening;
- any performance improvement;
- any P5-A reopening;
- any GPU/Vulkan backend;
- any production telemetry change;
- any replacement of the verifier-first route.

## Validation

- `git diff --check`
- `cargo fmt --check`
- `git status --short`
