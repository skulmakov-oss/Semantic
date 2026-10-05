# PB-04 — Module Graph, Identity & Provenance Decision

Status: decided (owner decisions on #1686, #1690 and #1684, 2026-10-05)
Scope: Phase-B PB-04: #1681–#1692, #1701–#1703, #1706
Historical audit umbrella: #1617 · Previous slices: #1985, #1986, #1988 · Residual: #1987 (untouched)
Base: `main` `3b12987ba59a985ff6726e3abd0e57d60b588b7e`
Reference oracle (unchanged): C1 `89641da8237f4fcefb50cf1958a50e4d4003aea7` = `v1.2.0`

Invariant: there is one resolved module graph, one module identity per node,
and one resolved edge per import declaration, and every later module-semantic
operation consumes that frozen graph.

## Re-adjudication on current main (`3b12987b`)

| Issue | Current-main reproduction | Classification |
|---|---|---|
| #1681 | Production loader consumes `LogosProgram::imports` (PB-02). The public raw scanner `parse_import_directives` still found an indented `Import "evil.sm"` inside a Law body. | PARTIALLY_RESOLVED (production RESOLVED_BY_PB02; dead public scanner remained) |
| #1682 | The production directive parser (moved there by PB-02) repaired input: `Import "a.sm` → spec `a.sm`; `Import "a.sm" { A` → empty select set. | STILL_REPRODUCED |
| #1683 | `{ Bogus:A }` against `Law A` → validation OK. | STILL_REPRODUCED |
| #1684 | `build_import_resolution_plan_core` unit-tested, unused in production; Logos analysis never bound imported names. | STILL_REPRODUCED |
| #1685 | `Import pub "dep" { Entity:A }` re-exported nothing. | STILL_REPRODUCED |
| #1686 | `Entity A` + `Law A` → both exported (2 items). | STILL_REPRODUCED |
| #1687 | `{ Entity:A }` with no kind metadata → OK. | STILL_REPRODUCED |
| #1688 | Duplicate module key → last-write-wins. | STILL_REPRODUCED |
| #1689 | Missing dependency export cache entry → `unwrap_or_default()` empty set. | STILL_REPRODUCED (code) |
| #1690 | `path_contract_key("dir\\name.sm")` → `dir/name.sm` on every platform; `to_string_lossy`. | STILL_REPRODUCED |
| #1691 | `normalize_lexical("../a.sm")` and `("../../a.sm")` → `a.sm`. | STILL_REPRODUCED |
| #1692 | `resolve_import` called 3× for one edge. | STILL_REPRODUCED |
| #1701 | `cargo check -p sm-sema --no-default-features --features alloc` passes (no `std`/serde), but no CI/7HELL gate runs it. | PARTIALLY_RESOLVED (build ok; enforcement missing) |
| #1702 | Fixture discovery via `read_dir(..).flatten()`, no inventory. | STILL_REPRODUCED (code) |
| #1703 | 3-hop re-export chain `["a","b","X"]` (lost `c`). | STILL_REPRODUCED |
| #1706 | `ExportOrigin::Imported` public, never constructed. | STILL_REPRODUCED (code) |

## Owner decisions

1. **#1686 — flat namespace.** One module plus one public name gives at most one `ExportItem`; the same name across any kinds is `E0242`. `{ Foo }` selects the unique item; `{ Entity:Foo }` additionally asserts its kind. There is no `(name, kind)` table.
2. **#1690 — fail closed on non-UTF-8.** The `String`/`&str` `ModuleProvider` API is kept. A module id is a provider-owned UTF-8 logical identifier. Filesystem paths convert to ids losslessly at the std/filesystem boundary (root conversion in `sm-sema::std_adapters`, resolution in the `smc-cli` providers); non-UTF-8 fails closed (`E0239` in the loader). No `to_string_lossy` is used for identity, and no `\`→`/` folding happens outside Windows. Provider-returned ids are consumed verbatim and never re-normalized.
3. **#1684 — bind imports from the frozen graph; preserve the namespace contract.** The documented order is now production authority: local → selected → namespace-qualified → wildcard (declaration order). **PB-04 preserves the pre-existing module-resolution contract. The frontend condition representation was minimally extended (`LogosAtom::QualifiedField { namespace, entity, field }`, built by `sm-front` from `Ident . Ident . Ident`) so production semantic analysis can consume namespace-qualified bindings from the frozen graph; no new lookup precedence or module semantics were invented.** `X.Entity.field` resolves only through alias `X` and is never a fallback for an unqualified name.

## Final architecture

| Concern | Owner / authority |
|---|---|
| Import syntax | `sm-front` preserves each directive (PB-02); `alloc_core::parse_import_directive` is the one strict directive interpreter (no repair) |
| Selector | `SelectedImport { public_name, expected_kind, local_name }`, parsed once and flowing through every phase |
| Resolution | `std_adapters::load_module_recursive` calls `resolve_import` exactly once per edge into `ResolvedImport { directive, target }` |
| Graph | `alloc_core::ModuleGraph` (frozen; rejects duplicate ids and dangling edges) |
| Exports | `build_export_sets_core(&ModuleGraph)`: flat namespace, no default fallback, complete chain, `source_module`/`source_name` preserved |
| Selection | `validate_select_imports_core(&ModuleGraph, &ExportSets)` on the unique `ExportItem` |
| Lookup | `resolve_unqualified_import` (selected, then wildcard) and `resolve_namespace_import` (alias only) |
| Provenance | `ExportOrigin::{Local, ReExport{chain}}`, both reachable and tested; `Imported` removed (#1706) |

## Identity roles

| Role | Type | Meaning |
|---|---|---|
| `ModuleId` | `String` | provider-owned semantic identity: graph key, cycle detection, export source |
| host path | `Path` | filesystem transport; converted only at the std boundary |
| display | `String` | `ModuleProvider::display_module(id)`, used for diagnostics and provenance text only |
| `provider_module_id` | `Option<String>` | the module id, copied as diagnostic provenance (SSF-09) |
| package-relative `SourceIdentity` | `smc-cli` | CLI/LSP presentation identity, unchanged and not a dependency of `sm-sema` |

## Public API changes (`sm-sema`)

| Change | Class |
|---|---|
| removed `parse_import_directives`, `parse_select_items` | dead-contract removal (#1681) |
| `parse_import_directive` → `Result<_, ImportPolicyError>` | fail-closed correction (#1682, #1683) |
| `SelectedImport`; `ImportDirective.select_items: Vec<SelectedImport>`; `ImportDirective::namespace_alias` | authority consolidation (#1683, #1685) |
| removed `ImportResolutionPlan`, `build_import_resolution_plan_core`, `SelectImportModule`, `SelectImportPolicyError`, `ExportBuildModule`, `ExportBuildError` | dead-contract removal / authority consolidation (#1684, #1687) |
| `ModuleGraph`, `ModuleNode`, `ResolvedImport`, `ModuleError`, `ExportSets`, `resolve_unqualified_import`, `resolve_namespace_import` | authority consolidation (#1684, #1688, #1692) |
| `build_export_sets_core` / `validate_select_imports_core` take the graph | authority consolidation |
| `collect_local_exports_core(id, display, locals) -> Result` | fail-closed correction (#1686) |
| `ExportItem.{source_module, source_name}`, `ExportSet::get` | provenance correction (#1703) |
| `ExportOrigin::Imported` removed | dead-contract removal (#1706) |
| `infer_*` field resolver gains an optional namespace | authority consolidation (#1684) |
| `sm-front`: `LogosAtom::QualifiedField` | minimal syntax projection (#1684) |

## Behavior changes (each maps to the issue set)

- Malformed `Import` directives and unknown qualifiers reject (#1682, #1683).
- The same public name across kinds rejects with E0242 (#1686).
- Kind-qualified selected re-exports export the item (#1685).
- Imported Entities are usable in conditions through the documented order (#1684).
- Module ids are verbatim and lossless (#1690, #1691); a non-UTF-8 root fails (E0239).
- Re-export provenance chains are complete (#1703).

The C1 module corpus (`tests/fixtures/imports`, imports matrix, SSF-09 project goldens) keeps its outcomes; the only diagnostic text change is an E0242 message for a local cross-kind collision ("export collision").

## Residual

- W0252 (unused field) is still computed per module: a field used only by an importing module is reported unused in its declaring module. This is a semantic-lint limit outside the PB-04 issue set.
