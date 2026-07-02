# text_collections_toolbox

- benchmark-class application anchor: practical toolbox example for the admitted
  control-flow, text, collection, and stdlib surface
- purpose: show a realistic small-program workflow over `Sequence(i32)`,
  `Map(i32, text)`, `text + text`, `to_text`, and branch selection
- demonstrates:
  - `len`, `push`, `prepend`, `contains`
  - `map_empty`, `map_set`, `map_get`
  - `to_text` and `text + text`
  - explicit control-flow branching over collection-derived state
  - assert-based proof
- commands:
  - `cargo run --bin smc -- check examples/canonical/text_collections_toolbox/src/main.sm`
  - `cargo run --bin smc -- run examples/canonical/text_collections_toolbox/src/main.sm`
  - `cargo run --bin smc -- compile examples/canonical/text_collections_toolbox/src/main.sm -o out.smc`
  - `cargo run --bin smc -- verify out.smc`
- expected output:
  - `check` succeeds
  - `run` exits successfully
  - `verify` accepts the compiled `.smc`
- non-goals:
  - no host effects
  - no package or release packaging work
