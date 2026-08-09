# Semantic Feature Maturity Matrix

Status: current-facing index
Scope: implementation depth and public-status routing

The authoritative one-status-per-feature inventory is now:

- [`docs/roadmap/stable_foundation/semantic_stable_foundation_matrix.md`](../roadmap/stable_foundation/semantic_stable_foundation_matrix.md)

That matrix records the exact SSF-00 evidence snapshot, owner layers, one of the
seven #1571 statuses for every public feature family, evidence boundaries, and
the later SSF phase that owns each unresolved decision. Status values must not
be duplicated here.

## Implementation depth

The following independent scale remains useful when describing how far an
implementation path reaches. It is not a release status and cannot promote a
feature.

| Level | Meaning |
|---|---|
| D0 | Documented or roadmap only |
| D1 | Parsed by the source frontend |
| D2 | Typechecked or semantically accepted |
| D3 | Lowered to IR |
| D4 | Emitted to SemCode |
| D5 | Accepted by `sm-verify` |
| D6 | Executed by `sm-vm` |
| D7 | Covered by focused positive, negative, golden, adversarial, or benchmark evidence |

A D7 feature may still be **Landed and qualified on `main`**, rather than
**Qualified limited release** or **Published stable**. Conversely, a parser-only
feature must not be described as executable merely because it has a source
spelling.

## Current top-level posture

- No feature currently meets the matrix's **Published stable** evidence test:
  `v1.1.1` is a git tag whose own checkpoint left exact-tag asset smoke
  blocking, and no corresponding GitHub Release exists.
- The bounded Gate 1 contour remains **Qualified limited release**.
- Current `main` contains wider qualified and unqualified implementation that
  remains unpromoted.
- Logos is the separate experimental declarative profile selected by SSF-02;
  it does not produce SemCode or share Rust-like execution authority.
- Named `std.*` modules, the controlled application capability set, lock and
  provenance records, a canonical language server, and Foundation migration
  tooling remain roadmap work.
- UI and Workbench evidence stays separate from language, verifier, VM, and
  release authority.

## Supporting authorities

- Status vocabulary: `docs/roadmap/public_status_model.md`
- Release-facing posture: `docs/roadmap/v1_readiness.md`
- Qualified contour: `reports/g1_release_scope_statement.md`
- Wider benchmark contour: `reports/application_completeness_benchmark_verdict.md`
- Stable Foundation target: `docs/roadmap/stable_foundation/stable_foundation_target_contract.md`

Historical matrices and completed reports remain evidence for their exact
commits and scopes. They do not override the current SSF-00 matrix.
