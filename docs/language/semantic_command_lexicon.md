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

Candidate vocabulary:

- `require`
- `requirement`
- `verify`
- `verifier`
- `admit`
- `admission`
- `check`
- `assert`

Clarify:

- `require` is the preferred Semantic-native direction for source-level
  requirement / precondition wording.
- `assert` remains bridge-only where current frontend / tests require it.
- `assert` must not be treated as canonical public Semantic vocabulary.
- `verify` belongs primarily to verifier / admission / tooling vocabulary.
- `admit` belongs to verifier-first execution policy.
- `check` is CLI / tooling vocabulary, not automatically source syntax.
- exact source syntax remains undecided.

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

Candidate vocabulary:

- `observe`
- `observation`
- `observable event`
- `observation sink`
- `emission`
- `emit`
- `print`
- `stdout`

Clarify:

- `observe` is the preferred Semantic-native direction for controlled
  observation.
- `print` is rejected-as-canonical for public Semantic source vocabulary.
- `print` may remain only as bridge / legacy comparison if current fixtures
  require it.
- `stdout` is an implementation / host-channel term, not canonical source
  vocabulary.
- `emit` is ambiguous: implementation emission vs observation emission must be
  separated.
- `observe` is not executable unless grammar / runtime / effect path later
  implements it.

### 4.7 Controlled Effects

Reserved for capability-bound effect vocabulary.

Candidate vocabulary:

- `controlled effect`
- `effect`
- `effect boundary`
- `capability`
- `capability gate`
- `observation/effect admission`
- `I/O`

Clarify:

- `controlled effect` is preferred over generic `I/O`.
- generic `I/O` must not be canonical source vocabulary.
- effects must remain capability / effect-bound where relevant.
- no capability / effect admission change is made by this PR.

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

### 5.4 `require`

- canonical term: `require`
- category: Verification and admission
- meaning: declares a required condition or semantic precondition before
  transition / admission
- accepted forms: planned / directional only
- forbidden / legacy synonyms: `assert`
- bridge status: `assert` remains current executable bridge where needed
- verifier/runtime layer involved: frontend / typecheck / verifier boundary,
  subject to future design
- observation/effect impact: none by itself
- example status: future directional sketch only
- current status: planned / undecided
- open questions: source grammar, relation to contracts, whether failed
  requirement traps or rejects admission, quad condition policy
- follow-up PR: `LEXICON-F` or later grammar planning

### 5.5 `verify`

- canonical term: `verify`
- category: Verification and admission
- meaning: validates source / IR / SemCode according to a defined policy
- accepted forms: CLI / tooling / admission vocabulary
- forbidden / legacy synonyms: unchecked run
- bridge status: implementation / tooling term
- verifier/runtime layer involved: verifier-first admission
- observation/effect impact: none
- example status: tooling docs only
- current status: implementation-detail / stable if existing CLI uses it
- open questions: source-surface vs tool-surface separation

### 5.6 `admit`

- canonical term: `admit`
- category: Verification and admission
- meaning: records acceptance of a verified artifact or transition into
  execution
- accepted forms: admission-policy vocabulary
- forbidden / legacy synonyms: unchecked execute
- bridge status: not source syntax yet
- verifier/runtime layer involved: verifier / runtime admission boundary
- observation/effect impact: indirect; gates execution before effects
- current status: implementation-detail / planned

### 5.7 `assert`

- canonical term: none / bridge term
- category: Diagnostic-only terms or Verification and admission
- meaning: current bridge / test assertion form
- accepted forms: current executable bridge only
- forbidden / legacy synonyms: public canonical `assert`
- bridge status: bridge-only
- current status: bridge
- open questions: migration path to `require` or diagnostics-only status

### 5.8 `check`

- canonical term: `check`
- category: CLI / tooling terms
- meaning: source checking / diagnostics command in the public CLI surface
- accepted forms: CLI / tooling vocabulary
- forbidden / legacy synonyms: source syntax by default
- bridge status: tooling term only
- verifier/runtime layer involved: CLI orchestration / source admission
- observation/effect impact: none by itself
- current status: implementation-detail / stable if existing CLI uses it
- open questions: whether the term ever becomes source-surface vocabulary

### 5.9 `evaluate` / `execute`

- canonical term: `evaluate` / `execute`
- category: Transition and completion
- meaning: source-level or runtime computation wording that should not be conflated with CLI `run`
- accepted forms: model / documentation term
- forbidden / legacy synonyms: generic `run` as source wording
- bridge status: CLI/runtime terms may remain, but they are not canonical source-surface names
- current status: implementation-detail / undecided

### 5.10 `observe`

- canonical term: `observe`
- category: Observation
- meaning: declares a controlled observable event or observation request
- accepted forms: planned / directional only
- forbidden / legacy synonyms: `print`
- bridge status: `print` rejected as canonical; may appear only as rejected
  legacy sketch / bridge comparison
- verifier/runtime layer involved: future observation / effect boundary,
  subject to design
- observation/effect impact: yes; future controlled observation
- example status: future directional sketch only
- current status: planned / undecided
- open questions: syntax, capability requirement, observation sink
  representation, determinism / audit requirements, whether observation is an
  effect
- follow-up PR: `LEXICON-F` or later observation implementation planning

### 5.11 `observation sink`

- canonical term: observation sink
- category: Observation / Controlled effects
- meaning: target or channel for controlled observation
- accepted forms: model / documentation term
- forbidden / legacy synonyms: `stdout` as canonical source term
- bridge status: implementation-detail
- verifier/runtime layer involved: future runtime / effect boundary
- observation/effect impact: yes
- current status: implementation-detail / planned

### 5.12 `controlled effect`

- canonical term: controlled effect
- category: Controlled effects
- meaning: bounded interaction with external world or host boundary
- accepted forms: model / policy term
- forbidden / legacy synonyms: generic `I/O`
- bridge status: not source syntax yet
- verifier/runtime layer involved: capability / effect admission
- observation/effect impact: yes
- current status: implementation-detail / planned

### 5.13 `print`

- canonical term: none / rejected legacy term
- category: Diagnostic-only terms or Observation
- meaning: legacy output primitive
- accepted forms: rejected legacy sketch / bridge comparison only
- forbidden / legacy synonyms: canonical public `print`
- bridge status: rejected-as-canonical
- observation/effect impact: would imply uncontrolled output if canonized
- current status: rejected

### 5.14 `stdout`

- canonical term: `stdout`
- category: Observation / Controlled effects
- meaning: host / process channel wording
- accepted forms: implementation / host-channel term only
- forbidden / legacy synonyms: canonical source vocabulary
- bridge status: implementation-detail
- verifier/runtime layer involved: host boundary / runtime channel
- observation/effect impact: indirect
- current status: implementation-detail

### 5.15 `emit`

- canonical term: `emit`
- category: Compiler / artifact emission
- meaning: compiler or artifact emission where already used
- accepted forms: implementation / documentation term
- forbidden / legacy synonyms: controlled observation vocabulary by default
- bridge status: implementation-detail / undecided depending on context
- verifier/runtime layer involved: compiler / artifact pipeline
- observation/effect impact: no, unless later decided otherwise
- current status: implementation-detail / undecided

## 6. First-Pass Command / Primitive Table

| concept | preferred direction | legacy / weak direction | proposed category | current status | notes |
|---|---|---|---|---|---|
| entrypoint | `entry` | `main` / `fn main` | Entry and lifecycle | planned / refined by `LEXICON-B` | Bridge entrypoint spelling remains in current executable fixtures, but it is not canonical. |
| observable event | `observe` | `print` | Observation | planned | Observation must remain controlled and not collapse into generic stdout. |
| completion | `complete` | `return` | Transition and completion | planned / refined by `LEXICON-B` | Current completion spelling is bridge syntax in fixtures, not a final decision. |
| requirement | `require` | `assert` | Verification and admission | planned / refined by `LEXICON-C` | Requirement vocabulary needs to stay distinct from diagnostics-only assertions. |
| admission | `admit` / `verify` | unchecked `run` | Verification and admission | planned / refined by `LEXICON-C` | Execution must remain verifier-gated. |
| output target | observation sink | `stdout` | Controlled effects / capability boundary | implementation-detail | Host output channel wording is not canonical source vocabulary. |
| external interaction | controlled effect | `I/O` | Controlled effects / capability boundary | implementation-detail | Keep effect admission explicit and bounded. |
| execution | `transition` / `evaluate` / `execute` | `run everywhere` | Transition and completion | undecided | The runtime meaning needs a clean surface split from CLI wording. |
| semantic state | `state` | variable-only model | State declaration | planned | State should carry meaning beyond a plain mutable variable story. |
| contradiction | `conflict` / `S` | boolean error | Quad values and quad relations | undecided | `S` remains a bridge/pattern term; future relation vocabulary still needs decision. |
| unknown | `unknown` / `N` | null-like value | Quad values and quad relations | undecided | Unknown-state vocabulary remains directional, not frozen. |
| text output proof | controlled observation proof | Hello World via `print` | Observation | planned / refined by `LEXICON-D` | Proof-of-life must stay controlled, not generic output. |
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
| `assert` | `require` | bridge | yes, where current frontend requires it | Assertion spelling remains bridge-only in tests / fixtures. |
| `check` | tooling check | CLI / tooling | yes | CLI terms stay distinct from source syntax. |
| `verify` | verifier / admission term | tooling / verifier | yes | Verifier-first meaning is primary; source syntax is undecided. |
| `admit` | admission-policy term | planned | yes as model wording | Admission wording belongs to verifier-first execution policy. |
| unchecked `run` | `verify` / `admit` / `transition` sequence | bridge / weak | no as canonical source wording | Keep the verifier-first pipeline explicit. |
| failed source requirement | reject / trap / diagnostic | undecided | yes as open question | Failure mode remains a policy decision. |
| quad condition in requirement | quad-policy decision | undecided | yes as open question | Whether requirements can use quad relations needs later design. |
| `print` | `observe` | bridge / legacy | yes, only as rejected legacy comparison | Public canonical `print` must not be restored. |
| `stdout` | observation sink | implementation-detail | yes | Keep host-channel wording separate from source vocabulary. |
| generic `I/O` | controlled effect | implementation-detail | yes as model wording only | Generic I/O is not canonical source vocabulary. |
| `emit` as compiler emission | implementation-detail | implementation-detail | yes | Keep compiler emission separate from controlled observation. |
| `emit` as output | undecided | undecided | yes as open question | Observation emission and artifact emission must remain distinct. |
| uncontrolled output | rejected | rejected | no as canonical source wording | Generic uncontrolled output remains out of bounds. |
| observation without capability | undecided / blocked | undecided | blocked | Capability requirement must be decided before implementation. |
| observation with capability / effect admission | planned | planned | not executable yet | Future effect admission path needs separate implementation. |
| Hello World via `print` | rejected-as-canonical | rejected | no | Legacy form is only a comparison sketch. |
| Hello World via controlled observation | planned / blocked | planned | blocked | Waits on observation vocabulary and canonical shape decisions. |

## 8. Labeled Sketches

### 8.1 Rejected legacy canonical sketch for entry / completion

```semantic
fn main() {
    return;
}
```

Label: rejected as canonical / bridge-only if executable.

### 8.2 Future directional sketch for entry / completion

```semantic
entry Example {
    complete T;
}
```

Label: future directional sketch, not executable claim.

### 8.3 Optional denser future sketch for entry / completion

```semantic
entry Example:
    complete T
```

Label: density experiment only, not grammar decision.

### 8.4 Rejected canonical requirement sketch

```semantic
assert(boot == T);
```

Label: bridge-only if executable; rejected as canonical public vocabulary.

### 8.5 Future directional requirement sketch

```semantic
require boot == T;
```

Label: future directional sketch, not executable claim.

### 8.6 Admission pipeline wording sketch

```text
check -> compile -> verify -> admit -> transition
```

Label: pipeline wording, not source syntax.

### 8.7 Rejected canonical observation sketch

```semantic
print("Hello, World!");
```

Label: rejected as canonical public vocabulary.

### 8.8 Future directional observation sketch

```semantic
observe "Hello, World!";
```

Label: future directional sketch, not executable claim.

### 8.9 Controlled observation policy sketch

```text
require -> verify -> admit -> observe -> audit
```

Label: policy wording, not source syntax.

### 8.10 Capability-bound effect sketch

```text
controlled effect = admitted capability + audited boundary
```

Label: model wording, not source syntax.

## 9. Open Questions

- exact grammar for `observe`
- whether `observe` is statement, event, transition, or effect request
- whether `observe` requires declared capability
- how observation sink is represented
- whether observation is deterministic output, audited effect, or both
- how observation interacts with verifier/admission
- whether observation can fail and how failure is represented
- whether Hello World can exist without general I/O
- whether `emit` should ever be user-facing
- how to keep compiler emission separate from observation emission
- exact grammar for `require`
- whether `require` supports bool only, quad relation, or both
- how failed requirement is represented: diagnostic, trap, rejected
  admission, or completion state
- distinction between `require`, `verify`, and `admit`
- whether `assert` remains only in tests / fixtures
- whether `check` remains strictly CLI
- how requirement semantics map to SemCode / verifier later
- relationship between source requirement and runtime traps

## 10. Bridge Compatibility Rule

Bridge forms may remain executable where the current frontend requires them,
but they are not canonical unless later accepted.

## 11. Dependency Notes

- `#477` remains blocked.
- Hello World remains blocked.
- No observation implementation starts here.
- Any future observation / effect implementation must be checked against CTF / capability policy.
- `LEXICON-F` will prepare Hello World canonical shape decision.

## 12. Hello World Dependency

Hello World remains required, but it is blocked until:

- lexicon / density decides observation vocabulary
- controlled observation shape is accepted
- implementation scope is opened separately under `#477` or a successor issue

## 13. Relationship to `#478`

`#478` produced the audit.
`#479` now defines the positive lexicon.

## 14. Relationship to Future PRs

Planned `#479` sequence:

- `LEXICON-A — docs(language): add Semantic command lexicon skeleton`
- `LEXICON-B — docs(language): define entry/lifecycle and completion vocabulary`
- `LEXICON-C — docs(language): define requirement/verification/admission vocabulary`
- `LEXICON-D — docs(language): define observation/effect vocabulary`
- `LEXICON-E — docs(language): define compact quad-style density rules`
- `LEXICON-F — docs(language): prepare Hello World canonical shape decision`
- `LEXICON-G — docs(language): close #479 lexicon/density phase`

## 15. Acceptance Checklist

- lexicon skeleton created
- entry schema defined
- category skeleton defined
- first-pass table added
- entry/lifecycle vocabulary section refined
- transition/completion vocabulary section refined
- verification/admission vocabulary section refined
- observation vocabulary section refined
- controlled effect vocabulary section refined
- `require` entry added
- `verify` entry added
- `admit` entry added
- `observe` entry added
- observation sink entry added
- controlled effect entry added
- `assert` bridge status clarified
- `check` tooling status clarified
- failed requirement / quad requirement questions listed
- `print` rejected-as-canonical clarified
- `stdout` implementation-detail clarified
- `I/O` generic term rejected as canonical source vocabulary
- examples labeled as non-executable sketches
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
