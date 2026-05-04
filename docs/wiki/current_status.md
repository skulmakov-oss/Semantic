# Semantic Language — Current Status

Status: current-main orientation page  
Audience: repository Wiki / first-contact project readers  
Last synced from repository audit: after PR #353 and tracking issue #354

## 1. Short positioning

Semantic Language is not only a programming language and not only syntax sugar over an existing VM.

It is a deterministic, contract-driven execution platform for meaning-oriented programs, reasoning rules, semantic state transitions, and controlled runtime effects.

The current architecture is best read as:

```text
Semantic source
  -> frontend / semantic analysis
  -> IR / deterministic passes
  -> SemCode
  -> verifier admission
  -> deterministic VM
  -> PROMETHEUS boundary
  -> state / rules / audit / UI boundary
```

## 2. Current public posture

Use the canonical status vocabulary from:

- `docs/roadmap/public_status_model.md`

The repository distinguishes four status families:

1. `published stable`
2. `qualified limited release`
3. `landed on main, not yet promised`
4. `out of scope`

Important rule:

```text
Landed on main does not automatically mean published stable.
```

The current repository should be read as a strong limited-release / current-main development line, with several post-stable capabilities already landed but not automatically promoted into a stable public promise.

## 3. Core architecture

The current owner-split workspace is organized around these layers.

### Semantic core

- `sm-front` — frontend, parsing, source typing surface
- `sm-sema` — semantic analysis support
- `sm-ir` — IR and deterministic optimization passes
- `sm-emit` — SemCode emission
- `sm-verify` — verifier / admission gate
- `sm-runtime-core` — runtime-safe shared execution vocabulary
- `sm-vm` — deterministic SemCode VM
- `smc-cli` — command-line tooling pipeline
- low-level quad primitive compatibility perimeter — retained as non-owning historical/support surface; see `docs/legacy-map.md` for the exact inventory

### PROMETHEUS integration layer

- `prom-abi` — host-call ABI vocabulary
- `prom-cap` — capability policy
- `prom-gates` — gate descriptors and binding layer
- `prom-runtime` — runtime session orchestration
- `prom-state` — semantic state store
- `prom-rules` — deterministic rule agenda and rule evaluation
- `prom-audit` — audit, trace, replay-oriented records

### UI / application boundary

- `prom-ui`
- `prom-ui-runtime`
- `prom-ui-demo`
- `apps/workbench`

The UI layer is an operator/application shell. It does not own compiler, verifier, VM, or Semantic runtime semantics.

## 4. Execution model

Semantic execution is verifier-first.

```text
source
  -> AST
  -> typed source model
  -> IR
  -> optimized IR
  -> SemCode
  -> verified program
  -> VM state transition
  -> optional PROMETHEUS host boundary
```

Mathematically, VM execution is a deterministic state transition system:

```text
sigma[k+1] = delta(sigma[k], instr[pc])
```

Where VM state includes at least:

```text
pc, registers, frames, locals, quotas, active ownership paths, capability context, host boundary
```

## 5. Quad logic

Semantic has a native `quad` value domain:

```text
N = unknown
F = false
T = true
S = conflict
```

A useful implementation model is two-plane logic:

```text
N = (0, 0)
F = (0, 1)
T = (1, 0)
S = (1, 1)
```

Important source rule:

```text
if quad_expr    // forbidden
if state == T   // explicit comparison required
```

This keeps branch control boolean while allowing semantic data to carry unknown and conflicting evidence.

## 6. Current landed capability highlights

Current `main` includes substantial language and runtime surface beyond the early stable line, including:

- native `quad` logic;
- `bool`, `i32`, `u32`, `f64`, `fx`, `text`, `unit`;
- measured numeric forms / units-of-measure surface;
- records and ADTs;
- enum constructor and enum match paths;
- `Option(T)` and `Result(T, E)` families;
- tuple and record destructuring paths;
- sequence values and first-wave sequence iteration;
- first-class closures with immutable capture;
- function contracts: `requires`, `ensures`, `invariant`;
- deterministic imports / selected executable imports in the currently admitted contour;
- SemCode version ladder through ownership, closures, sequence iteration, and host-call capabilities;
- verifier-admitted VM execution;
- runtime quotas and bounded execution model;
- runtime ownership slice for tuple and direct record-field paths;
- PROMETHEUS host-call boundary through ABI/capability layers;
- semantic state, rules, agenda, rollback/audit-oriented substrate.

These capabilities must still be read through the public status model: not every landed capability is automatically a published stable promise.

## 7. Runtime ownership status

The current runtime ownership contract is intentionally narrow and frozen around tuple and direct record-field paths.

Supported:

- tuple `AccessPath`;
- direct record field `AccessPath`;
- `Borrow` and `Write` ownership events;
- `OWN0` SemCode section;
- `SEMCOD11` tuple ownership transport;
- `SEMCOD12` direct record-field ownership transport;
- frame-local borrow lifetime;
- runtime write rejection on overlap.

Explicitly unsupported in the current ownership contract:

- ADT payload paths;
- schema paths;
- partial borrow release before frame exit;
- advanced aliasing / region reasoning;
- inter-frame borrow persistence;
- indirect field selection;
- smart path normalization.

## 8. Current cleanup milestone

Repository-tail cleanup is tracked in:

- GitHub issue `#354` — `M-Tail: Repository Tail Cleanup`

Scope of that cleanup milestone:

- classify and clean stale `codex/*` branches;
- resolve closed-unmerged PR / branch tails such as `#324`;
- sync application ledger truth without pulling in snake implementation;
- audit `panic!` surface;
- audit `allow(dead_code)` / compatibility allowances;
- verify legacy/perimeter truth;
- separate Workbench backlog from Semantic core cleanup.

Explicitly excluded from that milestone:

- self-learning snake / benchmark-class application-completeness work;
- new language/runtime feature implementation;
- runtime ownership expansion;
- Workbench feature expansion;
- public release claim widening.

## 9. Active application-completeness stream

The self-learning snake / benchmark-class application stream is intentionally separate from repository-tail cleanup.

It is tracked through:

- `docs/roadmap/application_completeness_pr_ledger.md`
- `tests/fixtures/snake_benchmark/README.md`
- `tests/snake_benchmark_gap_matrix.rs`

Current benchmark-positive baseline includes:

- same-family text equality;
- same-family text concatenation and explicit scalar `to_text(...)`;
- enum/control-flow basics;
- same-family plain `i32` relational operators;
- ordered `Sequence(T)` indexing and iteration;
- first-class closure capture.

Known benchmark-family blockers remain in the application-completeness stream rather than the cleanup milestone, including:

- public integer arithmetic;
- statement loops and control exits;
- sequence utility layer;
- first-wave map surface;
- deterministic seeded PRNG;
- narrow stdout experiment surface.

## 10. Legacy and compatibility perimeter

The repository intentionally retains a narrow non-owning compatibility perimeter. The exact path inventory is intentionally kept in the dedicated legacy map rather than repeated here:

- `docs/legacy-map.md`

This perimeter is historical/compatibility-oriented. It is not a second owner of the `sm-*` Semantic platform contracts.

Any new architecture must land in the appropriate owner crate, not in legacy or compatibility paths.

## 11. Practical reading order

For a new reader, the recommended order is:

1. `README.md`
2. `docs/roadmap/public_status_model.md`
3. `docs/roadmap/v1_readiness.md`
4. `reports/g1_release_scope_statement.md`
5. `docs/spec/syntax.md`
6. `docs/spec/types.md`
7. `docs/spec/source_semantics.md`
8. `docs/spec/semcode.md`
9. `docs/spec/verifier.md`
10. `docs/spec/vm.md`
11. `docs/spec/runtime_ownership.md`
12. `docs/architecture/blueprint.md`

## 12. Current engineering rule

The current repository discipline is:

```text
one logical change
  -> one PR
  -> tests where behavior changes
  -> docs/spec sync where contract changes
  -> no silent release claim widening
```

If a cleanup task starts requiring new language/runtime capability, it should leave the cleanup milestone and move into the appropriate feature or application-completeness stream.
