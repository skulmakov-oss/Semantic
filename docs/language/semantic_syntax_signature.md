# Semantic Syntax Signature

Status: public identity note for Linguist readiness

## Purpose

This note describes the visible syntax profile that makes `.sm` files
recognizable as Semantic source.

It does not define new language behavior. It only summarizes the current public
surface so external tools can classify Semantic source consistently.

## Canonical Identity Markers

Semantic source files are recognized by a combination of these markers:

- `fn` function declarations
- `record` declarations
- `schema` declarations
- `Import` module/import statements
- `quad`-oriented values and branching
- explicit `if`, `match`, and `return` control flow
- block-delimited executable structure

The canonical public source extension is `.sm`.

The compiled artifact extension is `.smc`.

## Executable Surface Profile

The common executable source shape is block-structured and explicit:

```sm
fn main() {
    return;
}
```

Representative public forms include:

- typed function signatures
- explicit local bindings with `let`
- record literals and field access
- `if` / `else if` / `else` branching
- `match` over `quad`, `Option(T)`, `Result(T, E)`, and nominal enum forms
- `return` statements and expression-bodied sugar where documented

The surface is intentionally not indentation-significant.

## Quad Identity

Semantic is visually distinct from Rust, Python, and config-file DSLs because
its public source uses a native four-valued logic vocabulary:

- `N`
- `F`
- `T`
- `S`

This is not boolean truthiness. The source surface keeps quad logic explicit and
names the values directly.

## Declaration Profile

Semantic source commonly begins with top-level declarations such as:

- `record Name { ... }`
- `schema Name { ... }`
- `fn name(...) { ... }`

This is different from ordinary config files, which usually center on data
keys rather than executable declarations.

## Import and Module Markers

Semantic source also shows a module-oriented profile through explicit imports:

```sm
Import "helper.sm"
Import "helper.sm" { symbol }
Import pub "shared.sm"
```

That import style is a strong differentiator from Rust-style `use`, Python
imports, and generic key/value configuration.

## Why `.sm` Should Classify As Semantic

`.sm` should be classified as Semantic because the file extension consistently
maps to a recognizable source identity:

- block-structured executable declarations are present
- `fn`, `record`, `schema`, and `Import` are visible markers
- quad logic is first-class and named explicitly
- source and compiled artifact identities are separated as `.sm` and `.smc`
- the canonical examples pack already uses `.sm` as the public sample surface

The classification should rely on those stable markers, not on any unstable or
out-of-scope syntax.

## Evidence Boundaries

Use current public source only.

Do not treat these as evidence for Linguist classification:

- experimental syntax
- unpromised future sugar
- generated `.smc` artifacts
- benchmark-only samples
- private qualification scaffolding

## Canonical Presentation

The preferred layout and density of these markers — top-level ordering,
indentation, line width, compact guard/match forms — is frozen separately in
`docs/spec/source_style.md` (Semantic Canonical Source Style v0). This note
covers identity markers; that document covers presentation.

## Related Documents

- `docs/LANGUAGE.md`
- `docs/NAMING.md`
- `docs/spec/syntax.md`
- `docs/spec/source_style.md`
- `examples/canonical/README.md`
