# Math Surface Planning

## Goal
Establish the naming direction and inventory strategy for Semantic math forms and literal forms under the Work Layer doctrine. This planning document will guide future parser changes.

## Inventory of Existing Forms

Currently, the semantic pipeline deals with several distinct math and literal representations:
- **Integer Literals**: Unsigned and signed boundary types (e.g., `u8`, `u32`, `i64`).
- **Quad-State Literals**: T/F/S/N logic literals.
- **Floating-Point Literals**: (Pending standardization)
- **Math Operators**: Standard arithmetic (`+`, `-`, `*`, `/`) and bitwise/quad-state logic operations.

## Proposed Naming Direction

To align with the Semantic Work Layer's emphasis on intent and determinism:

### 1. Explicit Literal Suffixes
Math forms should avoid implicit widening. 
- Use strict suffixes for ambiguity resolution: e.g., `42_u32` instead of relying entirely on inference if the boundary is critical.
- Quad-state literals should use clear canonical prefixes or keywords (e.g., `0q1100` or explicit `True/False/Conflict/Unknown`).

### 2. Deterministic Operator Naming
When exposed in AST dumps or `reveal` traces, operators should map to unambiguous textual names to aid diagnostics:
- `+` -> `add_strict` or `add_wrap`
- `&` -> `quad_meet` or `bit_and`

### 3. Separation of Concerns
- The syntax surface remains intuitive (using standard symbols).
- The intermediate representation (IR) strictly separates math forms based on the backing deterministic hardware rules (e.g., CPU math vs. Quad-State logic).
