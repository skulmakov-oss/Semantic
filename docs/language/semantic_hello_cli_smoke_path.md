# Semantic Hello CLI Smoke Path

Status: CLI smoke-path contract note for `#477`

## 1. Purpose

This document defines the CLI smoke path for controlled text observation after
verifier / VM route / capability / audit are accepted.

It does not define general stdout, formatting, file I/O, stdin, network, debug
output, or README / user-facing promotion.

## 2. CLI Smoke Model

```text
source file
  -> smc check
  -> smc compile
  -> smc verify
  -> smc run / run-smc
  -> verified VM execution
  -> controlled observation event
  -> capability allow
  -> audit decision
  -> CLI observation sink
```

The CLI must only render the observation after the full controlled route
succeeds.
The CLI must not synthesize the output independently.

## 3. CLI Output Contract

```text
input observation class: ControlledText
allowed output: exact controlled text payload
ordering: deterministic sequence order
sink: CLI observation sink only
non-goals: general stdout, formatting, interpolation, file, stdin, network
```

For the current Hello smoke scope:

```text
Expected payload: Hello, World!
Expected behavior: one controlled text observation rendered once
```

This is not a stable public stdout API claim.

## 4. Required Lower-Layer Prerequisites

| Prerequisite | Owner | Required before CLI output |
| --- | --- | --- |
| source check | frontend / checker path | source must pass check |
| compile | emit / SemCode path | source must compile |
| verify | `sm-verify` | SemCode must be admitted |
| VM route | `sm-vm` | controlled observation event produced |
| capability gate | `prom-cap` | explicit controlled sink allow |
| audit policy | `prom-audit` | record / redact / no_store / deny decision |
| CLI sink | `smc-cli` | render only approved observation envelope |

## 5. CLI Denial Matrix

| Case | Expected CLI result |
| --- | --- |
| check fails | no output |
| compile fails | no output |
| verify fails | no output |
| VM route missing | no output |
| capability denied | no output |
| audit denied | no output |
| audit policy missing | no output |
| observation class not ControlledText | no output |
| file / stdin / network target appears | reject / no output |
| formatting / interpolation requested | reject / no output |
| valid controlled text observation approved | render exact payload once |

## 6. Smoke Commands

Target commands:

```bash
cargo run --bin smc -- check examples/hello_world.sm
cargo run --bin smc -- compile examples/hello_world.sm -o hello_world.smc
cargo run --bin smc -- verify hello_world.smc
cargo run --bin smc -- run examples/hello_world.sm
cargo run --bin smc -- run-smc hello_world.smc
```

If this PR is docs-only, these are target commands, not implemented behavior.

## 7. Code Implementation Gate

Code implementation may start only when the lower-layer route is already
implemented and testable.
Until then, this document is the CLI contract for the later implementation PR.

## 8. Owner Boundary

| Concern | Owner | M-HELLO-12A action |
| --- | --- | --- |
| verifier admission | `sm-verify` | prerequisite only |
| VM event route | `sm-vm` | prerequisite only |
| capability gate | `prom-cap` | prerequisite only |
| audit decision / storage policy | `prom-audit` | prerequisite only |
| CLI smoke path | `smc-cli` | define or wire narrow smoke path |
| README / user examples | README / examples | out of scope until 12B |

## 9. Not Implemented

```text
no CLI output implemented
no smc run output claim
no run-smc output claim
no example promotion
no README promotion
no general stdout
no formatting / interpolation
no implicit scalar-to-text conversion
no file / stdin / network
no closure of #477
```

## 10. Next PR Split

```text
12A-code - implement CLI smoke path once lower layers are real
12B - user-facing examples / README promotion only after CLI smoke path passes
```

## 11. Issue State

```text
This document does not close #477.
This document does not satisfy #477 acceptance criteria.
```

## 12. Acceptance Checklist

```text
[ ] CLI smoke-path contract documented
[ ] lower-layer prerequisites explicit
[ ] CLI denial matrix documented
[ ] target smoke commands documented without claiming they pass
[ ] owner boundary correct
[ ] no fake output path exists
[ ] #477 remains open
```

See also: [`semantic_hello_implementation_closeout.md`](semantic_hello_implementation_closeout.md)

