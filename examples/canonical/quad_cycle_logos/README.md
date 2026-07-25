# quad_cycle_logos

- purpose: canonical Logos declarative profile example — `System` context,
  `Entity` state/prop fields, and priority-ordered `Law`/`When` rules
- language profile: **Logos declarative surface** (`docs/spec/logos.md`) — this
  is not the Rust-like executable surface and is not combined with one in this
  file
- supported status: **parse-qualified and IR-lowering-qualified only**. It is
  explicitly **not** `check`/`compile`/`verify`/`run`-qualified: those commands
  target the Rust-like SemCode/VM path, which does not accept Logos source.
  Feeding this file to `smc check` produces a Rust-like parser/type diagnostic,
  not a Logos result — that is expected and is exercised by
  `tests/canonical_source_style.rs` as an honesty-boundary check.
- demonstrates (`docs/spec/source_style.md` section B.12):
  - one blank line between `System`, `Entity`, and `Law` blocks
  - 4-space indentation for `Entity` fields and `Law`/`When` clauses even
    though Logos indentation is semantically significant
  - `System` parameters using the parser's actual accepted `name = value` form
  - `Law` declarations are reordered by descending `priority` by the parser,
    independent of source order
- commands:
  - `cargo run --bin smc -- dump-ast examples/canonical/quad_cycle_logos/src/main.sm`
  - `cargo run --bin smc -- dump-ir examples/canonical/quad_cycle_logos/src/main.sm --profile logos`
- expected result:
  - `dump-ast` prints a `LogosProgram` with one `System`, one `Entity`, and one
    `Law` carrying two ordered `When` clauses
  - `dump-ir --profile logos` prints one `LogosIrLaw` for `"CheckSignal"` with
    `when_count: 2`
  - `smc check` / `smc run` on this file fail with a Rust-like frontend
    diagnostic — this is the expected, documented boundary, not a bug
- non-claims:
  - does not execute through SemCode, the verifier, or the VM
  - `When` condition/effect bodies are structured text fragments at this
    stage, not type-checked Rust-like expressions
  - not a claim that `System`/`Entity`/`Law` can appear inside a Rust-like
    `fn main()` program
