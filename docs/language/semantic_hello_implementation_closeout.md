# Semantic Hello Implementation Closeout

Status: closeout draft for `#477`

See also:

- [`semantic_hello_cli_smoke_path.md`](semantic_hello_cli_smoke_path.md)
- [`semantic_hello_real_semcode_encoding.md`](semantic_hello_real_semcode_encoding.md)
- [`semantic_hello_runtime_sink.md`](semantic_hello_runtime_sink.md)
- [`semantic_hello_audit_event.md`](semantic_hello_audit_event.md)
- [`semantic_hello_policy_fixtures.md`](semantic_hello_policy_fixtures.md)
- [`semantic_hello_verifier_admission.md`](semantic_hello_verifier_admission.md)

## 1. Purpose

This document closes the skeleton / planning implementation phase for `#477`
and records the remaining blockers before any real production CLI work may
begin.

- docs-only
- no code changes
- no tests
- no fixtures
- no CLI integration
- no runtime output
- no accepted Hello World behavior
- `#477` remains open

## 2. What Is Now Covered

| Area | Status | Evidence |
|---|---|---|
| parser path | covered | `#628` |
| sema path | covered | `#629` |
| IR lowering | covered | `#633` |
| real SemCode-level skeleton | covered | `#649` |
| verifier admission skeleton | covered | `#650` |
| negative verifier coverage | covered | `#651` |
| runtime explicit sink skeleton | covered | `#652` |
| capability-gated route harness | covered | `#653` |
| audit decision harness | covered | `#654` |
| isolated CLI smoke harness | covered | `#655` |

## 3. Current Accepted Boundary

Accepted:

```text
source fixture
→ isolated parser
→ isolated sema
→ isolated lowering
→ typed SemCode-level skeleton
→ isolated verifier admission
→ isolated capability gate
→ isolated explicit sink route
→ isolated audit decision
```

Not accepted:

```text
source
→ smc check
→ smc compile
→ smc verify
→ smc run
→ VM execution
→ user-visible output
```

## 4. Remaining Blockers Before Real CLI

- [ ] Decide real SemCode byte encoding / opcode allocation or admitted typed host-call form
- [ ] Implement real SemCode byte emission for Hello operation sequence
- [ ] Add production verifier admission for real encoded observation operation
- [ ] Connect VM/runtime execution path to explicit observation sink
- [ ] Gate route through production capability/effect policy
- [ ] Decide and implement production audit behavior or explicit audit-deferred policy
- [ ] Add CLI / smc smoke path only after verifier/runtime/capability/audit are accepted
- [ ] Add accepted golden SemCode fixture
- [ ] Add examples/hello_world.sm only when it actually passes check → compile → verify → run
- [ ] Update README only after real behavior exists
- [ ] Ensure negative fixtures reject through production path
- [ ] Keep observation as controlled sink, not stdout/print/general I/O

## 5. Explicitly Blocked Actions

- no README claim
- no example claim
- no `smc run` claim
- no "Hello World works" claim
- no stdout claim
- no print support claim
- no accepted golden SemCode claim
- no closure of `#477`

## 6. Real CLI Readiness Gate

Real CLI PR may start only when:

- real SemCode representation is implemented or explicitly admitted
- production verifier can admit/reject it
- runtime route is explicit sink only
- capability gate is production-wired
- audit policy is production-decided
- negative cases remain rejected
- user-visible output is produced only through the controlled observation sink

## 7. Issue State

- `#477` must remain open after this closeout
- this closeout does not satisfy the acceptance criteria of `#477`
- the closeout only prepares the transition from skeleton harnesses to real implementation PRs

## 8. Suggested Next PR Sequence

```text
M-HELLO-10A — semcode: assign controlled observation opcode/encoding decision or typed host-call decision
M-HELLO-10B — semcode: emit real Hello SemCode bytes behind gated path
M-HELLO-10C — verify: admit real controlled observation encoding
M-HELLO-10D — tests: reject real stdout/print/general I/O encodings
M-HELLO-11A — runtime/vm: execute admitted controlled observation to explicit sink
M-HELLO-11B — capability: wire production observation route gate
M-HELLO-11C — audit: wire production observation audit/deferred policy
M-HELLO-12A — cli: add smc check/compile/verify/run smoke path
M-HELLO-12B — examples/docs: add real hello example after behavior is accepted
M-HELLO-12C — close #477 only after acceptance criteria pass
```

## 9. Acceptance Checklist

- [ ] skeleton phase coverage table added
- [ ] accepted vs not accepted boundary documented
- [ ] real CLI blockers listed
- [ ] blocked claims listed
- [ ] readiness gate documented
- [ ] next implementation sequence proposed
- [ ] docs-only
- [ ] no code/test/fixture changes
- [ ] no CLI/smc integration
- [ ] no runtime output
- [ ] no accepted Hello World behavior
- [ ] `#477` remains open

