# 7hell PCC Qualification Mini-Runner

The `7hell` mini-runner is a deterministic local qualification harness that proves our critical SemCode and VM boundaries are intact. It is an intentional gateway designed to prevent regressions in Core Trust boundaries, SemCode format structures, and the PCC-ADT payload ownership slice.

This is **not** the full release qualification runner.

### Known Quirks
- The `sm-verify` and `sm-vm` test gates currently explicitly enable `sm-ir/profile-rust` because verifier fixtures depend on Rust-like profile paths to successfully compile.

## Execution
To run all 7 gates locally, use the full runner for your OS:

**Windows:**
```powershell
.\tools\7hell\run.ps1
```

**Linux/macOS:**
```bash
./tools/7hell/run.sh
```

## CI Usage
The CI pipeline uses the fast gate (`tools/7hell/run_ci.ps1`) to keep PR checks short. The full runner remains available locally via `run.ps1` / `run.sh` for the complete qualification pack.
The full runner also has a separate manual/nightly GitHub workflow for cases where the complete pack needs to run in CI without blocking PR checks.

## The 7 Gates
The script will fail fast if any gate is broken.
- **Hell 1:** Workspace Health (`cargo fmt`, `check`)
- **Hell 2:** Trust Boundary Guards (`cargo tree` anti-dependency checks)
- **Hell 3:** SemCode Format Authority (`sm-format` isolation and `rg` leakage checks)
- **Hell 4:** Verifier Negative Corpus (`sm-verify` admission tests)
- **Hell 5:** VM Ownership Semantics (`sm-vm` borrow/overlap tests)
- **Hell 6:** Source to SemCode Smoke (golden pipeline compilation, tuple/record/Option-Result smoke, public CLI smoke matrix, and PCC control-flow + text + collections + stdlib negative diagnostics)
- **Hell 7:** PCC Documentation Integrity (matrix and architecture docs exist)

Read more at: `docs/roadmap/pcc/7hell_mini_runner.md`
