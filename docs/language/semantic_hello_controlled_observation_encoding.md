# Semantic Hello Controlled Observation Encoding

Status: decision document for `#477`

See also:

- [`semantic_hello_real_semcode_encoding.md`](semantic_hello_real_semcode_encoding.md)
- [`semantic_hello_implementation_closeout.md`](semantic_hello_implementation_closeout.md)

## 1. Purpose

This document decides the controlled observation encoding strategy for future
real Hello SemCode.

- docs-only
- no opcode implementation
- no numeric opcode ID assignment
- no bytecode format change
- no encoder / verifier / VM / runtime / capability / audit / CLI behavior
  change
- `#477` remains open

## 2. Decision

```text
Decision:
Use a dedicated controlled observation encoding form for Hello text observation.

Symbolic operation name:
OBSERVE_TEXT_LITERAL

Surface / skeleton spelling:
observe_text_literal "Hello, World!"
```

Clarifications:

- `OBSERVE_TEXT_LITERAL` is a symbolic reserved operation name only.
- No numeric opcode ID is assigned here.
- It is not executable in this PR.
- It is not accepted golden SemCode.
- It is not stdout.
- It is not print.
- It is not generic I/O.
- It is not a general host call.

## 3. Why Not Generic Host-Call

| Option | Decision | Reason |
|---|---|---|
| dedicated controlled observation encoding | selected | keeps observation semantically distinct from generic host calls |
| typed admitted host-call | deferred fallback only | reserved as a later compatibility fallback, not the primary contract |
| generic host-call | rejected | widens the boundary and makes verifier admission less precise |
| stdout encoding | rejected | collapses observation into host output |
| print encoding | rejected | canonizes legacy output vocabulary |
| general I/O encoding | rejected | expands the runtime surface beyond controlled observation |

## 4. Encoding Shape

Future conceptual encoding fields, without byte layout:

```text
OBSERVE_TEXT_LITERAL {
    text_const_ref: ConstRef<Text>,
    observation_class: ControlledText,
    sequence_index: SequenceIndex,
    policy_ref: ObservationPolicyRef,
}
```

Clarifications:

- field names are conceptual
- not final binary layout
- no byte offsets
- no numeric opcode IDs
- `text_const_ref` must refer to a deterministic text literal
- `observation_class` must be controlled text only
- `sequence_index` preserves deterministic ordering
- `policy_ref` links verifier / capability / audit policy later

## 5. Const / Data Boundary

- payload is a text literal const entry
- no interpolation
- no formatting
- no implicit scalar-to-text conversion
- no host-dependent generation
- no stdout / terminal source
- no clock / random / env dependency

## 6. Verifier Contract

Future verifier must reject:

- generic host-call observation
- stdout / print / io.write / file / network / stdin encodings
- non-text payload
- text payload not in const table
- missing policy metadata if required
- missing deterministic sequence index
- operation order violations
- observation without preceding requirement
- observation after completion
- bytecode / opcode shape mismatch

No verifier implementation is added here.

## 7. Runtime / Capability / Audit Contract

Future runtime:

- routes admitted operation only to explicit observation sink
- never defaults to stdout
- does not expose general I/O

Future capability:

- gates observation sink route through production policy
- does not reuse broad host-call capability

Future audit:

- records or defers controlled observation event by explicit policy
- payload identity should use hash / ref shape, not raw host-output stream

No runtime / capability / audit implementation is added here.

## 8. Rejected Encodings

| Rejected form | Reason |
|---|---|
| `HOST_CALL stdout` | collapses controlled observation into host output |
| `HOST_CALL print` | canonizes legacy print vocabulary |
| `HOST_CALL io.write` | widens the observation surface into generic output APIs |
| generic host-call with string payload | loses the controlled observation boundary |
| function return value as observation | changes observation into result flow |
| debug / log channel | is not a controlled observation sink |
| terminal / stdout byte stream | is host output, not explicit observation |
| file / network / stdin route | expands beyond controlled observation |
| implicit print lowering | hides the semantic contract |
| reused unrelated opcode | causes silent contract drift |

## 9. Migration From Skeleton Spelling

```text
Skeleton:
observe_text_literal "Hello, World!"

Future symbolic operation:
OBSERVE_TEXT_LITERAL(text_const_ref, ControlledText, sequence_index, policy_ref)
```

Clarifications:

- skeleton tests stay valid as high-level shape
- future implementation will add real encoding tests
- existing skeleton harnesses are not accepted runtime behavior

## 10. Remaining Blockers After This Decision

- allocate numeric opcode ID or equivalent typed encoding ID
- implement byte emission
- update SemCode docs/spec when binary layout is real
- add production verifier admission
- add VM/runtime dispatch to explicit sink
- wire capability gate
- wire audit or audit-deferred policy
- add accepted golden SemCode
- add CLI/smc smoke path
- update examples/README only after real behavior exists
- keep `#477` open

## 11. M-HELLO-10B Boundary Note

- gated provisional byte-emission skeleton exists
- representation is deterministic but not production SemCode
- `OBSERVE_TEXT_LITERAL` remains symbolic / not a numeric opcode ID
- no stable bytecode format is claimed
- no production encoder integration
- no verifier / VM / runtime / capability / audit / CLI behavior
- no accepted golden SemCode
- `#477` remains open

## 12. M-HELLO-10C Bridge Note

- bridge API now makes the handoff from the Hello real SemCode skeleton to the gated provisional observation bytes explicit
- the bridge remains provisional and non-production
- the bridge preserves the same canonical validation rules as M-HELLO-10B
- no final numeric opcode ID is assigned
- no stable bytecode format is claimed
- no production encoder integration
- no verifier / VM / runtime / capability / audit / CLI behavior
- no accepted golden SemCode
- `#477` remains open

## 13. Issue State

- this PR does not close `#477`
- this PR does not satisfy `#477` acceptance criteria
- this PR only removes the design fork between dedicated encoding and host-call fallback

## 14. Acceptance Checklist

- [ ] dedicated encoding decision recorded
- [ ] typed host-call fallback documented but not selected
- [ ] stdout / print / generic I/O rejected
- [ ] conceptual encoding fields documented
- [ ] verifier contract documented
- [ ] runtime / capability / audit contract documented
- [ ] remaining blockers listed
- [ ] docs-only
- [ ] no code / test / fixture changes
- [ ] no numeric opcode IDs
- [ ] no bytecode format changes
- [ ] no accepted runtime behavior
- [ ] `#477` remains open
