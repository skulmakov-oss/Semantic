# Machine-Readable Language Surface v1

Status: normative v1 (S2043, OWNER-approved schema and spelling). Not a
stable-release promise: `main` is not stable (`CONSTRAINTS.md`); the
stability of this document is the versioning policy below, nothing more.

Schema id: `semantic.language-surface`, version `1`.

Owners: `sm-front` (`crates/sm-front/src/language_surface.rs`) owns the one
canonical registry of features, statuses and probes; `smc-cli`
(`crates/smc-cli/src/language_surface.rs`) owns only the wire projection.
There is no second, hand-maintained list anywhere else.

## Producing the document

```text
smc language describe --format json
```

- Output: exactly one UTF-8 JSON document on stdout, two-space indentation,
  `\n` line endings, one trailing newline. Nothing on stderr. Exit status `0`.
- Any other argument list (no subcommand, another subcommand, a missing or
  other `--format` value, extra arguments) is a usage error: non-zero exit,
  nothing on stdout, `usage: smc language describe --format json` on stderr.
- The command reads no source, runs nothing, performs no host effect and
  grants nothing.
- Existing commands keep their meaning: `smc features` reports Cargo build
  features, `smc version --json` reports tool identity, `smc check --format
  json` reports diagnostics. None of them is a language catalog.

## Document

```json
{
  "schema_id": "semantic.language-surface",
  "schema_version": 1,
  "compiler_identity": {
    "source_commit": null,
    "source_fingerprint": "<16 hex digits>" | null,
    "build_id": null
  },
  "profile": {"source": "semantic.foundation", "compilation": "O0+O1"},
  "features": [Feature],
  "semcode": {"floor": "SEMCOD22", "promotions": ["SEMCOD23", "SEMCOD24", "SEMCOD25"]},
  "runtime": {"envelope": null, "capability_grants": null}
}
```

- `compiler_identity.source_commit` is always `null`: the binary carries no
  trustworthy Git provenance. `source_fingerprint` is the non-cryptographic
  compiler source fingerprint also reported by `smc version --json`; it is
  never a commit id or a digest. `build_id` is `null`.
- `profile.source` is the parser profile identity the statuses were
  established under; `profile.compilation` names the optimization levels
  every lowering-and-later stage was qualified at (both `O0` and `O1`).
- `semcode.floor` is the canonical artifact header; `promotions` are the
  headers a program is promoted to only when it actually uses the
  corresponding opcodes or capabilities. Neither is a capability grant.
- `runtime` is always `null`/`null`: describing the language selects no
  execution envelope and grants no capability.

### Feature

```json
{
  "id": "control.if_let.expression",
  "release_state": "current-main-experimental",
  "stage_status": {
    "lex": "qualified", "parse": "qualified", "typecheck": "qualified",
    "lower": "qualified", "emit": "qualified", "verify": "qualified", "run": "qualified"
  },
  "evidence": {"suite": "tests/s2043_language_surface.rs::declared_stages_match_the_running_pipeline"}
}
```

Features are sorted by `id` (byte order) and unique. Exactly these four keys.

Two independent axes:

- `release_state`: `published-stable`, `foundation-included` (listed as
  Included by `foundation_source_profile_v1.md`), `current-main-experimental`
  (accepted on current `main`, no compatibility promise), `draft-only`,
  `unsupported` (deliberately not admitted), `unknown`.
- `stage_status` per stage `lex`, `parse`, `typecheck`, `lower`, `emit`,
  `verify`, `run`: `qualified`, `rejected` (the first stage that
  deterministically rejects the feature), `not-qualified` (not proven either
  way), `not-applicable` (a previous stage rejects).

A stage is `qualified` only when the feature's registry probe passes it
through the real pipeline of the same build (`check --format json`, then
`compile` at `O0` and `O1`, `verify` and `run-smc` of both artifacts), and an
executable feature also carries a negative probe that the same pipeline must
reject at a declared stage. `typecheck` is the canonical source check
(`smc check --format json`): for the inspection-only Logos profile it means
Logos semantic analysis, while the legacy unformatted `smc check` rejects
Logos input by the PB-03 executable-admission boundary (`logos.md`).
`lower` failing covers the whole compile step; `emit` is never reported
`rejected` separately.

## Determinism

Key order, indentation, escaping and array order are fixed; identical builds
produce byte-identical documents.

## Versioning policy

- `schema_version` is incremented for any removed or renamed key, changed key
  type or meaning, new key, or new enumeration value.
- Adding, removing or re-classifying a feature entry is not a schema change;
  it must keep the registry probes passing.
- A version bump requires a new `language_surface_vN.md` and a compatibility
  note; v1 is never edited in place except for editorial clarification.

## Evidence

`tests/s2043_language_surface.rs` re-derives every declared stage from the
pipeline, checks the negative probes, the registry shape (sorted ids, stage
shape, release/stage agreement), byte determinism, exact keys, the honest
identity/runtime fields and that `smc features` / `smc version --json` are
unchanged.
