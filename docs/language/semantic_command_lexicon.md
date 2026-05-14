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

Candidate vocabulary:

- `entry`
- `entry contract`
- `lifecycle`
- `start`
- `admit`
- `evaluate`
- `transition`

Clarify:

- `entry` is the preferred Semantic-native direction for program/module entry.
- `fn main` remains bridge-only where the current frontend requires it.
- `main` must not be treated as canonical Semantic vocabulary.
- `entry` is not executable unless grammar later implements it.

### 4.2 State Declaration

Reserved for state-bearing and precondition vocabulary.

### 4.3 Quad Values and Quad Relations

Reserved for quad-state values, relations, and directional value vocabulary.

### 4.4 Verification and Admission

Reserved for admit / verify / checker vocabulary.

### 4.5 Transition and Completion

Reserved for transition / execute / evaluate / complete vocabulary.

Candidate vocabulary:

- `complete`
- `completion state`
- `transition result`
- `halt`
- `yield completion`
- `return`

Clarify:

- `complete` is the preferred Semantic-native direction for explicit completion.
- `return` remains bridge-only where the current frontend requires it.
- `return` must not be used as canonical public Semantic vocabulary.
- `complete` is not executable unless grammar later implements it.
- Whether completion is explicit or implicit remains an open design question.

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

## 5. Entry Records

### 5.1 `entry`

- canonical term: `entry`
- category: Entry and lifecycle
- meaning: declares the semantic entrypoint / admission boundary for a program or module
- accepted forms: planned / directional only
- forbidden / legacy synonyms: `fn main`, `main`
- bridge status: bridge syntax still required in current executable fixtures
- verifier/runtime layer involved: frontend admission / module execution boundary, subject to future design
- observation/effect impact: none by itself
- example status: future directional sketch only
- current status: planned / undecided
- open questions: exact grammar, module-vs-program entry, multiple entrypoints, entry contracts
- follow-up PR: `LEXICON-F` or later grammar planning

### 5.2 `complete`

- canonical term: `complete`
- category: Transition and completion
- meaning: declares successful completion / completion state of a semantic transition
- accepted forms: planned / directional only
- forbidden / legacy synonyms: `return`
- bridge status: `return` remains current executable bridge where needed
- verifier/runtime layer involved: VM / transition semantics, subject to future design
- observation/effect impact: none by itself
- example status: future directional sketch only
- current status: planned / undecided
- open questions: explicit vs implicit completion, completion values, quad completion state
- follow-up PR: `LEXICON-F` or later grammar planning

### 5.3 `transition`

- canonical term: `transition`
- category: Transition and completion
- meaning: describes deterministic semantic state movement
- accepted forms: model / documentation term
- forbidden / legacy synonyms: generic `run everywhere`
- bridge status: not source syntax yet
- current status: implementation-detail / undecided

### 5.4 `evaluate` / `execute`

- canonical term: `evaluate` / `execute`
- category: Transition and completion
- meaning: source-level or runtime computation wording that should not be conflated with CLI `run`
- accepted forms: model / documentation term
- forbidden / legacy synonyms: generic `run` as source wording
- bridge status: CLI/runtime terms may remain, but they are not canonical source-surface names
- current status: implementation-detail / undecided

## 6. First-Pass Command / Primitive Table

| concept | preferred direction | legacy / weak direction | proposed category | current status | notes |
|---|---|---|---|---|---|
| entrypoint | `entry` | `main` / `fn main` | Entry and lifecycle | planned / refined by `LEXICON-B` | Bridge entrypoint spelling remains in current executable fixtures, but it is not canonical. |
| observable event | `observe` | `print` | Observation | planned | Observation must remain controlled and not collapse into generic stdout. |
| completion | `complete` | `return` | Transition and completion | planned / refined by `LEXICON-B` | Current completion spelling is bridge syntax in fixtures, not a final decision. |
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

## 7. Decision Table

| legacy / bridge term | proposed direction | status | allowed now? | notes |
|---|---|---|---|---|
| `fn main` | `entry` | bridge | yes, where current frontend requires it | Bridge entry spelling remains executable-only. |
| `main` | `entry` | bridge / undecided | yes, where current frontend requires it | `main` must not be treated as canonical Semantic vocabulary. |
| `return` | `complete` | bridge | yes, where current frontend requires it | Completion syntax remains bridge-only in current fixtures. |
| generic `run` as source wording | `transition` / `evaluate` | undecided | yes as CLI/runtime wording only | Do not promote generic `run` into canonical source syntax. |
| implicit program exit | `completion state` | undecided | yes as model wording only | Needs explicit vs implicit completion decision. |
| module start | `entry contract` | planned | yes as directional terminology | Contract form must be settled separately from executable grammar. |

## 8. Labeled Sketches

### 8.1 Rejected legacy canonical sketch

```semantic
fn main() {
    return;
}
```

Label: rejected as canonical / bridge-only if executable.

### 8.2 Future directional sketch

```semantic
entry Example {
    complete T;
}
```

Label: future directional sketch, not executable claim.

### 8.3 Optional denser future sketch

```semantic
entry Example:
    complete T
```

Label: density experiment only, not grammar decision.

## 9. Bridge Compatibility Rule

Bridge forms may remain executable where the current frontend requires them,
but they are not canonical unless later accepted.

## 10. Hello World Dependency

Hello World remains required, but it is blocked until:

- lexicon / density decides observation vocabulary
- controlled observation shape is accepted
- implementation scope is opened separately under `#477` or a successor issue

## 11. Relationship to `#478`

`#478` produced the audit.
`#479` now defines the positive lexicon.

## 12. Relationship to Future PRs

Planned `#479` sequence:

- `LEXICON-A — docs(language): add Semantic command lexicon skeleton`
- `LEXICON-B — docs(language): define entry/lifecycle and completion vocabulary`
- `LEXICON-C — docs(language): define requirement/verification/admission vocabulary`
- `LEXICON-D — docs(language): define observation/effect vocabulary`
- `LEXICON-E — docs(language): define compact quad-style density rules`
- `LEXICON-F — docs(language): prepare Hello World canonical shape decision`
- `LEXICON-G — docs(language): close #479 lexicon/density phase`

## 13. Acceptance Checklist

- lexicon skeleton created
- entry schema defined
- category skeleton defined
- first-pass table added
- entry/lifecycle vocabulary section refined
- transition/completion vocabulary section refined
- `entry` documented as planned/directional, not executable
- `complete` documented as planned/directional, not executable
- `fn main` remains bridge-only
- `return` remains bridge-only
- bridge compatibility rule recorded
- Hello World dependency preserved
- `#477` remains blocked/dependent
- `#356` Linguist readiness remains deferred
- no code/test/fixture changes
- no grammar changes
- no `print` / `observe` implementation
- no README/examples rewrite
- no UI / Workbench / I70
