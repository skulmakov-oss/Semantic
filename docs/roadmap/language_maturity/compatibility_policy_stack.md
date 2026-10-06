# Compatibility Policy Stack

Status: current governance companion to SSF-10 (#1378). The canonical technical
compatibility contract is `docs/architecture/artifact_identity_and_trust.md`
(SSF-10 §2); deprecation states and migration rules are in
`docs/roadmap/language_maturity/deprecation_and_migration.md`. This document
is not a release qualification authority and marks no surface stable by itself.

## Purpose

This document defines the intended compatibility stack across the main public
Semantic surfaces.

The point is to stop treating compatibility as one bytecode question. A mature
language platform needs different but coordinated rules for each layer.

## Layer 1: Source Language

Scope:

- syntax
- source typing
- source semantics
- module/import/export forms
- Logos surface

Compatibility rule:

- once a source feature is marked `stable`, its meaning must not silently
  change
- removing or reinterpreting a stable source form requires deprecation and
  migration guidance

Current honest note:

- much of the source-language contract is still in `draft v0` spec form

## Layer 2: CLI Surface

Scope:

- documented commands
- documented flags
- machine-readable JSON output

Compatibility rule:

- stable machine-readable CLI fields must not change silently
- CLI subcommands can evolve faster while still labeled `beta` or `draft`

## Layer 3: Standard Library

Scope:

- module names
- exported public functions/types
- documented library semantics

Compatibility rule:

- once a stdlib family is marked stable, its module identity and public API
  shape become compatibility-sensitive
- moving a stdlib helper into a builtin or changing argument order is a
  breaking change

## Layer 4: Package Ecosystem

Scope:

- manifest format
- lockfile format
- dependency resolution rules
- published package naming/versioning expectations

Compatibility rule:

- stable manifest fields must not be repurposed silently
- lockfile meaning must remain deterministic for the same versioned format
- package resolution behavior must not drift silently between patch releases

## Layer 5: SemCode And Profile Contracts

Scope:

- SemCode headers/opcodes/capabilities
- ParserProfile schema and semantics

Compatibility rule:

- these are already version-sensitive contracts and require explicit version
  review when meaning changes

This is currently the strongest compatibility-disciplined layer in the
repository.

## Layer 6: Runtime And Boundary Surfaces

Scope:

- verified execution path
- PROMETHEUS capability/gate/runtime contracts
- audit/runtime compatibility-sensitive records

Compatibility rule:

- narrow implemented boundary surfaces may be compatibility-sensitive even while
  the wider long-term runtime roadmap remains out of scope
- changes require spec and validation updates in the same series

## Cross-Layer Rule

A change is compatibility-sensitive if it affects any stable public surface in
any of the layers above.

That means a release review should ask:

1. Which layer changed?
2. What is the current stability label of that layer?
3. Is a migration note required?
4. Is a version or release-note boundary required?

## Review Triggers

The following should trigger explicit compatibility review:

- source syntax/meaning changes
- machine-readable CLI changes
- stdlib API changes
- manifest/lockfile changes
- SemCode/Profile meaning changes
- boundary/runtime contract changes

## Cross-References

This stack works together with:

- `docs/roadmap/language_maturity/stability_and_compatibility.md`
- `docs/roadmap/language_maturity/deprecation_and_migration.md`
