# Semantic Command Lexicon

Status: initial lexicon skeleton

## 1. Purpose

This document is the initial command / primitive lexicon skeleton for Semantic.

It is a design dictionary skeleton.
It does not freeze final grammar.
It does not implement syntax.
It translates the `#478` audit outputs into a positive lexicon structure.

## 2. Non-Goals

This document does not:

- freeze the final lexicon
- change grammar
- change parser or typechecker behavior
- rename syntax
- rewrite README content
- rewrite examples
- rewrite fixtures
- rewrite tests
- implement Hello World
- implement `print` / `observe`
- add general I/O
- implement `#477`
- start Linguist readiness
- touch UI / Workbench / I70

## 3. Lexicon Entry Schema

Future command entries should use this schema:

- canonical term
- category
- meaning
- accepted forms
- forbidden / legacy synonyms
- bridge status
- verifier / runtime layer involved
- observation / effect impact
- example status
- current status
- open questions
- follow-up PR

Status vocabulary:

- stable
- planned
- bridge
- rejected
- implementation-detail
- undecided

## 4. Category Skeleton

### 4.1 Entry and Lifecycle

Reserved for entrypoint and program lifecycle vocabulary.

### 4.2 State Declaration

Reserved for state-bearing and precondition vocabulary.

### 4.3 Quad Values and Quad Relations

Reserved for quad-state values, relations, and directional value vocabulary.

### 4.4 Verification and Admission

Reserved for admit / verify / checker vocabulary.

### 4.5 Transition and Completion

Reserved for transition / execute / evaluate / complete vocabulary.

### 4.6 Observation

Reserved for controlled observation vocabulary and proof-of-life wording.

### 4.7 Controlled Effects

Reserved for capability-bound effect vocabulary.

### 4.8 Memory / State Access

Reserved for state access, read/write, and ownership vocabulary.

### 4.9 Capability Boundary

Reserved for capability / effect boundary wording.

### 4.10 Audit / Trace

Reserved for review, trace, and diagnostic vocabulary.

### 4.11 Module / Import Surface

Reserved for module, import, and helper-surface vocabulary.

### 4.12 Diagnostic-Only Terms

Reserved for terms that are only valid in diagnostics or bridge tests.

### 4.13 CLI / Tooling Terms

Reserved for command-line and workflow vocabulary.

## 5. First-Pass Command / Primitive Table

| concept | preferred direction | legacy / weak direction | proposed category | current status | notes |
|---|---|---|---|---|---|
| entrypoint | `entry` | `main` / `fn main` | Entry and lifecycle | planned | Bridge entrypoint spelling remains in current executable fixtures, but it is not canonical. |
| observable event | `observe` | `print` | Observation | planned | Observation must remain controlled and not collapse into generic stdout. |
| completion | `complete` | `return` | Transition and completion | planned | Current completion spelling is bridge syntax in fixtures, not a final decision. |
| requirement | `require` | `assert` | Verification and admission | planned | Requirement vocabulary needs to stay distinct from diagnostics-only assertions. |
| admission | `admit` / `verify` | unchecked `run` | Verification and admission | planned | Execution must remain verifier-gated. |
| output target | observation sink | `stdout` | Controlled effects / capability boundary | implementation-detail | Host output channel wording is not canonical source vocabulary. |
| external interaction | controlled effect | `I/O` | Controlled effects / capability boundary | implementation-detail | Keep effect admission explicit and bounded. |
| execution | `transition` / `evaluate` / `execute` | `run everywhere` | Transition and completion | undecided | The runtime meaning needs a clean surface split from CLI wording. |
| semantic state | `state` | variable-only model | State declaration | planned | State should carry meaning beyond a plain mutable variable story. |
| contradiction | `conflict` / `S` | boolean error | Quad values and quad relations | undecided | `S` remains a bridge/pattern term; future relation vocabulary still needs decision. |
| unknown | `unknown` / `N` | null-like value | Quad values and quad relations | undecided | Unknown-state vocabulary remains directional, not frozen. |
| text output proof | controlled observation proof | Hello World via `print` | Observation | planned | Proof-of-life must stay controlled, not generic output. |
| command-line check | `check` | source truth claim | CLI / tooling terms | implementation-detail | CLI verbs are tooling surface, not canonical source vocabulary. |
| SemCode execution | `run-smc` | generic `run` | CLI / tooling terms | implementation-detail | Persisted artifact execution must remain verifier-admitted. |

## 6. Bridge Compatibility Rule

Bridge forms may remain executable where the current frontend requires them,
but they are not canonical unless later accepted.

## 7. Hello World Dependency

Hello World remains required, but it is blocked until:

- lexicon / density decides observation vocabulary
- controlled observation shape is accepted
- implementation scope is opened separately under `#477` or a successor issue

## 8. Relationship to `#478`

`#478` produced the audit.
`#479` now defines the positive lexicon.

## 9. Relationship to Future PRs

Planned `#479` sequence:

- `LEXICON-A — docs(language): add Semantic command lexicon skeleton`
- `LEXICON-B — docs(language): define entry/lifecycle and completion vocabulary`
- `LEXICON-C — docs(language): define requirement/verification/admission vocabulary`
- `LEXICON-D — docs(language): define observation/effect vocabulary`
- `LEXICON-E — docs(language): define compact quad-style density rules`
- `LEXICON-F — docs(language): prepare Hello World canonical shape decision`
- `LEXICON-G — docs(language): close #479 lexicon/density phase`

## 10. Acceptance Checklist

- lexicon skeleton created
- entry schema defined
- category skeleton defined
- first-pass table added
- bridge compatibility rule recorded
- Hello World dependency preserved
- `#477` remains blocked/dependent
- `#356` Linguist readiness remains deferred
- no code/test/fixture changes
- no grammar changes
- no `print` / `observe` implementation
- no README/examples rewrite
- no UI / Workbench / I70
