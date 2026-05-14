# Semantic Hello Parser / Sema Plan

Status: parser/sema planning draft for `#477`

See also:

- [`semantic_hello_grammar_slice.md`](semantic_hello_grammar_slice.md)
- [`semantic_hello_fixtures.md`](semantic_hello_fixtures.md)

## 1. Purpose

This document plans parser and semantic-analysis admission for the future Hello
grammar slice.

This is planning only.
This is not parser change.
This is not typechecker / sema change.
This is not accepted behavior change.
This is not fixture wiring.

## 2. Non-Goals

This document does not:

- add Rust code
- implement parser behavior
- implement typechecker / sema behavior
- change IR or lowering
- change SemCode
- change verifier behavior
- change VM / runtime behavior
- change capability / effect admission
- wire tests into passing CI
- add accepted golden output
- implement Hello World
- implement `observe`
- implement `print`
- implement `entry` / `state` / `require` / `complete`
- rewrite README content
- rewrite examples
- start Linguist readiness
- touch UI / Workbench / I70

## 3. Parser Scope Proposal

Future parser recognition scope:

- recognize `entry IDENT { ... }`
- recognize `state IDENT: quad = QUAD_LIT;`
- recognize `require IDENT == QUAD_LIT;`
- recognize `observe TEXT_LIT;`
- recognize `complete QUAD_LIT;`

Explicit exclusions:

- no arbitrary expressions
- no nested entries
- no parameters
- no imports or modules
- no interpolation
- no numeric observation payload
- no `print` alias
- no `io.write`
- no early-return semantics

## 4. AST / Surface Model Proposal

| source element | proposed AST / surface node | required fields | notes |
|---|---|---|---|
| `entry` | `HelloEntry` / `EntryDecl` | name, body statements, span | planning labels only; not an implementation commitment |
| `state` | `StateDecl` | name, quad literal, span | first-slice local semantic state binding |
| `require` | `RequireQuadEq` | state symbol, quad literal, span | fixed-shape precondition node |
| `observe` | `ObserveText` | text literal, span | controlled observation request node |
| `complete` | `CompleteQuad` | quad literal, span | explicit completion node |

These names are planning labels only.

## 5. Sema Admission Proposal

| element | semantic check | failure class | open question |
|---|---|---|---|
| `entry` | exactly one entry in the Hello slice | duplicate or missing entry diagnostic | whether later slices permit more than one entry |
| `state` | quad-only state binding | unsupported type or duplicate binding diagnostic | whether first slice permits more than one state |
| `require` | only `state == quad literal` | invalid requirement expression diagnostic | whether requirements remain quad-only beyond the first slice |
| `observe` | text literal only | non-text observation diagnostic | whether observation sink admission is checked here or later |
| `complete` | quad literal only | invalid completion diagnostic | whether completion remains explicit in later slices |

## 6. Name / Binding Rules

- `state boot: quad = T;` introduces a local semantic state symbol.
- `require boot == T;` may only reference declared state symbols.
- no shadowing in the first slice.
- no mutation in the first slice.
- no heap or string allocation decision beyond text literal handling.
- `observe` does not introduce bindings.
- `complete` terminates the entry slice.

## 7. Diagnostic Planning

Planned diagnostic categories, not final error codes:

- unknown state symbol
- duplicate state symbol
- invalid quad literal
- unsupported state type
- invalid requirement expression
- observation payload must be text literal
- observation or effect not admitted yet
- complete requires quad literal
- statement after complete
- legacy print not canonical

## 8. Pending Fixture Admission Mapping

| fixture | future parser outcome | future sema outcome | current status |
|---|---|---|---|
| `positive_hello_verbose_directional.sm` | parse yes | sema yes eventually | pending |
| `positive_hello_minimal_observe_directional.sm` | parse yes | sema maybe yes later, secondary onboarding shape | pending |
| `negative_hello_print_legacy_canonical.sm` | parse no or legacy rejection | sema no if parsed | pending |
| `negative_hello_observe_non_text_payload.sm` | parse maybe yes | sema no | pending |
| `negative_hello_require_side_effect_shape.sm` | parse no or sema no | sema no | pending |
| `negative_hello_general_io_shape.sm` | parse no | sema no | pending |

The mapping is only a planning forecast.
It does not wire these fixtures into current accepted test truth.

## 9. Implementation Sequencing Recommendation

Recommended future split:

- `M-HELLO-3B` - parser only, behind an isolated Hello grammar path
- `M-HELLO-3C` - sema only, with diagnostics but no lowering
- `M-HELLO-3D` - pending fixture test harness, still not accepted runtime behavior
- `M-HELLO-4` - IR / SemCode lowering plan
- `M-HELLO-5` - verifier / runtime / capability plan

This document proposes the sequence only.
It does not start any of these PRs.

## 10. Acceptance Checklist

- parser scope proposed
- AST / surface model proposal added
- sema admission proposal added
- name / binding rules listed
- diagnostic categories listed
- pending fixture mapping added
- implementation sequencing proposed
- no code changes
- no fixture changes
- no tests wired into passing CI
- no parser / sema implementation
- no runtime / verifier / capability changes
- `#477` remains open

## 11. M-HELLO-3B Boundary Note

`M-HELLO-3B` is the parser-only recognition step for the Hello grammar slice.

It may add isolated parser structures and parser-only tests for pending
fixtures, but it still does not add sema admission, lowering, verifier,
runtime, or capability behavior.

Pending fixtures remain outside accepted runtime truth until later phases.

## 12. M-HELLO-3C Boundary Note

`M-HELLO-3C` is the isolated sema-admission step for the parser-only Hello AST.

It may add semantic validation diagnostics and sema-only tests, but it still
does not add IR, lowering, SemCode, verifier, runtime, or capability
behavior.

Pending fixtures remain outside accepted runtime truth until later phases.
