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
    `top-level executable Import admits direct local-path and package-qualified helper modules plus selected local imports`
