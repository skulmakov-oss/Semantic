# Semantic Canonical Source Style v0

Status: frozen v0
Primary owners: `smc-cli` (formatter), `docs/spec` (contract)
Tracking issue: #1538

## Purpose

This document freezes the canonical presentation style for Semantic `.sm`
source: how a supported program should look, not what it is allowed to mean.

It is a style and layout contract. It does not redefine:

- parser grammar (`syntax.md`)
- source-level execution semantics (`source_semantics.md`)
- the type contract (`types.md`)
- the Logos declarative surface grammar (`logos.md`)
- SemCode, verifier, VM, or PROMETHEUS ABI behavior

Every rule below classifies into exactly one of four categories, and no rule
implies a stronger guarantee than its category allows:

| Category | Meaning | Enforced by |
|---|---|---|
| **A. Required lexical/file invariant** | Machine-checked baseline; a canonical example that violates it is malformed | `smc fmt --check`, `tests/canonical_source_style.rs` |
| **B. Canonical presentation rule** | The preferred public shape; canonical examples and current-facing docs follow it | code review + `tests/canonical_source_style.rs` shape checks |
| **C. Permitted alternative author style** | Still accepted by the language and not rejected by any tool; simply not the preferred public presentation | none (author discretion) |
| **D. Future formatter behavior** | Documented intent; **not** currently applied automatically by `smc fmt` | none yet — see [Section D](#d-formatter-contract) |

A recommended layout is never a parser or verifier rule unless `syntax.md` or
`source_semantics.md` already says so independently.

## Compactness Is Not Minification

The target shape is:

```text
compact local logic
+ visible architectural structure
+ native Semantic vocabulary
+ low syntactic ceremony
+ deterministic reviewability
!=
minification
```

Horizontal space is spent on one coherent, complete causal action. Vertical
space is spent on boundaries between architectural phases (data/contracts,
domain types, domain transformations, validation, orchestration), not on every
brace or trivial statement. A line that mixes unrelated responsibilities is
never canonical, no matter how short it is.

## Mandatory Honesty Boundary: Two Source Surfaces

Semantic currently exposes two distinct, non-interchangeable source surfaces.
This document presents canonical style **separately for each** and never
implies they combine into one executable program today.

### Rust-like executable surface

Parsed by `sm-front`'s rustlike parser, type-checked by `sm-sema`, lowered by
`sm-ir`, emitted by `sm-emit`, admitted by `sm-verify`, executed by `sm-vm`.
Qualified end-to-end through `smc check`, `smc compile`, `smc verify`, and
`smc run` (see `tests/canonical_examples.rs`).

Currently includes (per `syntax.md` / `source_semantics.md`): `record`;
compile-time-only `schema` (including `config schema` / `api schema` /
`wire schema` and `version(N)`); `fn` (block- and expression-bodied);
`requires` / `ensures` / `invariant`; `let` / `const` / destructuring /
`let-else`; `quad` and its literals `N`, `F`, `T`, `S`; `if` / `else if` /
`else` (condition must be `bool` — `quad` is never an implicit condition);
`match` (over `quad`, nominal enums, `Option`, `Result`); `for` / `while` /
`loop` / `guard`; `assert`; `Result` / `Option`; `Sequence(T)`; imports;
`fn main()`.

### Logos declarative surface

Parsed by `sm-front`'s Logos parser (`parse_logos_program_with_profile`).
`System` / `Entity` / `Law` / `When` describe systems, entities, and priority
laws. **This surface does not compile, verify, or run through the Rust-like
SemCode/VM path.** Its current qualification path is parse-level
(`smc dump-ast`) and IR-lowering-level for laws only
(`smc dump-ir --profile logos`, which lowers `Law`/`When` bodies to
`LogosIrLaw` — condition/effect text fragments, not executable SemCode).
`smc check` / `compile` / `verify` / `run` do not accept Logos source; passing
Logos source to them produces a Rust-like parser diagnostic, not a Logos
result. **A program is never presented as one canonical example unless the
exact commands used to qualify it are the ones documented here for its
surface.**

These two surfaces are never merged into a single "canonical executable
program" claim. A document or example that shows both `schema`/`fn main()`
Rust-like forms and `System`/`Entity`/`Law` Logos forms together must label
the Logos portion as its own separately-qualified profile example, not as
part of the same executable program.

## A. Required Lexical/File Invariants

These apply to every canonical `.sm` file and are enforced by `smc fmt --check`
and `tests/canonical_source_style.rs`.

1. Source is UTF-8.
2. Line endings are LF (`\n`) only; no CR.
3. No trailing spaces or tabs at the end of any line.
4. No tab characters anywhere in a canonical file.
5. Exactly one trailing newline; no trailing blank lines.
6. No formatting-dependent semantic claim: layout never changes what a program
   means. (`quad` control flow, in particular, is governed entirely by
   `source_semantics.md`, never by indentation or spacing.)

Rules 1–3 and 5 are already what `smc fmt` enforces today: CRLF/CR
normalization, trailing-whitespace trimming, and final-newline normalization
(see [Section D](#d-formatter-contract)). Rule 4 (no tabs) and the "exactly
one final newline" part of rule 5 are also asserted directly against
canonical examples by `tests/canonical_source_style.rs`.

**Current tooling does not structurally validate indentation depth or
nesting.** The absence of tabs (rule 4) means a canonical file cannot mix tabs
and spaces, but it does not by itself prove any particular indentation width
is followed — that is a Section B canonical presentation rule (B.2), checked
only by code review today, not by `smc fmt` or by an automated depth check.

## B. Canonical Presentation Rules

### B.1 Top-level order (Rust-like surface)

Recommended, not a parser restriction:

1. imports
2. `schema` / compile-time contract declarations
3. `record` / domain type declarations
4. module-level `const` where supported
5. pure domain transformation functions
6. stateful or aggregation functions
7. validation functions
8. `fn main()`

`main` is always last. Declarations that later functions depend on come
before their use where practical; this list is a default reading order, not a
dependency-resolution rule enforced by the compiler.

### B.2 Indentation

- 4 spaces per nesting level. This is the frozen canonical presentation,
  enforced by code review — **no tool currently validates nesting depth**.
  The Section A tab prohibition is a separate, machine-checked invariant; it
  rules out mixed tab/space indentation but does not by itself prove 4-space
  depth is followed.
- Continuation lines (wrapped call arguments, wrapped parameter lists) add one
  further 4-space level relative to the line being continued.
- Braces open on the same line as the construct they belong to (`fn f() {`,
  `if cond {`, `match x {`) and close aligned with that construct's starting
  column.

### B.3 Line width

- **100 columns** is the canonical target (matches the workspace's default
  `rustfmt` width; no `rustfmt.toml` overrides it at the repository root).
- **120 columns** is the review ceiling — a canonical example must not exceed
  it.
- Narrow exceptions are permitted for long string literals, diagnostic error
  codes/identifiers, generated values, and URLs that cannot be wrapped without
  changing their meaning.
- `smc fmt` does not perform destructive automatic line-wrapping to satisfy
  this target (see [Section D](#d-formatter-contract)); authors wrap manually.

### B.4 Blank lines

- One blank line between top-level declarations.
- One blank line between distinct semantic phases inside a function (for
  example: between guard checks and the main computation, or between
  computation and the final `return`).
- No blank line between tightly coupled guard clauses that check related
  preconditions in sequence.
- No decorative or repeated blank lines (never more than one consecutive blank
  line).

### B.5 Compact guard returns

Canonical for one simple condition and one immediate `return`:

```sm
if slot == 0 { return N; }
if slot == 1 { return F; }
```

Expand vertically once the condition or the returned expression stops being
simple:

```sm
if ctx.override_state == T && ctx.ready {
    return Result::Ok(T);
}
```

An unrelated second effect never shares the line with a compact guard:

```sm
// not canonical: two responsibilities on one line
if slot == 0 { n_count += 1; audit(slot); }
```

### B.6 Match arms

```sm
match state {
    N => { 0 }
    F => { 1 }
    T => { 2 }
    S => { 3 }
}
```

- A single-expression or single-statement arm stays on one line as
  `Pattern => { expr }`.
- An arm with more than one statement, or with nested control flow, expands
  vertically:

```sm
match state {
    T => {
        let boosted: i32 = priority * 2;
        boosted
    }
    _ => { 0 }
}
```

- Guarded arms (`Pattern if cond => { ... }`) follow the same one-line/expand
  rule based on the arm body, not the guard.
- `quad` matches always keep the required `_` default arm (`source_semantics.md`);
  it is formatted like any other arm.

### B.7 Expression-bodied functions

```sm
fn dispatch_code(state: quad) -> i32 = match state {
    N => { 0 }
    F => { 1 }
    T => { 2 }
    _ => { 3 }
};
```

Canonical for small, pure, single-expression transformations. Block-bodied
functions are not mechanically converted to expression-bodied form — the
choice is the author's, based on whether the whole function is genuinely one
expression.

### B.8 Multiple statements on one physical line

- One ordinary statement per line.
- The one documented exception is the compact guard-return form in B.5.
- Multiple unrelated declarations packed onto one line are not canonical, even
  though the parser accepts semicolon-separated statements.
- Multiple `assert(...)` calls on one line are syntactically permitted but not
  canonical — see [Section C](#c-permitted-alternative-author-style).

### B.9 Function parameters

- Keep the full signature (name, parameters, return type, contract clauses) on
  one line while it fits the B.3 line-width target.
- When it does not fit, wrap one parameter per continuation line, each at one
  additional indent level, with the closing `)` and return type on their own
  line:

```sm
fn validate_distribution(
    n_count: i32,
    f_count: i32,
    t_count: i32,
    s_count: i32,
    code_sum: i32,
) -> bool {
    ...
}
```

- `requires(...)`, `ensures(...)`, and `invariant(...)` clauses each take
  their own line, after the parameter list and before the opening `{`, in the
  order the language accepts them (`requires`, then `ensures`, then
  `invariant`).

### B.10 `main` orchestration

`main` constructs or selects inputs, calls domain functions, coordinates the
computation, invokes final validation, and returns. A domain decision that
needs a named condition or a multi-step transformation belongs in a named
function, not inlined into `main`.

### B.11 Comments

- A comment explains a boundary, an invariant, a rationale, or a non-obvious
  domain meaning — never syntax that names and structure already communicate.
- No decorative banner comments (`// ============`) inside canonical examples.
- Canonical examples stay self-documenting primarily through naming and
  structure; comments are the exception, not the default.

### B.12 Logos declarative surface presentation

Logos indentation is significant and is not reformatted using Rust-like
brace/indent assumptions:

```sm
System QuadCycle(sample_count = 48, state_period = 4):

Entity Sensor:
    state val: quad
    prop threshold: f64

Law "CheckSignal" [priority 10]:
    When Sensor.val == T -> Log.emit("Signal OK")
    When Sensor.val == S -> System.recovery()
```

- One blank line between `System`, each `Entity`, and each `Law` block.
- `Entity` and `Law` bodies indent their fields/`When` clauses by 4 spaces,
  matching the Rust-like indentation step even though indentation is
  semantically significant here (unlike the Rust-like surface).
- Multiple `When` clauses inside one `Law` are one per line, ordered as
  authored; the compiler orders `Law` declarations themselves by descending
  `priority`, not by source order (`logos.md`).
- `System` parameters use `name = value` (the current parser's accepted form —
  `logos.md`'s illustrative `param: type` spelling is not what
  `parse_logos_system` accepts; this document reflects the implemented
  parser).

## C. Permitted Alternative Author Style

These remain valid, accepted source; they are not rejected by the parser or
by `smc fmt --check`, but they are not the preferred public presentation and
should not appear in canonical examples:

- Expanding a simple guard (B.5) into a vertical `if { ... }` block even when
  the compact one-line form would fit.
- Multiple `assert(...)` statements chained on one physical line.
- Fully vertical `match` arms even when every arm is a single expression.
- Block-bodied functions for logic that could be expressed with the B.7
  expression-bodied sugar.
- Wrapping a signature that would still fit on one line under the B.3 target.
- A different, but still internally consistent and tab-free, indentation
  width chosen by an author for a non-canonical, non-repository-owned
  program. Canonical, repository-owned examples still follow the 4-space
  presentation rule in B.2, but nothing currently checks that automatically.

## D. Formatter Contract

### What `smc fmt` currently enforces

`smc fmt` (`crates/smc-cli/src/formatter.rs`) is intentionally conservative.
Given any `.sm` file under the target path, it currently and only:

1. normalizes `\r\n` and `\r` to `\n`;
2. trims trailing spaces/tabs from every line;
3. removes trailing blank lines and writes exactly one final `\n`
   (or an empty file for empty input);
4. walks directories in sorted order, skipping `.git`, `target`,
   `node_modules`, `dist`, and `.semantic-cache`.

`smc fmt --check` reports files that would change without writing them;
`smc fmt <path>` writes the normalized bytes. Both modes are deterministic and
idempotent: running either twice in a row produces no further changes on the
second run.

`smc fmt` does **not** currently:

- re-indent code;
- wrap or unwrap lines to a width target;
- rewrite `if { return ...; }` in either direction;
- expand or collapse `match` arms;
- convert between block-bodied and expression-bodied functions;
- combine or split statements;
- reorder declarations;
- touch string or comment contents beyond trailing-whitespace trimming at
  end-of-line;
- distinguish Rust-like from Logos indentation — it never rewrites indentation
  at all, so Logos's significant indentation is preserved by construction.

Every Section B rule beyond the four points above is documentation guidance,
enforced by code review and by the shape assertions in
`tests/canonical_source_style.rs`, not by automatic rewriting.

### Future formatter behavior

If a future change teaches `smc fmt` to enforce a Section B rule
automatically (for example, canonical match-arm collapsing), that change
must, in the same series:

- move the rule's row in the category table from **B** to a formatter-owned
  behavior documented in this section;
- add positive tests (the rule is applied), negative preservation tests (the
  rule does not fire when it should not), idempotence tests, profile-specific
  tests (Logos indentation is still untouched unless explicitly in scope), and
  comment/string preservation tests;
- keep the change narrowly scoped — this document does not authorize a full
  AST-based formatter as an incidental side effect of any single rule.

## Canonical Examples

Three examples demonstrate this contract most directly (the full canonical
pack's authoritative inventory — every example, profile, qualification
level, expected result, and migration status — lives in
[`examples/canonical/README.md`](../../examples/canonical/README.md#canonical-examples--authoritative-inventory)):

- `examples/canonical/match_control_flow/` — compact quad decision program
  (Rust-like, executable, `check`/`compile`/`verify`/`run`-qualified).
- `examples/canonical/rule_state_decision/` — structured practical program
  with `record`, domain transformations, validation, and orchestration
  (Rust-like, executable, `check`/`compile`/`verify`/`run`-qualified).
- `examples/canonical/quad_cycle_logos/` — Logos profile example
  (`System`/`Entity`/`Law`), parse- and lowering-qualified only via
  `dump-ast` / `dump-ir --profile logos`, explicitly not executable through
  `check`/`compile`/`verify`/`run`.

## Contract Rule

Any change that widens what `smc fmt` rewrites automatically, or that changes
which category (A/B/C/D) a rule belongs to, must update this document in the
same change series.
