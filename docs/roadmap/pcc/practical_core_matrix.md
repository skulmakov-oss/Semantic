# PCC Practical Core Matrix

Status: working PCC matrix
Scope: documentation / qualification map only

This matrix is a preparation artifact for Core Trust Freeze planning. It is
honest about qualified behavior, conservative fallbacks, partial proof, and
deferred work.

Core Trust Boundary Repair v1 is closed, but Core Trust Freeze is not
automatically closed.

SemCode format authority is already split:

- `sm-format` owns SemCode format and decode;
- `sm-verify` owns structural admission;
- `sm-vm` owns deterministic verified execution.

Runtime ownership is conservatively qualified, not symbolically precise.

## Main Matrix

| Feature family | Parse | Typecheck | Lowering | SemCode | Verify | VM/runtime | Ownership contour | Tests/evidence | Status | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `fn` / `let` / `return` / `if` / `else` | present | present | present | present | present | present | n/a | `docs/spec/source_semantics.md`; `docs/status/feature_maturity_matrix.md`; `tests/pcc1_control_flow_gate.rs`; `tests/pcc1_control_flow_lowering_stability.rs` | READY | Core executable surface; not a Core Trust Freeze claim. |
| `while` / `loop` / `break` / `continue` | present | present | present | present | present | present | n/a | `docs/spec/source_semantics.md`; `docs/status/feature_maturity_matrix.md`; `tests/pcc1_control_flow_gate.rs`; `tests/pcc1_control_flow_diagnostics.rs` | READY | Practical control-flow surface is qualified, but broad language freeze remains separate. |
| `bool` / `quad` / `unit` | present | present | present | present | present | present | n/a | `docs/spec/source_semantics.md`; `docs/status/feature_maturity_matrix.md` | READY | Native value domain is established; `quad` preserves conflict as a first-class state. |
| `i32` / `u32` | present | present | present | present | present | present | n/a | `docs/spec/source_semantics.md`; `docs/status/feature_maturity_matrix.md`; `tests/pcc2_numeric_core_gate.rs` | READY | Integer family is qualified for the practical core contour. |
| `f64` / `fx` | present | present | present | present | present | present | n/a | `docs/spec/source_semantics.md`; `docs/status/feature_maturity_matrix.md`; numeric acceptance tests | READY | Qualified arithmetic surface exists, but broader mixed-numeric guarantees remain bounded. |
| `text` / `string` | present | present | present | present | present | present | n/a | `docs/spec/source_semantics.md`; `docs/status/feature_maturity_matrix.md`; `tests/pcc3_text_core_gate.rs` | READY | Text concatenation / `to_text` are qualified; host-facing text ABI remains separate. |
| `records` | present | present | present | present | present | present | record field ownership qualified | `docs/roadmap/pcc/record_field_ownership_matrix.md`; `docs/roadmap/pcc/runtime_ownership_conservative_contour_closeout.md`; `tests/record_field_ownership_golden.rs` | READY | Direct named record fields are qualified end-to-end. |
| `tuples` | present | present | present | present | present | present | tuple index ownership qualified | `docs/roadmap/pcc/tuple_ownership_matrix.md`; `docs/roadmap/pcc/runtime_ownership_conservative_contour_closeout.md`; `tests/tuple_ownership_golden.rs` | READY | Direct tuple element paths are qualified end-to-end. |
| `ADT` / enum constructors | present | present | present | present | present | present | ADT payload ownership qualified | `docs/roadmap/pcc/adt_payload_ownership_matrix.md`; `docs/architecture/adt_payload_ownership_paths.md`; `tests/pcc5_adt_acceptance.rs`; `tests/pcc5_match_acceptance.rs` | READY | Enum construction and payload handling are qualified within the current ADT slice. |
| `ADT payload access` | present | present | present | present | present | present | variant + payload-index ownership qualified | `docs/roadmap/pcc/adt_payload_ownership_matrix.md`; `docs/architecture/adt_payload_ownership_paths.md`; `tests/runtime_ownership_e2e.rs` | READY | Payload path is variant-qualified; different variants may be disjoint. |
| `sequences / arrays / collections` | present | present | present | present | present | present | sequence ownership is conservative where dynamic indices appear | `docs/roadmap/pcc/sequence_conservative_ownership_contour_closeout.md`; `docs/roadmap/pcc/sequence_ownership_contract_audit.md`; `tests/pcc7_sequence_acceptance.rs` | PARTIAL | Runtime sequence support exists; ownership is qualified only for the conservative contour. |
| `maps` | present | present | present | present | present | present | n/a | `tests/pcc7_map_acceptance.rs`; `tests/pcc7_collections_diagnostics.rs`; `docs/status/feature_maturity_matrix.md` | PARTIAL | Map behavior is practical and test-backed, but broader collection semantics remain bounded. |
| Record field ownership | present | present | present | present | present | present | precise for direct named fields | `docs/roadmap/pcc/record_field_ownership_matrix.md`; `tests/runtime_ownership_e2e.rs`; `tests/record_field_ownership_golden.rs` | READY | Same-field, sibling-field, and parent/child overlap are qualified. |
| Tuple index ownership | present | present | present | present | present | present | precise for direct tuple elements | `docs/roadmap/pcc/tuple_ownership_matrix.md`; `tests/runtime_ownership_e2e.rs`; `tests/tuple_ownership_golden.rs` | READY | Direct tuple element paths are qualified. |
| ADT payload ownership | present | present | present | present | present | present | precise by variant + payload index | `docs/roadmap/pcc/adt_payload_ownership_matrix.md`; `docs/architecture/adt_payload_ownership_paths.md`; `tests/runtime_ownership_e2e.rs` | READY | Variant-qualified payload paths are qualified. |
| Static sequence index ownership | present | present | present | present | present | present | precise static `SequenceIndexStatic(u32)` | `docs/roadmap/pcc/sequence_conservative_ownership_contour_closeout.md`; `tests/runtime_ownership_e2e.rs`; `tests/sequence_ownership_golden.rs` | READY | Static `seq[0]` / `seq[1]` paths are qualified. |
| Dynamic sequence index fallback | present | present | present | present | present | present | conservative `seq[i] -> seq` | `docs/roadmap/pcc/sequence_dynamic_ownership_contract_audit.md`; `docs/roadmap/pcc/sequence_symbolic_dynamic_ownership_contract_audit.md`; `docs/roadmap/pcc/sequence_conservative_ownership_contour_closeout.md`; `tests/runtime_ownership_e2e.rs` | CONSERVATIVE | Safe fallback is qualified; it is intentionally over-approximated. |
| Symbolic dynamic sequence ownership | deferred | deferred | deferred | deferred | deferred | deferred | deferred | `docs/roadmap/pcc/sequence_symbolic_dynamic_ownership_contract_audit.md` | DEFERRED | No `SequenceIndexDynamic` contract is active yet. |
| Range ownership | deferred | deferred | deferred | deferred | deferred | deferred | deferred | `docs/roadmap/pcc/sequence_dynamic_ownership_contract_audit.md`; `docs/roadmap/pcc/sequence_symbolic_dynamic_ownership_contract_audit.md` | DEFERRED | Range / region precision is explicitly postponed. |
| Iterator ownership | deferred | deferred | deferred | deferred | deferred | deferred | deferred | `docs/roadmap/pcc/sequence_dynamic_ownership_contract_audit.md`; `docs/roadmap/pcc/sequence_symbolic_dynamic_ownership_contract_audit.md` | DEFERRED | Iterator cursor ownership is not modeled yet. |
| Advanced alias reasoning | deferred | deferred | deferred | deferred | deferred | deferred | deferred | `docs/roadmap/pcc/sequence_symbolic_dynamic_ownership_contract_audit.md`; `docs/roadmap/pcc/runtime_ownership_conservative_contour_closeout.md` | DEFERRED | No full symbolic alias analysis is claimed. |
| Imports | present | present | present | n/a | n/a | n/a | n/a | `docs/imports.md`; `docs/spec/source_semantics.md`; `tests/canonical_examples.rs`; `tests/pcc9_project_model_acceptance.rs` | READY | Direct local-path helper imports and selected imports are qualified. |
| Exports | partial | partial | partial | n/a | n/a | n/a | n/a | `docs/imports.md`; `docs/spec/source_semantics.md` | PARTIAL | Export sets exist, but broader export/re-export semantics remain bounded. |
| Project-root compilation | present | present | present | present | present | present | n/a | `docs/roadmap/pcc/cli_public_sample_qualification_matrix.md`; `docs/roadmap/pcc/cli_public_sample_qualification_audit.md`; `tests/canonical_examples.rs`; `tests/pcc9_project_model_acceptance.rs` | READY | `smc check` / `smc compile` over project roots is qualified for the canonical smoke set. |
| CLI `check` / `compile` / `verify` / `run` / `run-smc` path | present | present | present | present | present | present | n/a | `docs/roadmap/pcc/cli_public_sample_qualification_matrix.md`; `tests/cli_public_smoke_matrix.rs`; `tests/canonical_examples.rs` | READY | Public CLI smoke is qualified for selected canonical fixtures only. |
| `schema` | present | present | present | present | present | present | n/a | `docs/spec/types.md`; `docs/spec/source_semantics.md`; `docs/roadmap/language_maturity/generated_api_contract_surface_scope.md`; `docs/roadmap/language_maturity/config_schema_contract_scope.md` | PARTIAL | Compile-time schema contracts exist, but broader schema/boundary-core work is still ongoing. |
| `requires` / `ensures` | present | present | present | present | present | present | n/a | `docs/spec/syntax.md`; `docs/spec/source_semantics.md`; `docs/roadmap/language_maturity/function_contract_invariant_scope.md` | PARTIAL | Narrow function-contract support is real, but not yet a full general proof system. |
| `invariant` | present | present | present | present | present | present | n/a | `docs/spec/syntax.md`; `docs/spec/source_semantics.md`; `docs/roadmap/language_maturity/function_contract_invariant_scope.md` | PARTIAL | First-wave invariant work is scoped, but broader invariant reasoning is deferred. |
| `Logos` / `System` / `Entity` / `Law` surface | present | partial | partial | partial | partial | partial | n/a | `docs/spec/source_semantics.md`; `docs/status/feature_maturity_matrix.md`; `tests/frontend_lexer_qualification.rs` | UNKNOWN | Tokens and lexicon evidence exist, but this refresh does not separately qualify the full surface. |

## Notes

- The matrix deliberately separates qualified behavior from stable-release
  claims.
- `READY` means the feature is qualified for the practical core contour, not
  that Core Trust Freeze is complete.
- `CONSERVATIVE` is reserved for safe over-approximation, currently only the
  dynamic sequence fallback `seq[i] -> seq`.
- `PARTIAL` means some layers work, but the end-to-end claim is still bounded.
- `DEFERRED` means the contract is intentionally postponed.
- `UNKNOWN` means the evidence base was not strong enough to classify the row
  more aggressively in this refresh.
- `OUT OF SCOPE` is reserved for rows that the project explicitly excludes from
  the current phase; this refresh did not need that label for the required rows.

## Explicit Non-Claims

This matrix does **not** claim:

- full language completion;
- full PCC completion;
- Core Trust Freeze completion;
- full dynamic ownership precision;
- symbolic alias analysis;
- range ownership;
- iterator ownership;
- full contract runtime semantics;
- full no_std qualification beyond available evidence.

## Final Position

The current practical core contour is qualified where evidence exists,
conservative where the repository intentionally over-approximates, partial
where pipeline proof is incomplete, deferred where contracts are not yet
defined, and unknown only where the evidence base was not strong enough for a
tighter call.
