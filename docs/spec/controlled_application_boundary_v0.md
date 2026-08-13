# Controlled Application Boundary v0

Status: Foundation candidate; not Published Stable

Contract ID: `semantic.foundation.application/0.1`

This contract admits a small, verifier-first host boundary for ordinary
command-line and file-transform programs. It does not grant ambient host
access. Source syntax requests an operation, the SemCode header declares the
required bit, the verifier admits the artifact, the runtime manifest grants or
denies the exact capability, and only then may the host adapter act.

## Capability catalog

| Capability ID | Source builtin | Input | Result | Class |
|---|---|---|---|---|
| `args.read` | `args_read(index)` | `u32` | `text` | captured observation |
| `stdin.read_text` | `stdin_read_text()` | none | `text` | captured observation |
| `stdout.write` | `stdout_write(text)` | `text` | `unit` | host write |
| `stderr.write` | `stderr_write(text)` | `text` | `unit` | host write |
| `path.inspect` | `path_inspect(path)` | relative `text` | `bool` | captured observation |
| `fs.read` | `fs_read_text(path)` | relative `text` | UTF-8 `text` | captured observation |
| `fs.write` | `fs_write_text(path, text)` | relative `text`, UTF-8 `text` | `unit` | host write |
| `time.duration` | `time_duration_ms()` | explicit injected value | `u32` milliseconds | captured observation |

The dotted names are capability identities. Foundation Source 1.1 remains
flat and Rust-like, so they are not source namespaces.

Every source builtin identifier in the capability catalog is reserved for the
application boundary. A function declaration using one of these identifiers is
rejected before lowering, so an ordinary local call cannot acquire host
authority through a name collision.

## Profiles

| Profile | Grants |
|---|---|
| `Pure` | none of the application capabilities |
| `CliReadOnly` | args, stdin, stdout, stderr, path inspection, file read, explicit duration |
| `CliFileTransform` | `CliReadOnly` plus file write |
| `UiBounded` | controlled observation plus explicit duration; catalogued only, not a CLI execution mode |

Profiles build a deny-by-default `CapabilityManifest`. A program request,
SemCode capability bit, package declaration, or UI intent is never a grant.

## Path and write policy

- Every filesystem path is relative to one canonical root supplied by the
  caller.
- Absolute paths, parent traversal, root/prefix components, symlinks, Windows
  reparse points, and resolved root escape are denied.
- Read targets must already exist. Write parents must already exist and resolve
  inside the root.
- Text observations and writes are limited to 16 MiB per call. Path inputs are
  limited to 4096 UTF-8 bytes.
- At least one host-bound observation must be captured before any stdout,
  stderr, or filesystem write.
- Network, child process, ambient environment, wall-clock, and unrestricted
  filesystem operations are absent.

## Determinism, replay, and audit

Args, stdin, path results, file contents, and duration are host-bound
observations. A replay implementation must supply their captured values in the
same call order. `time.duration` never reads wall-clock time; the CLI exposes it
only through `--duration-ms`.

The CLI emits ordered audit records for admitted and denied operations under
`semantic.foundation.application.audit/0.1`. Records contain sequence,
capability ID, byte length, and deterministic path/payload hashes. Raw paths,
arguments, stdin, file contents, and output payloads are not written to the
audit stream. A denied operation has `decision=deny`, no payload hashes, and is
also returned as a structured runtime or ABI error. Records for any preceding
admitted operations are still emitted.

Application-controlled `stderr_write` payload and the CLI's own audit records
share process stderr, so untrusted application text is never written raw.
Every physical line the application writes to stderr is prefixed with
`application| ` and always newline-terminated, regardless of whether the
application's own text ended in a newline. Audit records always start with
`schema=` and are never prefixed that way. This keeps the two evidence
classes unambiguous even if application text embeds newlines or literally
contains audit-schema-looking text, and prevents an unterminated application
write from concatenating onto a following audit line.

## CLI contour

The existing one-argument `smc run <input>` behavior remains the controlled
observation path. Explicit application execution uses:

```text
smc run <input.sm|project-root> \
  --profile <pure|cli-read-only|cli-file-transform> \
  --root <directory> \
  [--duration-ms <u32>] \
  [-- <application-args...>]
```

`examples/qualification/ssf04_file_transform.sm` is the canonical
args -> read -> transform -> write flow.

## Authority and exclusions

`HostCallId`, `CapabilityKind`, SemCode capability bits, verifier admission,
VM capability checks, and `ApplicationHostAbi` are the authority chain.
`PrometheusHostAbi` v1 is unchanged. `print(text)` remains the separate
controlled-observation operation and is not an alias for `stdout.write`.

This phase adds no UI behavior, network/process/environment access, package
grant semantics, Stable promotion, or release publication.
