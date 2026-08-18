# PR 1185 7hell Platform Contour Audit

Status:
  DRAFT / AUDIT ONLY

Core Trust Freeze:
  NOT DECLARED COMPLETE

This audit classifies the remaining failing PR #1185 check
`pcc-qualification-7hell` and separates the current GitHub Linux contour from
the local Windows qualification contour.

Basis:

- [CI workflow](../../../.github/workflows/ci.yml)
- [7hell PowerShell runner](../../../tools/7hell/run.ps1)
- [7hell Bash runner](../../../tools/7hell/run.sh)
- [Raw Execution Compatibility Inventory](raw_execution_compatibility_inventory.md)
- [Core Trust Freeze Checklist](core_trust_freeze_checklist.md)
- [Semantic UI DNA](../../dna/SEMANTIC_UI_DNA.md)

## 1. Current Failing Check

Current failing PR check:

- `pcc-qualification-7hell`

Observed on GitHub Actions:

- runner: `ubuntu-latest`
- command: `bash tools/7hell/run.sh`
- failure: `winit` compile error on Linux

Current log evidence shows the job is not failing in the trust-core / PCC
guard stages. It fails while compiling the UI/native path that brings in
`winit`.

## 2. Exact Failing Command

The failing command is the GitHub workflow step:

```bash
bash tools/7hell/run.sh
```

Within that script, the first failing contour is the Hell 1 workspace-health
stage:

```bash
cargo check --workspace --all-features
```

That workspace-wide command pulls in the UI/native/demo surface on Linux and
eventually reaches `winit`.

## 3. Dependency Path To `winit`

The current workspace evidence shows:

- `prom-ui-backend-native` depends on `winit` directly.
- `prom-ui-demo` depends on `prom-ui-runtime`.
- `prom-ui-runtime` depends on `prom-ui`.
- `Cargo.toml` includes `crates/prom-ui-demo`, `crates/prom-ui-runtime`, and
  `crates/prom-ui-backend-native` as workspace members.

So the effective path is:

`workspace all-features` -> `prom-ui-demo` / `prom-ui-backend-native` ->
`winit`

This is not a trust-core ownership path. It is a UI/native/demo workspace
surface that is currently included by the 7hell workspace-wide build.

## 4. `run.ps1` vs `run.sh`

> **Correction (FA-05-005 / #1735):** the bullet below originally claimed both
> runners start with `cargo check --workspace --all-features` *and*
> `cargo test --workspace --all-features`. At the audited runner state, and
> still on current main, neither runner executes a workspace-wide
> `cargo test`; Hell 1 is `cargo fmt --check` followed by
> `cargo check --workspace --all-features` only. This section is corrected in
> place rather than silently rewritten, since the document's own finding (the
> Linux `winit` failure path) depends only on the `cargo check` command and is
> unaffected by this correction. See `docs/roadmap/pcc/7hell_mini_runner.md`
> for the current, accurate Hell 1 description.

The two runners are structurally similar:

- both run the same Hell 1 / Hell 2 / Hell 3 / Hell 4 / Hell 5 / Hell 6 / Hell
  7 sequence;
- both run `cargo fmt --check` followed by
  `cargo check --workspace --all-features`; neither runs a workspace-wide
  `cargo test`;
- both include the same trust-boundary and SemCode checks.

The differences are platform and shell:

- `run.ps1` is the local Windows parity runner;
- `run.sh` is the GitHub Linux runner;
- `run.sh` uses `bash`, `grep`, and `rm`;
- `run.ps1` uses PowerShell syntax and Windows file operations.

The important point is not shell syntax drift. The important point is that the
same workspace-wide build is executed on two different OS contours, and Linux
fails when `winit` is pulled into the build.

## 5. Contour Classification

Current classification for the failing path:

| Contour | Classification | Notes |
| --- | --- | --- |
| PCC trust-core / boundary checks | PCC trust-core required | These already pass before the UI/native failure. |
| UI/native/demo surface | UI/native/demo optional, but currently included by workspace-wide 7hell | The build includes `prom-ui-demo`, `prom-ui-runtime`, `prom-ui-backend-native`, and `winit`. |
| GitHub `pcc-qualification-7hell` job | accidental workspace-wide inclusion for Linux | The job currently qualifies more than the trust-core contour on a Linux runner. |

The current GitHub 7hell job therefore does not match a narrow trust-core-only
contour. It exercises the full workspace, including UI/native/demo crates.

## 6. Recommended Fix Path

Recommended smallest safe repair:

**Option A - Windows parity**

- keep the 7hell qualification contour aligned with the current local
  evidence;
- run the GitHub `pcc-qualification-7hell` job on `windows-latest`;
- invoke `tools/7hell/run.ps1` instead of the Linux shell runner.

Why this is the smallest safe repair:

- it matches the local qualification evidence already used by the branch;
- it avoids redefining the 7hell contour through a new Linux-safe package
  subset;
- it does not require changing `winit` features or the UI/native dependency
  graph.

Alternative options remain valid but are higher risk:

- **Option B - Linux-safe PCC subset:** keep Ubuntu but narrow the 7hell
  commands to a smaller trust-core contour that excludes the UI/native/demo
  surface.
- **Option C - Full Linux UI support:** fix `winit` / feature support for Linux
  so the entire workspace builds there. This is the highest-risk option and is
  not justified by the current PCC evidence.

## 7. Non-Goals

This audit does not:

- change CI workflows;
- change `winit` features;
- change UI/native behavior;
- change runtime, verifier, VM, or SemCode behavior;
- change sequence ownership logic;
- widen Core Trust Freeze claims;
- remove 7hell from CI.

## 8. Validation For The Future Fix

If Option A is chosen, the future validation should be:

```powershell
cargo fmt --check
powershell -ExecutionPolicy Bypass -File .\tools\7hell\run.ps1
git diff --check
```

If Option B is chosen, add the Linux workflow command set that defines the
narrow PCC subset and verify it separately on `ubuntu-latest`.

If Option C is chosen, validate the Linux UI/native path explicitly with the
targeted `winit`/feature configuration before expanding the claim surface.

## 9. Final Verdict

The current failure is a Linux platform-contour mismatch, not a sequence
ownership regression.

The GitHub 7hell job currently qualifies a workspace-wide UI/native/demo
surface on Ubuntu, while the local evidence for the branch is Windows parity.

Core Trust Freeze remains **not declared complete**.
