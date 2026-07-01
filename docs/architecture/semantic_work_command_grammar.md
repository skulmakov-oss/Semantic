# Semantic Work Command Grammar and Control Frames

## Goal
Define the deterministic grammar and the internal control-frame model for the Semantic Work Layer. The goal is to provide a highly predictable CLI and programmatic interface that avoids natural-language ambiguity.

## Grammar Specification

The Work Layer parses commands using a deterministic, token-based grammar. It strictly rejects unstructured natural language to guarantee predictability.

The current dispatcher admits these intent shapes:
1. `work <subject> check`
2. `work <subject> prove`
3. `work <subject> wake`
4. `work <subject> seal to <target> [with <profile>]`

### Token Roles
- **`work`**: The universal dispatch verb.
- **`<subject>`**: The entity being operated upon (e.g., a project, file, module, or specific namespace).
- **`<intent>`**: The canonical operation requested (e.g., `check`, `prove`, `seal`, `wake`, as defined in the vocabulary).
- **`to <target>`**: Optional for `seal`. Defines the destination output artifact path.
- **`with <profile>`**: Optional for `seal`. Defines the compile profile to use when sealing.

## Lowering to Typed Control Frames

Upon successful parsing, the grammar is lowered into a typed internal control frame. This frame acts as the definitive execution intent passed to the dispatcher, insulating the core pipeline from CLI parsing logic.

```rust
pub struct WorkControlFrame {
    pub subject: WorkSubject,
    pub intent: WorkIntent,
    pub target: Option<WorkTarget>,
    pub profile: Option<WorkProfile>,
}
```

Each parsed string token maps to a typed enum or structured path/identifier within this frame.
The current CLI consumes `target` and `profile` only for `seal`; other intents keep the fields available in the control frame but do not forward them as raw lower-layer flags.

## Error Handling and Feedback

To ensure a smooth user experience, the parser implements strict boundary checks. If a command deviates from the grammar:
- The parser immediately halts.
- It does **not** attempt heuristic recovery.
- It yields a friendly, structured error message.

### Example Error Shapes
- **Unknown Intent:**
  `Error: Unknown intent 'compile'. Did you mean 'work <subject> prove' or 'work <subject> seal'?`
- **Invalid Shape:**
  `Error: Unexpected token 'using'. The grammar supports: work <subject> <intent> [to <target>] [with <profile>]`

This strict feedback loop teaches the user the canonical shapes without causing unpredictable internal behavior.
