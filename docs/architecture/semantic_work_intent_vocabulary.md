# Canonical Work Intent Vocabulary

This document outlines the small, canonical vocabulary for the Semantic Work Layer. Every command exposed to the user will follow the `work <subject> <intent> ...` pattern using one of the primary intents listed below.

## Primary Intents

Each intent maps to exactly one deterministic internal operation. No intent implies release readiness or production stability.

- **check**: Perform syntactic and baseline semantic validation of the subject. It does not execute the code or evaluate deeper proofs.
- **prove**: Compile source subjects to SemCode when needed, then run the verifier to guarantee semantic soundness, type safety, and memory properties without generating execution side effects.
- **wake**: Bring the subject into an active, resident state (e.g., launching a persistent service or actor) awaiting interaction.
- **replicate**: Produce an identical, verifiable copy of the subject's artifact or environment state according to the canonical specification.
- **seal**: Finalize the current state or artifact of the subject, creating an immutable, cryptographically verifiable boundary and writing the requested output artifact to the `to` target.
- **reveal**: Expose the underlying internal data structures, raw lowered AST, or unverified output of the subject for inspection.
- **trace**: Execute the subject while recording deterministic step-by-step state transitions and events for diagnostic playback.
- **explain**: Provide a human-readable synthesis or breakdown of the subject's structure, proof boundaries, or execution flow.
- **observe**: Attach to an already `wake`d subject to passively consume metrics, logs, or state snapshots without mutating its state.

## Synonyms and Aliases

To maintain a strict and predictable vocabulary, synonyms are explicitly deprecated in the canonical interface. However, for familiar developer workflows, certain legacy commands act purely as compatibility aliases to canonical intents:

- **build / compile / package**: Aliased to `seal` or `prove` depending on context, but officially deprecated as intent names.
- **run**: Aliased to `wake` or `trace` depending on the subject.
- **test**: Aliased to a specific `prove` or `trace` profile.

These aliases exist solely for compatibility and must ultimately route through the strict canonical intent definitions without introducing separate execution paths.

## Modifier Use

In the current CLI, `to <target>` and `with <profile>` are consumed by `seal`. Other intents keep the control-frame fields available for future expansion, but they do not forward them as unsupported lower-layer flags.
