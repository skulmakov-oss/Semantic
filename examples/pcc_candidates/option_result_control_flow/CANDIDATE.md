# option_result_control_flow probe notes

Command:

```bash
cargo run --bin smc -- check examples/pcc_candidates/option_result_control_flow/src/main.sm
```

Observed admitted surface:

- Option type form: `Option(T)`
- Result type form: `Result(T, E)`
- constructors: namespace form
- match fallback `_`: required for the current probe sample to parse cleanly

Conclusion:

`Option` / `Result` control flow is admitted enough for probe-level use, but
the sample remains a PCC candidate until the rest of the practical contour is
confirmed.
