# Machine-Readable Diagnostics Schema v1

Status: normative v1 (SSF-09, `#1580` AC1). Not a stable-release promise:
`main` is not stable (`CONSTRAINTS.md`), and this schema's stability is the
versioning policy below, nothing more.

Owner: `smc-cli` (`crates/smc-cli/src/diagnostic_schema.rs`), the external
schema owner. The canonical internal carrier (`sm-diagnostic`) is a separate
artifact with its own change control and is **not** this schema: the schema
is an explicit projection of the carrier, never a `Serialize` derive
(`ssf09_canonical_carrier_contract.md` section 4).

## Producing the document

```text
smc check <input.sm|project-root> --format json
```

- Output: exactly one document on stdout, UTF-8, `\n` line endings, one
  trailing newline. Nothing is written to stderr.
- Exit status: `0` when `status` is `passed`; `1` when it is `failed` or
  `error`.
- `--format human` renders the same report with the canonical human
  renderer (plain text, no ANSI escapes). `--format` cannot be combined with
  the legacy `--no-cache`/`--metrics`/`--deny`/`--color` flags; such a
  combination is rejected, not ignored.
- `smc check` without `--format` keeps its legacy output unchanged.

## Document

```json
{
  "schema": "semantic.diagnostics",
  "schema_version": 1,
  "status": "passed | failed | error",
  "sources": [Source],
  "diagnostics": [Diagnostic],
  "summary": {"errors": 0, "warnings": 0},
  "failure": null | {"message": "..."}
}
```

- `status`: `failed` iff at least one top-level diagnostic has severity
  `error`; `passed` otherwise; `error` when the check could not run at all
  (unreadable input, package admission or project-root failure). An `error`
  document has empty `sources`/`diagnostics`, zero counts, and a `failure`
  message. A `failure` is not a diagnostic: no producer assigned it a code.
- `summary` counts top-level diagnostics only (causes are not counted).

### Source

```json
{"index": 0, "path": "src/main.sm" | null, "identity": Identity | null}
```

- `index`: position in `sources`; diagnostics refer to sources by index.
  Index `0` is always the checked root.
- `path`: presentation-only path relative to the current working directory
  with `/` separators, or the root argument as typed. `null` when no such
  relative form exists. Never an absolute host path synthesized by the tool.
- `identity`: the canonical logical identity (SSF-09 Decision C), present
  only when package admission proves it:
  `{"package": "<package name>", "module": "<module path relative to the package module_root>"}`
  - e.g. `src/util/helper.sm` under `module_root src` is
  `"module": "util/helper.sm"`, never the basename. A rootless standalone
  source has `"identity": null`.

### Diagnostic

```json
{
  "code": "E0005",
  "severity": "error | warning",
  "family": "frontend | semantic | verification | runtime",
  "message": "...",
  "source": 0 | null,
  "range": Range | null,
  "related": [{"source": 0 | null, "range": Range | null, "message": "..." | null}],
  "notes": ["..."],
  "fix": "..." | null,
  "cause": null | {"kind": "diagnostic | report", "diagnostics": [Diagnostic]}
}
```

- `code`, `severity`, `family`: the producer's own identity (Decision B),
  relayed unchanged. Codes are opaque strings; consumers must not parse
  them. The retired placeholder `E0000` is never emitted.
- `message`: presentation-free diagnostic text (no carets, no ANSI, no
  line/column prefix).
- `source`: index of the source the producer analyzed, or `null` when the
  check cannot bind the diagnostic to one source (for example a diagnostic
  about a composed executable bundle, or an import cycle).
- `range`: present only when producer authority proves a genuine UTF-8 byte
  range (Decision D). An absent range is `null`, never `[0,0)` or `1:1`.
- `related`, `notes`: producer order, never sorted or deduplicated.
- `fix`: human guidance only; no machine-applicable edit exists in v1.
- `cause`: structured nested diagnostics (for example `E0239` wrapping the
  module's frontend parse error), never flattened text.

### Range

```json
{"start": Position, "end": Position}
Position = {"offset": 12, "line": 2, "column": 9}
```

- `offset`: canonical zero-based UTF-8 byte offset; the range is half-open
  `[start.offset, end.offset)`.
- `line`, `column`: derived, 1-based; `line` counts `\n`; `column` counts
  Unicode scalar values from the line start. Consumers needing another
  convention (LSP UTF-16) must derive it from `offset` and the source text.

## Determinism

Key order, indentation (two spaces), escaping (`"`, `\`, `\n`, `\r`, `\t`,
and every other control character as `\u00XX`) and array order are fixed.
Identical inputs produce byte-identical documents
(`tests/ssf09_editor_baseline.rs::schema_output_is_byte_deterministic`,
golden files under `tests/golden_snapshots/ssf09_editor_baseline/`).

## Versioning policy

- `schema_version` is incremented for any change that removes or renames a
  key, changes a key's type or meaning, or changes an enumeration value.
- Adding a new `family` value, a new `status` value, or a new key is also a
  version increment in v1: consumers may treat unknown values as errors.
- New diagnostic **codes** are not schema changes (codes are opaque data).
- A version bump requires a new `diagnostics_machine_schema_vN.md`, updated
  goldens, and a compatibility note; v1 is never edited in place except for
  editorial clarification.

## Producer coverage (v1)

| Producer | Canonical adapter | Code authority | Range authority |
|---|---|---|---|
| `sm-front` lexer | `FrontendDiagnostic::from_lex_failure` | lexer codes `E0001`-`E0004`, `E0101` | offending byte(s) |
| `sm-front` RustLike parser | `FrontendDiagnostic::from_parse_error` | `E0005` syntax, `E0006` policy | exact extent of the token at the reported offset |
| `sm-front` Logos parser | `FrontendDiagnostic::from_parse_error` (structured `FrontendErrorDetail`) | the parser's own `E02xx` codes | anchor token extent; recovered errors as `related` |
| `sm-front` surface resolution | `FrontendDiagnostic::surface_resolution` | `E0007` ambiguous, `E0008` no claim | none (whole input) |
| `sm-front` type checker | `FrontendDiagnostic::from_type_check_error` | `E0201` | none: the type checker has no position authority |
| `smc-cli` executable bundle composition | `bundler_semantic_error` | `E0009` | none |
| `sm-sema` | `SemanticDiagnostic::to_canonical` | `sm-sema` codes (`E02xx`, `W02xx`) | extent of the lexical token whose mark the analyzer reported |
| `sm-verify` | `VerificationDiagnostic::to_canonical`, `RejectReport::to_canonical` | stable `V0001`-`V0032` tokens | none: artifact offsets are notes, never ranges |
| `sm-vm` runtime | `RuntimeError::to_canonical` (explicit admission table) | `R0001`-`R0030` | none: a program counter is not a source position |

`smc check` runs only the source stages; the verifier and runtime adapters
are qualified by their crates' own tests and are available to later
tooling.
