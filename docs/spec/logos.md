# Logos Surface Specification

Status: experimental declarative profile v0.1
Contract identifier: `semantic.logos.declarative/0.1`
Primary frontend owners: `sm-front`, `sm-sema`

## Purpose

This document defines the current declarative Logos-oriented source surface used
for system, entity, and law descriptions inside Semantic.

It complements the Rust-like executable surface described in `syntax.md`.

## Authoritative Relationship

SSF-02 selects **Model B: declarative Logos profile**.

- Rust-like Semantic owns executable behavior and the SemCode/verifier/VM path.
- Logos describes systems, entities, laws, and policy-shaped `When` fragments.
- Logos parsing, semantic analysis, and `LogosIrLaw` projection do not imply
  executable lowering.
- No tool may reinterpret Logos text fragments as Rust-like expressions or
  bypass verifier-first execution.

The decision evidence and rejected Model A alternative are recorded in
`../roadmap/stable_foundation/rustlike_logos_coherence_decision.md`.

## Current Top-Level Forms

The current Logos surface recognizes these top-level declarations:

- `System`
- `Entity`
- `Law`

Current legacy compatibility directives may still exist in guarded paths, but
they are not the primary long-term public contract for the Logos surface.

## System

Current `System` form:

```sm
System Name(param = value, ...)
```

Current rule:

- `System` declares one top-level system descriptor
- parameters are `name = value` pairs (an identifier or numeric-literal value),
  not typed parameter declarations; `parse_logos_system` in `sm-front` admits
  `name`/`=`/value, not `name`/`:`/type

## Entity

Current `Entity` form:

```sm
Entity Sensor:
    state val: quad
    prop threshold: f64
```

Current entity-field kinds include:

- `state`
- `prop`

Current rule:

- entity bodies are indentation-delimited
- each field has a kind, a name, and a type

## Law

Current `Law` form:

```sm
Law "CheckSignal" [priority 10]:
    When Sensor.val == T -> Log.emit("Signal OK")
```

Current law properties:

- law names are string-literal based
- `priority` is optional and numeric
- law bodies are indentation-delimited
- law bodies contain one or more `When` clauses

## When Clauses

Current `When` form:

```sm
When condition -> effect
```

Current rule:

- empty `When` conditions are rejected
- empty `When` effects are rejected
- the current frontend stores condition and effect as structured text fragments
  at this surface, not as the Rust-like executable AST

## Ordering Rule

Current rule:

- parsed laws are ordered by descending priority in the current Logos program

This behavior is part of the current public source contract and should not be
changed silently.

## Tool and Projection Contract

The admitted Logos workflow is inspection-only:

| Stage | Current status |
|---|---|
| Parse to `LogosProgram` | implemented and qualified |
| Semantic analysis | implemented with Logos diagnostics |
| Project laws to `LogosIrLaw` | implemented and qualified |
| Lower to Rust-like function IR or SemCode | unsupported |
| Verifier/VM execution | unsupported because no Logos SemCode is produced |

`LogosLaw` and `LogosWhen` retain source marks in the frontend model. The
current `LogosIrLaw` projection contains only law name, priority, and `When`
count. It is an inspection summary, not a generated executable or a source-map
promise for future binding.

Current CLI support is `dump-ast`, `dump-ir --profile logos`, and
`hash-ir --profile logos`. SemCode-producing and execution commands reject
Logos input before artifact emission.

## Files, Modules, and Packages

- one source file belongs to one surface for the current tool invocation;
- Rust-like items and Logos declarations may not be mixed in one admitted file;
- Logos has no Stable Foundation package/module or cross-profile import model;
- Rust-like package behavior remains owned by SSF-05 and SSF-06;
- any future binding must be an explicit versioned validated artifact, not an
  implicit import or a second execution authority.

## Maturity and Version Policy

`semantic.logos.declarative/0.1` is **Experimental**. The identifier versions
the declarative grammar, semantic checks, and inspection projection described
here; it is not a stable compatibility or execution promise. Incompatible
changes require a new contract identifier version and updated fixtures.

This status is separate from Rust-like
`semantic.foundation.source/1.0`, which is the qualified-limited executable
Stable Foundation candidate. The shared parser-profile version is an admission
envelope and must not be used to collapse these maturity states.

## Policy Rule

The Logos surface is policy-gated:

- it may be disabled by the active parser profile
- profile rejection is a source-level policy violation, not a runtime error

## Current Limits

The current Logos contract does not yet claim stable support for:

- a fully separate package or module ecosystem for Logos-only projects
- rich user-defined statement semantics inside `When` beyond the current
  text-fragment contract
- broad legacy directives as first-class long-term source features
- executable lowering, SemCode production, verifier admission, or VM execution
- mixed Rust-like/Logos files or implicit cross-profile imports

## Contract Rule

Any public change to `System`, `Entity`, `Law`, `When`, field kinds, priority
ordering, or profile-gating behavior should update this document in the same
change series.
