# Semantic Feature Maturity Matrix

Status: draft status document  
Scope: documentation/status discipline only  
Owner: project status / roadmap documentation

This document separates **architecture maturity**, **feature implementation**, and
**public release claims** for Semantic.

Semantic is being built as a **verifier-first deterministic execution platform**
with a semantic bytecode contract. The execution-core architecture is currently
more mature than the everyday practical language surface required for general
application development.

A feature being present on `main` does **not** automatically make it part of the
published stable release surface.

```text
landed on main != published stable
```

## Maturity levels

| Level | Meaning |
|---|---|
| D0 | Documented only / roadmap intent |
| D1 | Parsed by the source frontend |
| D2 | Typechecked / semantically accepted |
| D3 | Lowered to IR |
| D4 | Emitted to SemCode |
| D5 | Accepted by `sm-verify` |
| D6 | Executed by `sm-vm` |
| D7 | Qualified by tests, golden evidence, 7hell, or benchmark-positive evidence |
| N/A | Explicitly out of scope for the current release surface |

## Status vocabulary

| Status | Meaning |
|---|---|
| Stable release surface | Publicly claimed current contract |
| Qualified limited release | Works in a narrow qualified path, but does not widen all related features |
| Implemented but unqualified | Landed or partially working, but not yet full release evidence |
| Experimental | Present for evaluation or future shaping |
| Roadmap | Planned or tracked, not current stable behavior |
| Roadmap blocker | Needed for application completeness, but not yet qualified |
| Out of scope | Deliberately excluded from the current bounded contract |

## Current matrix

| Feature | Maturity level | Confidence | Evidence type | Status | Notes |
|---|---:|---|---|---|---|
| Native quad logic | D6 | High | Docs claim, VM/spec evidence | Stable release surface | `N / F / T / S` is a native semantic value domain. |
| i32 relational operators | D7 | High | Test evidence | Qualified limited release | Covers relational/equality-style operators (`==`, `!=`, `<`, `<=`, `>`, `>=`). |
| same-family i32 arithmetic | D7 | High | Test evidence | Qualified application-completeness contour | Covers `+`, `-`, `*`, `/`, `%` and unary `-`. Not published stable. |
| Mutable locals & reassignment | D7 | High | Test evidence | Qualified application-completeness contour | Supports `let mut` declarations and plain reassignments. Not published stable. |
| Loops and control exits | D7 | High | Test evidence | Qualified application-completeness contour | Supports `while` loops, statement `loop`, and exits `break`/`continue`. Not published stable. |
| Text concatenation and to_text | D7 | High | Test evidence | Qualified application-completeness contour | Supports `text + text` and explicit `to_text(scalar)`. Not published stable. |
| Sequence indexing and iteration | D7 | High | Test evidence | Qualified limited release | Qualified for sequence iteration and indexing. |
| First-class immutable closures | D7 | High | Test evidence | Qualified limited release | Immutable closure path only; mutable capture semantics are separate. |
| Map surface | D7 | High | Test evidence | Qualified application-completeness contour | Supports `Map(K, V)` functional get, set, contains. Not published stable. |
| Deterministic seeded PRNG | D7 | High | Test evidence | Qualified application-completeness contour | Deterministic seeded PRNG (xorshift64) via `random_seed` / `random_next_i32`. Not published stable. |
| Controlled observation (stdout) | D7 | High | Test evidence | Qualified application-completeness contour | Narrow `print(text)` via `CAP_STDOUT` capability. Not published stable. |
| Bounded project-root CLI baseline | D7 | High | Test evidence | Qualified application-completeness contour | Supports resolving and running routes from documented manual project-root layouts (`semantic.toml` / `Semantic.package`). Excludes registry, multi-package resolution, package manager semantics, and `smc new` scaffolding. Not published stable. |
| Runtime ownership OWN0 | D6 | High | Docs claim, VM/spec evidence | Stable release surface, frozen | Tuple and direct record-field access paths only. |
| Function contracts: `requires` / `ensures` | D5 | Medium | Docs claim, verifier spec | Implemented but unqualified | Requires syntax, typecheck, lowering, verifier and runtime qualification. |
| PROMETHEUS ABI / host boundary | D6 | Medium | Docs claim, CLI evidence | Implemented but unqualified | Needs qualification of ABI, capability policy, audit, and host bridge. |
| Units-of-measure surface | D2 | High | Docs claim, crate/status evidence | Experimental | Type/semantic surface only. |
| ADT payload paths for ownership | N/A | High | Runtime ownership docs | Out of scope | Explicitly excluded from the current OWN0 slice. |

## FR-1 Source Surface Classification

This section classifies the public source contour for FR-1 using the canonical
status vocabulary in `docs/roadmap/public_status_model.md`.

| Feature / syntax family | Current status | Evidence | Public claim allowed | Notes / limitation |
|---|---|---|---|---|
| `fn` / `let` / `if` / `else` / `return` | qualified limited release | spec, README, tests | limited | core executable control surface; published-stable status requires stable-line contract evidence |
| `quad` / `bool` / `unit` | qualified limited release | spec, README, tests | limited | native value and condition families; stable promotion remains governed by public status model |
| `i32` / `u32` primitive types | qualified limited release | spec, tests | limited | primitive types only; arithmetic and relational behavior are tracked separately |
| `match` | qualified limited | spec, tests, roadmap | limited | quad and narrow enum / standard-form match behavior is documented; full pattern ergonomics remain bounded |
| `record` | qualified limited | spec, examples, tests, roadmap | limited | nominal record values are admitted and benchmarked, but not published stable |
| `Option` / `Result` | qualified limited | spec, tests, Gate 1 report | limited | explicit variants are in the narrow qualified contour |
| `Sequence` | qualified limited | spec, tests, Gate 1 + benchmark reports | limited | iteration/indexing are qualified; broader collection API remains limited |
| direct local-path helper imports | qualified limited release | spec, tests, roadmap | limited | bare and selected local-path helper imports are admitted in the qualified contour |
| broader import / export surface | landed on main / current-main only | spec, tests, roadmap | limited | alias, wildcard, public re-export, package-qualified executable import, and namespace-qualified access remain current-main only |
| mutable locals / assignment | landed on main / current-main only | spec, tests, matrix | limited | benchmark-qualified on main; not promoted to published stable |
| `while` / `loop` / `break` / `continue` | landed on main / current-main only | spec, tests, roadmap | limited | benchmark-qualified on main; not promoted to published stable |
| `schema` | landed on main / current-main only | spec, tests, roadmap | limited | compile-time-only family; current-main support remains bounded |
| ADT / enum surface | landed on main / current-main only | spec, tests | limited | enum forms are documented and tested; public promotion not yet made |
| function contracts | landed on main / current-main only | spec, tests | limited | `requires` / `ensures` / `invariant` exist, but release-facing qualification is still pending |
| `f64` / `fx` / `text` | landed on main / current-main only | spec, tests, roadmap | limited | admitted on current main in bounded slices; not a published stable promise |
| unsupported / diagnostic-only syntax families | out of scope | diagnostics, spec | no | general package registry, broad I/O, generic traits, and other non-FR-1 families remain excluded |

## Important distinctions

### same-family i32 arithmetic is qualified

The current status includes qualified support for:

```text
same-family i32 arithmetic:
  +, -, *, /, %, unary -
```

This is distinct from multi-family numeric compatibility or implicit float conversions.

### Text concatenation is not general formatting

Text concatenation and explicit `to_text` are qualified, but general template formatting and implicit conversion of complex structures are roadmap/non-goals.

### Controlled observation is not general stdout

The active controlled-observation path is intentionally narrow:

```text
verified SemCode
  -> VM controlled observation event
  -> capability gate
  -> audit decision
  -> CLI rendering envelope
```

Narrow `print(text)` is qualified under the `CAP_STDOUT` capability, but general file I/O, command-line arguments (argv), or unconstrained stdout remain out of scope.

### OWN0 is intentionally narrow

The frozen runtime ownership slice supports:

- tuple access paths;
- direct record-field access paths;
- frame-local borrow lifetime;
- overlap rejection for exact, parent-child, and child-parent paths;
- sibling writes when paths do not overlap.

It explicitly does not support:

- ADT payload paths;
- schema paths;
- partial borrow release before frame exit;
- advanced alias / region reasoning;
- inter-frame borrow persistence;
- indirect projections.

## FR-6 Runtime Closure Classification

This matrix classifies the deterministic runtime surfaces that FR-6 keeps in
scope. It does not add runtime behavior or promote any surface to stable
release by itself.

| FR-6 surface | Current classification | Evidence owner | Boundary | Notes |
|---|---|---|---|---|
| Runtime value set | core runtime contract | `sm-vm` / `sm-runtime-core` | runtime value / execution model | Current value families are documented in `docs/spec/vm.md`; this is a bounded runtime carrier set, not a general-purpose value guarantee. |
| Symbol identity model | core runtime contract | `sm-runtime-core` | deterministic identity / hot-path model | `SymbolId` is used in the VM hot path and canonical access-path transport; the model is deterministic, but not a public stable naming ABI. |
| Quota / fuel taxonomy | core runtime contract | `sm-runtime-core` / `sm-vm` | bounded execution | Quota kinds, baseline profiles, and enforcement ownership are already frozen in `docs/spec/quotas.md`. |
| Quota exhaustion behavior | core runtime contract | `sm-vm` | deterministic failure | Exhaustion is reported deterministically via `QuotaExceeded` or the surfaced `StackOverflow` compatibility path. |
| Trap taxonomy | core runtime contract | `sm-vm` | runtime failure model | Runtime traps remain distinct from verifier rejection, CLI diagnostics, and capability denial. |
| Verifier rejection vs runtime trap split | core runtime contract | `sm-verify` / `sm-vm` | verifier-before-execution boundary | Standard SemCode admission rejects malformed or unsupported artifacts before execution; runtime traps only describe admitted execution failure. |
| Trace / audit event shape | PROMETHEUS boundary policy | `prom-runtime` / `prom-audit` | orchestration / audit policy | Trace and audit shape are release-facing evidence, but the shape is owned by PROMETHEUS-facing orchestration and audit layers rather than core VM semantics. |
| Deterministic rerun invariant | downstream proof surface | `sm-vm` / tests | replay / golden evidence | Same SemCode, same config, same capability context, and same input stream must yield the same result / trap / trace class. |
| Seeded pseudo-random behavior | current-main evidence | `sm-vm` / tests | deterministic helper surface | Deterministic seeded PRNG (`random_seed` / `random_next_i32`) is qualified evidence, not a broad randomness contract. |
| Capability / observation boundary | PROMETHEUS boundary policy | `prom-cap` / `prom-runtime` | host-effect admission | Narrow `print(text)` / `CAP_STDOUT` remains bounded and capability-aware; this matrix does not widen host effects. |
| PROMETHEUS runtime policy | PROMETHEUS boundary policy | `prom-runtime` / `prom-audit` | orchestration / release-facing policy | PROMETHEUS runtime policy stays separate from core runtime determinism and does not imply release qualification. |

## FR-9 Release Qualification Classification

This matrix classifies the release-qualification surfaces that FR-9 keeps in
scope. It distinguishes local admission gates from release-facing policy and
does not itself create release artifacts or claims.

| FR-9 surface | Current classification | Evidence owner | Gate role | Notes |
|---|---|---|---|---|
| Public status vocabulary | public-status guard | roadmap/status docs | release-facing policy | Distinguishes `published stable`, `qualified limited release`, `landed on main`, `current-main only`, `experimental`, and `out of scope` without promoting any of them. |
| Qualified limited release posture | release policy / required | roadmap policy docs | release-candidate posture | Describes the bounded qualified contour; it is not a public stable declaration. |
| Published stable posture | explicit non-claim until later decision | roadmap policy docs | release publication target | Remains distinct from landed-on-main behavior and is not implied by FR-9 planning. |
| `PRReady` | local admission gate | `scripts/admission_guard.ps1` | local admission | Useful for PR admission, but not sufficient alone for release qualification. |
| `Readiness` | local readiness gate | `scripts/admission_guard.ps1` | local readiness | Stronger than `PRReady`, but still not equivalent to publishing a release. |
| `FullPreflight` | heavy local gate candidate | `scripts/admission_guard.ps1` | local full-preflight gate | A broad local gate used for deeper validation; it is not itself a public release claim. |
| Release bundle verification | bundle verification surface | `scripts/verify_release_bundle.ps1` | release-candidate gate | Verifies bundle contents and required release documentation without producing a release artifact in this PR. |
| Release asset smoke verification | asset smoke surface | `scripts/verify_release_assets.ps1` | release-candidate gate | Checks release assets and smoke behavior separately from ordinary PR readiness. |
| Release-facing docs alignment | release policy / required | roadmap/status docs | release publication support | Keeps release notes and status wording honest before any publication decision. |
| Release notes / release candidate wording | public-status guard | roadmap/status docs | release-facing policy | Must preserve non-claims and avoid implying stable or production-ready status. |
| GitHub CI role | not authoritative | process policy docs | non-gate | GitHub CI may report evidence, but it is not the authoritative admission gate for this repository. |
| Stable runtime / binary ABI claims | explicit non-claim | roadmap/status docs | out of scope | FR-9 planning does not imply a stable runtime ABI or stable binary ISA. |
| Package ecosystem / `smc new` claims | explicit non-claim | roadmap/status docs | out of scope | Package-registry semantics and `smc new` support remain separate from release qualification planning. |
| Production-ready claim | explicit non-claim | roadmap/status docs | out of scope | FR-9 planning does not imply production-ready status. |

## Workbench Readiness Classification

This matrix classifies the Workbench readiness surfaces after the non-UI
readiness first pass. It does not add UI behavior, promote Workbench to stable
release, or move Semantic core ownership into the desktop shell.

| Workbench surface | Current classification | Source of truth | Boundary | Notes |
|---|---|---|---|---|
| Workbench foundation | foundation policy | `docs/workbench/architecture.md`, `docs/workbench/scope.md` | UI / orchestration | Workbench is a desktop orchestration and presentation layer over existing repository contracts, not a second Semantic core. |
| Source-of-truth policy | foundation policy | repository docs, command outputs, release artifacts | truth ownership | Workbench presents repository truth derived from docs, public tools, scripts, artifacts, and recorded outputs; it must not create Semantic truth. |
| Public surface rule | foundation policy | `smc`, `svm`, `cargo`, public release scripts, public docs | integration boundary | First integration is process-based over public surfaces; private crate internals remain off-limits unless a later public facade is explicitly supported. |
| Process adapter / command runner | proposed v1 / current beta evidence | `apps/workbench_ts_tauri_legacy`, Workbench beta docs | orchestration-only | May request `smc`, `svm`, `cargo`, and release-script commands and display outputs; command dispatch must not rewrite semantics or bypass verifier admission. |
| Overview cockpit | proposed v1 / current beta evidence | Workbench scope, beta notes, view models | presentation-only | May summarize branch, commit, validation, readiness, bundle, asset-smoke, and known-limit signals; it must not invent readiness percentages or alternate scores. |
| Jobs history | proposed v1 / current beta evidence | `JobViewModel`, Workbench app scaffold | presentation-only | Stores requested command metadata, stdout/stderr, exit codes, duration, and related files; it must not hide or reinterpret command failure. |
| Spec navigator | proposed v1 / current beta evidence | `docs/spec/*`, `docs/roadmap/*`, `SpecDocumentViewModel` | read-only projection | May index canonical docs, paths, headings, and declared stability labels; it must not silently edit or mirror docs as a new contract. |
| Editor shell | proposed v1 / current beta evidence | Workbench scope, beta notes | authoring shell | May provide tabs, save/reload, dirty markers, and current-file actions; it must not become a parser, typechecker, or full IDE protocol owner. |
| Diagnostics hub | proposed v1 / current beta evidence | public diagnostics, command outputs, `DiagnosticsViewModel` | diagnostic presentation | May group diagnostics, preserve codes/locations/messages, and link specs; it must not define diagnostic categories independently. |
| Inspector views | proposed v1 / current beta evidence | `svm disasm`, verify outputs, trace/quota/capability summaries | output projection | May display disassembly, verify, trace, quota, and capability summaries when present in outputs; it must not implement a second VM interpretation layer. |
| Release console | proposed v1 / current beta evidence | release docs, release scripts, `ReleaseViewModel` | gate visibility / not release authority | May show real gate, bundle, asset-smoke, and docs-alignment signals; it must not publish, qualify, or compute release truth independent from repository gates. |
| View models | proposed v1 | `docs/workbench/view_models.md` | derived presentation cache | View models are derived, explainable, and refreshable from public inputs; they are never canonical source of truth. |
| Beta packaging / beta notes | current beta evidence | `docs/workbench/beta_packaging.md`, `docs/workbench/beta_release_notes.md`, beta smoke artifacts | beta evidence / not stable release | Existing beta package and smoke evidence are current-main Workbench evidence, not production-ready or published-stable claims. |
| `smlsp` bridge | experimental / deferred | Workbench beta notes, scope deferred path | editor protocol boundary | Optional and experimental; not required for the primary Workbench loop and not a stable editor-semantics promise. |
| Private crate internals | explicitly forbidden | Workbench architecture | out of scope | Workbench must not couple to private crate internals or absorb core ownership through convenience integrations. |
| Parser / typechecker ownership | explicitly forbidden | Workbench scope and architecture | no second compiler frontend | Workbench may request public checks, but it must not own parsing or typechecking semantics. |
| Verifier / VM / runtime ownership | explicitly forbidden | Workbench scope and architecture | no second execution core | Workbench may display verifier, VM, trace, quota, and runtime outputs, but verifier admission, VM execution, and runtime contracts remain outside Workbench. |
| Alternate release scoring | explicitly forbidden | Workbench scope, release policy | not release authority | Workbench must not create release scoring independent from local gates, release scripts, release docs, and final human release decisions. |
| PROMETHEUS private state editing | explicitly forbidden | Workbench scope and architecture | host-boundary protection | Workbench must not edit private PROMETHEUS state or grant capabilities outside admitted boundary policy. |
| Workbench stability claim | explicit non-claim | Workbench beta docs, status docs | public-status guard | Workbench readiness classification does not claim stable Workbench, production-ready status, release readiness, or UI-driven widening of Semantic language/runtime behavior. |

## Documentation rule

README, examples, and public docs should avoid presenting roadmap or
unqualified features as stable behavior.

A feature should be promoted in public-facing documentation only when it has:

- a documented contract;
- test or golden coverage;
- verifier / VM evidence where applicable;
- CLI-visible behavior where applicable;
- explicit inclusion in the current release surface.

## Current project framing

Semantic should be described as:

```text
an emerging verifier-first deterministic execution platform
under active Practical Core Completion,
with a limited qualified release surface.
```

It should not be described as a mature general-purpose application language yet.
