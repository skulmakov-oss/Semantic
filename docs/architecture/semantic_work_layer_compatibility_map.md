# Command Compatibility Map

## Goal
Inventory existing CLI commands and propose their future Work Layer equivalents. This ensures backwards compatibility and smooth migration to the new intent-driven vocabulary.

*Note: This is planning only. Existing commands are not removed during this phase.*

## Inventory and Mapping

| Legacy Command | Future Work Layer Equivalent | Justification |
| -------------- | ---------------------------- | ------------- |
| `smc check`    | `work <subject> check`       | Direct mapping. Semantic validation without side-effects. |
| `smc verify`   | `work <subject> prove`       | "Verify" implies a strong proof generation phase. "Prove" aligns better with the intent vocabulary. |
| `smc compile`  | `work <subject> seal` (or `prove`) | Compilation finalizes an artifact (`.smc`). "Seal" reflects this boundary. |
| `smc run`      | `work <subject> wake`        | "Run" initiates an active execution environment. "Wake" represents bringing a subject into residence. |
| `smc run-smc`  | `work <subject> wake`        | Unified under the same intent regardless of whether the subject is source or compiled artifact. |
| `smc dump-ast` | `work <subject> reveal with ast` | "Dump" is an internal diagnostic action. "Reveal" safely exposes internal representations. |
| `smc dump-ir`  | `work <subject> reveal with ir`  | Same as above. |
| `smc disasm`   | `work <subject> reveal with disasm` | Same as above. |
| `smc watch`    | `work <subject> trace` (continuous) | TBD, potentially maps to a continuous trace or an observe pipeline. |

## Strategy
Legacy commands will remain as hidden aliases in the CLI parser, transparently routing to their `work` command equivalents to preserve existing CI pipelines until a formal deprecation schedule is enacted.
