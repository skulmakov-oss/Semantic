# Output Design for Reveal, Trace, Explain Intents

## Goal
Design the user-facing output formats for the `explain`, `reveal`, and `trace` intents in the Semantic Work Layer. The output must be readable, conceal irrelevant internal details by default, and avoid widening trust claims.

## Principles
1. **Readable by Default**: Output is formatted for human consumption unless a raw format (like JSON or binary) is explicitly requested via a profile.
2. **Selective Revelation**: Internal details (such as AST indices or raw memory bounds) are omitted unless the intent directly targets those layers.
3. **No Trust Widening**: These tools are analytical and diagnostic. Their output must not be framed as an execution guarantee or proof of correctness.

## Designs

### `work <subject> explain`
- **Purpose**: High-level structural synthesis.
- **Output**:
  - Summarizes the entry points, exported modules, and public dependencies.
  - Lists the active proof boundaries or required capabilities.
  - Formatted as a concise console report with sections.

### `work <subject> reveal`
- **Purpose**: Inspection of underlying artifacts or intermediate representations.
- **Output**:
  - Dumps the lowered representation or specific pipeline state.
  - Excludes noisy diagnostic annotations unless `with verbose` is used.
  - If outputting SemCode or IR, uses a standardized mnemonic text format (e.g., `<addr>: <op> <args>`).

### `work <subject> trace`
- **Purpose**: Step-by-step diagnostic playback of state transitions.
- **Output**:
  - Displays a bounded ledger of events: `[step_id] [node] [delta]`.
  - Abides by deterministic truncation limits (e.g., max 64 nodes) to prevent log flooding.
  - Highlights the first mismatch or error in red, keeping the rest compact.
