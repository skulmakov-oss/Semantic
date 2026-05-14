# Semantic Hello Grammar Slice

Status: grammar-slice planning draft

See also:

- [`semantic_hello_implementation_readiness.md`](semantic_hello_implementation_readiness.md)
- [`semantic_hello_world_shape.md`](semantic_hello_world_shape.md)

## 1. Purpose

This document defines the proposed minimal grammar slice for later Hello World
implementation planning.

This is not implementation.
This is not parser change.
This is not final general grammar.
This is the narrow grammar slice needed to scope `#477` safely.

## 2. Non-Goals

This document does not:

- change parser or typechecker behavior
- implement grammar
- implement runtime / effect behavior
- change capability / effect admission
- implement SemCode lowering
- implement verifier changes
- implement VM / runtime changes
- add tests or fixtures
- rewrite README content
- rewrite examples
- implement Hello World
- implement `observe`
- implement `print`
- implement `entry` / `complete` / `require`
- implement a formatter
- start Linguist readiness
- touch UI / Workbench / I70

## 3. Proposed Minimal Hello Shape

```semantic
entry HelloWorld {
    state boot: quad = T;
    require boot == T;
    observe "Hello, World!";
    complete T;
}
```

Label: proposed grammar slice target.
Not executable yet.
Implementation requires later PRs.

## 4. Grammar Elements to Scope

| element | proposed role | minimal accepted form | explicit exclusions |
|---|---|---|---|
| `entry` | named entrypoint / program boundary | `entry Name { ... }` | multiple entries, parameters, generics, imports, modules |
| `state` | local semantic state declaration | `state name: quad = T;` | mutable state model, arbitrary types beyond required first wave, heap allocation |
| `require` | precondition / admission requirement | `require boot == T;` | arbitrary expression language, side effects, non-deterministic checks |
| `observe` | controlled observation request | `observe "Hello, World!";` | formatting, interpolation, scalar conversion, file/stdin/network/general I/O |
| `complete` | explicit completion state | `complete T;` | arbitrary return values, early return control-flow, exceptions |

## 5. Minimal Grammar Sketch

```ebnf
hello_file      ::= entry_decl ;
entry_decl      ::= "entry" IDENT "{" hello_stmt* "}" ;
hello_stmt      ::= state_decl | require_stmt | observe_stmt | complete_stmt ;
state_decl      ::= "state" IDENT ":" "quad" "=" quad_lit ";" ;
require_stmt    ::= "require" IDENT "==" quad_lit ";" ;
observe_stmt    ::= "observe" text_lit ";" ;
complete_stmt   ::= "complete" quad_lit ";" ;
quad_lit        ::= "T" | "F" | "N" | "S" ;
```

This is a planning sketch, not accepted grammar.

## 6. Bridge Mapping

| future term | old bridge equivalent | current status |
|---|---|---|
| `entry` | `fn main` | bridge-only mapping concept |
| `state` | `let` | bridge / model comparison only |
| `require` | `assert` | bridge-only mapping concept |
| `observe` | `print` | bridge / rejected canonical comparison only |
| `complete` | `return` | bridge-only mapping concept |

Clarify:

- old bridge equivalents are not canonical
- mapping is conceptual only
- no automatic migration is defined here

## 7. Policy Constraints

- `observe` accepts text literal only in the first Hello slice.
- no implicit `to_text`.
- no interpolation.
- no general stdout.
- no file / stdin / network.
- observation order must be deterministic.
- requirement must not perform effects.
- completion is explicit.
- any failure semantics must be planned before implementation.

## 8. Open Questions

- whether `state` should allow only `quad` for first wave
- whether `require` accepts only quad equality in first wave
- whether `complete` accepts only quad literal in first wave
- whether text literal observation should include newline behavior
- whether observation sink requires capability declaration in source
- whether observation produces audit event in all runtimes
- how diagnostics should point at `observe` misuse
- whether bridge `print` compatibility is ever implemented or avoided entirely

## 9. Implementation Handoff

This document prepares `M-HELLO-2` / `M-HELLO-3` planning.

Suggested next PRs:

- `M-HELLO-2 — tests(hello): add pending/admission fixtures for Hello grammar slice`
- `M-HELLO-3 — parser/sema planning or implementation, only after acceptance`

This PR starts none of them.

## 10. Acceptance Checklist

- grammar slice target recorded
- minimal grammar elements scoped
- EBNF-like sketch added
- bridge mapping added
- policy constraints listed
- open questions listed
- no implementation
- no tests / fixtures
- no runtime / verifier / capability changes
- `#477` remains open
