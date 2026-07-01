# Semantic Style

Status: first-pass compact quad-style density doctrine

See also:

- [`semantic_command_lexicon.md`](semantic_command_lexicon.md)
- [`semantic_documentation_tone.md`](semantic_documentation_tone.md)
- [`../examples/semantic_language_experience_examples.md`](../examples/semantic_language_experience_examples.md)

## 1. Purpose

This document defines first-pass compact quad-style density rules for Semantic
source and documentation examples.

This is style / design guidance, not grammar.
It does not implement a formatter.
It does not rewrite fixtures.
It defines how future canonical examples should visually communicate Semantic's
model.

## 2. Non-Goals

This document does not:

- change grammar
- change parser or typechecker behavior
- implement a formatter
- rewrite source
- rewrite fixtures
- rewrite tests
- rewrite README content
- rewrite examples
- implement Hello World
- implement `print` / `observe`
- implement `entry` / `complete` / `require` / `observe`
- define a final no-brace or indentation grammar
- start Linguist readiness
- touch UI / Workbench / I70

## 3. Density Principles

- Compactness follows semantic relation, not minification.
- Adjacent phases should stay visually adjacent:
  - state
  - require
  - transition / evaluate
  - observe
  - complete
- Avoid excessive blank lines inside one semantic transition.
- Avoid C/Rust-like ceremonial rhythm when the construct is semantic-native.
- Preserve verifier clarity over visual cleverness.
- Keep quad-state expressions visually compact but readable.
- Group related quad conditions together.
- Separate independent semantic transitions with one blank line.
- Avoid burying capability / observation boundaries.
- Do not make density hide effects.

## 4. Legacy-Shaped Style Anti-Pattern

```semantic
fn main() {
    let boot: quad = T;

    assert(boot == T);

    print("Hello, World!");

    return;
}
```

Label: rejected legacy-shaped style / bridge comparison only.

Problems:

- airy imperative block rhythm
- legacy `fn main` / `assert` / `print` / `return`
- observation looks like generic stdout
- does not visually express Semantic phase relation

## 5. Semantic-Native Directional Shape

```semantic
entry HelloWorld {
    state boot: quad = T;
    require boot == T;
    observe "Hello, World!";
    complete T;
}
```

Label: future directional sketch, not executable claim.

This shape:

- keeps phases adjacent
- makes observation explicit and vocabulary-driven
- makes completion explicit
- shows the state / require / observe / complete relation

## 6. Denser Future Shape

```semantic
entry HelloWorld:
    boot:quad = T
    require boot==T
    observe "Hello, World!"
    complete T
```

Label: density experiment only, not grammar decision.

This shape:

- removes ceremony
- keeps semantic phases adjacent
- may be considered by future grammar / style work
- is not accepted as executable syntax

## 6.1 Dense Quad Selection Examples

Quad-heavy source should stay compact without becoming opaque.

See also:

- [`quad_language_design.md`](quad_language_design.md)
- [`../examples/quad_language_examples.md`](../examples/quad_language_examples.md)

```semantic
if q==T { ... } else if q==F { ... } else { ... }
```

```semantic
match q {
    N=>{ ... }
    F=>{ ... }
    T=>{ ... }
    S=>{ ... }
    _=>{ ... }
}
```

```semantic
let boot:quad = T;
if boot==T { observe "ready"; }
```

These are visual examples only.
They are not a grammar expansion and do not alter verifier or runtime meaning.

## 7. Density Decision Table

| concern | legacy-shaped risk | Semantic density direction | status |
|---|---|---|---|
| vertical whitespace | breaks a single transition into airy blocks | keep tightly related phases adjacent | planned |
| braces | can imply legacy imperative surface by default | use only when the current or future grammar needs them | undecided |
| semicolons | can add ceremony where the relation is already clear | reduce when they do not add verifier clarity | planned |
| phase adjacency | hides the relation between state / require / transition / observe / complete | keep the semantic chain visible | planned |
| quad expressions | become noisy when padded or scattered | keep them compact but readable | planned |
| observation boundary | can disappear into generic print-style output | make controlled observation explicit | planned |
| capability / effect visibility | can be buried behind generic I/O phrasing | keep the boundary visible | planned |
| verifier clarity | can be obscured by clever brevity | preserve explicitness first | planned |
| comments | can clutter or mislead phase boundaries | use sparingly and attach to the relevant semantic phase | undecided |
| examples in README | can accidentally canonize bridge rhythm | label future canonical examples explicitly | blocked by #479 |
| bridge fixtures | can leak legacy shape into public examples | keep bridge syntax untouched unless a migration PR exists | bridge-only |
| formatter / linter | can overfit to style before grammar stabilizes | defer implementation until the shape is accepted | blocked |
| Linguist highlighting | can obscure token categories if style is too loose | keep token categories explicit and stable | deferred |

## 8. Formatting Guidelines

- Use one semantic transition block per logical transition.
- Keep state / require / transition / observe / complete close unless
  separation clarifies meaning.
- Use blank lines between independent transitions, not inside tightly coupled
  phase chains.
- Avoid gratuitous Rust/C layout in future canonical examples.
- Never compress enough to obscure verifier-relevant boundaries.
- Use labels for bridge examples and future sketches.
- Keep bridge fixtures untouched unless a migration PR exists.
- Do not format non-executable sketches as if they are accepted grammar.

## 9. Bridge Fixture Rule

Bridge fixtures may keep current executable formatting.
This style guide applies to future canonical examples and future density
decisions, not to mass fixture rewrite.

## 10. Hello World Dependency

Hello World remains blocked until:

- lexicon / density decision is accepted
- observation vocabulary is accepted
- canonical shape is prepared in `LEXICON-F`
- implementation scope is opened separately

## 11. Open Questions

- final brace vs no-brace grammar
- indentation sensitivity
- semicolon policy
- whether formatter should enforce density later
- how to preserve source maps and diagnostics with denser forms
- how to expose quad syntax without visual noise
- whether `boot:quad = T` is acceptable density
- how comments attach to dense semantic phases
- whether density rules differ between public examples and internal bridge
  fixtures
- whether Linguist highlighting needs explicit token categories

## 12. Future Work

- `LEXICON-F` - prepare Hello World canonical shape decision
- later formatter / linter issue if grammar stabilizes
- later README/examples alignment after canonical shape decision
- later Linguist readiness after grammar and examples stabilize

## 13. Acceptance Checklist

- density principles recorded
- legacy-shaped anti-pattern recorded
- semantic-native directional shape recorded
- denser future shape recorded as non-executable
- density decision table added
- formatting guidelines added
- bridge fixture rule preserved
- Hello World dependency preserved
- no grammar changes
- no code/test/fixture changes
- no formatter implementation
- no `print` / `observe` implementation
- no README/examples rewrite
- no Linguist readiness
