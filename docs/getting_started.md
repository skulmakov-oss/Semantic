# Getting Started

Status: current-main onboarding guide for the current public toolchain surface

## Purpose

This guide gives an external engineer the shortest honest path from clone to:

- building the public CLI entrypoints
- checking and running a minimal program
- compiling and verifying a `.smc` artifact
- running the verified artifact and inspecting it when needed
- optionally reviewing the current diagnostic/readiness path

This is an onboarding guide, not a release-promotion document. Current `main`
includes landed work beyond the published stable line, so release reading still
follows the status model in `docs/roadmap/public_status_model.md`.

## External Onboarding Map

Use this path if you are reading the repository for the first time and want a
single honest route from clone to the current documented surface:

1. Understand status and non-claims
   - `README.md`
   - `docs/roadmap/public_status_model.md`
   - `docs/roadmap/v1_readiness.md`
   - `docs/status/feature_maturity_matrix.md`

2. Follow the shortest practical path
   - this guide

3. Move to the curated proof surface
   - `docs/examples_index.md`
   - `examples/canonical/README.md`

4. Read the public command and diagnostic contracts
   - `docs/spec/cli.md`
   - `docs/spec/diagnostics.md`

5. Review release posture and non-claims
   - `docs/roadmap/stable_release_policy.md`
   - `docs/roadmap/public_maturity_snapshot.md`

The canonical examples pack is the practical proof surface for the current
readiness contour. It is the right place to see the admitted `check`, `run`,
`compile`, and `verify` path in action before reading release-facing status.

## Prerequisites

- Rust toolchain installed
- repository cloned locally
- commands run from repository root

## Build The Public Entry Points

```powershell
cargo build --bin smc --bin svm
```

## Minimal Source Loop

Create a minimal source file:

```powershell
@'
fn main() {
    return;
}
'@ | Set-Content program.sm
```

Check the source:

```powershell
cargo run --bin smc -- check program.sm
```

Compile to SemCode:

```powershell
cargo run --bin smc -- compile program.sm -o program.smc
```

Verify the compiled artifact:

```powershell
cargo run --bin smc -- verify program.smc
```

Run the verified `.smc` artifact:

```powershell
cargo run --bin smc -- run-smc program.smc
```

Disassemble the compiled artifact:

```powershell
cargo run --bin svm -- disasm program.smc
```

If you want to run from source instead of the verified artifact route, `smc run program.sm` remains the source-execution workflow command. The practical onboarding order is still:

1. write or use a small `.sm` example
2. check source
3. compile to SemCode
4. verify the compiled artifact
5. run the verified artifact
6. disassemble if needed

The admitted Practical Core path uses explicit source/project-root entry resolution through the current bounded admission model.

The current baseline also exposes `smc 7hell program.sm [--json]` as a diagnostic/readiness path. Use it for report-quality checks and qualification review, not as the normal first-run route.

## Canonical Example Loop

The current curated examples pack lives in:

- `examples/canonical/`

Start with:

- `examples/canonical/cli_batch_core/src/main.sm`

Check it:

```powershell
cargo run --bin smc -- check examples/canonical/cli_batch_core/src/main.sm
```

Run it:

```powershell
cargo run --bin smc -- run examples/canonical/cli_batch_core/src/main.sm
```

Compile and verify it:

```powershell
cargo run --bin smc -- compile examples/canonical/cli_batch_core/src/main.sm -o cli_batch_core.smc
cargo run --bin smc -- verify cli_batch_core.smc
```

If you want a broader tour of the curated pack, see `docs/examples_index.md`.

## Current Public CLI References

For the current admitted CLI surface, see:

- `docs/spec/cli.md`

For the canonical spec bundle, start at:

- `docs/spec/index.md`

## Validation

Useful repository-level checks during onboarding:

```powershell
cargo test -q
cargo test -q --test public_api_contracts
cargo test -q --test canonical_examples
```

## Boundary Reminder

The canonical examples pack includes one honest boundary example:

- `examples/canonical/boundary_alias_import/`

It exists to show a real current limit, not a supported workflow, and future support for that alias-import form would require an explicit language/source-admission change.
