# Workbench Spec Index Reader v0 — Read-only Source Contract

## Purpose

The Workbench Spec Index Reader is **presentation-only**.

It exists to give developers a convenient surface for navigating canonical repository documentation directly from the Workbench shell. It does not create, fork, interpret, or replace any document. It does not hold release authority, semantic authority, or readiness authority of any kind. All source of truth remains in the repository documents it reads.

## Canonical Source Roots

The Spec Index Reader is permitted to read from exactly two source roots:

- `docs/spec/**`
- `docs/roadmap/**`

No other file trees may be presented as spec or roadmap content by this reader.

## Boundary Rules

The following rules are binding on the Spec Index Reader implementation:

1. **Workbench may read repository documentation but must not become a source of truth.**

   The reader is a navigation aid. Any document it displays remains authoritative only at its canonical path in the repository.

2. **Canonical sources remain `docs/spec/**` and `docs/roadmap/**`.**

   The reader must not synthesize, duplicate, or shadow these documents into a separate data store under Workbench control.

3. **Workbench must not interpret docs as release authority.**

   Displaying a document from `docs/roadmap/**` does not grant Workbench any role in release decisions, readiness scoring, or gate adjudication.

4. **Workbench must not create parser, typechecker, verifier, VM, or runtime semantics.**

   The reader must not infer, derive, or expose language semantics from documentation content. All such semantics remain in the compiler, verifier, VM, and runtime components.

5. **Workbench must not edit docs through this reader.**

   The Spec Index Reader is strictly read-only. No write, create, delete, or rename path may be reached through the reader surface.

6. **Local Admission Guard remains authoritative.**

   All Workbench functionality, including the Spec Index Reader, is subject to the Local Admission Guard. GitHub CI is not a substitute for the Local Admission Guard.

7. **GitHub CI is not a Workbench gate.**

   CI pass/fail status is not surfaced by the reader as a readiness signal. The reader presents repository document content only.

## Minimal Displayed Fields

The reader is permitted to display only the following fields per document entry:

- `path` — repository-relative path to the document.
- `filename` — filename component of the path.
- `directory group` — derived from the parent directory: `docs/spec` or `docs/roadmap`.
- `title` (optional) — first markdown heading extracted verbatim from the document.
- `short description` (optional) — first non-heading paragraph if already present in the document.
- `freshness / status` (optional) — only if derived directly from evidence present in the repository, such as a status field in the document's own frontmatter.

All displayed content must be derived verbatim or structurally from the document itself. No field may be synthesised by Workbench.

## Forbidden Fields

The following fields are explicitly forbidden and must never appear in the reader surface:

- **UI-owned readiness score** — Workbench may not compute or display a readiness score for any document.
- **UI-owned release verdict** — Workbench may not emit a pass/fail release verdict derived from displayed documents.
- **Inferred stability claim** — Workbench may not label any document or its subject as stable, unstable, or production-ready unless that label appears verbatim in the document itself.
- **Inferred production-readiness claim** — Workbench may not infer or display production-readiness from documentation content.
- **Hidden GitHub CI authority** — Workbench may not use CI status to derive or gate any field shown in the reader.
- **Generated semantic interpretation not present in the docs** — Workbench may not add explanatory or interpretive text that does not appear in the source document.

## Validation

The following lightweight checks apply to any PR that introduces or modifies the Spec Index Reader implementation:

- `git diff --check` — must pass with no whitespace or end-of-line errors.
- `git status --short --branch` — working tree must be clean at review time.
- **No FullPreflight** — the Spec Index Reader is a presentation slice; it does not require a full preflight run.
- **No release artifacts** — implementation of this contract does not produce or modify release artifacts.

## Implementation Note

This document is a scope contract only; implementation must be handled in a separate bounded PR.
