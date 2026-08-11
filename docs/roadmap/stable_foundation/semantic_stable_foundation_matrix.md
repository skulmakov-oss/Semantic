# Semantic Stable Foundation Matrix

Status: SSF-00 truth freeze
Authority: current-facing feature-status inventory for issues #1569 and #1571
Evidence snapshot: `main` at `89a014b66e7c1e40502dbd764c94bf5f9445677f`
Toolchain snapshot: Rust `1.97.1`; workspace package version `0.1.0`

## Reading this matrix

This file assigns exactly one status to every public language, tooling, runtime,
project, package, compatibility, and onboarding family named by the Stable
Foundation umbrella. Code presence is not release status.

Statuses used here are the seven required by #1571:

| Status | Meaning |
|---|---|
| **Published stable** | Promised by a published stable release and its validated assets. |
| **Qualified limited release** | Explicitly admitted by the completed Gate 1 evidence. |
| **Landed and qualified on `main`** | Implemented and backed by focused executable evidence, but not release-promised. |
| **Landed but unqualified** | Implemented or partially implemented without sufficient Foundation qualification. |
| **Experimental** | Present for evaluation behind a deliberately non-stable contour. |
| **Roadmap** | Required or considered by a later SSF phase; not current supported behavior. |
| **Out of scope** | Explicitly excluded from the Stable Foundation milestone unless separately approved. |

Owner-layer abbreviations are: **F** frontend/semantic analysis, **I** IR/SemCode,
**V** verifier, **R** VM/runtime, **C** CLI/tooling, and **D** docs/release.
Multiple owners mean the public claim crosses those layers.

## Release truth snapshot

| Fact | Evidence | Reading |
|---|---|---|
| Exact starting `main` | commit `89a014b66e7c1e40502dbd764c94bf5f9445677f` | SSF-00 base after governance PR #1584. |
| Workspace package version | root `Cargo.toml` | `0.1.0`; it is not aligned to repository tag names. |
| `v1.1.1` | git tag `087f2f6dd244221ff0c4c9c00b40683570209643` | Tag exists. Its own version-cut decision says candidate only and leaves exact-tag downloaded-asset smoke blocking. |
| Published stable assets | GitHub Releases inventory | No GitHub Release exists for `v1.1.1`, `v1.0.0`, or `v0.1.0`; therefore no feature currently satisfies the **Published stable** evidence test. |
| Current prerelease | GitHub Release `v1.2.0-beta.1` | Prerelease, not stable promotion. |
| Current practical verdict | `reports/g1_release_scope_statement.md` | **Qualified limited release**, restricted to its named contour. |
| Wider application evidence | `reports/application_completeness_benchmark_verdict.md` | Landed and benchmark-qualified on `main`; explicitly not a public release. |

The former current-facing statement “published stable `v1.1.1`” conflicts with
the tag’s own checkpoint and the absence of published assets. SSF-00 resolves
that wording drift; it does not delete or rewrite historical evidence.

## Source-language surface

| Public feature | Owner layer | Status | Evidence and exact boundary | Foundation routing |
|---|---|---|---|---|
| Functions and `fn main` entrypoint | F/I/V/R/C/D | **Qualified limited release** | Gate 1 single-file and helper-module programs traverse the verified pipeline. | Candidate; freeze in SSF-01. |
| Immutable bindings and `const` | F/I/V/R | **Landed and qualified on `main`** | Frontend/lowering tests and current executable examples; not named as a distinct Gate 1 promise. | Decide exact contract in SSF-01. |
| Mutable bindings and assignment | F/I/V/R | **Landed and qualified on `main`** | `tests/mutable_binding_qualification.rs` and application benchmark. | Candidate; SSF-01/07. |
| `if` / `else` | F/I/V/R | **Qualified limited release** | Gate 1 admitted core control surface. | Candidate; SSF-01. |
| `match` and guards | F/I/V/R | **Qualified limited release** | Quad and bounded enum/Option/Result cases have positive and negative qualification. | Exact patterns deferred to SSF-01/07. |
| Rust-like `when` expression | F/I/V/R | **Landed but unqualified** | Parser/typechecker/lowering unit evidence exists; no named Gate 1 or canonical full-path contour. | Decide inclusion in SSF-01. |
| `while` | F/I/V/R | **Landed and qualified on `main`** | PCC control-flow and application benchmark evidence. | Candidate; SSF-01. |
| statement `loop`, `break`, `continue` | F/I/V/R | **Landed and qualified on `main`** | PCC positive, negative, and stable-SemCode tests. | Candidate; SSF-01. |
| range and iterable `for` | F/I/V/R | **Landed and qualified on `main`** | PCC sequence iteration and frontend/lowering tests; generalized iterable dispatch remains narrower. | Bound in SSF-01/07. |
| blocks and value-producing expressions | F/I/V/R | **Landed and qualified on `main`** | Parser/typechecker/lowering coverage for block tails and expression-valued control. | Candidate; SSF-01. |
| `return` and assertions | F/I/V/R | **Qualified limited release** | Gate 1 examples plus `return_assert_surface_qualification`. | Candidate; SSF-01. |
| Records | F/I/V/R | **Qualified limited release** | Gate 1 rule/state and direct-record Iterable contour. | Candidate; SSF-01. |
| Tuples | F/I/V/R | **Landed and qualified on `main`** | IR, SemCode, VM, and tuple ownership goldens. | Candidate; SSF-01/08. |
| Enums / nominal ADTs | F/I/V/R | **Landed and qualified on `main`** | PCC5 acceptance, match, diagnostics, and VM paths. | Candidate subset; SSF-01/07/08. |
| Schemas | F/C/D | **Landed but unqualified** | Schema metadata, compatibility, and migration helpers exist; executable/stable contract is not qualified. | Decide boundary in SSF-01/10. |
| `Option(T)` | F/I/V/R | **Qualified limited release** | Gate 1 plus PCC6 positive/negative/ownership evidence. | Candidate; SSF-01. |
| `Result(T, E)` | F/I/V/R | **Qualified limited release** | Gate 1 plus PCC6 positive/negative/ownership evidence. | Candidate; SSF-01. |
| `Sequence(T)` core iteration/indexing | F/I/V/R | **Qualified limited release** | Gate 1 explicitly admits built-in sequence iteration; PCC7 qualifies broader core. | Stable subset in SSF-01/07. |
| `Sequence(T)` mutation/persistent helpers | F/I/V/R | **Landed and qualified on `main`** | PCC7 and application benchmark (`len`, `push`, `pop`, `prepend`, `contains`). | Decide stable API in SSF-03/07. |
| `Map(K, V)` functional helpers | F/I/V/R | **Landed and qualified on `main`** | PCC7 and snake-learning benchmark. | Decide stable API in SSF-03/07. |
| Immutable first-wave closures | F/I/V/R | **Landed and qualified on `main`** | Short-lambda positive/negative qualification; captureful and multi-arg exclusions are explicit. | Bound in SSF-01/07. |
| Mutable, async, or host-transport closures | F/I/V/R | **Out of scope** | Explicitly beyond the current closure slice and umbrella contour. | Separate approval required. |
| First-wave generics / monomorphisation | F/I/V/R | **Landed but unqualified** | Generic-bound and conformance unit evidence exists, but no Foundation-wide full-path qualification contour. | Exact subset in SSF-01/07. |
| Direct-record `Iterable` static-trait dispatch | F/I/V/R | **Qualified limited release** | Gate 1 explicitly admits direct-record Iterable dispatch. | Preserve the bounded slice in SSF-01/07. |
| Broader static traits/protocols and impl coherence | F/I/V/R | **Landed but unqualified** | Trait/impl parsing, duplicate/coherence, and conformance evidence exists beyond the admitted direct-record slice. | Exact subset in SSF-01/07. |
| Trait objects, associated types, blanket impls, specialization, default methods | F/I/V/R | **Roadmap** | Named as decisions, with no Foundation-wide admitted contract. | SSF-07 may defer them explicitly. |
| Patterns and destructuring | F/I/V/R | **Landed and qualified on `main`** | Match, records, tuples, Option/Result positives and diagnostics. | Freeze bounded subset in SSF-01/07. |
| Direct local-path bare/selected imports | F/C | **Qualified limited release** | Explicit Gate 1 helper-module contour. | Candidate; SSF-01/05. |
| Alias, wildcard, re-export, namespace and package-qualified imports | F/C | **Landed but unqualified** | Broader parsing/module/package work exists; executable qualification remains deliberately narrower. | SSF-01/05/06. |
| `requires` | F/I/V/R | **Landed but unqualified** | Parser/typechecker and bounded lowering evidence exist; no complete public qualification contour. | Decide exact contract in SSF-01. |
| `ensures` | F/I/V/R | **Landed but unqualified** | Boolean validation and lowering evidence exist; no complete public qualification contour. | Decide exact contract in SSF-01. |
| `invariant` | F/I/V/R | **Landed but unqualified** | Bounded typecheck/lowering support exists; no complete public qualification contour. | Decide or defer in SSF-01. |
| Native `quad` (`N/F/T/S`) | F/I/V/R/D | **Qualified limited release** | Gate 1 rule/state contour and end-to-end verifier-first evidence. | Core candidate; SSF-01. |
| `bool` | F/I/V/R | **Qualified limited release** | Gate 1 admitted control/value family. | Core candidate; SSF-01. |
| `text`, concatenation, `to_text` | F/I/V/R | **Landed and qualified on `main`** | PCC3 and application benchmark; no general formatting/indexing promise. | SSF-01/03/07. |
| `i32` values and comparisons | F/I/V/R | **Qualified limited release** | Gate 1/current matrix and focused numeric evidence. | Core candidate; SSF-01/07. |
| same-family `i32` arithmetic | F/I/V/R | **Landed and qualified on `main`** | PCC2 and application benchmark. | Overflow contract in SSF-07. |
| `u32` values/equality | F/I/V/R | **Qualified limited release** | Primitive/value qualification is present. | Narrow role or arithmetic decision in SSF-01/07. |
| general `u32` arithmetic and overflow policy | F/I/V/R | **Roadmap** | No complete source/runtime arithmetic and overflow contract. | SSF-07. |
| `f64` arithmetic/math | F/I/V/R | **Landed and qualified on `main`** | Numeric surface qualification and SemCode/VM math path. | Exact deterministic promise in SSF-01/07. |
| `fx` fixed-point | F/I/V/R | **Landed and qualified on `main`** | Focused numeric qualification; cross-family/measured gaps remain. | Bound in SSF-01/07. |
| `unit` | F/I/V/R | **Qualified limited release** | Admitted primitive/value family. | Core candidate; SSF-01. |
| measured numeric forms | F/I | **Experimental** | Narrow carriers/type semantics exist; the excluded-arithmetic boundary is pinned per-operator in `crates/sm-front/src/typecheck.rs`: fx `+`/`-` (`measured_fx_addition_still_reports_narrow_slice_gap`, `measured_fx_subtraction_reports_narrow_slice_gap`), `*`/`/`/`%` (`measured_arithmetic_rejects_mul`, `_div`, `_mod`), mismatched units (`measured_arithmetic_rejects_mismatched_units`), and unary `+`/`-` on `i32`/`u32` (`measured_i32_unary_minus_rejects`, `measured_u32_unary_minus_rejects`); matching same-unit `f64` addition typechecks (`measured_f64_addition_typechecks`). | Boundary pinned in SSF-07; widening remains a later decision. |

## Profiles, standard library, and application boundary

| Public feature | Owner layer | Status | Evidence and exact boundary | Foundation routing |
|---|---|---|---|---|
| Rust-like executable profile | F/I/V/R/C/D | **Qualified limited release** | Gate 1 establishes a narrow executable contour. | Primary contract in SSF-01. |
| Logos declarative profile | F/I/C/D | **Experimental** | SSF-02 selected Model B: `semantic.logos.declarative/0.1` supports parse, semantic analysis, and non-executable `LogosIrLaw` inspection; SemCode/VM paths reject it. | Preserve the boundary through SSF-09/11/12; no automatic promotion. |
| `std.core` | F/I/V/R/D | **Qualified limited release** | `semantic.foundation.std/0.1` selects language-owned `assert`; `std.*` is not an import namespace. | Preserve through SSF-12. |
| `std.quad` | F/I/V/R/D | **Qualified limited release** | `semantic.foundation.std/0.1` selects explicit qtruth maps with N/S evidence preserved. | Preserve through SSF-12. |
| `std.math` | F/I/V/R/D | **Roadmap** | SSF-03 selects no API; current f64 builtins remain experimental pending determinism policy. | SSF-07/10. |
| `std.text` | F/I/V/R/D | **Qualified limited release** | Exact UTF-8 text equality/concat and bounded scalar `to_text`; no indexing/normalization. | Bound further in SSF-07. |
| `std.seq` | F/I/V/R/D | **Qualified limited release** | Ordered persistent sequence helpers selected as language-owned equivalents. | Bound further in SSF-07. |
| `std.map` | F/I/V/R/D | **Qualified limited release** | Persistent lookup/update helpers selected; no observable iteration/order API. | Bound further in SSF-07. |
| `std.option` | F/I/V/R/D | **Qualified limited release** | Language-owned type/constructors/exhaustive match selected; no helper expansion. | Preserve through SSF-12. |
| `std.result` | F/I/V/R/D | **Qualified limited release** | Language-owned type/constructors/exhaustive match selected; no helper expansion. | Preserve through SSF-12. |
| `std.serde` | F/I/V/R/D | **Roadmap** | SSF-03 selects no API or encoding; JSON in Rust tooling is not Semantic stdlib. | Explicit future approval only. |
| `std.rand` | F/I/V/R/D | **Qualified limited release** | Versioned xorshift64/13-7-17 seeded VM stream; no host entropy. | Compatibility window in SSF-10. |
| `args.read` | V/R/C | **Landed and qualified on `main`** | `args_read(u32)` is verifier- and manifest-gated, captured, hashed, and replay-order tested. | Preserve through SSF-12. |
| `stdin.read_text` | V/R/C | **Landed and qualified on `main`** | `stdin_read_text()` is an explicit captured UTF-8 observation; never ambient by default. | Preserve through SSF-12. |
| `stdout.write` | V/R/C | **Landed and qualified on `main`** | `stdout_write(text)` is a distinct explicit host write; it is not `print(text)`. | Preserve through SSF-12. |
| `stderr.write` | V/R/C | **Landed and qualified on `main`** | `stderr_write(text)` is independently capability-gated and audited. | Preserve through SSF-12. |
| `path.inspect` | V/R/C | **Landed and qualified on `main`** | Relative root-contained inspection with traversal/symlink/reparse denial. | Preserve through SSF-12. |
| `fs.read` | V/R/C | **Landed and qualified on `main`** | `fs_read_text` admits existing UTF-8 files only inside the canonical root. | Consume in SSF-05/06/11. |
| `fs.write` | V/R/C | **Landed and qualified on `main`** | `fs_write_text` requires an explicit grant and a prior captured observation; root escape is denied before write. | Consume in SSF-05/06/11. |
| bounded duration/time input | V/R/C | **Landed and qualified on `main`** | `time_duration_ms()` uses explicit `--duration-ms`; no wall-clock read exists. | Preserve through SSF-12. |
| `Pure`, `CliReadOnly`, `CliFileTransform`, `UiBounded` profiles | V/R/C/D | **Landed and qualified on `main`** | Exact deny-by-default manifests are frozen by `semantic.foundation.application/0.1`; `UiBounded` is catalogued, not a CLI mode. | Preserve through SSF-12. |
| Narrow `print(text)` / `CAP_STDOUT` observation | V/R/C | **Landed and qualified on `main`** | Capability-gated/audited benchmark path; not general stdout. | Input to SSF-04, not its completion. |

## Project and package surface

| Public feature | Owner layer | Status | Evidence and exact boundary | Foundation routing |
|---|---|---|---|---|
| Single-file fallback | C/F | **Landed and qualified on `main`** | Project Model v0 preserves direct `.sm` inputs; PCC9 exercises the full single-file CLI path. | Preserve through SSF-12. |
| `semantic.toml` project root and entrypoint | C/F | **Landed and qualified on `main`** | `semantic.foundation.project/0.1`; PCC9 positive/negative/root-preference and command-route tests. | Preserve through SSF-12. |
| `Semantic.package` baseline | C/F | **Landed and qualified on `main`** | Explicit compatibility/package input; `semantic.toml` wins when both exist. | SSF-06 owns its local-package contract. |
| Source/module root resolution | C/F | **Landed and qualified on `main`** | Project Model v0 plus PCC9 import/root qualification. | Preserve through SSF-12. |
| Test/example discovery | C | **Landed and qualified on `main`** | Sorted `tests/**/*.sm` execution and explicit `examples/**/*.sm` contract; SSF05 focused tests. | Preserve through SSF-12. |
| Deterministic path normalization/root-escape rejection | C/F | **Landed and qualified on `main`** | Project/import escape fixtures plus link/reparse rejection in test discovery. | Requalify local dependencies in SSF-06. |
| Project identity/content hashing boundary | C/D | **Landed and qualified on `main`** | Project Model v0 freezes descriptive name versus `hash-smc` content and SemCode epoch/revision boundaries; cryptographic provenance remains SSF-10. | Preserve boundary; SSF-10 owns trust policy. |
| `smc check <project-root>` | C/F | **Landed and qualified on `main`** | PCC9 full project-root check path. | SSF-05. |
| `smc compile <project-root>` | C/I | **Landed and qualified on `main`** | PCC9 compile and artifact command tests. | SSF-05. |
| `smc verify <artifact>` | C/V | **Qualified limited release** | Core Gate 1 verified path and artifact diagnostics. | Preserve in SSF-05/10. |
| `smc verify <project-root>` | C/F/V | **Landed and qualified on `main`** | Resolves the canonical entry, compiles in memory, and verifies without execution or artifact persistence. | Preserve through SSF-12. |
| `smc run <project-root>` | C/V/R | **Landed and qualified on `main`** | PCC9 run path. | SSF-05. |
| `smc test <project-root>` | C/F/V/R | **Landed and qualified on `main`** | Root-contained sorted discovery; every test compiles, verifies, and runs under the pure profile. | Preserve through SSF-12. |
| `smc new` | C | **Out of scope** | Project Model v0 is manually creatable; scaffolding is not required. | Excluded from Stable Foundation. |
| Package identity | C/F/D | **Landed and qualified on `main`** | `semantic.foundation.package/0.1`: name is descriptive; deterministic manifest/content/graph fingerprints carry reproducible identity, not trust. | Cryptographic trust remains SSF-10. |
| Local path dependencies | C/F | **Landed and qualified on `main`** | Relative local-only dependency loading; absolute paths and undeclared root escape reject. | No remote fetch. |
| Deterministic package graph and cycle diagnostics | C/F | **Landed and qualified on `main`** | Full declared graph is sorted; cycles, missing nodes, and duplicate identities reject deterministically. | SSF-06 closed contour. |
| Package-qualified imports | C/F | **Landed and qualified on `main`** | `<alias>::<module>` remains contained by the dependency module root and may enforce pinned identity. | SSF-06 closed contour. |
| Manifest/content hashes | C/D | **Landed and qualified on `main`** | Normalized `fnv1a64` fingerprints are deterministic change detectors and explicitly non-cryptographic. | Cryptographic artifacts remain SSF-10. |
| Capability-request inventory | C/V/R | **Landed and qualified on `main`** | Sorted manifest inventory is recorded; request is proven not to grant or propagate capability authority. | Runtime grants remain SSF-04 authority. |
| Lock/provenance record | C/D | **Landed and qualified on `main`** | Read-only `smc package inspect` emits canonical provenance-equivalent JSON and writes no lockfile. | Signing remains SSF-10. |
| Bounded workspaces | C/F | **Out of scope** | Local package composition needs no second workspace model. | Reconsider only after Stable Foundation. |

## Runtime, tooling, compatibility, and onboarding

| Public feature | Owner layer | Status | Evidence and exact boundary | Foundation routing |
|---|---|---|---|---|
| Source -> IR -> SemCode -> verifier -> VM | F/I/V/R/C | **Qualified limited release** | Gate 1 execution-integrity evidence. | Preserve through every phase. |
| Deterministic VM rerun | V/R | **Landed and qualified on `main`** | VM goldens, compatibility suites, benchmarks, and 7hell. | Final contract in SSF-08/10/12. |
| Quotas/fuel and trap taxonomy | V/R | **Landed and qualified on `main`** | VM/spec and trap/quota tests. | Freeze public promise in SSF-08. |
| OWN0 tuple/direct-record paths | V/R | **Landed and qualified on `main`** | Frozen tuple+record spec and ownership goldens. | Public Position A/B decision in SSF-08. |
| ADT/schema/sequence/map indirect ownership paths | V/R | **Landed but unqualified** | Some value-family tests exist; no complete stable path model. | SSF-08. |
| Partial move/release and inter-frame ownership | V/R | **Roadmap** | Explicit gap beyond OWN0. | SSF-08 may defer under Position A. |
| Capability and audit boundary | V/R/C | **Landed and qualified on `main`** | Controlled observation plus SSF-04 application capabilities; request/manifest distinction, structured denial, hashed audit, and replay-order evidence. | Preserve through SSF-12. |
| Human-readable diagnostics and source spans | F/I/V/C | **Landed and qualified on `main`** | PCC negative suites and CLI diagnostic parity tests. | Machine contract in SSF-09. |
| Stable machine-readable diagnostics | F/I/V/C | **Roadmap** | No single frozen JSON/protocol taxonomy across phases. | SSF-09/10. |
| Canonical formatter | C | **Landed and qualified on `main`** | Formatter idempotence, on-disk and canonical style tests. | Freeze contract in SSF-09. |
| Canonical diagnostics-only language server | C/F | **Roadmap** | Only a legacy Workbench bridge is visible; no canonical language-owned server baseline. | SSF-09. |
| Project-aware hover/go-to-definition/document symbols | C/F | **Roadmap** | No canonical external tooling contract. | SSF-09. |
| Formatter bridge and editor integration | C/D | **Roadmap** | No distributable canonical editor baseline. | SSF-09. |
| Source/SemCode compatibility policy | I/V/R/D | **Landed but unqualified** | Version headers and compatibility docs/tests exist; Foundation window is not frozen. | SSF-10. |
| Manifest/stdlib/diagnostic compatibility | C/D | **Roadmap** | No unified promised window. | SSF-10. |
| `smc version` | C | **Landed but unqualified** | CLI version output exists, but package/tag/toolchain identities remain misaligned. | SSF-10. |
| `smc artifact inspect` | C | **Roadmap** | No canonical command of this shape. | SSF-10. |
| `smc artifact hash` | C | **Roadmap** | Hash routes exist under other commands; no canonical artifact-trust command. | SSF-10. |
| `smc migrate check` / dry-run migration | C/D | **Roadmap** | Schema helpers exist, but no general source/artifact migration command. | SSF-10. |
| Release manifest, checksums, signing/unsigned policy | C/D | **Landed but unqualified** | Bundle verification exists; exact stable asset publication is unresolved and signing policy is not complete. | SSF-10/12. |
| Canonical examples pack | F/I/V/R/C/D | **Landed and qualified on `main`** | Twelve executable positives, one rejection boundary, one honest Logos example. | Rebuild against final contour in SSF-11. |
| External clean-clone onboarding | C/D | **Landed but unqualified** | README/cold-start evidence exists, but final Foundation procedure and artifacts do not. | SSF-11/12. |
| Benchmark-class application logic | F/I/V/R/C | **Landed and qualified on `main`** | Snake core and learning benchmark verdict. | Canonical proof in SSF-11. |
| Native Workbench/UI | R/C/D | **Landed but unqualified** | Separate beta/current-main evidence; DNA says UI is projection, never language authority. | Evidence-only path in SSF-11; not Foundation authority. |

## Explicitly out of scope

| Surface | Owner layer | Status | Evidence and boundary | Routing |
|---|---|---|---|---|
| async/await | F/I/V/R | **Out of scope** | Explicit #1569 non-goal. | Separate roadmap approval. |
| general multithreading/distributed execution | V/R | **Out of scope** | Explicit #1569 non-goal. | Separate roadmap approval. |
| macro system | F/I | **Out of scope** | Explicit #1569 non-goal. | Separate roadmap approval. |
| unrestricted reflection/dynamic dispatch | F/I/V/R | **Out of scope** | Explicit #1569 non-goal. | Separate roadmap approval. |
| garbage-collected object runtime | V/R | **Out of scope** | Explicit #1569 non-goal. | Separate roadmap approval. |
| browser/mobile platform support | R/C/D | **Out of scope** | Explicit #1569 non-goal. | Separate product track. |
| unrestricted filesystem/network/process access | V/R/C | **Out of scope** | Explicit #1569 non-goal. | Only bounded capabilities may be considered in SSF-04. |
| public registry/remote solver/build scripts/install hooks | C/D | **Out of scope** | Explicit package-baseline non-goals. | Separate ecosystem track. |
| plugin marketplace | C/D | **Out of scope** | Explicit #1569 non-goal. | Separate ecosystem track. |
| ALM/autonomous source mutation | C/D | **Out of scope** | Explicit #1569 non-goal. | Separate governed track. |
| Semantic Studio completion | C/D | **Out of scope** | Explicit #1569 non-goal. | Separate product track. |
| Andromeda implementation | R/C/D | **Out of scope** | Explicit #1569 non-goal. | Separate roadmap. |
| broad permanent ABI/ISA guarantee | I/V/R/D | **Out of scope** | Only the explicitly selected compatibility window may be promised. | SSF-10 defines the bounded window. |

## Current-facing authority and historical evidence

Current-facing readers must use this matrix together with:

- `stable_foundation_target_contract.md` for candidate scope;
- `stable_foundation_dependency_map.md` for ownership of unresolved work;
- `reports/g1_release_scope_statement.md` for the qualified limited-release
  contour;
- `reports/application_completeness_benchmark_verdict.md` for wider current-main
  application evidence.

Documents tied to an old tag, completed PR, or closed qualification cycle remain
historical evidence. They do not override this current-main inventory.
