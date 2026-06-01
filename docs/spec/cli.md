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
- `run`
- `run-smc`
- `disasm`

Current accepted usage forms are:

- `smc compile <input.sm> -o|--out <out.smc> [--profile auto|rust|logos] [--opt-level O0|O1|--opt] [--debug-symbols] [--metrics]`
- `smc check <input.sm|project-root> [--no-cache] [--trace-cache] [--metrics] [--deny warnings|<CODE>] [--color auto|always|never]`
- `smc lint <input.sm> [--no-cache] [--trace-cache] [--deny warnings|<CODE>] [--color auto|always|never]`
- `smc watch <input.sm> [--metrics] [--color auto|always|never]`
- `smc fmt [--check] <path>`
- `smc dump-ast <input.sm>`
- `smc dump-ir <input.sm> [--profile auto|rust|logos] [--opt-level O0|O1|--opt]`
- `smc dump-bytecode <input.sm> [--profile auto|rust|logos] [--opt-level O0|O1|--opt] [--debug-symbols]`
- `smc hash-ast <input.sm>`
- `smc hash-ir <input.sm> [--profile auto|rust|logos] [--opt-level O0|O1|--opt]`
- `smc hash-smc <input.sm> [--profile auto|rust|logos] [--opt-level O0|O1|--opt] [--debug-symbols] [--trace-cache]`
- `smc snapshots [--update]`
- `smc features`
- `smc explain <error-code|--list>`
- `smc repl`
- `smc verify <input.smc>`
- `smc run <input.sm>`
- `smc run-smc <input.smc>`
- `smc disasm <input.smc>`

This draft does not claim that every command is permanently frozen, but it defines the current public CLI surface that tooling may rely on.

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
- `smc run-smc`
- `smc features`

The following inspection commands are public workflow surface, but their plain-text rendering is not yet a frozen machine-readable format:

- `smc dump-ast`
- `smc dump-ir`
- `smc dump-bytecode`
- `smc hash-ast`
- `smc hash-ir`
- `smc hash-smc`
- `smc disasm`

## Output Modes

Current output families are:

- human-readable text
- plain-text dumps and hashes for inspection commands

There is currently no admitted machine-readable JSON output contract in `smc-cli`.

Current output rules:

- `smc features` reports enabled and disabled feature sets as text
- `smc dump-*`, `smc hash-*`, and `smc disasm` emit plain text for inspection
- `smc check`, `smc lint`, and `smc watch` support colorized human-readable diagnostics via `--color auto|always|never`

## Verified Execution Rule

Current execution-facing commands follow this split:

- `smc run <input.sm>` compiles source input and executes the produced in-memory SemCode path
- `smc verify <input.smc>` performs verifier admission without execution
- `smc run-smc <input.smc>` executes compiled SemCode through the verified artifact path

Public rule:

- persisted `.smc` execution must not bypass verification
- `smc run` is a source-execution workflow command, not the persisted artifact admission path

## Source Admission Rule

Commands that ingest source input through `<input.sm>` operate through the current package-admission and helper-module loading rules rather than unrestricted filesystem execution.

Current rule:

- source-reading commands inherit the current executable bundle admission boundary
- `smc check` also accepts a bounded admitted project-root entrypoint through `Semantic.package` + `src/main.sm`
- widening package resolution, helper import loading, or source-root admission is a public CLI and source-boundary change

## Tooling Helper Rule

The following commands are workflow helpers rather than source-language contract owners:

- `smc fmt`
- `smc snapshots`
- `smc repl`
- `smc explain`

Current helper behavior:

- `smc fmt` either writes formatting changes or fails under `--check`
- `smc snapshots` shells out to `cargo test --test golden_snapshots`, with `--update` enabling snapshot refresh
- `smc repl` runs interactive check-mode analysis
- `smc explain` renders diagnostic help text or lists known error codes

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
