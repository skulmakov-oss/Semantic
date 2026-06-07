# boundary_alias_import

- purpose: intentional boundary example for the still-blocked top-level alias
  import form on the executable path
- demonstrates:
  - `Import "helper.sm" as Helper`
  - current executable-path narrowing
- note:
  - this is not a supported workflow in the current baseline
  - future support requires an explicit language/source-admission change
- commands:
  - `cargo run --bin smc -- check examples/canonical/boundary_alias_import/src/main.sm`
- expected output:
  - `check` fails
  - the diagnostic contains:
    `top-level executable Import currently admits direct local-path helper-module imports plus selected imports in wave2`
