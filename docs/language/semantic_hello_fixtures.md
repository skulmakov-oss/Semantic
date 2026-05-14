# Semantic Hello Fixtures

Status: pending fixture registry for `#477`

## 1. Purpose

This document registers pending Hello grammar-slice fixtures.

## 2. Status

- pending only
- not executable yet
- not part of accepted test truth
- not wired into cargo tests as passing behavior
- implementation later must decide how to admit or reject them

## 3. Fixture Table

| fixture | intended future class | expected future outcome | reason | current status |
|---|---|---|---|---|
| `tests/fixtures/pending/hello/positive_hello_verbose_directional.sm` | positive pending Hello grammar slice | accepted once grammar / sema / verifier / runtime support exists | records the recommended canonical direction | pending |
| `tests/fixtures/pending/hello/positive_hello_minimal_observe_directional.sm` | possible secondary onboarding shape | possibly accepted later, but not canonical architecture-bearing proof | useful as a lighter shape, but incomplete | pending |
| `tests/fixtures/pending/hello/negative_hello_print_legacy_canonical.sm` | rejected legacy Hello sketch | rejected as canonical / bridge-only if ever executable | legacy `fn main` / `print` / `return` shape | pending |
| `tests/fixtures/pending/hello/negative_hello_observe_non_text_payload.sm` | negative pending Hello rejection fixture | rejected until first slice only admits text literal observation payload | tests observation payload boundary | pending |
| `tests/fixtures/pending/hello/negative_hello_require_side_effect_shape.sm` | negative pending Hello rejection fixture | rejected until requirement is side-effect-free | tests requirement boundary | pending |
| `tests/fixtures/pending/hello/negative_hello_general_io_shape.sm` | negative pending Hello rejection fixture | rejected until general I/O is explicitly scoped | tests that generic I/O is not canonical Hello observation | pending |

## 4. Admission Boundary

- positive fixtures are not accepted until parser / sema / verifier / runtime
  support exists
- negative fixtures define future rejection intent
- no current compiler behavior is changed
- no existing test expectation is changed

## 5. Relationship to `#477`

- prepares `#477`
- does not close `#477`
- next implementation planning still required

## 6. Acceptance Checklist

- pending fixture directory added
- positive pending fixtures added
- negative pending fixtures added
- docs registry added
- fixtures not wired into passing cargo tests
- no parser / typechecker changes
- no grammar implementation
- no runtime / verifier / capability changes
- no Hello World implementation
- `#477` remains open
