# Semantic Work Layer Doctrine

## Goal
Establish the doctrine for the Semantic Work Layer. The Work Layer is designed to expose a controlled semantic command layer where user-facing commands express **intent** rather than internal pipeline stages.

## Canonical Shape
The canonical shape for all commands exposed via the Semantic Work Layer is:
`work <subject> <intent> ...`

This structure ensures that the vocabulary remains user-centric and intent-driven. 

## Distinguishing Intent from Internal Stages
The core principle of the Work Layer is abstraction of pipeline complexities:
- **User-Facing Intent:** Commands should describe what the user wants to accomplish (e.g., build, run, test) rather than the mechanism.
- **Hidden Internal Stages:** The underlying execution stages (compilation, parsing, lowering, verification, etc.) are encapsulated and remain hidden by default.
- **Advanced Inspection:** Advanced or core commands (such as raw AST dumps or verifier bypass commands) remain available for developers and CI inspection, but they are clearly separated from the standard Work Layer path.

## Non-goals
- The Work Layer is **not** a replacement for the core pipeline architecture.
- It is **not** intended to expose raw VM instructions to the standard user.
- It does **not** rewrite or reorganize the core execution stages, but merely orchestrates them under intent-driven facades.

## Trust Boundary Constraints
- **Core Trust Freeze:** The introduction of the Work Layer **must not widen the active Core Trust Freeze contour**. It operates purely as a structural entrypoint layer.
- The layer merely routes intent to already-verified pipeline stages without introducing new privileges or bypassing existing verifier checks.
