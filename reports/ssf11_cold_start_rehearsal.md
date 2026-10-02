# SSF-11 Cold-Start Rehearsal

Status: SSF-11 (#1582) onboarding evidence. Not a release or promotion record.

## Candidate

| Field | Value |
|---|---|
| Candidate SHA | `396a0a0ce42bdded1b60e72b6c3b14eaa2500d81` (SSF-11 branch after the onboarding-docs commit; base `main` `ea0d6dcacc70cada5b1fc80e23a5560ecddccb94`) |
| Source | fresh `git clone --branch <SSF-11 branch> https://github.com/skulmakov-oss/Semantic.git` into a new empty temporary directory |
| Build outputs | fresh `target/` inside the clone; nothing was copied from the development worktree |
| Guidance used | `docs/getting_started.md` at the candidate SHA only, no aliases or private notes |
| OS / environment | Ubuntu 24.04.4 LTS, x86_64, Linux 6.18 (ephemeral cloud container) |
| `cargo --version` | `cargo 1.97.1 (c980f4866 2026-06-30)` |
| `rustc -Vv` | `rustc 1.97.1 (8bab26f4f 2026-07-14)`, host `x86_64-unknown-linux-gnu`, LLVM 22.1.6 (selected by `rust-toolchain.toml`) |

## Commands, expected and actual results

| # | Command (`smc` = `cargo run -q --bin smc --`) | Expected | Actual |
|---|---|---|---|
| 1 | `git clone ...`; `git status --short` | clean tree | clean |
| 2 | `cargo build -q --bin smc` | builds | exit 0 |
| 3 | `smc version` | version text | exit 0 |
| 4 | `smc check examples/qualification/ssf11/f01_minimal/main.sm` | `smc check passed` | exit 0 |
| 5 | `smc compile ... -o minimal.smc` | artifact written | exit 0 |
| 6 | `smc verify minimal.smc` | `header=SEMCOD22, epoch=0.23` | exit 0 |
| 7 | `smc run-smc minimal.smc` | silent, exit 0 | exit 0 |
| 8 | `smc artifact inspect minimal.smc` | `Verifier: Admitted`, `Signing State: unsigned` | exit 0 |
| 9 | `smc artifact hash minimal.smc` | `sha256:<64 hex>` | exit 0 |
| 10 | `smc disasm minimal.smc` | disassembly | exit 0 |
| 11 | `smc run examples/canonical/cli_batch_core/src/main.sm` | exit 0 | exit 0 |
| 12 | `smc run examples/canonical/match_control_flow/src/main.sm` | exit 0 | exit 0 |
| 13 | `smc check` / `run` / `test examples/qualification/ssf11/f06_project` | `ok tests/double.sm`, `1 passed` | exit 0 (all three) |
| 14 | `smc run examples/qualification/ssf11/f06_packages/app` | exit 0 | exit 0 |
| 15 | `smc package inspect examples/qualification/ssf11/f06_packages/app` | provenance JSON | exit 0 |
| 16 | in `sandbox/`: `smc run ../examples/qualification/ssf11/f08_file_transform/main.sm --profile cli-file-transform --root . -- input.txt output.txt`; `cat output.txt` | stdout `transform complete`; file `transformed:hello` | as expected, exit 0 |
| 17 | `smc verify examples/qualification/ssf11/f09_verifier_rejection/unsupported_header.smc` | `[UnsupportedVersion]`, exit 1 | as expected |
| 18 | `smc run .../f10_runtime_failure/division_by_zero.sm` | `runtime trap: DivisionByZero`, exit 1 | as expected |
| 19 | `smc run .../f10_runtime_failure/step_quota.sm` | `quota exceeded: Steps limit=100000 used=100001`, exit 1 | as expected |
| 20 | `smc check .../f02_quad_bare_condition/main.sm --format json` | `semantic.diagnostics` v1, code `E0201`, exit 1 | as expected (with a relative `sources[].path`) |
| 21 | `smc explain E0201` | explanation | exit 0 |
| 22 | `smc migrate check examples/qualification/ssf11/f06_project --json` | `Compatible`, `mutations_performed: 0` | as expected |
| 23 | `cargo test -q --test ssf11_canonical_applications` | all corpus cases pass | exit 0 |
| 24 | `cargo test -q --test canonical_examples` | pass | exit 0 |

## Defects discovered

| Defect | Class | Disposition |
|---|---|---|
| A first build on Linux failed with `rust-lld: error: unable to find library -lopenblas`. The previous Getting Started listed no system prerequisite. CI installs `libopenblas-dev`, but the onboarding docs never mentioned it. | Documentation defect | **Fixed in SSF-11**: the Prerequisites and Troubleshooting sections of `docs/getting_started.md` now name it, guarded by `tests/ssf11_onboarding_docs.rs`. The package was installed during the development session before the clean clone, so step 2 above ran with it present. |
| The previous Getting Started used only PowerShell here-strings and `Set-Content`, which are unusable from a POSIX shell. | Documentation defect | **Fixed in SSF-11**: the commands now use checked-in files and `cargo run --bin smc --`, which behave the same in every shell; the one file-creating step (section 6) has POSIX and PowerShell forms (the PowerShell form was added after this rehearsal and was not executed here). |
| `"\n"` in a string literal is written as a backslash and `n` (no escape processing, undocumented). | Earlier-phase contract gap | **RETURN-TO-OWNER SSF-01 / SSF-04**; documented in Troubleshooting. Not blocking. |
| `known(x)`, `unknown(x)` and `conflict(x)` are documented quad predicates but are rejected as unknown functions. | Earlier-phase doc/admission drift | **RETURN-TO-OWNER SSF-01**. Not blocking. |

## Final result

**PASS.** Using only the published repository guidance, an external engineer
can clone, build, check, compile, verify, run, inspect, hash, use a project
and a local package, run a capability-controlled transform, and reproduce
deterministic verifier, trap and quota failures. Status warnings say
plainly that `main` is not a published stable release and that SSF-12
(#1583) has not started.

No credentials, tokens or personal paths are recorded here.
