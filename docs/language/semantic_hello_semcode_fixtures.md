# Semantic Hello SemCode Fixtures

Status: pending conceptual fixture registry for `#477`

## 1. Purpose

This document registers conceptual SemCode planning fixtures for Hello.

## 2. Status

- pending only
- conceptual only
- not executable
- not final opcode names
- not bytecode format
- not accepted golden SemCode
- not runtime truth
- no emitter exists yet

## 3. Fixture Table

| fixture | source shape | intended future class | current status | reason |
|---|---|---|---|---|
| `tests/fixtures/pending/hello_semcode/positive_hello_verbose_conceptual.semcode.txt` | canonical verbose Hello IR shape | conceptual SemCode planning fixture | pending | records the intended future conceptual sequence without committing to opcodes or bytecode |

## 4. Conceptual Sequence

- `declare_local_quad boot = T`
- `require_quad_eq boot T`
- `request_observation_text "Hello, World!"`
- `complete_quad T`

## 5. Boundary

- no SemCode emission
- no opcode implementation
- no verifier admission
- no VM/runtime behavior
- no capability/effect admission
- no audit implementation
- no CLI pipeline integration
- no accepted golden SemCode

## 6. Relationship to `#477`

- prepares `#477`
- does not close `#477`
- future emitter skeleton must treat this as conceptual planning guidance only
