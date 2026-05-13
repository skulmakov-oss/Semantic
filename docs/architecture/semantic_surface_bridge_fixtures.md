# Semantic Surface Bridge Fixtures

Status: bridge fixture registry draft

See also:

- [`semantic_surface_examples_classification.md`](semantic_surface_examples_classification.md)
- [`semantic_surface_vocabulary_audit.md`](semantic_surface_vocabulary_audit.md)

## 1. Purpose

This document records which current fixtures and executable examples are bridge
fixtures.

Bridge fixture means:

- required by the current executable frontend
- allowed for tests and pipeline proof
- not canonical public Semantic vocabulary
- not a final surface decision

## 2. Non-Goals

This document does not:

- rewrite fixtures
- rewrite tests
- rename syntax
- change grammar
- change parser or typechecker behavior
- implement Hello World
- implement `print` / `observe`
- add general I/O
- freeze the final vocabulary
- implement `#477`
- implement `#479`
- start Linguist readiness
- touch UI / Workbench / I70

## 3. Bridge Fixture Definition

- `executable bridge fixture`: current executable source that must keep bridge
  syntax to run under the current frontend
- `diagnostic bridge fixture`: current executable source that exists to prove a
  stable rejection path
- `lowering/emission bridge fixture`: current executable source that exists to
  prove deterministic lowering or SemCode emission behavior
- `CLI/pipeline bridge fixture`: current example or fixture that exercises the
  public CLI or pipeline path without defining canonical source vocabulary
- `rejected legacy sketch`: a legacy-looking form that must not be promoted to
  canonical public Semantic
- `future directional sketch`: a proposed Semantic-native direction that is not
  claimed executable here

## 4. Fixture Group Registry

| fixture / location group | bridge vocabulary used | current role | classification | allowed for now? | public-doc risk | follow-up action |
|---|---|---|---|---|---|---|
| PCC-1 control-flow fixtures (`tests/fixtures/pcc1_control_flow/`, `tests/pcc1_control_flow_gate.rs`, `tests/pcc1_control_flow_diagnostics.rs`, `tests/pcc1_control_flow_lowering_stability.rs`) | `fn main`, `return`, `assert` | control-flow gate, diagnostics, and lowering stability coverage | `bridge-only` | yes | These fixtures can be mistaken for canonical source wording if copied into public docs without labels. | Keep them labeled as current executable bridge syntax in future docs; do not rewrite now. |
| PCC-2 numeric fixtures (`tests/fixtures/pcc2_numeric/`, `tests/pcc2_numeric_core_gate.rs`, `tests/pcc2_numeric_diagnostics.rs`, `tests/pcc2_numeric_lowering_stability.rs`) | `fn main`, `return`, `assert` | numeric gate, diagnostics, and lowering stability coverage | `bridge-only` | yes | Same risk as PCC-1: executable bridge syntax can look canonical if presented without context. | Keep bridge labels explicit in future documentation. |
| PCC-3 text fixtures (`tests/fixtures/pcc3_text/`, `tests/pcc3_text_core_gate.rs`, `tests/pcc3_text_diagnostics.rs`, `tests/pcc3_text_lowering_stability.rs`) | `fn main`, `return`, `assert` | text gate, diagnostics, and lowering stability coverage | `bridge-only` | yes | Especially sensitive because `fn main` / `assert` appear in text examples and could be mistaken for canonical surface vocabulary. | Keep `fn main` / `assert` labeled as bridge syntax, not canonical vocabulary. |
| Snake benchmark / benchmark fixtures (`tests/fixtures/snake_benchmark/`, `tests/snake_benchmark*.rs` if present) | `fn main`, `return`, `assert`, `print` in some benchmark cases | benchmark and gap-surface coverage | `bridge-only` or `implementation-detail` depending on use | yes | Benchmark programs are often read as examples and can accidentally set public expectations. | Keep benchmark intent explicit; do not promote benchmark code to canonical docs by default. |
| `examples/` | mixed bridge syntax and Semantic-facing wording | example pack and testable sample programs | `undecided` / `bridge-only` until reviewed individually | partially | Examples are public-facing and therefore the highest accidental-canonization risk after README/docs. | Classify each example before promoting it to canonical documentation. |
| `README.md`, `docs/examples_index.md`, `docs/LANGUAGE.md`, `docs/spec/*.md` | public wording, CLI verbs, bridge samples, and conceptual surface language | public documentation | `public-risk` / `bridge-only` where explicit bridge examples appear | yes, with review | These files can easily blur canonical source vocabulary with bridge syntax or tool vocabulary. | Review and label public-facing examples before canonical promotion. |
| Rejected Hello World legacy sketch (`fn main() { print("Hello, World!"); return; }`) | `fn main`, `print`, `return` | legacy proof-of-life comparison | `rejected legacy sketch` / `rejected-as-canonical` | yes, only as anti-example | Highest canonization risk if copied verbatim into public examples. | Keep only as rejected legacy sketch, bridge comparison, or anti-example. |
| Future directional Hello World sketch (`entry / state / require / observe / complete`) | `entry`, `require`, `observe`, `complete` | future Semantic-native direction | `future directional sketch` | yes, as direction only | Directional wording can be mistaken for executable syntax if not clearly labeled. | Keep as non-executable direction until later grammar work exists. |

## 5. Bridge Labeling Policy

Future docs and tests should label bridge examples using one of these labels:

- Bridge executable fixture
- Bridge diagnostic fixture
- Bridge lowering fixture
- Internal/tooling example
- Future directional sketch
- Rejected legacy sketch
- Canonical Semantic example

This document defines the policy only.
It does not add those labels to every fixture in this PR.

## 6. Public Copy Rule

Fixture code using `fn main`, `return`, or `assert` may remain executable, but
it must not be copied into public canonical documentation unless it is
explicitly labeled as bridge executable syntax.

## 7. Hello World Boundary

Hello World remains required as proof of life.

But the legacy form:

```sm
fn main() {
    print("Hello, World!");
    return;
}
```

is rejected as canonical.

It may only appear as:

- rejected legacy sketch
- bridge comparison
- anti-example

The future directional form:

```sm
entry / state / require / observe / complete
```

remains non-executable unless later grammar work implements it.

## 8. Follow-up PRs

- `M-SURFACE-D — docs(surface): prepare Hello World observation vocabulary decision`
- `M-SURFACE-E — docs(surface): align README/examples after vocabulary decision`
- `M-SURFACE-F — docs(surface): hand off to Lexicon/Density #479`

Optional:

- `M-SURFACE-C2 — docs(surface): add bridge labels to selected docs examples`

This document does not implement those follow-ups.

## 9. Acceptance Checklist

- bridge fixture registry created
- PCC-1/PCC-2/PCC-3 fixture groups classified
- examples/docs public-risk groups identified
- bridge labeling policy recorded
- public copy rule recorded
- Hello World boundary preserved
- no fixture/test/code changes
- no grammar changes
- no Hello World implementation
- no `print` / `observe` implementation
- `#477` remains blocked/dependent
- `#479` remains future work
- `#356` Linguist readiness remains deferred
- no UI / Workbench / I70
