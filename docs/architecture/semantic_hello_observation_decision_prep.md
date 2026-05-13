# Semantic Hello / Observation Decision Prep

Status: decision-prep draft

See also:

- [`semantic_surface_vocabulary_audit.md`](semantic_surface_vocabulary_audit.md)
- [`semantic_surface_examples_classification.md`](semantic_surface_examples_classification.md)
- [`semantic_surface_bridge_fixtures.md`](semantic_surface_bridge_fixtures.md)

## 1. Purpose

This document prepares the future Hello World / controlled observation
vocabulary decision.

Hello World is required.

It is a proof-of-life path for mature engineers and external readers.

It must demonstrate Semantic's model, not disguise Semantic as a C/Rust-like
language with `print`.

This document is decision-prep only, not implementation.

## 2. Non-Goals

This document does not:

- implement Hello World
- implement `print`
- implement `observe`
- change grammar
- change parser or typechecker behavior
- change the CLI
- add tests or fixtures
- implement runtime effects
- change capability or effect admission
- add general stdout
- add general I/O
- freeze the final vocabulary
- implement `#477`
- implement `#479`
- start Linguist readiness
- touch UI / Workbench / I70

## 3. Decision Constraints

- Hello World must be observable.
- Observation must be controlled, not general stdout.
- Future observation must remain capability/effect-bound where relevant.
- Legacy `print` must not become canonical.
- Existing bridge fixtures may remain, but they cannot define canonical Hello
  World.
- Directional syntax must not be presented as executable until implemented.
- Public examples must be labeled as one of:
  - Canonical Semantic example
  - Bridge executable example
  - Future directional sketch
  - Rejected legacy sketch

## 4. Rejected Legacy Form

The legacy form is explicitly rejected as canonical:

```sm
fn main() {
    print("Hello, World!");
    return;
}
```

Classification:

- rejected-as-canonical
- allowed only as anti-example / bridge comparison
- not allowed as canonical public Hello World

## 5. Candidate Semantic-Native Shape

Directional sketch only:

```sm
entry HelloWorld {
    state boot: quad = T;
    require boot == T;
    observe "Hello, World!";
    complete T;
}
```

This is:

- a future directional sketch
- not currently claimed executable
- not a grammar decision
- not an implementation plan by itself

Exact syntax is still subject to `#479` Lexicon/Density and later grammar
work.

## 6. Vocabulary Decision Table

| concept | rejected / bridge term | candidate Semantic-native direction | current status | notes |
|---|---|---|---|---|
| entrypoint | `fn main` | `entry` | undecided | Bridge entrypoint spelling remains in use for current executable fixtures, but it is not canonical. |
| output | `print` | `observe` | undecided | `print` must not become the public canonical story. Observation must remain controlled. |
| completion | `return` | `complete` | undecided | Legacy completion spelling is still a bridge form, not a canonical public claim. |
| assertion / requirement | `assert` | `require` | undecided | Current fixtures may use `assert`, but future public wording should prefer requirement vocabulary only after decision. |
| stdout | `stdout` | observation sink | implementation-detail | Host output channel wording is not canonical source vocabulary. |
| I/O | `I/O` | controlled observation / effect | implementation-detail | Capability/effect boundary wording, not user-facing source syntax. |
| executable proof | Hello World legacy form | controlled observation proof | undecided | Proof-of-life is required, but legacy proof form is rejected as canonical. |
| state precondition | none / ad-hoc bool | `state boot: quad = T` | undecided | This is a candidate structural shape, not an executable claim yet. |
| admission | implicit execution | verified / admitted transition | implementation-detail | Admission should remain separated from source syntax naming. |
| result | program return | completion state | undecided | Result/completion semantics need the later surface decision. |

## 7. Candidate Labels

Future Hello World docs should use one of these labels:

- Rejected legacy sketch
- Bridge executable sketch
- Future directional sketch
- Canonical Semantic Hello World

Only the last label may be used as canonical public onboarding once implemented
and accepted.

## 8. Open Design Questions

This document does not answer:

- exact grammar for `entry`
- whether `state` belongs in minimal Hello World or in a richer example
- whether `require` is the right surface term or a contract-only term
- whether `observe` requires capability declaration
- whether `complete` is explicit or implicit
- how the observation sink is represented
- whether Hello World should compile to SemCode before `observe` is
  implemented
- whether bridge executable example should remain available during transition

## 9. Dependency Order

Required order:

1. `M-SURFACE-D` decision prep
2. `M-SURFACE-F` handoff to `#479` Lexicon/Density
3. `#479` lexicon/density decision
4. `#477` M-Hello implementation planning
5. implementation only after accepted surface decision

Do not start those later steps here.

## 10. Acceptance Checklist

- Hello World requirement recorded
- legacy form rejected as canonical
- Semantic-native direction recorded as non-executable sketch
- vocabulary decision table added
- open design questions listed
- dependency order recorded
- `#477` remains blocked/dependent
- `#479` remains future decision track
- `#356` Linguist readiness remains deferred
- no code/test/fixture changes
- no grammar changes
- no `print` / `observe` implementation
- no UI / Workbench / I70
