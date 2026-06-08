# data_audit_record_iterable

- benchmark-class application anchor: deterministic data-processing / record
  workflow on the admitted surface
- purpose: data-heavy audit pass over direct-record `Iterable` dispatch
- demonstrates:
  - direct-record `Iterable` impls
  - `for value in samples`
  - immutable record update with `with { ... }`
  - boolean accumulation over record data
  - assert-based proof
- commands:
  - `cargo run --bin smc -- check examples/canonical/data_audit_record_iterable/src/main.sm`
  - `cargo run --bin smc -- run examples/canonical/data_audit_record_iterable/src/main.sm`
  - `cargo run --bin smc -- compile examples/canonical/data_audit_record_iterable/src/main.sm -o out.smc`
  - `cargo run --bin smc -- verify out.smc`
- expected output:
  - `check` succeeds
  - `run` exits successfully
  - `verify` accepts the compiled `.smc`
- non-goals:
  - no host effects
  - no UI
  - no package or release packaging work
