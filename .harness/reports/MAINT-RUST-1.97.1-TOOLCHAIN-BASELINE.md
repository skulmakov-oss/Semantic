# Rust 1.97.1 Toolchain Baseline Qualification

Status: PASS

## Baseline

- Main SHA: `c71242d04c1f6962c9bc816b535b9136e9113d23`
- Branch: `maintenance/rust-1.97.1-toolchain-baseline`
- Previous PR-ready toolchain: Rust `1.93.1`
- Previous remaining-CI toolchain: floating `stable`
- Previous scheduled 7hell toolchain: floating `stable`
- Root `rust-toolchain` file previously absent: `YES`
- Root `rust-toolchain.toml` file previously absent: `YES`

## New baseline

- Toolchain: Rust `1.97.1`
- Profile: `minimal`
- Components: `rustfmt`, `clippy`
- Root toolchain file: `rust-toolchain.toml`
- CI jobs pinned: `pr-ready`, `boundary-enforcement`, `public-api-guard`,
  `runtime-release-gates`, `pcc-qualification-7hell`, `test-std`,
  `check-no-std`
- Scheduled workflow pinned: `7hell Full Qualification`

## MSRV boundary

- `Cargo.toml` modified: `NO`
- `Cargo.lock` modified: `NO`
- Workbench `rust-version` modified: `NO`
- MSRV claim changed: `NO`
- Repository development/CI baseline changed: `YES`

## Validation

| Command | Result |
| --- | --- |
| `rustup toolchain install 1.97.1 --profile minimal --component rustfmt --component clippy` | `PASS` |
| `rustup show active-toolchain` | `PASS — 1.97.1-x86_64-pc-windows-msvc` |
| `rustc --version --verbose` | `PASS — rustc 1.97.1 (8bab26f4f 2026-07-14)` |
| `cargo --version --verbose` | `PASS — cargo 1.97.1 (c980f4866 2026-06-30)` |
| `rustfmt --version` | `PASS — rustfmt 1.9.0-stable` |
| `cargo clippy --version` | `PASS — clippy 0.1.97` |
| `pwsh -File scripts/harness-check.ps1` | `PASS` |
| `cargo +1.97.1 fmt --all --check` | `PASS` |
| `cargo +1.97.1 clippy --workspace --all-targets -- -D warnings` | `BLOCKED — 2 Rust 1.97.1 Clippy errors` |
| `cargo +1.97.1 fmt --all --check` after the two authorized corrections | `BLOCKED — rustfmt requires a one-line form in parser.rs` |
| `cargo +1.97.1 fmt --all --check` after canonicalization | `PASS` |
| `cargo +1.97.1 clippy -p sm-front --all-targets -- -D warnings` | `PASS` |
| `cargo +1.97.1 test -p sm-front` | `PASS — 449 tests` |
| `cargo +1.97.1 check -p sm-front --all-targets` | `PASS` |
| `cargo +1.93.1 check -p sm-front --all-targets` | `PASS` |
| resumed `pwsh -File scripts/harness-check.ps1` | `PASS` |
| resumed `cargo +1.97.1 fmt --all --check` | `PASS` |
| resumed `cargo +1.97.1 clippy --workspace --all-targets -- -D warnings` | `BLOCKED — 3 new Rust 1.97.1 Clippy errors outside the authorized paths` |
| reproduced `cargo +1.97.1 clippy --workspace --all-targets --message-format=short -- -D warnings` | `BLOCKED — exact expected 3-error set reproduced` |
| targeted `cargo +1.97.1 fmt --all --check` after the three authorized corrections | `BLOCKED — prescribed let-chain requires Rust 2024 edition` |
| targeted `sm-sema` / `sm-ir` Clippy | `NOT RUN — fail-closed after targeted formatting blocker` |
| targeted `sm-sema` / `sm-ir` tests | `NOT RUN — fail-closed after targeted formatting blocker` |
| targeted `sm-sema` / `sm-ir` checks | `NOT RUN — fail-closed after targeted formatting blocker` |
| `sm-front` regression checks | `NOT RUN — fail-closed after targeted formatting blocker` |
| combined Rust 1.93.1 sanity check | `NOT RUN — fail-closed after targeted formatting blocker` |
| `cargo +1.97.1 fmt --all --check` after Rust 2021 match-guard correction | `PASS` |
| `cargo +1.97.1 clippy -p sm-ir --all-targets -- -D warnings` | `PASS` |
| `cargo +1.97.1 test -p sm-ir` | `PASS — 99 tests` |
| `cargo +1.97.1 check -p sm-ir --all-targets` | `PASS` |
| modified-crate Clippy for `sm-front`, `sm-sema`, and `sm-ir` | `PASS` |
| modified-crate tests | `PASS — sm-front 449, sm-sema 44, sm-ir 99` |
| combined Rust 1.93.1 sanity check after match-guard correction | `PASS` |
| resumed harness after match-guard correction | `PASS` |
| resumed workspace formatting after match-guard correction | `PASS` |
| resumed workspace Clippy after match-guard correction | `BLOCKED — 1 new Rust 1.97.1 Clippy error in smc-cli` |
| reproduced workspace Clippy before the `smc-cli` correction | `BLOCKED — exact sole smc-cli diagnostic reproduced` |
| targeted formatting after the `smc-cli` correction | `PASS` |
| targeted `smc-cli` Clippy | `PASS` |
| targeted `smc-cli` tests | `PASS — 78 tests` |
| targeted `smc-cli` check | `PASS` |
| Rust 1.93.1 `smc-cli` sanity check | `PASS` |
| modified-crate Clippy including `smc-cli` | `PASS` |
| modified-crate tests including `smc-cli` | `PASS — sm-front 449, sm-sema 44, sm-ir 99, smc-cli 78` |
| resumed harness after the `smc-cli` correction | `PASS` |
| resumed workspace formatting after the `smc-cli` correction | `PASS` |
| resumed workspace Clippy after the `smc-cli` correction | `BLOCKED — 1 new Rust 1.97.1 Clippy error in test support` |
| targeted `cargo +1.97.1 fmt --all --check` after the test-support correction | `PASS` |
| targeted workspace Clippy after the test-support correction | `PASS` |
| `cargo +1.97.1 test --test g1_frontend_trust` | `PASS — 2 tests` |
| `cargo +1.97.1 test --test g1_execution_integrity` | `PASS — 3 tests` |
| `cargo +1.97.1 test --test g1_benchmark_baseline` | `PASS — 1 test` |
| `cargo +1.97.1 test --test g1_real_program_trial` | `PASS — 6 tests` |
| `cargo +1.93.1 check --tests` | `PASS` |
| full-qualification toolchain identity checks | `PASS — Rust 1.97.1, rustfmt 1.9.0, Clippy 0.1.97` |
| full-qualification `pwsh -File scripts/harness-check.ps1` | `BLOCKED — authorized test-support path still matches forbidden tests/** scope` |
| workspace all-targets/all-features check | `NOT RUN — fail-closed after resumed workspace Clippy blocker` |
| standard tests | `NOT RUN — fail-closed after resumed workspace Clippy blocker` |
| no-default-features gate | `NOT RUN — fail-closed after resumed workspace Clippy blocker` |
| fast 7hell | `NOT RUN — fail-closed after resumed workspace Clippy blocker` |
| full 7hell | `NOT RUN — fail-closed after resumed workspace Clippy blocker` |
| restarted full-qualification toolchain identity checks | `PASS — Rust 1.97.1, rustfmt 1.9.0, Clippy 0.1.97` |
| harness after scope-conflict correction | `PASS` |
| restarted workspace formatting | `PASS` |
| restarted workspace Clippy with denied warnings | `PASS` |
| workspace all-targets/all-features check with denied warnings | `PASS` |
| `cargo +1.97.1 test --all-targets --quiet` | `PASS` |
| `cargo +1.97.1 check --no-default-features --quiet` | `PASS — limited gate; not a workspace-wide no_std claim` |
| fast 7hell | `PASS` |
| full 7hell | `PASS` |
| ten explicit boundary and runtime gates | `PASS` |
| final `git diff --check` | `PASS` |

## Compatibility findings

Rust `1.97.1` Clippy with `-D warnings` reports:

```text
error: consider using `sort_by_key`
    --> crates\sm-front\src\parser.rs:2978:9
     |
2978 |         out.laws.sort_by(|a, b| b.priority.cmp(&a.priority));
     |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
     |
     = note: `-D clippy::unnecessary-sort-by` implied by `-D warnings`
help: try
     |
2978 -         out.laws.sort_by(|a, b| b.priority.cmp(&a.priority));
2978 +         out.laws.sort_by_key(|b| std::cmp::Reverse(b.priority));

error: explicit call to `.into_iter()` in function argument accepting `IntoIterator`
    --> crates\sm-front\src\typecheck.rs:1281:53
     |
1281 |             for (item, item_ty) in items.iter().zip(item_tys.into_iter()) {
     |                                                     ^^^^^^^^^^^^^^^^^^^^
     |
     = note: `-D clippy::useless-conversion` implied by `-D warnings`
help: consider removing the `.into_iter()`
     |
1281 -             for (item, item_ty) in items.iter().zip(item_tys.into_iter()) {
1281 +             for (item, item_ty) in items.iter().zip(item_tys) {

error: could not compile `sm-front` (lib) due to 2 previous errors
```

Responsible repository paths:

- `crates/sm-front/src/parser.rs`
- `crates/sm-front/src/typecheck.rs`

## Authorized source compatibility corrections

The two authorized mechanical corrections were applied:

| Path | Diagnostic | Correction | Behavior impact |
| --- | --- | --- | --- |
| `crates/sm-front/src/parser.rs` | `clippy::unnecessary-sort-by` | stable `sort_by_key` with `Reverse` | none |
| `crates/sm-front/src/typecheck.rs` | `clippy::useless-conversion` | removed redundant `.into_iter()` | none |

No other source correction, lint suppression, CI-command weakening, or
dependency change was performed.

## Initial rustfmt blocker

The first targeted command stopped the resumed qualification:

```text
Diff in crates/sm-front/src/parser.rs:2975:
-        out.laws
-            .sort_by_key(|law| core::cmp::Reverse(law.priority));
+        out.laws.sort_by_key(|law| core::cmp::Reverse(law.priority));
```

Rustfmt `1.97.1` required the authorized parser correction to use a one-line
form.

## Rustfmt canonicalization

The initially authorized parser correction was written across two lines.
Rust 1.97.1 rustfmt requires the expression in its canonical one-line form:

```rust
out.laws.sort_by_key(|law| core::cmp::Reverse(law.priority));
```

This adjustment changed formatting only. Sorting behavior, stability,
priority ordering, parser semantics, and public API remained unchanged.

## Historical workspace Clippy blockers

After all five targeted checks passed, the resumed workspace Clippy command
reported three new diagnostics outside the authorized source paths:

```text
error: this block may be rewritten with the `?` operator
   --> crates\sm-sema\src\alloc_core.rs:441:12
    |
441 |       } else if let Some(x) = e.strip_prefix("fx.div(") {
    |  ____________^
442 | |         ("div", x)
443 | |     } else {
444 | |         return None;
445 | |     };
    | |_____^
    |
    = note: `-D clippy::question-mark` implied by `-D warnings`

error: consider using `sort_by_key`
   --> crates\sm-ir\src\legacy_lowering.rs:572:5
    |
572 |     laws.sort_by(|a, b| b.priority.cmp(&a.priority));
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `-D clippy::unnecessary-sort-by` implied by `-D warnings`

error: this `if` can be collapsed into the outer `match`
    --> crates\sm-ir\src\legacy_lowering.rs:1025:17
     |
1025 | /                 if !labels.contains_key(label) {
1026 | |                     return Err(FrontendError {
1027 | |                         pos: idx,
1028 | |                         message: format!("jump to unknown label '{}' in '{}'", label, f.name),
1029 | |                     });
1030 | |                 }
     | |_________________^
     |
     = note: `-D clippy::collapsible-match` implied by `-D warnings`
```

Affected paths:

- `crates/sm-sema/src/alloc_core.rs`
- `crates/sm-ir/src/legacy_lowering.rs`

Classification: Clippy compatibility blockers. These paths are not authorized
for mutation. The next-failure policy therefore stopped qualification before
the remaining full checks. No additional source fix, commit, push, or PR was
performed.

Rust 1.97.1 qualification required two mechanical Clippy corrections and one
formatting-only canonicalization of the already authorized parser correction.
The resumed workspace check exposed three additional Clippy blockers, so the
qualification remained blocked at that checkpoint.

## Second workspace compatibility correction

Rust 1.97.1 subsequently exposed three additional denied workspace Clippy
diagnostics:

1. `crates/sm-sema/src/alloc_core.rs`
   - `clippy::question-mark`
   - final `fx.div` recognition branch expressed with `?`

2. `crates/sm-ir/src/legacy_lowering.rs`
   - `clippy::unnecessary-sort-by`
   - stable descending sort expressed with `sort_by_key` and `Reverse`

3. `crates/sm-ir/src/legacy_lowering.rs`
   - `clippy::collapsible-match`
   - jump-validation pattern and label-existence condition collapsed without
     changing the error path

| Path | Diagnostic | Correction | Behavior impact |
| --- | --- | --- | --- |
| `crates/sm-sema/src/alloc_core.rs` | `clippy::question-mark` | `fx.div` fallback expressed with `?` | none |
| `crates/sm-ir/src/legacy_lowering.rs` | `clippy::unnecessary-sort-by` | stable `sort_by_key` with `Reverse` | none |
| `crates/sm-ir/src/legacy_lowering.rs` | `clippy::collapsible-match` | collapsed jump-validation match | none |

No additional source correction was authorized or performed.

## Historical targeted formatting blocker

The first targeted validation command failed:

```text
error: let chains are only allowed in Rust 2024 or later
    --> crates\sm-ir\src\legacy_lowering.rs:1023:12
     |
1023 |         if let IrInstr::Jmp { label } | IrInstr::JmpIf { label, .. } = instr
     |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Error writing files: failed to resolve mod `legacy_lowering`: cannot parse
crates\sm-ir\src\legacy_lowering.rs
```

Command:

```text
cargo +1.97.1 fmt --all --check
```

Classification: formatting/parser compatibility blocker. The prescribed
let-chain form requires Rust 2024 edition, while this crate uses an earlier
edition. Replacing it with the Clippy-rendered match-guard form is outside this
exact authorization. The remaining targeted and full checks were not run. No
commit, push, or PR was created.

## Rust 2021 edition compatibility correction

The initially prescribed collapsed `if let` chain required Rust edition 2024
and was therefore incompatible with the repository's existing Rust 2021
edition.

The jump-validation condition was expressed instead as a Rust-2021-compatible
match guard:

```rust
match instr {
    IrInstr::Jmp { label } | IrInstr::JmpIf { label, .. }
        if !labels.contains_key(label) =>
    {
        // unchanged error path
    }
    _ => {}
}
```

The replacement preserves instruction matching, label lookup, error position,
error text, and return behavior. No crate edition or manifest was changed.

## Historical workspace Clippy blocker after edition correction

After all targeted checks passed, the resumed workspace Clippy command found
one additional diagnostic:

```text
error: consider using `sort_by_key`
   --> crates\smc-cli\src\executable_bundle.rs:484:5
    |
484 |     replacements.sort_by(|a, b| b.0.cmp(&a.0));
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `-D clippy::unnecessary-sort-by` implied by `-D warnings`
help: try
    |
484 -     replacements.sort_by(|a, b| b.0.cmp(&a.0));
484 +     replacements.sort_by_key(|b| std::cmp::Reverse(b.0));
```

Command:

```text
cargo +1.97.1 clippy --workspace --all-targets -- -D warnings
```

Classification: Clippy compatibility blocker. The affected
`crates/smc-cli/src/executable_bundle.rs` path is outside this authorization.
The remaining full qualification commands were not run. No source correction,
commit, push, or PR was performed.

## `smc-cli` compatibility correction

Rust 1.97.1 exposed one additional denied Clippy diagnostic in
`crates/smc-cli/src/executable_bundle.rs`:

- diagnostic: `clippy::unnecessary-sort-by`
- correction: stable descending `sort_by_key` with `Reverse`
- purpose: preserve reverse-offset replacement order
- behavior impact: none

No adjacent bundle, token, rename, range-replacement, or output logic changed.

## Historical test-support Clippy blocker

After all targeted checks passed, the resumed workspace Clippy command found
one additional diagnostic:

```text
error: consider using `sort_by_key`
   --> tests\support\executable_bundle_support.rs:440:5
    |
440 |     replacements.sort_by(|a, b| b.0.cmp(&a.0));
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `-D clippy::unnecessary-sort-by` implied by `-D warnings`
help: try
    |
440 -     replacements.sort_by(|a, b| b.0.cmp(&a.0));
440 +     replacements.sort_by_key(|b| std::cmp::Reverse(b.0));
```

Command:

```text
cargo +1.97.1 clippy --workspace --all-targets -- -D warnings
```

Classification: Clippy compatibility blocker in test support. The affected
`tests/support/executable_bundle_support.rs` path is outside this authorization,
and test modification is explicitly forbidden. The remaining full
qualification commands were not run. No test correction, commit, push, or PR
was performed.

## Test-support compatibility correction

Rust 1.97.1 exposed one additional denied Clippy diagnostic in the shared
executable-bundle test-support helper:

- path: `tests/support/executable_bundle_support.rs`
- diagnostic: `clippy::unnecessary-sort-by`
- correction: stable descending `sort_by_key` with `Reverse`
- behavior impact: none

The helper continues to apply text replacements in descending source-offset
order. No test case, assertion, fixture, expected result or production source
behavior changed.

## Historical harness scope blocker

All seven targeted checks passed after the test-support correction. The full
qualification then stopped at its first harness check:

```text
[harness:error] forbidden path changed: tests/support/executable_bundle_support.rs
```

Command:

```text
pwsh -File scripts/harness-check.ps1
```

The task manifest lists `tests/support/executable_bundle_support.rs` under
`scope.allowed_paths` and separately authorizes that exact compatibility
correction, but the same manifest still includes `tests/**` under
`scope.forbidden_paths`. The harness applies the forbidden pattern and rejects
the changed path. Classification: harness authorization conflict. Per the
fail-closed and stop-on-next-failure constraints, no further qualification
command, manifest correction, commit, push or PR was performed.

## Harness scope-conflict correction

The test-support file was explicitly listed in `allowed_paths`, while the
broader `tests/**` pattern remained in `forbidden_paths`.

The repository harness evaluates forbidden patterns before allowed patterns,
so the exact authorization could not override the broad prohibition.

The conflicting `tests/**` entry was removed from `forbidden_paths`.
The exact test-support file remains the only allowed test path. All other test
paths remain outside `allowed_paths` and therefore remain rejected.

No harness script or enforcement precedence changed.

## Scope

- Rust source modified: `YES — exactly five authorized paths across sm-front, sm-sema, sm-ir, and smc-cli`
- Shared test-support Rust modified: `YES — exactly one authorized helper path`
- Test cases, assertions, fixtures or expected results modified: `NO`
- Dependencies modified: `NO`
- Public API modified: `NO`
- Runtime modified: `NO`
- Architecture modified: `NO`

Rust 1.97.1 qualification required seven mechanical Clippy corrections,
one rustfmt canonicalization, one Rust-2021-compatible structural rewrite,
and one task-manifest scope-conflict correction.

No test case, assertion, fixture, dependency, Cargo manifest, lockfile, MSRV,
edition, public API, runtime, architecture, harness script, Gate D or
production-status change was required.

The qualification intentionally added the root `rust-toolchain.toml`, updated
the local harness task manifest, and pinned the two authorized GitHub Actions
workflow files.

- Current blocker count: `0`
- Qualification status: `PASS`

## Governance

- Gate D: `CLOSED`
- Production promotion: `NOT AUTHORIZED`
- Follow-on implementation: `NOT AUTHORIZED`
- NEXT AUTHORIZED IMPLEMENTATION SLICE: `NONE`
