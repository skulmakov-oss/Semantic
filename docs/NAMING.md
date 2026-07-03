# Naming

This note defines the project's naming rules and the meaning of its short forms.

## Principles

- Keep names short.
- Keep names readable.
- Prefer stable abbreviations over long branded prefixes.
- Reserve long names for user-facing identity only.

## Product Names

- `Semantic Language`: the canonical public language name and overall project identity.
- `Semantic`: the short form used in prose, headings, and tooling references.
- `TON618 Core`: the low-level core/engine identity.

## File Extensions

- `.sm`: the primary public source code extension for Semantic Language.
- `.sem`: textual machine/intermediate representation.
- `.smc`: compiled SemCode bytecode generated from `.sm` source.

## Tools

- `smc`: the compiler/tooling command.
  - Responsibilities: compile, check, lint, watch, hashes, snapshots.
- `svm`: the virtual machine command.
  - Responsibilities: run and disassemble `.smc`.

## Internal Crates

- `ton618-core`: low-level core primitives and packed quad-logic runtime.
- `sm-front`: frontend layer.
  - Lexer, parser, typing-facing AST helpers.
- `sm-sema`: semantic analysis layer.
- `sm-ir`: IR lowering and optimization layer.
- `sm-emit`: SemCode emission layer.
- `sm-vm`: SemCode VM layer.
- `smc-cli`: shared CLI pipeline crate.

## Prefixes and Terms

- `sm`: shorthand for `Semantic`.
- `sema`: shorthand for `semantic analysis`.
- `ir`: shorthand for `intermediate representation`.
- `vm`: shorthand for `virtual machine`.
- `cli`: shorthand for command-line interface.

## Format Terms

- `SemCode`: the bytecode/runtime format family.
- `SEMCODE0`, `SEMCODE1`: concrete SemCode header versions.

## Practical Rule

When naming a new file, module, crate, or command:

1. Use the shortest name that stays obvious.
2. Reuse the abbreviations from this file.
3. Do not introduce new prefixes if an existing one already fits.
