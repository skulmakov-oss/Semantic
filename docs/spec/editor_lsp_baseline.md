# Editor and Language Server Baseline

Status: SSF-09 baseline (`#1580` AC4-AC7). Diagnostics-first; not a
stable-release promise.

Owner: `smc-cli` (`crates/smc-cli/src/lsp.rs`, `canonical_check.rs`,
`formatter.rs`).

## Authority boundary

`smc check` and the canonical compiler services remain the only source of
language truth. The server:

- runs **the same** canonical check as `smc check --format human|json`
  (`smc_cli::canonical_check::check_canonical`): the frozen Decision E/F
  surface authority, the #1933 bundling rules, and the `sm-sema` project
  mechanism, unchanged;
- never parses, type-checks, verifies, classifies, or invents a code,
  severity, range, or identity of its own;
- performs only transport conversions: canonical UTF-8 byte ranges become
  LSP UTF-16 positions (line terminators `\n`, `\r\n`, `\r`), and each
  diagnostic is routed to the document URI the client supplied, or - for a
  diagnostic bound to another source of the same check (an imported
  module) - to that source's `file:` URI;
- formats only through the canonical formatter, including its refusals.

`tests/ssf09_editor_baseline.rs::cli_lsp_parity_*` proves the LSP publishes
exactly the canonical diagnostics of `smc check --format json` (same code,
severity, family, message, and the UTF-16 image of the same byte range, on
the URI of the same source) for every baseline fixture.

## Running the server

```text
smc lsp [--stdio]
```

JSON-RPC 2.0 with `Content-Length` framing over stdin/stdout; `--stdio` is
accepted because clients commonly pass it and is the only transport. The
server opens no socket, reads only stdin and the project files the canonical
check reads, writes only protocol frames to stdout, and sends no telemetry.

### Capabilities

| Capability | Behaviour |
|---|---|
| `positionEncoding` | `utf-16` |
| `textDocumentSync` | `openClose`, full-document `change` (1), `save` |
| `textDocument/publishDiagnostics` | canonical diagnostics; `version` = the document version checked |
| `textDocument/formatting` | one whole-document `TextEdit` from the canonical formatter; `[]` when already formatted; error `-32803` when the formatter refuses |
| `workspace/didChangeWorkspaceFolders` | recorded; all open documents are re-checked |

Every open document is checked with all open documents applied as an
in-memory overlay: every project source read - the root, Logos project
modules, and RustLike executable-helper modules - goes through one
overlay-first source seam (`smc-cli`'s `SourceAccess`), so an unsaved
imported module of either grammar is seen by its importers. Closing a
document drops its overlay (the saved file is read again); reopening it
restores the overlay
(`lsp_unsaved_rustlike_helper_is_seen_by_its_importer_over_stdio`).
A `file:` URI whose file does not exist yet, or any non-`file:` URI, is
checked as a rootless standalone source (no project context, no identity).

Diagnostic `data`: `family`, `rangeAbsent`, `sourceBound`, `notes`,
`proposal`, `cause` (direct cause codes/messages). LSP requires a range; a
diagnostic whose canonical range is absent is anchored at `0:0` with
`rangeAbsent: true` - a transport requirement, never a claimed location.
Related locations and the direct cause's location appear as
`relatedInformation`. A check that could not run is published as one
`hostFailure` diagnostic without a code, so the editor never looks clean
while `smc check` fails.

### Determinism, staleness, cancellation

- Messages are processed strictly in arrival order; every request is
  answered before the next message is read. Identical input streams produce
  byte-identical output streams.
- `didChange` must carry exactly one full-document change and a version
  newer than the stored one. A stale or incremental change is rejected with
  a `window/logMessage` and changes nothing (no republish).
- `$/cancelRequest` can only name an already-answered request: no-op.
- `didClose` publishes an empty diagnostic list for the document (and for
  any other URI only it contributed to).

### Protocol errors

| Input | Result |
|---|---|
| body not valid JSON / not UTF-8 | `-32700`, session continues |
| body not an object, missing `jsonrpc: "2.0"`, invalid id | `-32600`, session continues |
| request before `initialize` | `-32002` |
| request after `shutdown` | `-32600` |
| unknown method | `-32601` |
| missing/invalid `Content-Length`, non-CRLF header, truncated body, body over 64 MiB | session ends (exit 1) |
| header line over 1 KiB (`MAX_HEADER_LINE_BYTES`), or headers over 8 KiB in total (`MAX_HEADER_BYTES`) | session ends (exit 1); at most one maximal line is buffered before rejection |
| `exit` after `shutdown` / without it / end of input | exit 0 / 1 / 1 |

### Deferred, with deterministic rejection

- Hover, go-to-definition, document symbols: no canonical symbol/definition
  span data exists yet; the server does not advertise them and answers
  `-32601`. No parallel symbol engine is built (#1580 step 7).
- Code actions: `FixProposal` carries guidance only, no machine edit; not
  advertised, `-32601` (#1580 step 9, "no silent source mutation").
- Incremental sync, pull diagnostics, TCP/pipe transports: not supported.

## Formatter contract (AC3)

`smc fmt [--check] <path>` and LSP formatting both use
`smc_cli::format_source_checked`:

- Normalizes line endings to `\n`, strips trailing spaces/tabs, ends a
  non-empty file with exactly one `\n`; nothing else.
- A change is applied only if the canonical lexer yields the identical
  `(kind, text)` token sequence before and after (the end-of-file run of
  blank-line newline tokens aside); otherwise the source is **refused**, not
  rewritten (for example a carriage return inside a string literal, or a
  source that does not lex).
- `smc fmt` formats (or refuses) every file before writing any; a refusal
  writes nothing.
- Deterministic and idempotent: `fmt(fmt(x)) == fmt(x)`.

## Documented editor path: VS Code (generic LSP client)

Any LSP client that can launch a stdio server works; the protocol-generic
recipe is "run `smc lsp` in the project directory for `*.sm` files".

VS Code, with a minimal generic client extension (for example one built on
`vscode-languageclient`):

```json
{
  "serverOptions": {"command": "smc", "args": ["lsp", "--stdio"]},
  "clientOptions": {"documentSelector": [{"scheme": "file", "language": "semantic"}]}
}
```

Neovim (built-in client):

```lua
vim.lsp.start({ name = "semantic", cmd = { "smc", "lsp" }, root_dir = vim.fs.root(0, { "Semantic.package", "semantic.toml" }) })
```

The qualified evidence for this path is the protocol-level suite
(`tests/ssf09_editor_baseline.rs`), including
`smc_lsp_binary_speaks_the_protocol_over_stdio`, which drives the real `smc`
binary over stdio exactly as an editor does. Editor-specific extensions are
not shipped by this repository.
