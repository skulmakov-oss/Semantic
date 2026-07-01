# Code Surface Planning

## Goal
Plan how the Work Layer doctrine influences the way internal Semantic code is written and structured. This document serves as a guideline for future refactoring and does not enact immediate behavior changes.

## Architectural Influence

### 1. Hard Boundaries Between Parsing and Execution
Internal pipeline stages must not rely on CLI-specific context. The `WorkControlFrame` must serve as the singular entrypoint for all user intent. Internal functions must accept structured data rather than raw `Vec<String>` arguments derived from the CLI.

### 2. Intent-Driven API Design
The internal SDK and library exports should begin reflecting the canonical vocabulary:
- Use terms like `prove_module()` instead of `compile_and_verify_module()`.
- Group utility functions by intent rather than by architectural subsystem where user-facing orchestration occurs.

### 3. Avoiding Vocabulary Drift in Source Code
Code variables, log lines, and structural boundaries should avoid deprecated synonyms (e.g., `build`, `run`) unless they explicitly refer to compatibility wrappers or specific subsystem nuances.

### 4. Modular Backend Adapters
Since the Work Layer hides backend complexities, internal code must isolate backend-specific adapters (e.g., WGPU, CPU) behind standard trait interfaces. The user's `intent` should trivially route to these adapters without requiring the user to specify complex pipeline flags.
