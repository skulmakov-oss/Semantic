# Semantic Surface to Lexicon Handoff

Status: handoff draft from surface audit to lexicon/density

See also:

- [`semantic_surface_vocabulary_audit.md`](semantic_surface_vocabulary_audit.md)
- [`semantic_surface_examples_classification.md`](semantic_surface_examples_classification.md)
- [`semantic_surface_bridge_fixtures.md`](semantic_surface_bridge_fixtures.md)
- [`semantic_hello_observation_decision_prep.md`](semantic_hello_observation_decision_prep.md)

## 1. Purpose

This document hands off the completed pass-1 and pass-2 surface audit results
to `#479` Lexicon/Density.

`#478` identified legacy and bridge vocabulary.
`#479` must define the positive canonical command lexicon and density/style
rules.

This is a handoff, not implementation.

## 2. Non-Goals

This document does not:

- define the final lexicon
- rename syntax
- change grammar
- change parser or typechecker behavior
- rewrite README content
- rewrite examples
- rewrite fixtures
- rewrite tests
- implement Hello World
- implement `print` / `observe`
- start `#477`
- implement `#479`
- start Linguist readiness
- touch UI / Workbench / I70

## 3. Surface Audit Outputs

### A. Vocabulary audit matrix

- file: `docs/architecture/semantic_surface_vocabulary_audit.md`
- result: legacy and bridge terms classified

### B. Examples classification

- file: `docs/architecture/semantic_surface_examples_classification.md`
- result: bridge vs public/canonical examples separated

### C. Bridge fixture registry

- file: `docs/architecture/semantic_surface_bridge_fixtures.md`
- result: fixtures may remain bridge-only, not canonical

### D. Hello observation decision prep

- file: `docs/architecture/semantic_hello_observation_decision_prep.md`
- result: Hello World required, legacy form rejected, future direction non-executable

## 4. Decisions Already Established

| area | established by `#478` audit | handoff to `#479` |
|---|---|---|
| `fn main` | bridge-only executable entry spelling; not canonical | decide entry/lifecycle vocabulary and migration path |
| `main` | overloaded as filename, function name, and entry concept | decide whether any public-facing `main` wording remains at all |
| `print` | rejected-as-canonical for public Semantic surface | decide observation vocabulary and whether any bridge spelling remains |
| `return` | bridge-only completion syntax in current fixtures/docs | decide completion vocabulary and migration path |
| `assert` | bridge-only in fixtures/docs; not canonical | decide requirement / verification / admission distinctions |
| `stdout` | implementation-detail output channel wording | decide whether it remains implementation-only or gets a named boundary term |
| `I/O` | implementation-detail / capability-boundary wording | decide exact capability/effect boundary wording |
| `run` | CLI/tooling verb, not source-surface vocabulary by default | decide whether any source-facing execution term exists separately |
| `observe` | undecided semantic-native direction; not executable claim yet | decide controlled observation vocabulary and any bridge spelling |
| `entry` | future directional semantic-native term | decide canonical public use and exact lifecycle placement |
| `complete` | future directional semantic-native term | decide whether completion is explicit, implicit, or contract-only |
| `require` | future directional semantic-native term / contract-vocabulary overlap | decide distinction from `assert` and verification wording |
| `Hello World` | required proof-of-life, but not implemented | define canonical surface before `#477` planning |
| executable fixtures | bridge-only where they require current frontend syntax | decide how bridge fixtures are labeled in future public docs/examples |
| public examples | not yet canonical by accident; must be reviewed | decide canonical example policy and labeling rules |
| CLI/tooling verbs | separated from source vocabulary | decide whether any lexicon terms need CLI/example separation notes |
| future directional sketch | recorded as non-executable direction | decide final public shape and whether density rules alter the sketch |

## 5. `#479` Decision Inputs

`#479` should decide:

- canonical command terms
- accepted bridge aliases, if any
- rejected legacy synonyms
- category boundaries for:
  - entry / lifecycle
  - state declaration
  - quad values and relations
  - verification / admission
  - transition / completion
  - observation
  - controlled effects
  - memory / state access
  - capability boundary
  - audit / trace
  - module / import surface
  - diagnostic-only terms
- compact quad-style density principles
- whether the directional Hello World shape should remain verbose or become
  denser
- how to label examples while bridge and canonical forms coexist

## 6. What Remains Blocked

The following remain blocked until `#479` or a later accepted decision:

- `#477` M-Hello implementation
- public canonical Hello World example
- README/examples alignment
- Linguist readiness / `#356..#362`
- grammar changes for `entry` / `observe` / `complete`
- any `print` / `observe` implementation
- any general observation / I/O path

## 7. Proposed `#479` PR Sequence

Proposed follow-up sequence only:

- `LEXICON-A — docs(language): add Semantic command lexicon skeleton`
- `LEXICON-B — docs(language): define entry/lifecycle and completion vocabulary`
- `LEXICON-C — docs(language): define requirement/verification/admission vocabulary`
- `LEXICON-D — docs(language): define observation/effect vocabulary`
- `LEXICON-E — docs(language): define compact quad-style density rules`
- `LEXICON-F — docs(language): prepare Hello World canonical shape decision`
- `LEXICON-G — docs(language): close #479 lexicon/density phase`

This document does not implement those steps.

## 8. `#478` Closeout Note

After this handoff lands, `#478` may be eligible for closeout if the maintainer
accepts that:

- audit matrix exists
- examples classification exists
- bridge fixture registry exists
- Hello observation decision prep exists
- handoff to `#479` exists

This PR does not close `#478` automatically unless explicitly requested.

## 9. Acceptance Checklist

- surface audit outputs summarized
- handoff to `#479` recorded
- already-established decisions listed
- `#479` decision inputs listed
- blocked items listed
- proposed `#479` PR sequence listed
- `#477` remains blocked/dependent
- `#356` Linguist readiness remains deferred
- no code/test/fixture changes
- no grammar changes
- no Hello World implementation
- no `print` / `observe` implementation
- no UI / Workbench / I70
