# Rust 1.97.1 7hell Rustfmt Component Qualification

Status: PASS

## Baseline

- Main SHA: `6219e67e6f5f233797cfe8047cc54e0148a5a223`
- Origin: post-merge P2 on PR `#1522`
- Predecessor PR: `#1522`
- Review thread: `PRRT_kwDOROOm386R_6YL`
- Rust toolchain: `1.97.1`
- Affected workflows: `2`
- Explicit component: `rustfmt`

## Finding

The 7hell workflows installed Rust 1.97.1 through
`dtolnay/rust-toolchain@master` without explicitly requesting `rustfmt`.

The action installs the minimal profile unless extra components are supplied.
The minimal profile does not include rustfmt.

## Correction

- `.github/workflows/ci.yml`
  - `pcc-qualification-7hell`
  - added `components: rustfmt`

- `.github/workflows/7hell-full.yml`
  - `full-qualification`
  - added `components: rustfmt`

## Non-changes

- Rust source changes: `0`;
- test changes: `0`;
- script changes: `0`;
- dependency changes: `0`;
- toolchain-version changes: `0`;
- no runtime or API change;
- no Gate D or production-status change.

## Validation

- harness: PASS
- formatting: PASS
- fast 7hell: PASS
- full 7hell: PASS
- diff check: PASS
