# SSF-09 Closeout - Diagnostics and Editor Baseline (`#1580`)

Status: completion evidence for the one-PR `#1580` completion. `#1580` stays
OPEN until that PR is merged under owner MERGE GO; SSF-10 has **not** started.

Base: implemented on `main` at `05613f9d874bc5d7099fcb8f237c3ad76bc8bcde`
(includes PR #1966, the Ledger Phase 1 frontend/semantic-core closure), then
integrated by a normal merge with `main` at
`f6b5f6d60230c7ff2229fef441df14562836fc5a` (PR #1971, which tracks the root
`Cargo.lock`; this PR's lockfile delta is exactly the six authorized
`sm-diagnostic` dependency edges).

Governing records: `ssf09_diagnostic_authority_decision.md` (Decisions A-F),
`ssf09_canonical_carrier_contract.md` (carrier contract + implementation
addendum), `docs/spec/diagnostics_machine_schema_v1.md`,
`docs/spec/editor_lsp_baseline.md`.

## Acceptance criteria

| AC | Criterion | Evidence | Result |
|---|---|---|---|
| AC1 | Machine-readable diagnostics have a versioned stable schema | `semantic.diagnostics` v1 with a written versioning policy; `smc check --format json`; byte-exact goldens (`tests/golden_snapshots/ssf09_editor_baseline/*.json`), `schema_output_is_byte_deterministic`, `schema_header_and_top_level_shape_are_stable`, `host_failure_is_a_schema_document_not_a_diagnostic` | SATISFIED |
| AC2 | Diagnostics preserve code, severity, file and range where source information exists | producer adapters C1B/C2/C3B/C4; `frontend_syntax_error_carries_code_severity_source_and_token_range`, `lexer_error_keeps_lexer_code_and_exact_range`, `logos_grammar_errors_keep_their_own_codes_and_recovered_errors`, `imported_module_failure_is_attributed_to_that_module`, `unprovable_ranges_are_absent_never_fabricated`, `retired_generic_code_is_never_emitted`, `utf8_multibyte_prefix_does_not_skew_byte_ranges`, `crlf_source_ranges_stay_exact`, `canonical_json_contains_no_absolute_fixture_path_anywhere`, `canonical_json_is_byte_identical_across_checkout_roots`, `portable_messages_name_modules_by_project_relative_path` | SATISFIED |
| AC3 | Formatter is deterministic and idempotent | `format_source_checked` token-stream proof; `formatter_is_idempotent_and_deterministic_on_all_fixtures` (fixtures + `examples/canonical`), `formatter_preserves_check_result_of_messy_sources`, formatter unit tests (refusals, no partial writes) | SATISFIED |
| AC4 | LSP/editor results derive from canonical tooling and cannot silently disagree with CLI truth | one shared `check_canonical`; `cli_lsp_parity_rootless_matrix`, `cli_lsp_parity_project_matrix`, `lsp_routes_imported_module_diagnostics_to_that_module_uri`, `lsp_formatting_bridge_equals_cli_formatter_including_refusals`, host failures published (never a clean editor while `smc check` fails) | SATISFIED |
| AC5 | Project-root awareness works | identity from package admission; `nested_module_identity_keeps_module_root_relative_path` (M11), `project_root_directory_resolves_like_its_entry`, `lsp_overlay_of_imported_module_changes_importer_result`, `lsp_workspace_folder_change_republishes_deterministically`, `rootless_source_has_no_fabricated_identity` | SATISFIED |
| AC6 | No hidden telemetry or source upload exists | `diagnostic_and_editor_surfaces_have_no_network_or_telemetry_code`, `editor_host_crates_declare_no_network_dependencies`, `lsp_writes_only_protocol_frames_to_its_output`; stdio-only transport | SATISFIED |
| AC7 | At least one documented external editor/protocol path works | `docs/spec/editor_lsp_baseline.md` (protocol-generic, VS Code generic client, Neovim); `smc_lsp_binary_speaks_the_protocol_over_stdio` drives the real binary over stdio; malformed/stale/lifecycle protocol fixtures | SATISFIED |
| AC8 | SSF-10 entry conditions are explicit | this document, section "SSF-10 entry conditions" | SATISFIED |

## Producer / adapter matrix

| Producer | Native authority (repaired) | Canonical adapter | Family | Range |
|---|---|---|---|---|
| `sm-front` lexer | `LexFailure { code, detail, range }` | `FrontendDiagnostic::from_lex_failure` | Frontend | offending bytes |
| `sm-front` RustLike parser | error kind -> `E0005`/`E0006` | `FrontendDiagnostic::from_parse_error` | Frontend | reported token extent |
| `sm-front` Logos parser | `FrontendErrorDetail` (own `E02xx`, bare message, anchor) | `FrontendDiagnostic::from_parse_error` | Frontend | anchor token extent; recovered errors as related |
| `sm-front` surface resolution | `E0007`/`E0008` | `FrontendDiagnostic::surface_resolution` | Frontend | absent (whole input) |
| `sm-front` type checker | `E0201` | `FrontendDiagnostic::from_type_check_error` | Frontend | absent (no position authority) |
| `smc-cli` bundle composition | `E0009` | `bundler_semantic_error` -> `SemanticDiagnostic` | Frontend | absent; source unbound |
| `sm-sema` | existing codes, `DiagLevel` | `SemanticDiagnostic::to_canonical` | Semantic (or relayed Frontend) | reported token extent |
| `sm-verify` | `code_token()` `V0001`-`V0032`, `severity()` | `VerificationDiagnostic::to_canonical`, `RejectReport::to_canonical` | Verification | absent (artifact offset kept as a note) |
| `sm-vm` | explicit admission table `R0001`-`R0030` | `RuntimeError::to_canonical` | Runtime | absent (PC is not a source position) |

## Integration with PR #1966 (base drift)

The earlier desktop implementation commits (`4189f464`, `e877779f`) were not
present in the cloud repository, so this completion was re-implemented
directly on `05613f9d`. PR #1966 is treated as current truth; nothing it
changed was reverted. Overlap handled:

- `crates/sm-front/src/parser.rs`, `crates/sm-front/src/typecheck.rs`
  (#1966: REM-002..REM-006 frontend/typecheck repairs) and
  `crates/sm-ir/src/{hello_ir,legacy_lowering}.rs` (#1966 lowering repairs):
  every #1966 check and its RED-to-GREEN tests are kept intact. This PR
  only adds `detail: None` mechanically at each pre-existing
  `FrontendError` construction site (including the ones #1966 added), the
  structured detail in the two Logos error helpers, `LexFailure`, and the
  diagnostic-authority module. Frontend behaviour and legacy messages are
  unchanged except the lexer's non-ASCII "unexpected character" message,
  which no longer prints a mojibake lead byte.
- `.harness/current.task.yaml`: #1966's doc-envelope authorization is kept
  verbatim; this PR appends the `#1580` completion authorization after it.
- #1966 did not touch `sm-sema`, `sm-verify`, `sm-vm`, `smc-cli`,
  `sm-diagnostic` or `tests/**`; no conflict exists there.
- `crates/sm-sema/src/std_adapters.rs` (not touched by #1966): the
  Decision E/F resolver/admission flow is unchanged; only the diagnostic
  construction was routed through the frontend authority, and module
  provenance is attached to module-local errors.
- `crates/smc-cli/src/app.rs`: the `smc check` dispatch
  (`check_root_with_project_authority`) is unchanged in behaviour; it now
  delegates to `check_root_with_attribution`, which the canonical path shares.

## Re-derived test expectations

These pre-existing expectations pinned behaviour this completion changes by
design (each re-derived from the frozen decisions, not relaxed):

- `E0000` -> producer codes: `crates/sm-sema/src/std_adapters.rs` tests
  (`rustlike_syntax_error_preserves_frontend_source_mark` -> `E0005`,
  `rustlike_policy_error_preserves_frontend_mark_and_kind` -> `E0006`,
  numeric-literal mapping tests -> `E0005`),
  `crates/smc-cli/src/app.rs::lex_failure_check_reports_lexer_e0004_and_direct_e0004_contracts`,
  and `src/bin/smc.rs` 7hell fixtures (`E0005`). Carrier contract section 8.1
  forbids the `E0000` placeholder.
- Module-local errors now carry `provider_module_id`
  (`provider_error_path_does_not_attach_warning_provenance`,
  `assert_module_parse_error_at`): Decision C requires the structurally
  known module identity instead of attribution through message text.
- `tests/dependency_boundaries.rs`: the C0-only "no package depends on
  `sm-diagnostic`" guard became an explicit authorized-consumer allowlist
  (governance authorization recorded in `.harness/current.task.yaml`).

## Mutation campaign

`python3 tests/ssf09_mutation_campaign.py` - each mutant violates one frozen
law and must be killed by a runtime test failure (compile failures do not
count):

| Id | Violated law | Killing test |
|---|---|---|
| M1 | fabricated zero-width range | `point_without_source_token_never_becomes_a_range` |
| M2 | policy violation relabelled as syntax | `rustlike_policy_error_preserves_frontend_mark_and_kind` |
| M3 | adapter rewrites severity | `schema_goldens_rootless` |
| M4 | `Debug`-derived verifier code | `code_tokens_are_unique_and_never_debug_derived` |
| M5 | runtime admits an internal VM fault | `internal_vm_faults_are_not_admitted` |
| M6 | verifier cause truncated | `verifier_rejection_keeps_structured_report_cause` |
| M7 | schema reorders diagnostics | `schema_goldens_rootless` |
| M8 | UTF-8 instead of UTF-16 positions | `lsp_utf16_positions_count_code_units` |
| M9 | stale `didChange` accepted | `lsp_stale_did_change_is_rejected_and_changes_nothing` |
| M10 | LSP formatting bypasses the canonical formatter | `lsp_formatting_bridge_equals_cli_formatter_including_refusals` |
| M11 | module identity reduced to basename | `nested_module_identity_keeps_module_root_relative_path` |
| M12a | editor overlay ignored | `lsp_uses_open_document_overlay_not_disk` |
| M12b | LSP drops another source's diagnostics | `lsp_routes_imported_module_diagnostics_to_that_module_uri` |

Result on the final tree: recorded in the completion PR.

## R3 review repairs (2026-09-30)

The independent R3 adversarial review and the Copilot review of PR #1972
found four defects, all repaired before merge (earlier drafts of this record
understated the first two as P3 limitations):

1. **Host paths in canonical machine output (was blocking).** Import cycle
   (`E0238`), failed import read (`E0239`, including rootless sources),
   missing selected symbol (`E0244`) and executable-bundle composition
   (`E0009`) messages embedded absolute checkout paths and OS error text,
   so the same project produced different JSON per checkout. Now module
   names come from the provider-owned `ModuleProvider::display_module`
   (relative to the checked root's directory) and failures are described by
   structured code / `io::ErrorKind`; the selected-helper prefix is derived
   from the same display name. Evidence:
   `canonical_json_is_byte_identical_across_checkout_roots`,
   `portable_messages_name_modules_by_project_relative_path`,
   `legacy_check_output_keeps_host_detail`.
2. **RustLike helpers ignored editor overlays (was blocking).** The
   executable bundler read helpers straight from disk. All project source
   reads now go through one overlay-first seam (`SourceAccess`). Evidence:
   `lsp_unsaved_rustlike_helper_is_seen_by_its_importer_over_stdio`,
   `lsp_rustlike_overlay_matches_cli_on_the_same_effective_sources`.
3. **Unbounded LSP header reads.** Header lines are capped at 1 KiB and all
   headers at 8 KiB, enforced before buffering. Evidence:
   `lsp_rejects_oversized_header_line_before_buffering_it`,
   `lsp_rejects_too_many_header_bytes`,
   `lsp_accepts_normal_headers_within_limits`.
4. **Mutation runner accepted unknown selectors.** Selection now fails
   closed (unknown ids or an empty selection exit 2). Evidence:
   `mutation_campaign_selection_fails_closed`.

Import select/export failures (`E0244`, `E0242`, `E0243` re-export) are now
also bound structurally to the module they were found in.

## Known limitations (P3, non-blocking)

1. RustLike type-checker errors have no position authority (`pos` is always
   `0`), so `E0201` carries no range; fixing this is typechecker work, not
   diagnostic plumbing.
2. The Logos parser reuses `E0201` for "expected 'System'", colliding with
   the type checker's `E0201` (pre-existing producer assignment, preserved
   per Decision B).
3. `smc check` without `--format` keeps its legacy human output, including a
   `1:1` header for positionless errors; only `--format` output is canonical.
4. `sm-sema` ranges cover the reported token (for example `state` of
   `state x: quad`), not the whole construct.
5. Hover, go-to-definition, document symbols, code actions, completion and
   incremental sync are deferred (not advertised, `-32601`).

## SSF-10 entry conditions

SSF-10 (`#1581`, frozen source/tooling/runtime contracts) may start only
when all of the following hold:

1. The `#1580` completion PR is merged under owner MERGE GO, `#1580` is
   closed as completed, and a separate governance PR moves
   `.harness/current.task.yaml` `active_phase` and the dependency map's
   **Active** row from SSF-09 to SSF-10 together
   (`tests/ssf_status_drift.rs` enforces they agree).
2. `semantic.diagnostics` v1 is the diagnostic schema SSF-10's compatibility
   window covers; any change SSF-10 needs is a v2 under the versioning policy
   in `diagnostics_machine_schema_v1.md`, never an in-place edit of v1.
3. SSF-10 compatibility work consumes the canonical carrier and the code
   registries as they are (`FRONTEND_DIAGNOSTIC_CODES`,
   `ALL_VERIFICATION_CODES`, `RUNTIME_DIAGNOSTIC_CODES`, and the legacy
   explanation catalog behind `smc explain`, mirrored in
   `docs/ERROR_CODES.md`); it does not introduce a second diagnostic
   identity authority.
4. The LSP/editor surface is not extended by SSF-10 (no hover/symbols) unless
   a canonical symbol authority is separately decided and authorized.
5. No stable-release claim: the SSF-09 surfaces are "Landed and qualified on
   `main`", not release-promised.
