# Semantic Project Model v0

Status: SSF-05 candidate contract; not a Stable release claim
Contract: `semantic.foundation.project/0.1`

## Canonical layout

```text
project-root/
  semantic.toml
  src/
    main.sm
    lib.sm        # optional module source; not a second entrypoint
  examples/       # explicit runnable programs
  tests/          # standalone test programs
```

`semantic.toml` is the only canonical Project Model v0 manifest. The older
`Semantic.package` form remains a compatibility input and the SSF-06 local
package baseline; when both files exist, `semantic.toml` wins.

The minimal manifest is:

```toml
[package]
name = "hello"

[project]
entry = "src/main.sm"
```

`[project].entry` defaults to `src/main.sm`. It must be a relative path inside
the project root. Absolute paths, `..`, missing entries, unknown sections, and
unknown fields are rejected deterministically. `[package].version` is accepted
as descriptive compatibility metadata and does not grant capabilities or
change verifier admission.

## Source and discovery rules

- The manifest directory is the project root.
- The entry's parent directory is its module root. `src/lib.sm`, when present,
  is an importable module source; it is not discovered as another executable.
- `examples/**/*.sm` are explicit programs and run only when named by path.
- `smc test <project-root>` discovers regular `tests/**/*.sm` files,
  recursively, in normalized relative-path order. Each file is a standalone
  program with `fn main()`, compiled, verified, and executed under the `pure`
  application profile. Empty test sets fail explicitly.
- Symbolic-link/reparse discovery paths and paths resolving outside the project
  root are rejected. Filesystem enumeration order is never observable.
- A direct `.sm` input remains the single-file form and needs no manifest.

## Command agreement

```text
smc check <file.sm|project-root>
smc compile <file.sm|project-root> -o <artifact.smc>
smc verify <artifact.smc|project-root>
smc run <file.sm|project-root>
smc test <project-root>
```

Project-root `check`, `compile`, `verify`, and `run` resolve the same canonical
entry and executable module bundle. Project-root `verify` compiles the resolved
source in memory and performs verifier admission without persisting or running
the artifact. `test` uses the separate bounded discovery rule above.

## Identity boundary

`[package].name` is a descriptive project identifier, not trust. The existing
`smc hash-smc <project-root>` output is the reproducible content identity of the
resolved source/module graph and selected compiler options. `smc verify`
records the SemCode header epoch/revision, which is the currently admitted
artifact-format/toolchain compatibility identity. Manifest spelling, package
names, and hashes do not grant capabilities and cannot override verification.

Project Model v0 makes no signing, cryptographic provenance, registry identity,
or source-compatibility promise. SSF-10 owns those contracts.

## Manual cold-start flow

Create the three required files shown above, put this in `src/main.sm`:

```semantic
fn main() {
    assert(true);
    return;
}
```

Then run:

```text
smc check .
smc compile . -o app.smc
smc verify .
smc verify app.smc
smc run .
smc test .
```

For the last command, add `tests/smoke.sm` containing the same program. From a
repository checkout, use `cargo run --bin smc --` before each argument list.

## SSF-06 entry boundary

SSF-06 may add only reproducible local path dependencies, deterministic graph
ordering/cycle rejection, package hashes, capability-request inventory, and a
minimal provenance/lock record if evidence requires it. Registry access,
remote solving, build/install hooks, implicit capability propagation, and a
broad workspace model remain excluded.
