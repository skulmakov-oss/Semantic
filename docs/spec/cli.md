# CLI Specification

Status: draft v0
Canonical owner crate: `smc-cli`
Current process entrypoints: root `smc` and `svm` binaries

## Purpose

This document defines the current public CLI contract for the Semantic toolchain.

Current owner rule:

- `smc-cli` owns the public CLI contract in the current baseline
- root `src/bin/smc.rs` and `src/bin/svm.rs` are process entrypoints, not second long-term owners
- the CLI must orchestrate public crate APIs rather than redefine compiler, verifier, VM, or profile semantics

## Current Command Surface

The admitted `smc` command surface is currently:

- `compile`
- `check`
- `lint`
- `watch`
- `fmt`
- `dump-ast`
- `dump-ir`
- `dump-bytecode`
- `hash-ast`
- `hash-ir`
- `hash-smc`
- `snapshots`
- `features`
- `explain`
- `repl`
- `verify`
- `test`
- `run`
- `run-smc`
- `disasm`
- `7hell`
- `look ui frame`
- `hub tools`
- `hub describe`
- `hub invoke`
- `hub session`
- `hub audit`

Current accepted usage forms are:

- `smc compile <input.sm|project-root> -o|--out <out.smc> [--profile auto|rust] [--opt-level O0|O1|--opt] [--debug-symbols] [--metrics]`
- `smc check <input.sm|project-root> [--no-cache] [--trace-cache] [--metrics] [--deny warnings|<CODE>] [--color auto|always|never]`
- `smc lint <input.sm> [--no-cache] [--trace-cache] [--deny warnings|<CODE>] [--color auto|always|never]`
- `smc watch <input.sm> [--metrics] [--color auto|always|never]`
- `smc check <input.sm|project-root> --format human|json` (SSF-09 canonical path; see `diagnostics_machine_schema_v1.md`)
- `smc fmt [--check] <path>`
- `smc lsp [--stdio]` (diagnostics-first language server; see `editor_lsp_baseline.md`)
- `smc dump-ast <input.sm|project-root>`
- `smc dump-ir <input.sm|project-root> [--profile auto|rust|logos] [--opt-level O0|O1|--opt]`
- `smc dump-bytecode <input.sm|project-root> [--profile auto|rust] [--opt-level O0|O1|--opt] [--debug-symbols]`
- `smc hash-ast <input.sm|project-root>`
- `smc hash-ir <input.sm|project-root> [--profile auto|rust|logos] [--opt-level O0|O1|--opt]`
- `smc hash-smc <input.sm|project-root> [--profile auto|rust] [--opt-level O0|O1|--opt] [--debug-symbols] [--trace-cache]`
- `smc snapshots [--update]`
- `smc features`
- `smc explain <error-code|--list>`
- `smc repl`
- `smc verify <input.smc|project-root>`
- `smc test <project-root>`
- `smc run <input.sm|project-root>`
- `smc run <input.sm|project-root> --profile <pure|cli-read-only|cli-file-transform> --root <directory> [--duration-ms <u32>] [--envelope <verified-local|trusted-compiler>] [--max-steps <N>] [--max-frames <N>] [--metrics json] [-- <application-args...>]`
- `smc run-smc <input.smc>`
- `smc disasm <input.smc>`
- `smc 7hell <input.sm> [--json]`
- `smc look ui frame --from <snapshot> [--frame <n>] [--format text|draw-json] [--out <path>]`
- `smc look ui frame <source-file> [--events <script>] [--frame <n>] [--format text|draw-json] [--out <path>]`
- `smc hub tools`
- `smc hub describe <tool-id>`
- `smc hub invoke <tool-id> <operation-id> --input <file> [--out <file>]`
- `smc hub session --requests <file> [--out <file>] [--max-requests <n>]`
- `smc hub audit --request <request-id>`

This draft does not claim that every command is permanently frozen, but it defines the current public CLI surface that tooling may rely on.

## Rust-like / Logos Profile Boundary

Rust-like Semantic is the only source profile admitted to SemCode-producing and
execution commands. Logos is the separate experimental declarative profile
defined by `semantic.logos.declarative/0.1`.

- `dump-ast` accepts either source surface for inspection.
- `dump-ir --profile logos` and `hash-ir --profile logos` expose the
  non-executable `LogosIrLaw` projection.
- `compile`, `dump-bytecode`, `hash-smc`, `verify`, `run`, and `run-smc` do not
  form a Logos execution path.
- Explicit or auto-detected Logos input reaching a SemCode-producing path is
  rejected before artifact emission with the existing Logos/SemCode boundary
  diagnostic.
- A file that mixes Rust-like items with Logos declarations is unsupported and
  is rejected; tools do not split it into two hidden compilation units.

The shared `ParserProfile::foundation_default()` admission envelope does not
give the two source profiles equal maturity or execution authority.

## Not In The Current Surface

The following commands or output modes are not part of the current admitted CLI surface:

- `smc doctor`
- `smc profile show`
- `smc profile train`
- `smc profile validate`
- CLI JSON output modes tied to those commands

Any reintroduction of those surfaces should be treated as a new public CLI change rather than assumed baseline behavior.

## Contract-Sensitive Commands

The following commands expose persisted artifact, admission, or build-surface behavior and should be reviewed as contract-sensitive:

- `smc compile`
- `smc verify`
- `smc test`
- `smc run-smc`
- `smc features`
- `smc hub invoke`
- `smc hub session`

The following inspection commands are public workflow surface, but their plain-text rendering is not yet a frozen machine-readable format:

- `smc dump-ast`
- `smc dump-ir`
- `smc dump-bytecode`
- `smc hash-ast`
- `smc hash-ir`
- `smc hash-smc`
- `smc disasm`

## Diagnostic / Readiness Path

`smc 7hell` is the current single-file diagnostic/readiness route. It is a
compatibility and qualification path, not the normal beginner onboarding flow.

Current command surface:

- `smc 7hell <input.sm> [--json]`

Current output rule:

- `smc 7hell` emits either human-readable text or JSON via `--json`

## Hub Tool Invocation Path

`smc hub` is the current CLI surface for the Semantic Hub v0 governed tool
execution boundary (see `docs/architecture/semantic_hub_v0.md` for the full
architecture). It exposes external computational tools -- currently one
reference tool, `vector.turbovec`, supporting eight operations
(`vector.index.create`, `vector.index.describe`, `vector.index.insert`,
`vector.index.remove`, `vector.search`, `vector.search.filtered`,
`vector.index.reset`, `vector.index.recover`) -- through a single typed
request/reply path rather than ad hoc per-feature integration. There is no
dedicated `smc hub recover` subcommand: recovery is just another bounded
operation, reachable through `invoke` or `session` like any other.

Current command surface:

- `smc hub tools` lists all registered Hub tools, one per line, as
  tab-separated `<tool_id>\t<tool_version>\t<execution_mode>\t<worker_state>`,
  in deterministic ascending order by `tool_id`; any extra argument is
  `InvalidArguments: unexpected argument '<arg>'`
- `smc hub describe <tool-id>` prints the full descriptor for one tool:
  `tool_id`, `name`, `version`, `hub_api_version`, `execution_mode`,
  `trust_class`, `adapter_provenance`, then each operation with
  `determinism=`, `mutates_tool_state=`, and `required_capabilities=[...]`;
  an unknown `tool-id` produces `UnknownTool: <tool-id>`
- `smc hub invoke <tool-id> <operation-id> --input <file> [--out <file>]`
  reads a JSON request file, admits it through the Hub, and writes a JSON
  reply to stdout or, atomically, to `--out <file>` (same write-temp-file,
  fsync, rename pattern as `smc look ui frame --out`)
- `smc hub session --requests <file> [--out <file>] [--max-requests <n>]`
  reads a newline-delimited JSON (NDJSON) file, admits each line in order
  through one bounded `HubSession` against one persistent worker instance,
  and writes one NDJSON reply per line plus a final `{"session_summary":
  {...}}` line -- see `docs/spec/hub/hub_session_v0.md` for the full
  contract (request-line shape, cancel-line control records, session
  ceiling, exit-code policy)
- `smc hub audit --request <request-id>` looks up one audit record by
  `request_id` and prints it as `key: value` lines; an unknown
  `request-id` produces `UnknownRequest: no audit record for request_id
  '<id>'`

Current `smc hub invoke` request file rule:

The `--input` file is a JSON object, bounded to 8 MiB and checked via file
metadata before it is read (an oversized or unreadable file produces
`InputRejected`), with these fields:

- `schema_version` -- optional, defaults to `1`; any other value produces
  `SchemaVersionUnsupported`
- `request_id` -- optional; auto-generated as `req-<pid>-<nanos>` when absent
- `session_id` -- optional, defaults to `"cli-session"`
- `caller_identity` -- optional, defaults to `"cli:local"`
- `capabilities` -- array of capability name strings; there is no auto-grant
  default, so every capability an operation needs must be listed explicitly,
  e.g. `["VectorSearch", "PrivateStorageRead"]`
- `privacy_class` -- optional, defaults to `"ProjectLocal"`; one of
  `PublicSafe`, `ProjectLocal`, `PrivateSource`, `OrganizationPrivate`,
  `SecretSuspected`
- `resource_budget` -- optional object overriding any subset of the built-in
  V0 ceiling (`wall_time_millis`, `memory_bytes`, `input_bytes`,
  `output_bytes`, `index_item_count`, `vector_dimensions`, `result_count`,
  `queue_depth`, `concurrent_requests`, `storage_read_bytes`,
  `storage_write_bytes`, `audit_bytes`)
- `payload` -- required, tool-and-operation-specific JSON object

Example request file for `vector.search`:

```json
{
  "capabilities": ["VectorSearch", "PrivateStorageRead"],
  "payload": {"index": "docs", "queries": [[0,1,0,0,0,0,0,0]], "k": 3}
}
```

The reply JSON has the shape `{schema_version, request_id,
logical_sequence, tool_id, tool_version, operation_id, status, fault_code,
fault_message, payload, resource_usage: {wall_time_millis, input_bytes,
output_bytes}, provenance: {input_digest, output_digest,
worker_state_after, artifact}, warnings}`, where `status` is one of
`Success`, `Rejected`, `ToolFailed`, `Crashed`, `HubFault`. `provenance` is
evidence of how the reply was produced, never a claim that its payload is
true or safe to commit (see `docs/architecture/semantic_hub_v0.md` section
4); `artifact` is `null` for a non-mutating operation, or
`{kind, id, digest}` for a mutating one. `warnings` is always `[]` in v0 --
no warning-producing path exists yet.

Current admission rule:

- every `smc hub invoke` call goes through full Hub admission -- capability
  check, resource budget check, registry lookup -- before it reaches the
  tool; there is no bypass route from the CLI to a tool's native
  implementation

Current persisted-state rule:

- Hub state is written under `.semantic/hub/` relative to the current
  working directory: one `.tvim` file per TurboVec index under
  `.semantic/hub/vector.turbovec/`, plus a single `.semantic/hub/audit.log`
  recording every invocation, which `smc hub audit` reads

Current exit rule:

- `smc hub invoke` exits `0` only when the reply `status` is `Success`; any
  other status is a non-zero exit whose error string leads with the fault
  code, e.g. `CapabilityDenied: missing or denied capabilities: VectorSearch,
  PrivateStorageRead` -- this follows the same single Ok-exit-0/Err-exit-1
  convention as other `smc` commands, with fine-grained status carried in
  the error string's leading token rather than in the OS exit code

## Output Modes

Current output families are:

- human-readable text
- plain-text dumps and hashes for inspection commands
- one admitted canonical machine-readable JSON format: `smc look ui frame --format draw-json` (Frame Snapshot v0; see `docs/spec/ui/ui_frame_inspection_cli.md`)
- the `smc hub invoke` JSON request/reply protocol (see the Hub Tool Invocation Path section above)

Outside those two, there is currently no other admitted machine-readable JSON output contract in `smc-cli`.

Current output rules:

- `smc features` reports enabled and disabled feature sets as text
- `smc dump-*`, `smc hash-*`, and `smc disasm` emit plain text for inspection
- `smc check`, `smc lint`, and `smc watch` support colorized human-readable diagnostics via `--color auto|always|never`
- `smc look ui frame` supports `--format text|draw-json`; both are deterministic and suitable for golden tests (see the dedicated spec)
- `smc hub tools` and `smc hub describe` emit deterministic plain text (tab-separated fields and `key: value` lines respectively); `smc hub invoke` emits the JSON reply described above

## Verified Execution Rule

Current execution-facing commands follow this split:

- `smc run <input.sm|project-root>` resolves source input, then compiles and executes the produced in-memory SemCode path
- `smc verify <input.smc|project-root>` performs verifier admission without execution; project roots compile the canonical entry in memory first
- `smc test <project-root>` discovers and executes `tests/**/*.sm` in normalized relative-path order under the pure application profile
- `smc run-smc <input.smc>` executes compiled SemCode through the verified artifact path

Public rule:

- persisted `.smc` execution must not bypass verification
- `smc run` is a source-execution workflow command, not the persisted artifact admission path

## Application Run Execution Envelope And Metrics

Status: SHF-R (#2030). Applies only to the application-mode form
`smc run <input> --profile ... --root ...`. The one-argument
`smc run <input>` and `smc run-smc <input.smc>` controlled-observation forms
are unchanged and do not accept these options.

### Envelope

- `--envelope <verified-local|trusted-compiler>` selects the execution
  envelope; the default is `verified-local`.
- Both envelopes execute under `ExecutionContext::VerifiedLocal`.
- `verified-local`, and `trusted-compiler` without overrides, use exactly
  `RuntimeQuotas::verified_local()`.
- `--max-steps <N>` (1..=200000000) and `--max-frames <N>` (1..=1024) are
  accepted only together with `--envelope trusted-compiler`.
  `--max-frames` also sets `max_stack_depth`.
- `max_calls` (16384), `max_effect_calls`, `max_registers`,
  `max_symbol_table` and `max_debug_symbols_per_function` are never changed
  by the envelope. There is no `--max-calls` option.
- The capability manifest is selected by `--profile` alone; the envelope
  never grants capabilities.
- One `ExecutionConfig` is constructed after argument validation and is
  used unchanged for verifier admission (`verify_semcode_token_with_quotas`,
  with the default `VerificationLimits` profile), for execution and for
  metrics.
- `trusted-compiler` limits are experimental contract candidates for
  explicitly selected local execution. They provide no aggregate heap bound
  and no wall-clock bound; see `docs/spec/quotas.md`.

### Argument validation

Validation happens before source resolution or compilation. A failure
exits non-zero, emits no metrics line, and reports the first failing rule in
this order:

1. unknown option: `unknown run option '<option>'` followed by usage
2. option without a value: usage
3. repeated `--envelope`, `--max-steps`, `--max-frames` or `--metrics`:
   `duplicate run option '<option>'`
4. unknown envelope:
   `unknown execution envelope '<value>'; expected verified-local or trusted-compiler`
5. unknown metrics format: `unsupported metrics format '<value>'; expected json`
6. a value that is not plain ASCII decimal digits (empty, sign, whitespace,
   prefix, exponent or non-ASCII digits):
   `--max-steps must be a decimal integer` (likewise `--max-frames`)
7. zero, a value above the maximum, or a value that overflows `u64`:
   `--max-steps must be between 1 and 200000000`,
   `--max-frames must be between 1 and 1024`
8. an override without `--envelope trusted-compiler`:
   `--max-steps requires --envelope trusted-compiler` (likewise `--max-frames`)
9. the pre-existing `--profile`, `--root` and `--duration-ms` rules

Rules 1-6 are applied while scanning options in command-line order; rules
7-9 after the scan.

### `--metrics json`

`--metrics json` is independent of `--envelope`. After argument validation
succeeds, exactly one line is written to stderr for every outcome, after the
application audit records and before the error text of a failing command.
stdout carries only application output. The exit status is unchanged by the
option.

The line is a JSON object serialized with `serde_json`:

| Field | Meaning |
|---|---|
| `schema` | `"semantic.run.metrics/0.1"` |
| `envelope` | `"verified-local"` or `"trusted-compiler"` |
| `effective_quotas` | `max_steps`, `max_calls`, `max_frames`, `max_stack_depth`, `max_effect_calls` of the one effective configuration |
| `phase_reached` | `"compile"` (source resolution and compilation), `"verify"`, `"entry"`, `"host_init"` or `"execute"` |
| `execution_started` | `true` exactly when a VM was constructed for the run |
| `outcome` | see below |
| `counters` | `null` when execution never started; otherwise `steps`, `calls`, `peak_frames`, `effect_calls` read from the VM at termination |
| `quota_exceeded` | `{kind, limit, used}` exactly as `RuntimeError::QuotaExceeded`, else `null` |
| `error` | the command's error text, else `null` |
| `nondeterministic` | `{wall_ms}`; excluded from every determinism comparison |

Outcomes:

| `outcome` | Condition | `counters` | Exit |
|---|---|---|---|
| `ok` | execution succeeded | measured | 0 |
| `compile_error` | source resolution or compilation failed | `null` | non-zero |
| `verify_rejected` | verifier admission rejected the artifact | `null` | non-zero |
| `entry_missing` | `main` not found in the admitted artifact | `null` | non-zero |
| `host_init_error` | the application host could not be created (for example an invalid `--root`) | `null` | non-zero |
| `quota_exceeded` | `RuntimeError::QuotaExceeded` (`Steps`, `Calls`, `Frames`, `Registers`, `EffectCalls`) | measured | non-zero |
| `capability_denied` | `RuntimeError::CapabilityDenied` | measured | non-zero |
| `trap` | `RuntimeError::Trap` (for example a failed `assert`) | measured | non-zero |
| `runtime_error` | any other runtime failure, including `StackOverflow` (the documented `StackDepth` remap) and host ABI errors | measured | non-zero |

Counter meaning:

- `steps` and `calls` are the `Steps` and `Calls` charge counters (#1759).
  On `quota_exceeded`, `used` reports the failing attempted charge and
  `counters` report the committed value.
- `peak_frames` is the call-stack high-water mark including the root frame.
- `effect_calls` is application-host effect calls plus VM-charged
  gate/state effect calls.
- A zero count is never substituted for a phase that did not run.

Currently unreachable through this command:

- `entry_missing`, because the compiler rejects programs without `main`.
- `StackOverflow`, because `max_stack_depth` always equals `max_frames`, so
  `Frames` is reported first.

The schema still defines both for completeness.

## Source Admission Rule

Commands that ingest source input through `<input.sm>` or admitted project-root forms operate through the current package-admission and helper-module loading rules rather than unrestricted filesystem execution.

Current rule:

- source-reading commands inherit the current executable bundle admission boundary
- project-root `smc compile` uses the bounded project entry resolution, then writes the SemCode artifact only to the requested `-o|--out` path
- `smc check` also accepts a bounded admitted project-root entrypoint through `Semantic.package` + `src/main.sm`
- `smc package inspect <project-root>` prints the deterministic, read-only local package provenance record defined by `semantic.foundation.package/0.1`
- project-root `smc check` also resolves a minimal `semantic.toml` manifest when present
- project-root `smc run` uses the same bounded project entry resolution, then follows the existing verifier-first source execution route
- project-root `smc dump-ast`, `smc dump-ir`, and `smc dump-bytecode` use the same bounded project entry resolution, then emit the existing dump output for the resolved source
- project-root `smc hash-ast` uses the same bounded project entry resolution and emits the existing AST hash for the resolved source file
- project-root `smc hash-ir` uses the same bounded project entry resolution and emits the existing IR hash for the resolved source file
- project-root `smc hash-smc` uses the same bounded project entry resolution and emits the existing SemCode hash for the resolved source or artifact path
- every package-aware operation (one CLI command, one `CliPipeline::semantic_check_file` call, one `check_canonical` request, one `smc watch` rebuild) is one fresh package-admission pass: trust evidence verified by an earlier operation never authorizes a later one, and no caller reset is required (#1776)
- `smc watch` rebuilds when either the imported module graph or the package trust surface changes; the trust surface is the full declared dependency graph, including dependencies no source imports (manifests, declared edges, pins, and module-root content), compared by content and never by modification time (#1777)
- widening package resolution, helper import loading, or source-root admission is a public CLI and source-boundary change
- `smc look ui frame <source-file>` is explicitly exempt from this `.sm`-source package-admission boundary: its input is Projection Source v0 text, a distinct source profile owned by `prom-ui` (not Semantic language source), read directly with a bounded file-size check; see `docs/spec/ui/ui_frame_inspection_cli.md` for its exact accepted source profile and limits
- `smc hub invoke --input <file>` is similarly exempt: its input is a Hub Tool Protocol JSON request file, not Semantic language source, read directly with the same bounded file-size check pattern (8 MiB); see the Hub Tool Invocation Path section above

## Tooling Helper Rule

The following commands are workflow helpers rather than source-language contract owners:

- `smc fmt`
- `smc snapshots`
- `smc repl`
- `smc explain`
- `smc look ui frame`

Current helper behavior:

- `smc fmt` either writes formatting changes or fails under `--check`
- `smc snapshots` shells out to `cargo test --test golden_snapshots`, with `--update` enabling snapshot refresh
- `smc repl` runs interactive check-mode analysis
- `smc explain` renders diagnostic help text or lists known error codes
- `smc look ui frame` is read-only UI frame inspection tooling: it reports existing admission/verification evidence produced by `prom-ui`/`prom-ui-runtime`, but owns no Semantic or admission authority itself (see `docs/spec/ui/ui_frame_inspection_cli.md`)

## Exit Behavior

Current rule:

- successful command execution exits successfully
- user-visible contract violations, usage failures, parsing failures, verification failures, formatting check failures, snapshot failures, and I/O failures produce non-zero termination through CLI error propagation

This draft does not yet formalize a complete numeric exit-code taxonomy.

## Change Review Rule

A CLI change requires explicit review if it changes:

- command names
- flag names
- usage shapes for contract-sensitive commands
- presence or absence of admitted commands listed above
- semantics of verified `.smc` execution behavior
- introduction of machine-readable output modes

Such changes should update this specification in the same change series.
