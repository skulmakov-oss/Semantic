# Getting Started

Status: current-main onboarding guide for the public toolchain surface
(SSF-11 / #1582 external onboarding path)

## Purpose

This guide is the shortest honest route from a fresh clone to:

- building the public `smc` CLI;
- checking, compiling, verifying and running a minimal program;
- inspecting and hashing the verified `.smc` artifact;
- running the canonical examples and the SSF-11 application corpus;
- using a project root and a local package;
- running a capability-controlled file transform;
- understanding failures, compatibility and release status.

You do not need any knowledge that lives outside this repository. The POSIX
commands below were executed on Linux during the SSF-11 cold-start rehearsal
(`reports/ssf11_cold_start_rehearsal.md`). The PowerShell form in section 6
was added afterwards and has not yet been executed in a rehearsal.

> **Release status.** Current `main` is **not** a published stable release.
> Landed work is not release-promised. The final Stable Foundation verdict
> belongs to SSF-12 (#1583) and an explicit human release decision, and
> neither has happened. Read `docs/roadmap/public_status_model.md` before
> treating any surface as stable.

## Prerequisites

| Requirement | Why | Check |
|---|---|---|
| Git | clone | `git --version` |
| `rustup` | the repository pins its toolchain in `rust-toolchain.toml` (Rust `1.97.1`, with `rustfmt` and `clippy`); `rustup` installs it automatically on first `cargo` use | `rustup --version` |
| C toolchain/linker | ordinary Rust linking (`cc` on Linux, MSVC Build Tools on Windows) | `cc --version` |
| **Linux only:** OpenBLAS development library | the workspace links `-lopenblas` through a Hub dependency; without it the `smc` link step fails with `unable to find library -lopenblas` | Debian/Ubuntu: `sudo apt-get install -y libopenblas-dev` |

Run all commands from the repository root.

## 1. Clone and build

```bash
git clone https://github.com/skulmakov-oss/Semantic.git
cd Semantic
cargo --version            # triggers the pinned 1.97.1 toolchain install
rustc -Vv
cargo build --bin smc
```

The binary is `target/debug/smc` (`target\debug\smc.exe` on Windows).
`cargo run --bin smc -- <args>` is equivalent. The examples below use
`cargo run --bin smc --`, which behaves the same in every shell; the one
step that creates files (section 6) gives both POSIX and PowerShell forms.

```bash
cargo run --bin smc -- --help
cargo run --bin smc -- version
```

## 2. Minimal program: check, compile, verify, run

The minimal program already exists at
`examples/qualification/ssf11/f01_minimal/main.sm`:

```semantic
fn main() {
    assert(1 + 1 == 2);
    return;
}
```

```bash
cargo run --bin smc -- check examples/qualification/ssf11/f01_minimal/main.sm
cargo run --bin smc -- compile examples/qualification/ssf11/f01_minimal/main.sm -o minimal.smc
cargo run --bin smc -- verify minimal.smc
cargo run --bin smc -- run-smc minimal.smc
```

Expected:

- `check` prints `smc check passed: 0 warning(s), 0 scheduled law(s)`;
- `compile` writes `minimal.smc` and reports its size;
- `verify` prints `verified ... header=SEMCOD22, epoch=0.23`;
- `run-smc` re-verifies the artifact, executes it and exits `0` silently.
  A failing `assert` exits non-zero with `assertion failed`.

Compiling is **not** verification. `run-smc` always passes the artifact
through the verifier before execution; an inadmissible artifact never runs.
`smc run <file.sm>` is the source workflow command. It compiles, verifies and
executes in one step.

To write your own file, create `program.sm` with the content above in any
editor and use the same commands.

## 3. Inspect and hash the artifact

```bash
cargo run --bin smc -- artifact inspect minimal.smc
cargo run --bin smc -- artifact hash minimal.smc
cargo run --bin smc -- disasm minimal.smc
```

`artifact inspect` reports the SemCode header, capabilities, functions, ADT
descriptors, verifier admission and `Signing State: unsigned`. `artifact
hash` prints `sha256:<64 hex>`. Artifacts are unsigned by design (SSF-10).
The digest is an identity, not a trust signature.

## 4. Canonical examples

```bash
cargo run --bin smc -- run examples/canonical/cli_batch_core/src/main.sm
cargo run --bin smc -- run examples/canonical/match_control_flow/src/main.sm
```

What each example proves, its profile, maturity and command:
`docs/examples_index.md`.

## 5. Project root and local package

A project root is a directory with `semantic.toml`
(`docs/spec/project_model_v0.md`):

```bash
cargo run --bin smc -- check examples/qualification/ssf11/f06_project
cargo run --bin smc -- run examples/qualification/ssf11/f06_project
cargo run --bin smc -- test examples/qualification/ssf11/f06_project
```

`smc test` prints `ok tests/double.sm` and `test result: ok. 1 passed`.

A local package graph uses `Semantic.package` with an explicit relative
dependency (`docs/spec/package_baseline_v0.md`):

```bash
cargo run --bin smc -- run examples/qualification/ssf11/f06_packages/app
cargo run --bin smc -- package inspect examples/qualification/ssf11/f06_packages/app
```

`package inspect` prints the deterministic
`semantic.foundation.package.provenance/0.1` record. It is read-only. There
is no registry, network fetch or implicit dependency search.

## 6. Controlled file transform (capabilities)

Host effects exist only through an explicit profile and root
(`docs/spec/controlled_application_boundary_v0.md`). The paths are resolved
inside `--root`. Use a scratch directory:

POSIX shells (bash, zsh):

```bash
mkdir -p sandbox && printf 'hello\n' > sandbox/input.txt
cd sandbox
cargo run --bin smc -- run ../examples/qualification/ssf11/f08_file_transform/main.sm \
  --profile cli-file-transform --root . -- input.txt output.txt
cat output.txt            # transformed:hello
cd ..
```

PowerShell (Windows):

```powershell
New-Item -ItemType Directory -Force sandbox | Out-Null
[System.IO.File]::WriteAllText("$PWD/sandbox/input.txt", "hello`n")
Set-Location sandbox
cargo run --bin smc -- run ../examples/qualification/ssf11/f08_file_transform/main.sm `
  --profile cli-file-transform --root . -- input.txt output.txt
Get-Content output.txt    # transformed:hello
Set-Location ..
```

stdout carries the program's own output (`transform complete`). stderr carries
one `semantic.foundation.application.audit/0.1` line per capability decision.
Try `--profile cli-read-only` (write denied, `MissingCapability`) or the output
path `../escaped.txt` (denied: parent traversal). Nothing is written in either
case.

## 7. The SSF-11 application corpus

`examples/qualification/ssf11/corpus.json` indexes 30 cases across the twelve
#1582 families. Positive programs, verifier rejections, runtime traps and
quotas, and justified exclusions are all listed, each with its exact
expected observable result. The family-by-family explanation is
`docs/roadmap/stable_foundation/ssf11_application_onboarding_matrix.md`.

```bash
cargo test --test ssf11_canonical_applications
cargo test --test ssf11_onboarding_docs
cargo test --test canonical_examples
```

Deterministic failures you can reproduce by hand:

```bash
cargo run --bin smc -- verify examples/qualification/ssf11/f09_verifier_rejection/unsupported_header.smc
#   verify error [UnsupportedVersion] ...   (exit 1; the artifact never runs)
cargo run --bin smc -- run examples/qualification/ssf11/f10_runtime_failure/division_by_zero.sm
#   runtime trap: DivisionByZero
cargo run --bin smc -- run examples/qualification/ssf11/f10_runtime_failure/step_quota.sm
#   quota exceeded: Steps limit=100000 used=100001
```

## 8. Diagnostics

```bash
cargo run --bin smc -- check examples/qualification/ssf11/f02_quad_bare_condition/main.sm --format json
cargo run --bin smc -- explain E0201
```

`--format json` emits the versioned `semantic.diagnostics` v1 schema
(`docs/spec/diagnostics_machine_schema_v1.md`). Compare diagnostics by `code`,
not by message wording.

## 9. Compatibility and migration

- Policy: `docs/roadmap/compatibility_statement.md`
  (active SemCode baseline `SEMCOD22`; older revisions are `Deprecated`,
  unknown ones are `Incompatible`/`Unsupported`).
- Non-destructive inspection:
  `cargo run --bin smc -- migrate check <path> --json`. It never writes.

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| `rust-lld: error: unable to find library -lopenblas` | Linux without OpenBLAS | `sudo apt-get install -y libopenblas-dev` (or your distribution's OpenBLAS dev package) |
| `cargo` downloads a toolchain on first use | `rust-toolchain.toml` pins `1.97.1` | expected; let it finish |
| `capability ... denied ... MissingCapability` | the profile does not grant that effect (`pure` grants none) | pick the profile the program needs; see `controlled_application_boundary_v0.md` |
| `path must be relative and must not contain parent traversal` | an application path escaped `--root` | keep file arguments inside the root |
| `generic function '...' is admitted by the frontend but is not executable` | generic execution (IR monomorphisation) is Roadmap | use a concrete function; see the matrix row F05 |
| `quota exceeded: Steps ...` | the default execution envelope bounds work deterministically | expected for unbounded loops; `examples/benchmarks/snake_learning.sm` fails closed under `smc run` by design |
| `verify error [...]` | the artifact is malformed or from an incompatible SemCode revision | recompile with the current `smc`; see the compatibility statement |
| `"\n"` in a string literal prints a backslash and `n` | string literals have no escape processing in the current contract | write literal text; tracked as an SSF-01 return in the SSF-11 matrix |

## Where to go next

| Topic | Owner |
|---|---|
| Language tour | `docs/LANGUAGE.md`, `docs/spec/source_semantics.md` |
| Semantic by example | `docs/examples_index.md` |
| Standard library | `docs/spec/foundation_stdlib_v0.md` |
| CLI | `docs/spec/cli.md` |
| Diagnostics | `docs/spec/diagnostics.md` |
| Verifier / runtime | `docs/spec/verifier.md`, `docs/spec/runtime.md`, `docs/spec/quotas.md` |
| Release status | `docs/roadmap/public_status_model.md`, `docs/status/feature_maturity_matrix.md` |

## Boundary reminder

`examples/canonical/boundary_alias_import/` shows a real current limit
(top-level alias import on the executable path), not a supported workflow.
The F05 and F07 corpus cases mark two more boundaries: generic execution and
serialization.
