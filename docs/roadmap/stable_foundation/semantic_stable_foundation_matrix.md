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
| measured numeric forms | F/I | **Experimental** | Narrow carriers/type semantics exist; measured `fx` arithmetic reports an explicit gap. | Decide/defer in SSF-01/07. |

## Profiles, standard library, and application boundary

| Public feature | Owner layer | Status | Evidence and exact boundary | Foundation routing |
|---|---|---|---|---|
| Rust-like executable profile | F/I/V/R/C/D | **Qualified limited release** | Gate 1 establishes a narrow executable contour. | Primary contract in SSF-01. |
| Logos declarative profile | F/I/C/D | **Experimental** | SSF-02 selected Model B: `semantic.logos.declarative/0.1` supports parse, semantic analysis, and non-executable `LogosIrLaw` inspection; SemCode/VM paths reject it. | Preserve the boundary through SSF-09/11/12; no automatic promotion. |
| `std.core` | F/I/V/R/D | **Roadmap** | Current helpers are builtins; no importable canonical module contract. | SSF-03. |
| `std.quad` | F/I/V/R/D | **Roadmap** | Native quad exists, but the named module/API does not. | SSF-03. |
| `std.math` | F/I/V/R/D | **Roadmap** | Math builtins exist; named module and compatibility contract do not. | SSF-03. |
| `std.text` | F/I/V/R/D | **Roadmap** | Bounded text operations exist; named module/UTF-8 API is not frozen. | SSF-03/07. |
| `std.seq` | F/I/V/R/D | **Roadmap** | Sequence helpers exist as language builtins, not a stable importable module. | SSF-03/07. |
| `std.map` | F/I/V/R/D | **Roadmap** | Map helpers exist as language builtins, not a stable importable module. | SSF-03/07. |
| `std.option` | F/I/V/R/D | **Roadmap** | Option is qualified; named module API is absent. | SSF-03. |
| `std.result` | F/I/V/R/D | **Roadmap** | Result is qualified; named module API is absent. | SSF-03. |
| `std.serde` | F/I/V/R/D | **Roadmap** | No canonical deterministic importable serialization family. | SSF-03. |
| `std.rand` | F/I/V/R/D | **Roadmap** | Seeded PRNG builtins are qualified on main; named/versioned module is absent. | SSF-03/10. |
| `args.read` | V/R/C | **Roadmap** | No canonical admitted capability of this name. | SSF-04. |
| `stdin.read_text` | V/R/C | **Roadmap** | No canonical admitted capability of this name. | SSF-04. |
| `stdout.write` | V/R/C | **Roadmap** | Narrow `print(text)`/`CAP_STDOUT` exists, but the proposed structured capability does not. | SSF-04. |
| `stderr.write` | V/R/C | **Roadmap** | No canonical admitted capability of this name. | SSF-04. |
| `path.inspect` | V/R/C | **Roadmap** | Project path validation exists in tooling, not as a program capability. | SSF-04. |
| `fs.read` | V/R/C | **Roadmap** | No admitted source-language file-read boundary. | SSF-04. |
| `fs.write` | V/R/C | **Roadmap** | No admitted source-language file-write boundary. | SSF-04. |
| bounded duration/time input | V/R/C | **Roadmap** | No replay-safe application time contract. | SSF-04. |
| `Pure`, `CliReadOnly`, `CliFileTransform`, `UiBounded` profiles | V/R/C/D | **Roadmap** | Proposed profiles are not yet canonical runtime contracts. | SSF-04. |
| Narrow `print(text)` / `CAP_STDOUT` observation | V/R/C | **Landed and qualified on `main`** | Capability-gated/audited benchmark path; not general stdout. | Input to SSF-04, not its completion. |

## Project and package surface

| Public feature | Owner layer | Status | Evidence and exact boundary | Foundation routing |
|---|---|---|---|---|
| Single-file fallback | C/F | **Qualified limited release** | Gate 1 explicitly admits single-file programs. | SSF-05. |
| `semantic.toml` project root and entrypoint | C/F | **Landed and qualified on `main`** | PCC9 positive/negative/root-preference and command-route tests. | Canonical manifest decision in SSF-05. |
| `Semantic.package` baseline | C/F | **Landed and qualified on `main`** | PCC9/project-manifest and local import evidence. | Relationship to `semantic.toml` in SSF-05/06. |
| Source/module root resolution | C/F | **Landed and qualified on `main`** | PCC9 and import qualification include deterministic root behavior. | Freeze in SSF-05. |
| Test/example discovery | C | **Roadmap** | Repository tests exist; no canonical project discovery contract or `smc test`. | SSF-05. |
| Deterministic path normalization/root-escape rejection | C/F | **Landed and qualified on `main`** | Project/import negative fixtures cover escapes and missing paths. | Requalify contract in SSF-05/06. |
| Project identity/content hashing boundary | C/D | **Landed but unqualified** | Hash routes exist, but the stable identity/provenance contract is not frozen. | SSF-05/10. |
| `smc check <project-root>` | C/F | **Landed and qualified on `main`** | PCC9 full project-root check path. | SSF-05. |
| `smc compile <project-root>` | C/I | **Landed and qualified on `main`** | PCC9 compile and artifact command tests. | SSF-05. |
| `smc verify <artifact>` | C/V | **Qualified limited release** | Core Gate 1 verified path and artifact diagnostics. | Preserve in SSF-05/10. |
| `smc run <project-root>` | C/V/R | **Landed and qualified on `main`** | PCC9 run path. | SSF-05. |
| `smc test <project-root>` | C | **Roadmap** | Command is not part of the current canonical CLI. | SSF-05. |
| `smc new` | C | **Roadmap** | Explicitly absent/non-required unless narrowly approved. | Optional SSF-05 decision. |
| Package identity | C/F/D | **Landed but unqualified** | Package names/manifests exist; long-term identity contract is not frozen. | SSF-06. |
| Local path dependencies | C/F | **Landed and qualified on `main`** | Deterministic local dependency loading and tests exist. | Requalify Foundation scope in SSF-06. |
| Deterministic package graph and cycle diagnostics | C/F | **Landed and qualified on `main`** | Import/package cycle positives and negatives. | SSF-06. |
| Package-qualified imports | C/F | **Landed but unqualified** | Landed beyond the Gate 1 executable import promise. | SSF-06. |
| Manifest/content hashes | C/D | **Landed but unqualified** | Hash helpers/routes exist without stable provenance promise. | SSF-06/10. |
| Capability-request inventory | C/V/R | **Roadmap** | Package metadata is not yet a complete explicit inventory contract. | SSF-06. |
| Lock/provenance record | C/D | **Roadmap** | No canonical lockfile/equivalent reproducibility record. | SSF-06/10. |
| Bounded workspaces | C/F | **Roadmap** | Not part of the current qualified package baseline. | SSF-06 only if examples require it. |

## Runtime, tooling, compatibility, and onboarding

| Public feature | Owner layer | Status | Evidence and exact boundary | Foundation routing |
|---|---|---|---|---|
| Source -> IR -> SemCode -> verifier -> VM | F/I/V/R/C | **Qualified limited release** | Gate 1 execution-integrity evidence. | Preserve through every phase. |
| Deterministic VM rerun | V/R | **Landed and qualified on `main`** | VM goldens, compatibility suites, benchmarks, and 7hell. | Final contract in SSF-08/10/12. |
| Quotas/fuel and trap taxonomy | V/R | **Landed and qualified on `main`** | VM/spec and trap/quota tests. | Freeze public promise in SSF-08. |
| OWN0 tuple/direct-record paths | V/R | **Landed and qualified on `main`** | Frozen tuple+record spec and ownership goldens. | Public Position A/B decision in SSF-08. |
| ADT/schema/sequence/map indirect ownership paths | V/R | **Landed but unqualified** | Some value-family tests exist; no complete stable path model. | SSF-08. |
| Partial move/release and inter-frame ownership | V/R | **Roadmap** | Explicit gap beyond OWN0. | SSF-08 may defer under Position A. |
| Capability and audit boundary | V/R/C | **Landed and qualified on `main`** | Narrow controlled observation evidence only. | Expand only through SSF-04. |
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
