# option_result_control_flow probe audit

Command:

```bash
cargo run --bin smc -- check examples/pcc_candidates/option_result_control_flow/src/main.sm
```

Observed result:

- `Option(T)` type surface: admitted
- `Result(T, E)` type surface: admitted
- `Option::Some` / `Option::None`: admitted
- `Result::Ok` / `Result::Err`: admitted
- `match` over `Option` / `Result`: admitted
- fallback `_` arm: required by current parse contract
- `smc check`: passed for the probe sample

Notes:

- matching without `_` failed to parse in the current surface
- the probe remains a PCC candidate until promoted deliberately

Conclusion:

The `Option` / `Result` contour is currently admitted enough to justify a
canonical-safe sample, but the probe itself should remain tracked as a PCC
candidate artifact.
