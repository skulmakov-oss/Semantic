# FA-02-042 — Logos Pulse/Profile Directive Preservation Decision

Residual cleanup, not a Phase-B slice. Umbrella: #1617 · Issue: #1987
Base: `37f988447b540760431e21515ad32ebc45ff0fc6`
C1 `89641da8237f4fcefb50cf1958a50e4d4003aea7` / `v1.2.0` are not touched.
Out of scope: #1993.

## Re-adjudication on current main (`37f98844`) — STILL_REPRODUCED

Under the default profile (`CompatibilityMode::LegacySupport`), both Logos parse paths accept and drop the directives:

| Source | `parse_logos_with_profile` | `admit_logos_program_with_profile` |
|---|---|---|
| `Pulse emit = sensor.edge\nProfile fast mode\n` | `Ok(LogosProgram { … imports: [] })`, no trace of either line | `Exclusive(Ok(…))`, same empty program |
| `Pulse\nProfile\n` (head only) | `Ok`, empty | `Exclusive(Ok)`, empty |
| `Pulse a\r\nProfile b` (CRLF, EOF) | `Ok`, empty | `Exclusive(Ok)`, empty |

Cause: the direct loop gated `Import`/`Pulse`/`Profile` on legacy compatibility, preserved only `Import`, and skipped the rest to the end of the line. The admission scan skipped `Pulse`/`Profile` the same way after promoting evidence to `Exclusive`.

Current behavior, kept unchanged:

- **Head-only directives:** `Pulse` and `Profile` with no payload are accepted by the current grammar. No payload rule is invented; they are preserved with text `"Pulse"` / `"Profile"`.

## Decision

`Pulse` and `Profile` are admitted, legacy-compatibility-gated, Logos-exclusive directive heads in the experimental Model-B contract. They have no executable or semantic meaning. Logos supports deterministic inspection, so an accepted directive must stay observable:

```text
accepted Pulse/Profile line
→ opaque LogosLegacyDirective in LogosProgram::legacy_directives
→ no semantic interpretation
→ no LogosIrLaw, no SemCode, no verifier, no VM
```

Two alternatives were rejected:

- **Reject the directives.** That would be an incompatible grammar change needing a new contract version.
- **Interpret the directives.** Nothing in the current architecture says what they mean.

## AST contract

- `LogosLegacyDirectiveKind { Pulse, Profile }` is a typed family, never re-derived from the text.
- `LogosLegacyDirective { kind, directive, span, mark }`.
- `directive` is the exact source from the keyword to the end of the line, excluding `\n` and a trailing `\r`. There is no token reconstruction, trimming or normalization. `span` is its byte range and `mark` is the keyword's source mark.
- `LogosProgram::legacy_directives` is one vector in source order; Pulse and Profile are not split into separate vectors and are not sorted.
- Text capture is one private helper, `capture_logos_directive_line`, shared with `LogosImport`, so both node kinds have identical text and span semantics.

## Unchanged boundaries

- **Import ownership:** `LogosImport` and `LogosProgram::imports` are unchanged. `sm-sema` stays the sole owner of import semantics. No `Pulse`/`Profile` semantic parser exists or was added.
- **Semantic analysis:** `sm-sema` does not interpret `legacy_directives`. That is the contract for opaque inspection nodes, not a drop. The parser dropping source was the defect; a semantic layer not interpreting an explicitly opaque node is intended.
- **Evidence basis (SSF-09 Decision E):** `Pulse`/`Profile` remain `Exclusive` and `Import` remains `Shared`.
- **Policy:**
  - With legacy compatibility disabled, `Pulse`/`Profile` still fail as `PolicyViolation` on both paths; no node is preserved and the failure is never ignored.
  - The policy gate runs before preservation.
- **Recovery and aggregation:** a directive preserved before a later malformed declaration does not create partial success. The existing all-errors contract still returns `Err` for the whole program, and diagnostics, error positions and the E0200–E0237 codes are unchanged.
- **Inspection:**
  - `dump-ast` shows the nodes through the standard Debug projection.
  - `LogosIrLaw` (`dump-ir`/`hash-ir`) stays a law-only summary: AST inspection is lossless for accepted directive lines, and the inspection IR is not a full AST serialization.
- **Execution:** compile, `dump-bytecode`, `hash-smc`, `run` and the library SemCode path still reject Logos sources that contain these directives.
- **Mixed surfaces:** `Pulse` plus RustLike evidence is still a deterministic rejection with no fallback.

## Historical records

The PB-02 and SSF-09 decision records correctly describe the skip behavior as it was at their time; they are not rewritten.

## Public API

- `sm_front` re-exports `LogosLegacyDirective` and `LogosLegacyDirectiveKind`.
- `LogosProgram` gains `legacy_directives`.
- The `entity_law.ast` golden gains one `legacy_directives: []` line.
- The new `legacy_directives.ast` golden shows `Import`, `Pulse` and `Profile` preserved.

## Non-goals

No Pulse/Profile semantics, capability policy, runtime profile selection, opcode mapping or compiler configuration. No Logos execution path. No #1993 work.
