# Semantic Surface Vocabulary Audit

Status: pass-1 audit matrix
Track: `#478` M-Surface-Audit
Layer: architecture / surface vocabulary
Scope: documentation only
Implementation: out of scope
Owner: language maturity stream

Related:

- `docs/roadmap/language_maturity/pcc3_text_closeout.md`
- `docs/roadmap/language_maturity/pcc3_text_surface_boundary_reset.md`
- `docs/roadmap/language_maturity/pcc3_text_ctf_impact_check.md`
- `docs/roadmap/language_maturity/pcc3_text_7hell_mapping.md`
- `docs/examples_index.md`
- `docs/LANGUAGE.md`
- `docs/spec/syntax.md`
- `docs/spec/source_semantics.md`
- `docs/spec/cli.md`
- `tests/fixtures/`

## 1. Purpose

This document audits current and proposed legacy / bridge vocabulary against
Semantic's intended model:

```text
meaning -> verification -> deterministic transition -> controlled observation/effect
```

The audit is pass 1. It classifies vocabulary without renaming anything yet.

## 2. Non-goals

This document does not:

- rename syntax;
- change grammar;
- change parser or typechecker behavior;
- rewrite fixtures or tests;
- implement Hello World;
- implement `print` / `observe`;
- add general I/O;
- make a final vocabulary decision;
- start Linguist readiness;
- touch UI / Workbench / I70.

Rule:

```text
audit only, no rename
```

## 3. Classification model

`#478` audit classification model:

1. Keep as implementation detail
2. Keep as compatibility bridge only
3. Rename in public examples/docs
4. Replace with Semantic-native construct
5. Remove from canonical surface

Status vocabulary for this pass:

- canonical
- bridge-only
- implementation-detail
- deprecated-candidate
- rejected-as-canonical
- undecided

This pass does not make final decisions where the audit is still incomplete.

## 4. Audit matrix

| current term | current / likely location | layer | problem | proposed Semantic-native direction | proposed status | migration impact | required follow-up PR |
|---|---|---|---|---|---|---|---|
| `fn main` | `tests/fixtures/`, `docs/spec/source_semantics.md`, `docs/examples_index.md`, proposed Hello World docs | executable bridge surface | anchors the legacy entrypoint spelling in tests/examples and can be mistaken for canonical source vocabulary. | `entry` | bridge-only | broad example/doc updates; keep bridge syntax only where current frontend still requires it | `M-SURFACE-B — docs(surface): classify canonical vs bridge examples` |
| `main` | `docs/spec/source_semantics.md`, `docs/spec/cli.md`, `docs/examples_index.md`, examples | executable entry / file naming | overloaded as filename, function name, and entry concept; easy to read as canonical rather than bridge. | `entry` | bridge-only | broad examples/doc updates and possible file-name conventions review | `M-SURFACE-B — docs(surface): classify canonical vs bridge examples` |
| `print` | `tests/fixtures/snake_benchmark/positive_print.sm`, `docs/roadmap/language_maturity/pcc3_text_*`, proposed Hello World docs | observation/output bridge | looks like direct stdout, which can collapse controlled observation into a generic output story. | `observe` | rejected-as-canonical | examples/docs must be reclassified; future observation wording must be explicit | `M-SURFACE-D — docs(surface): prepare Hello World observation vocabulary decision` |
| `return` | `tests/fixtures/`, `docs/spec/source_semantics.md`, `docs/spec/syntax.md`, proposed Hello World docs | control-flow / bridge syntax | currently does real control-flow work in bridge syntax and can be misread as canonical surface vocabulary. | `complete` | bridge-only | examples/docs need canonical-vs-bridge labels before any surface rename | `M-SURFACE-C — docs(surface): mark current bridge fixtures explicitly` |
| `assert` | `tests/fixtures/`, `docs/spec/diagnostics.md`, `docs/roadmap/language_maturity/*` | test / diagnostics bridge | used heavily in fixtures and docs, but can imply a canonical assertion surface before the vocabulary decision is made. | `require` | bridge-only | fixture/doc examples need explicit bridge labeling; future semantic-native requirement syntax remains pending | `M-SURFACE-C — docs(surface): mark current bridge fixtures explicitly` |
| `run` | `docs/spec/cli.md`, `docs/examples_index.md`, `tests/fixtures/`, `smc-cli` docs | CLI / execution verb | overloaded between CLI command and execution meaning; not a clean canonical source-surface term. | `execute` / `transition` / `evaluate` | implementation-detail | CLI wording and examples need careful separation from language vocabulary | `M-SURFACE-B — docs(surface): classify canonical vs bridge examples` |
| `stdout` | `tests/fixtures/snake_benchmark/README.md`, proposed observation docs, runtime/effect docs | output channel | suggests generic process output, which is too broad for a controlled observation boundary. | observation sink | implementation-detail | requires explicit output-boundary wording; should not become canonical surface vocabulary | `M-SURFACE-D — docs(surface): prepare Hello World observation vocabulary decision` |
| `I/O` | `docs/spec/abi.md`, `docs/spec/audit.md`, `docs/roadmap/language_maturity/core_trust_freeze/*` | host boundary | too broad for source surface; can hide effect admission and host-policy distinctions. | controlled observation/effect | implementation-detail | must stay separated from generic language output and host-effect policy wording | `M-SURFACE-D — docs(surface): prepare Hello World observation vocabulary decision` |
| `effect` | `docs/spec/abi.md`, `docs/spec/audit.md`, `docs/spec/runtime.md`, `docs/roadmap/language_maturity/*` | runtime / host boundary | broad umbrella term that can blur deterministic computation versus effectful host interaction. | controlled observation/effect | undecided | needs careful boundary language in specs and examples before any canonical surface claim | `M-SURFACE-A` / `M-SURFACE-D` |
| `emit` | `docs/spec/semcode.md`, `sm-ir`, `sm-emit`, `docs/roadmap/language_maturity/semcode_version_discipline.md` | lowering / artifact pipeline | implementation word for producing SemCode, not a candidate user-facing surface term. | lower / produce artifact | implementation-detail | no surface rename needed; keep ownership wording stable in docs | `M-SURFACE-A` |
| `observe` | `pcc3_text_surface_boundary_reset.md`, `pcc3_text_closeout.md`, `pcc3_text_ctf_impact_check.md`, proposed Hello World docs | controlled observation | required as future direction, but still not implemented; can’t be canonicalized yet. | controlled observation | undecided | needs explicit no-claim wording until a dedicated observation scope is approved | `M-SURFACE-D — docs(surface): prepare Hello World observation vocabulary decision` |
| `entry` | `docs/spec/source_semantics.md`, `docs/spec/cli.md`, `docs/roadmap/language_maturity/executable_module_entry_scope.md` | source / execution entry | promising future native term, but not yet part of a committed canonical Hello World surface. | `entry` | undecided | needs surface decision before canonical examples or executable grammar claims | `M-SURFACE-D — docs(surface): prepare Hello World observation vocabulary decision` |
| `complete` | `pcc3_text_closeout.md`, `practical_core_completion_v0_3.md`, closeout docs | phase / future surface term | useful as a future semantic-native completion token, but not yet a committed executable surface form. | `complete` | undecided | needs explicit scope decision before appearing as canonical surface syntax | `M-SURFACE-D — docs(surface): prepare Hello World observation vocabulary decision` |
| `require` | `docs/spec/source_semantics.md`, `docs/spec/syntax.md`, `docs/spec/diagnostics.md` | contract / future surface term | currently used as `requires(...)` in contracts, but exact `require` surface vocabulary is still a future decision. | `require` | undecided | needs vocabulary decision so contracts and surface syntax do not get conflated | `M-SURFACE-D — docs(surface): prepare Hello World observation vocabulary decision` |
| `verify` | `docs/spec/verifier.md`, `docs/spec/cli.md`, `docs/spec/semcode.md`, `smc-cli` | admission / tool surface | essential admission verb, but implementation and CLI contract should not be mistaken for canonical language surface vocabulary. | verify / admit | implementation-detail | keep as tooling/admission term; avoid loading it into canonical language examples | `M-SURFACE-A` |
| `admit` | `docs/spec/verifier.md`, `docs/spec/cli.md`, `core_trust_freeze/*` | verifier / trust lane | belongs to admission policy language, not to canonical program syntax. | admit | implementation-detail | maintain as trust-lane vocabulary, not user-facing syntax | `M-SURFACE-A` |
| `execute` | `docs/spec/source_semantics.md`, `docs/spec/vm.md`, `docs/spec/runtime.md` | runtime meaning | general execution verb is too broad to be canonical surface syntax by default. | deterministic transition | implementation-detail | keep as runtime/model language, not direct source-syntax claim | `M-SURFACE-A` |
| `transition` | `docs/spec/state.md`, `docs/spec/runtime.md`, `docs/spec/audit.md` | state model | useful modeling word, but not yet a surfaced entrypoint for program syntax. | deterministic transition | implementation-detail | keep in model/docs; do not imply executable surface | `M-SURFACE-A` |
| `evaluate` | `docs/spec/runtime.md`, `docs/spec/source_semantics.md`, `docs/spec/state.md` | runtime / computation model | describes computation semantics, not a public canonical surface term. | deterministic evaluation / transition | implementation-detail | no direct syntax rename needed; keep in runtime docs | `M-SURFACE-A` |
| `Hello World` | `pcc3_text_surface_boundary_reset.md`, `pcc3_text_closeout.md`, proposed examples/docs | proof-of-life concept | required as a future proof-of-life story, but the legacy `fn main` / `print` / `return` form must not become canonical. | `entry` / `state` / `require` / `observe` / `complete` | undecided | highest surface sensitivity; needs dedicated vocabulary decision before canonical examples | `M-SURFACE-D — docs(surface): prepare Hello World observation vocabulary decision` |

## 5. Bridge vocabulary rule

Current executable tests may continue using bridge syntax such as `fn main`,
`return`, and `assert` where required by the current frontend.

Those forms must be labeled bridge-only until the canonical surface decision is
made.

## 6. Hello World rule

Hello World remains required as proof of life.

But the legacy form is rejected as canonical:

```text
fn main() {
    print("Hello, World!");
    return;
}
```

Canonical direction remains:

```text
entry / state / require / observe / complete
```

This document does not claim that directional syntax is executable.

## 7. Surface boundary

The surface vocabulary decision must happen before:

- `#477` M-Hello implementation
- any public canonical examples
- Linguist readiness / `#356..#362`

## 8. Follow-up PR list

- `M-SURFACE-B — docs(surface): classify canonical vs bridge examples`
- `M-SURFACE-C — docs(surface): mark current bridge fixtures explicitly`
- `M-SURFACE-D — docs(surface): prepare Hello World observation vocabulary decision`
- `M-SURFACE-E — docs(surface): align README/examples after vocabulary decision`
- `M-SURFACE-F — docs(surface): hand off to Lexicon/Density #479`

## 9. Acceptance checklist

- vocabulary audit matrix created
- legacy terms classified at least preliminarily
- bridge-only rule recorded
- Hello World legacy form rejected as canonical
- directional vocabulary recorded without executable claim
- `#477` remains blocked / dependent
- `#479` remains future lexicon/density work
- `#356` Linguist readiness remains deferred
- no code / test / fixture / grammar changes
- no UI / Workbench / I70
