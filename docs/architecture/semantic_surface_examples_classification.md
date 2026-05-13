# Semantic Surface Examples Classification

Status: pass-2 surface classification draft

See also the pass-1 vocabulary audit matrix in
[`semantic_surface_vocabulary_audit.md`](semantic_surface_vocabulary_audit.md).

## 1. Purpose

This document classifies current Semantic examples, docs, and fixtures into
surface buckets:

- canonical
- bridge-only
- implementation-detail
- rejected-as-canonical
- deprecated-candidate
- undecided

The goal is to separate current executable bridge syntax from public
Semantic-facing wording without renaming anything yet.

## 2. Non-Goals

This document does not:

- rename syntax
- change grammar
- change parser or typechecker behavior
- rewrite fixtures
- rewrite tests
- rewrite README content
- implement Hello World
- implement `print` / `observe`
- add general I/O
- freeze the final vocabulary
- implement `#477`
- implement `#479`
- start Linguist readiness
- touch UI / Workbench / I70

## 3. Classification Buckets

- `canonical`: safe public Semantic-facing wording
- `bridge-only`: executable or temporary frontend vocabulary, not canonical
- `implementation-detail`: internal, tooling, or runtime term, not source-surface vocabulary
- `rejected-as-canonical`: must not be shown as canonical public Semantic
- `deprecated-candidate`: likely migration target later, not changed now
- `undecided`: needs `#479` or a later surface decision

## 4. Example / Location Classification Matrix

| location / pattern | current example shape | detected vocabulary | classification | reason | allowed for now? | follow-up |
|---|---|---|---|---|---|---|
| `tests/fixtures/**/*.sm` with `fn main`, `return`, `assert` | executable fixture sources for PCC-1/PCC-2/PCC-3 and benchmark-style checks | current frontend bridge syntax | `bridge-only` | These fixtures keep the executable path green under the current frontend, but they are not the canonical public Semantic vocabulary. | yes | Keep labeling them as current executable bridge syntax in future docs/tests; do not promote to public canonical wording yet. |
| `README.md`, `docs/examples_index.md`, `examples/**/*.sm` | onboarding examples, canonical example index, benchmark/example programs | mixed public-facing wording plus executable bridge syntax | `undecided` for public-facing examples; `bridge-only` where the file is clearly an executable bridge sample | These files are what users read first, so they must not accidentally canonize legacy syntax. Some rows are executable bridge examples, but the surface as a whole is not yet fully reviewed. | partially | Classify individual examples before promoting them to canonical docs; keep bridge labels explicit. |
| `smc` CLI verbs: `run`, `check`, `compile`, `verify`, `run-smc` | CLI docs and workflow examples | command vocabulary | `implementation-detail` | These are public CLI terms, but they are tooling surface, not source-language syntax by default. | yes | Keep CLI wording separated from source-surface wording in docs and examples. |
| observation / output terms: `print`, `stdout`, `I/O`, `observe`, `emit`, `Hello World` | benchmark examples, roadmap docs, proposed proof-of-life wording | output and effect vocabulary | `rejected-as-canonical` for `print`; `implementation-detail` for `stdout`, `I/O`, and `emit`; `undecided` for `observe`; `undecided` for `Hello World` as a future proof-of-life concept | `print` should not become the public canonical surface. `stdout` and `I/O` describe implementation or capability boundaries. `observe` and `Hello World` are directionally important, but they are not executable claims here. | yes, with labels | Keep `print` out of canonical examples; keep `Hello World` only as a rejected legacy sketch or future directional discussion. |
| semantic-native direction: `entry`, `complete`, `require`, `observe`, `transition`, `evaluate` | roadmap docs and future-facing surface discussion | proposed canonical direction | `undecided` | These terms express the intended Semantic-native direction, but this document does not claim executable grammar support. | yes, as direction only | Keep them in directional docs until `#479` and the surface decision settle the canonical form. |

## 5. Bridge Fixture Rule

Current executable tests may continue using bridge syntax such as `fn main`,
`return`, and `assert` where required by the current frontend.

That syntax must stay labeled as bridge-only and must not be copied into public
canonical examples without an explicit bridge label.

## 6. Public Examples Rule

Any future public example must be classified before it is promoted to canonical
documentation.

Required public labels:

- Canonical Semantic example
- Bridge executable example
- Internal/tooling example
- Future directional sketch
- Rejected legacy sketch

## 7. Hello World Boundary

Hello World remains required.

But it must not be promoted using the legacy canonical form:

```sm
fn main() {
    print("Hello, World!");
    return;
}
```

That form may only appear as a rejected legacy sketch or as a bridge
comparison.

The directional sketch remains:

```sm
entry / state / require / observe / complete
```

This document does not claim that the directional sketch is executable.

## 8. Surface Boundary

The surface vocabulary decision must happen before:

- `#477` M-Hello implementation
- any public canonical examples
- Linguist readiness / `#356..#362`

## 9. Follow-up PRs

Refined follow-up list from `M-SURFACE-A`:

- `M-SURFACE-C — docs(surface): mark current bridge fixtures explicitly`
- `M-SURFACE-D — docs(surface): prepare Hello World observation vocabulary decision`
- `M-SURFACE-E — docs(surface): align README/examples after vocabulary decision`
- `M-SURFACE-F — docs(surface): hand off to Lexicon/Density #479`

This document does not implement those follow-ups.

## 10. Acceptance Checklist

- example/location classification created
- bridge fixtures classified
- public examples are not promoted to canonical by accident
- CLI verbs are separated from source vocabulary
- observation/output terms are classified
- Hello World boundary preserved
- future semantic-native terms recorded without executable claim
- `#477` remains blocked/dependent
- `#479` remains future work
- `#356` Linguist readiness remains deferred
- no code/test/fixture changes
- no grammar changes
- no UI / Workbench / I70
