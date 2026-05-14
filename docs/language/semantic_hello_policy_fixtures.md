# Semantic Hello Policy Fixtures

Status: pending policy fixture registry for `#477`

## 1. Purpose

This document registers pending verifier / capability / runtime / audit policy fixtures for Hello controlled observation.

## 2. Status

- pending only
- declarative planning fixtures
- not executable
- not consumed by production verifier / runtime
- not accepted runtime truth
- no verifier / capability / runtime / audit implementation exists here

## 3. Fixture Table

| fixture | class | expected future outcome | reason | current status |
|---|---|---|---|---|
| `positive_observation_policy_admitted.toml` | positive policy case | admit | all required admission conditions are present | pending |
| `negative_missing_observation_capability.toml` | capability denial | deny | missing_observation_capability | pending |
| `negative_stdout_fallback.toml` | runtime shortcut rejection | deny | stdout_not_default_sink | pending |
| `negative_missing_audit_when_required.toml` | audit denial | deny | audit_required_but_unavailable | pending |
| `negative_nondeterministic_sink.toml` | determinism denial | deny | nondeterministic_sink_configuration | pending |

## 4. Policy Coverage

- verifier admission: `docs/language/semantic_hello_verifier_admission.md`
- capability requirement: `docs/language/semantic_hello_observation_policy.md`
- runtime sink model: `docs/language/semantic_hello_runtime_sink.md`
- audit requirement: `docs/language/semantic_hello_audit_event.md`
- deterministic ordering: `docs/language/semantic_hello_runtime_sink.md`

## 5. Boundary

- no verifier implementation
- no capability implementation
- no runtime / sink implementation
- no audit implementation
- no SemCode / opcode changes
- no CLI integration
- no accepted runtime behavior

## 6. Relationship to `#477`

- prepares `#477`
- does not close `#477`
- future implementation must decide how these fixture expectations become executable tests
