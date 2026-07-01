# M8 Everyday Expressiveness Roadmap

Status: active post-stable language-maturity package

Related documents:

- `docs/roadmap/language_maturity/m8_everyday_expressiveness_blueprint.md`
- `docs/roadmap/language_maturity/m8_everyday_expressiveness_phased_implementation_plan.md`
- `docs/roadmap/language_maturity/semantic_language_experience_roadmap.md`
- `docs/language/semantic_language_experience.md`
- `docs/language/quad_language_design.md`

## Goal

Define the next language-facing phase after the current post-stable platform and
runtime tracks.

This roadmap does not reopen the published `v1.1.1` line. It describes a
future forward-only language-maturity package for current `main`.

## Strategic Reading

Before opening larger abstraction systems, Semantic should first gain stronger
everyday expressiveness.

Primary order:

1. text / strings
2. package ecosystem
3. collections
4. first-class closures

Only after those foundations become stable should the language open wider
abstraction surfaces such as:

- generics
- traits / protocols
- richer iterable abstraction
- broader pattern systems

Platform-specific expansion such as UI boundary, graphics, or broader runtime
families remains a separate class of work and must not be mixed into this
language-maturity stream.

## Proposed Milestone Reading

### M8 — Everyday Expressiveness Foundation

Tracks:

- M8.1 Text / String Surface
- M8.2 Package Ecosystem Baseline
- M8.3 Collections Surface
- M8.4 First-Class Closures

Exit reading:

Semantic can express common application-facing data and modular code more
naturally without weakening its semantic core discipline.

### M9 — General Abstraction Layer

Tracks:

- M9.1 Generics / Parametric Polymorphism
- M9.2 Traits / Protocols / Interfaces
- M9.3 Iterable Abstraction
- M9.4 Richer Pattern Surface

Exit reading:

Semantic can express reusable APIs and reusable abstractions without overloading
concrete nominal forms.

### M10 — Application And Platform Expansion

Tracks:

- M10.1 UI Application Boundary
- M10.2 Broader Package/Distribution Layout
- M10.3 Optional Concurrency Model
- M10.4 Optional Extended Runtime Families

Exit reading:

Semantic can serve as an application/platform language without compromising
release discipline.

## Top-Priority Reading

Immediate next language-facing priorities in this proposal:

- text / strings
- package ecosystem baseline
- collections
- first-class closures

Current completed first subtrack checkpoint:

- `docs/roadmap/language_maturity/text_type_full_scope.md`

Current completed second subtrack checkpoint:

- `docs/roadmap/language_maturity/package_ecosystem_baseline_scope.md`

Current completed third subtrack checkpoint:

- `docs/roadmap/language_maturity/collections_surface_full_scope.md`

Current active fourth subtrack checkpoint:

- `docs/roadmap/language_maturity/first_class_closures_full_scope.md`

## Explicit Non-Goals

This roadmap package does not by itself:

- silently widen the published `v1.1.1` line
- reopen closed first-wave tracks
- mix language maturity with PROMETHEUS/runtime widening
- treat UI/platform work as if it were just syntax growth

## Decision Rule

Each proposed track under this package should be opened only when:

- the current active stream is no longer in conflict with it
- a scope checkpoint exists
- the admitted contract is explicit
- exclusions are explicit
- stable/main distinction stays honest
