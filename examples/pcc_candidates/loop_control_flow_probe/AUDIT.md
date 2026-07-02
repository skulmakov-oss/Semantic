# loop_control_flow_probe audit

Command:

```bash
cargo run --bin smc -- check examples/pcc_candidates/loop_control_flow_probe/src/main.sm
```

Observed result:

- `while condition`: admitted
- `loop` body: admitted as statement-loop surface
- `break;` inside `loop`: admitted
- `continue;` inside `while`: admitted
- mutable rebinding inside loop bodies: admitted
- terminal return paths after loops: admitted
- `smc check`: passed for the probe sample

Notes:

- the probe uses the admitted statement-loop surface only
- it does not rely on `break expr;`
- it does not use speculative loop-expression sugar

Conclusion:

The current practical contour supports `while`, `loop`, `break`, and
`continue` well enough for a canonical-safe candidate sample. This probe can
now be used as evidence for `PCC-CF-4` follow-up work.

Promoted to canonical:

- `examples/canonical/loop_control_flow/`
