# SSF-08 Lane 5 / #1902 — snake_learning Execution-Envelope Decision

## 1. Status

**CONTRACT DECISION ONLY. NO PRODUCTION BEHAVIOR CHANGE, NO BENCHMARK EDIT,
NO QUOTA CHANGE, NO CLI CHANGE.**

Baseline SHA: `18830d54a49706ba26bdcdd3b24a12498837fdb0` (`main`, confirmed
current via `git rev-parse origin/main`).
Target: `#1902` / `FA-08-012`.
Umbrella: SSF-08 umbrella `#1579` remains OPEN.
Lane: SSF-08 Lane 5, the last residual after `#1759`/`#1900`/`#1760`/
`#1761`/`#1762`/`#1763` (all CLOSED).

This document freezes the exact contract for why
`examples/benchmarks/snake_learning.sm` fails under `smc run`'s default
`VerifiedLocal` execution context, whether either published execution
context (`VerifiedLocal` or `KernelBound`) is the right home for it, and
what the narrowest contract-preserving remedy is - before any code,
test, or documentation file is touched.

## 2. Scope

In scope: classifying the artifact, measuring its true resource cost,
auditing the CLI/context/quota contract surface, evaluating five
candidate remedies, and freezing an exact future implementation
mechanic for a **separately authorized** implementation checkpoint.

## 3. Non-goals

This checkpoint does not:

- change `RuntimeQuotas` values for any context;
- add a CLI flag or environment variable;
- change `smc run`'s default behavior for any program;
- change `ExecutionConfig` semantics;
- change audit/provenance format;
- change verifier limits, `RuntimeError`/`RuntimeTrap`, or Steps/Calls
  charge semantics;
- change `snake_learning.sm`'s parameters;
- unignore or otherwise edit `tests/snake_learning_benchmark.rs`;
- touch `#1579`'s lifecycle state.

## 4. Exact baseline

`origin/main` = `18830d54a49706ba26bdcdd3b24a12498837fdb0`, confirmed via
`git fetch origin && git rev-parse origin/main` immediately before this
investigation began. Guards confirmed fresh at that moment: `#1913`
MERGED, `#1763` CLOSED, `#1579` OPEN, `#1902` OPEN. Hosted push
qualification green on that exact SHA (13/13 check-runs
`completed`/`success`).

## 5. Problem statement

`examples/benchmarks/snake_learning.sm`, run via `smc run <file>` (the
single-argument CLI form, which dispatches to
`cmd_run_controlled_observation` → `collect_controlled_observation_envelope`,
`crates/smc-cli/src/app.rs:2967,2789`), executes under the `VerifiedLocal`
execution context (`RuntimeQuotas::verified_local()`, `max_steps =
100_000`, hardcoded at `app.rs:2799` with no override surface) and fails:

```
QuotaExceeded { kind: Steps, limit: 100000, used: 100001 }
```

`tests/snake_learning_benchmark.rs::snake_learning_passes_check_run_compile_verify`
bundles `smc check` → `smc run` → `smc compile` → `smc verify` into one
`#[test]` and is entirely `#[ignore]`d citing this issue, so today the
benchmark has **zero test coverage of any kind** - not even the purely
static `check`/`compile`/`verify` guarantees, which never execute the VM
at all.

## 6. What the current failure proves

Only: `required_steps > 100000`. Nothing else. The `used: 100001` value
is the point at which `charge_counter` (`crates/sm-vm/src/semcode_vm.rs:2051`)
fired, not the workload's true completion cost - failed executions do not
run to completion, so this number carries no information about how much
further execution would have been needed.

## 7. What the current failure does NOT prove

It does not prove `required_steps <= 250000` (`KernelBound`'s budget),
that `KernelBound` is sufficient, that `VerifiedLocal` is wrong, that
`snake_learning`'s parameters are wrong, that the CLI needs a context
selector, or that the published baseline should change. Each of these
is evaluated on its own fresh evidence below (§13-§17).

## 8. Current execution-context model

`ExecutionContext` (`crates/sm-runtime-core/src/lib.rs:126-132`) has four
variants, none of which carries a doc comment anywhere in the workspace
explaining its intended role - the only role language that exists at
all is `docs/spec/vm.md:94-97`: *"context selects the runtime quota
baseline; context does not weaken verifier admission or SemCode safety
checks."*

| Context | Quota baseline | Reachable from `smc` CLI | Real construction sites (non-degenerate) |
|---|---|---|---|
| `PureCompute` | `max_effect_calls = 0`, otherwise `verified_local`-sized (`lib.rs:191-202`) | No | **None** - never constructed anywhere except the `for_context` match arm itself and its audit-label round-trip |
| `VerifiedLocal` | `max_steps = 100_000`, `max_calls = 16_384`, `max_frames = 256`, `max_registers = 4_096`, `max_stack_depth = 256`, `max_symbol_table = 16_384`, `max_effect_calls = 1_024` (`lib.rs:178-189`) | **Yes - the only CLI-reachable context**, hardcoded, no override | CLI (3 sites, §11), `sm-vm` library-default no-config shims, dozens of unit/integration tests |
| `RuleExecution` | identical values to `verified_local()` (`lib.rs:251-253`) | No | **None** - same as `PureCompute` |
| `KernelBound` | `max_steps = 250_000`, `max_calls = 32_768`, `max_registers = 8_192`, `max_effect_calls = 4_096`, `max_frames = 256`, `max_stack_depth = 256`, `max_symbol_table = 16_384` (`lib.rs:204-215`) | No | `prom-runtime`'s `ExecutionSession`/`GateExecutionSession` convenience constructors, `sm-vm`'s `PrometheusHostAbi`-typed compatibility shim, `tests/prometheus_*.rs` - see §17 |

`ExecutionConfig::for_context`'s match arm (`lib.rs:248-257`) only
selects a `RuntimeQuotas` value for any of the four - no capability
policy or verifier-admission behavior branches on context (confirmed by
grep: no `match`/`if` on `ExecutionContext::KernelBound` exists outside
that one arm and the `prom-audit` label serializer).

## 9. `snake_learning` artifact classification

| Dimension | Evidence | Confidence | Implication |
|---|---|---|---|
| Path | `examples/benchmarks/snake_learning.sm` | n/a | Both "example" and "benchmark" in path; no repo-wide doc distinguishes the two terms as "illustrative" vs. "stress" (confirmed: no top-level `examples/README.md` exists) |
| Expected user | `docs/roadmap/application_completeness_pr_ledger.md:19-21`: *"benchmark-class application experiments such as a self-learning snake"* - framed as a feature-completeness demonstration audience, not a performance-evaluator audience | High | Not authored for or by anyone measuring VM resource cost |
| Expected execution mode | Origin commit (`bb317691`, PR #415) commit message: *"Demonstrates a seeded deterministic training loop over 10 episodes... using the full admitted application surface"* (`Map`, `Sequence`+`prepend/pop/contains`, `random_seed`/`random_next_i32`, `while`, `print`/`to_text`) | High | Default local run, not an explicit high-budget invocation - the author never mentions resource budgets |
| Expected workload size | Golden assertions baked in at authoring time: `total_score == 8`, `total_steps == 1417` (game-loop iterations, not VM Steps) across 10 episodes, `max_steps: i32 = 200` per episode | High | Small, fully deterministic, fixed-shape functional example by domain-level measure |
| Result meaning | Semantic correctness (exact golden score/step values) - not performance, not learning quality (there is no varying-hyperparameter sweep) | High | A correctness fixture, not a benchmark in the performance sense |
| Public compatibility significance | Zero external references found (§10) - no CHANGELOG, no release doc, no CTF/PCC qualification doc. The closest is `reports/application_completeness_benchmark_verdict.md`, which self-limits: *"This verdict does not mean Semantic is public release"* / *"does not widen the published stable contour."* | High | Internal roadmap/test fixture only |

**Directory naming ("benchmarks") is evidence, not authority** (per this
checkpoint's own governing instruction) - and here the evidence it
carries points toward "feature-completeness experiment," not
"performance stress test," per the ledger's own definition of the term
for this exact file family.

## 10. Historical intent evidence

- `examples/benchmarks/snake_learning.sm` was authored complete, in a
  single commit (`bb317691`, PR #415, 2026-05-05), and **has never been
  modified since** (`git log --follow` returns exactly one commit).
  `n_episodes = 10`, `max_steps = 200`, and both golden assertions are
  original and unchanged.
- `RuntimeQuotas::verified_local().max_steps = 100_000` was set once, in
  commit `22c803b4` (2026-03-13), fully formed alongside its
  `pure_compute`/`kernel_bound` siblings, with **no stated numeric
  rationale** in the commit message. It has never been changed since.
- The number was **inert** (never actually enforced) for the entire
  five months both facts co-existed - it only began being checked when
  `#1759`'s enforcement PR (`#1901`, "fix(vm): enforce Steps and Calls
  runtime quotas") merged on 2026-09-07, the same day the failure was
  discovered and the `#[ignore]` was added. `ssf08_lane5_resource_failure_closure_audit.md:769-772`
  (§8 finding 5) frames this explicitly: *"FIRST REAL QUOTA VIOLATION
  SURFACED BY #1759's ACTIVATION - not a defect in #1759's
  implementation."*
- **This is not workload drift.** The file is byte-for-byte unchanged
  since authoring. The mismatch between the always-present true cost and
  the always-present published ceiling simply went unmeasured until
  enforcement existed to check it - falsifying H5 as literally "drift"
  while confirming its underlying premise (an unmeasured mismatch).
- `tests/snake_learning_benchmark.rs`'s four-verb (`check`+`run`+
  `compile`+`verify`) bundling was inherited wholesale, via two prior
  copy-paste generations, from a shared `check_run_compile_verify()`
  helper originally written for a generic `canonical_examples.rs`
  pattern (2026-04-24) - it was never designed around this specific
  benchmark's resource profile.
- The `#[ignore]` annotation's own doc comment (`tests/snake_learning_benchmark.rs:64-82`)
  already explicitly declines three remedies as "legitimate but separate
  decisions belonging to #1902": editing benchmark parameters, adding a
  CLI quota override, or raising the published baseline - and defers all
  of them here, as this document now does.

## 11. Current CLI contract

`smc run <single-file>` (the exact form the ignored test uses) and `smc
run <file> <app-args...>` (the multi-arg application-host form) and `smc
test <project-root>` are the **only three** `ExecutionConfig`
construction sites in `crates/smc-cli/` - at `app.rs:2799`, `:2950`, and
`:3124` respectively - and **all three hardcode
`ExecutionConfig::for_context(ExecutionContext::VerifiedLocal)` with no
override surface**. `cmd_check` (`app.rs:460`), `cmd_compile`
(`app.rs:292`), and `cmd_verify` (`app.rs:3060`) never construct an
`ExecutionConfig` at all - `check`/`compile` never execute the VM, and
`verify` calls `verify_semcode`/`verify_semcode_token` (static,
pre-execution admission, per `#1763`'s frozen taxonomy) - confirmed by
direct grep of every `ExecutionConfig`/`RuntimeQuotas`/`run_verified`/
`run_semcode`/`exec_loop` reference in `app.rs`, none of which fall
inside those three functions.

No CLI flag, subcommand option, or environment variable anywhere selects
a different `ExecutionContext` or overrides quota values (exhaustive
audit of `app.rs`'s flag parsing and env-var reads found only unrelated
uses: a source-parser `--profile` for `compile`/`dump-*` commands, an
`ApplicationCapabilityProfile` for `run`'s **capability manifest** - not
`ExecutionContext` - and toolchain/feature-hash/`NO_COLOR` env vars).
`docs/spec/quotas.md:126` is the only public documentation naming the
default: *"default execution for standard verified runs is
`VerifiedLocal`."* No `--help` text or `docs/spec/cli.md` entry mentions
execution context or quotas at all.

Custom envelopes are already fully supported at the library level:
`ExecutionConfig::new(context, quotas)` (frozen by `#1762`,
`ssf08_1762_execution_envelope_provenance_decision.md`) permits any
`context`+`quotas` pairing, with audit/provenance (`AuditSessionMetadata`,
`RuntimeSessionDescriptor`) recording the **actual** passed-in envelope,
never re-derived from `context` alone - confirmed live in
`prom-runtime/src/lib.rs:243-248,409-414` and
`smc-cli/src/app.rs:2828-2833`. No new plumbing is required to construct
and run under a custom envelope from Rust code; the CLI simply never
exposes that capability to end users today.

## 12. Actual workload parameters

`fn main()` in `snake_learning.sm:117-118`: `n_episodes: i32 = 10`,
`max_steps: i32 = 200` (per-episode cap on the game-domain training
loop, distinct from the VM's own `Steps` quota unit). 10 episodes, each
running the seeded Q-learning training loop to either death or the
200-iteration cap; golden invariant `total_steps == 1417` (total
game-loop iterations summed across all 10 episodes).

## 13. Exact runtime-cost evidence

Measured via `run_verified_entry_semcode_with_profile`
(`crates/sm-vm/src/semcode_vm.rs:1000-1021`, the existing
`vm-profile`-feature-gated measurement harness already used by
`crates/sm-vm/tests/vm_opcode_profile.rs`), which shares the exact same
`vm.steps = charge_counter(...)` charge point
(`semcode_vm.rs:2051`) as production execution, immediately followed by
`profile.record_opcode(opcode)` (`:2052`) - the profile's
`total_instructions()` is therefore charge-for-charge identical to the
real `Steps` quota consumption, not an approximation. Measured via a
temporary, uncommitted diagnostic test (deleted before this document was
committed; never part of any commit).

**Unmodified `snake_learning.sm` (`n_episodes = 10`), run under a
sufficiently large custom `ExecutionConfig` (`VerifiedLocal` context,
all quota fields raised) to reach completion:**

- **Total Steps: 753,864**
- **Total Calls (`Opcode::Call`): 42,481** (`Opcode::ClosureCall`: 0 - no
  closures are used)
- Completion result: success, both golden asserts (`total_score == 8`,
  `total_steps == 1417`) hold.

**Minimal-raise precision check**: raising *only* `max_steps` to
`1_500_000` and `max_calls` to `90_000`, leaving every other
`VerifiedLocal` quota field (`max_frames = 256`, `max_registers = 4096`,
`max_stack_depth = 256`, `max_effect_calls = 1024`, `max_symbol_table =
16384`) at its exact default, still **succeeds** with the identical
753,864/42,481 counts. This confirms exactly **two** of `VerifiedLocal`'s
seven quota fields are the workload's real binding constraints; none of
the other five needs to move at all.

## 14. Determinism evidence

Three consecutive runs of the unmodified benchmark under the same
high-budget custom envelope:

| Run | Total instructions |
|---|---|
| 1 | 753,864 |
| 2 | 753,864 |
| 3 | 753,864 |

**PASS - fully deterministic.** Consistent with the source's explicit
per-episode seeding (`random_seed(episode * 137 + 42)`,
`snake_learning.sm:127`). No divergence observed; this is not a larger
problem masked by an envelope decision.

## 15. VerifiedLocal evidence

Fails: `753,864 > 100,000` (Steps, 7.5×) and `42,481 > 16,384` (Calls,
2.6×) - **both** dimensions exceeded, not just the one the original
failure message names. `VerifiedLocal` is the only CLI-reachable
context; no test in the workspace pins the literal `100_000` value
except the already-`#[ignore]`d `snake_learning_benchmark.rs` itself
(confirmed by exhaustive audit of every `QuotaKind::Steps`-related
assertion in `sm-vm`'s test suite - all use deliberately small,
locally-crafted quotas via `ExecutionConfig::new`, never the real
baseline; `tests/ssf04_effect_quota.rs` only ever overrides
`max_effect_calls`).

## 16. KernelBound evidence

Fails: `753,864 > 250,000` (Steps, 3.0×) and `42,481 > 32,768` (Calls,
1.3×) - **H3 is falsified**. `KernelBound`'s own published budget is not
merely "a little short," it is roughly a third of what the unmodified
workload needs on the Steps dimension. Selecting `KernelBound` instead
of `VerifiedLocal` changes nothing besides the `RuntimeQuotas` values
picked (confirmed: no code branches on `ExecutionContext::KernelBound`
for capability policy or verifier admission); audit/provenance fidelity
under `KernelBound` is preserved by the same generic `#1762` mechanism
that already covers every context (`ssf08_1762_execution_envelope_provenance_decision.md:541`
explicitly covers "KernelBound + custom stricter envelope" as correctly
distinguishable in provenance).

**Semantic fit is also doubtful, independent of the numeric shortfall.**
Every real (non-degenerate) construction site of `ExecutionContext::KernelBound`
in the workspace is consistently paired with the Prometheus
gate-registry/rule-engine/host-integration family: `prom-runtime`'s
`ExecutionSession::kernel_bound()`/`GateExecutionSession::kernel_bound()`
convenience constructors (the *only* named convenience constructor
either type provides - no `verified_local()` sibling exists),
`sm-vm`'s `run_verified_semcode_with_host_and_capabilities` (a
compatibility shim typed specifically over `PrometheusHostAbi`+
`CapabilityChecker`), and `tests/prometheus_*.rs`. **No production
(non-test) implementor of `PrometheusHostAbi` exists anywhere in the
workspace** - `smc-cli`'s `CliApplicationHost` implements a different
trait (`ApplicationHostAbi`) entirely. No doc comment or decision
document states *why* `KernelBound`'s budget is higher (whether "just
more resources at the same trust level" or something narrower), but its
consistent real-world pairing with the gate/rule-engine surface is the
closest thing to an implicit semantic boundary that exists - reusing it
for a plain CLI script benchmark would blur that boundary, not merely
grant more Steps.

**Conclusion: `KernelBound appropriate` = NO/UNPROVEN-AND-NUMERICALLY-INSUFFICIENT.**
Both grounds (insufficient budget, doubtful semantic fit) independently
disqualify it as a sole remedy.

## 17. Custom-envelope evidence

`ExecutionConfig::new(ExecutionContext::VerifiedLocal, custom_quotas)`
with only `max_steps` and `max_calls` raised (§13's minimal-raise
result) reaches completion deterministically, using an envelope that is
explicit, honestly labeled, and fully supported by the already-frozen
`#1762` provenance mechanism if ever routed through an audited path.
Constructing this envelope requires zero new production code - the
library API already exists and is already used elsewhere in exactly
this shape (`tests/ssf04_effect_quota.rs`'s struct-update overrides).

## 18. Test-contract evidence

`tests/snake_learning_benchmark.rs::snake_learning_passes_check_run_compile_verify`
bundles four independently-meaningful guarantees into one `#[ignore]`d
test:

- **check** - language acceptance (parse/typecheck/lower). Never touches
  `ExecutionConfig`/`RuntimeQuotas`, never executes the VM.
- **compile** - bytecode generation. Same: never executes.
- **verify** - static bytecode admission via `verify_semcode`
  (`cmd_verify`, `app.rs:3060`). Static, pre-execution, per `#1763`'s
  frozen taxonomy returns `RejectReport` directly - never touches
  runtime quotas.
- **run** - the only guarantee that executes the VM and can hit
  `QuotaExceeded`.

**This test conflates language/bytecode correctness with an operational
execution-budget check, by inheritance from a generic helper never
designed around this benchmark** (§10). Because the whole function is
`#[ignore]`d, **all four guarantees currently have zero coverage** - not
just the `run` guarantee. Splitting the test would *increase* current
coverage (restoring check/compile/verify immediately, at zero risk,
since neither depends on any quota) rather than weaken it, while giving
the `run`-to-completion guarantee its own dedicated, correctly-scoped
test rather than silently deleting it.

## 19. Blast-radius analysis

**`VerifiedLocal`** (142 occurrences across 18 files): CLI (3 sites, all
hardcoded, §11), `sm-vm` no-config production convenience shims (each
with a `_with_config` sibling accepting caller-supplied config), dozens
of unit/integration tests, `prom-audit` label round-trips,
`prom-runtime` host-integration types (which, notably, provide **no**
`verified_local()` convenience constructor at all - only `kernel_bound()`),
and docs-only mentions. **No test in the workspace pins the exact
`100_000` value** except the already-ignored `snake_learning` test
itself - raising it globally would not break any other passing test, but
per this checkpoint's own governing instruction, absence-of-breakage is
not sufficient evidence *for* a global numeric-contract change; only
one benchmark's evidence exists, which the brief explicitly disqualifies
as sufficient justification (§25).

**`KernelBound`** (all real construction sites are test/example code
tied to the Prometheus gate/rule-engine family, §16): zero production
callers exist. A future implementation that routed `snake_learning`
through `KernelBound` unchanged would still fail (§16), so this question
is moot unless `KernelBound`'s own values were also raised - which would
be an equally out-of-scope, equally single-benchmark-justified baseline
change.

## 20. Candidate A — reduce workload

Cut `n_episodes` until the workload fits `VerifiedLocal`. Measured
scaling (unmodified per-episode logic, golden asserts stripped for the
experiment only):

| `n_episodes` | Total Steps | Total Calls | Fits VerifiedLocal (100k/16384)? | Fits KernelBound (250k/32768)? |
|---|---|---|---|---|
| 1 | 2,342 | 125 | Yes | Yes |
| 2 | 5,111 | 281 | Yes | Yes |
| 3 | 10,491 | 586 | Yes | Yes |
| 4 | 116,672 | 6,571 | **No** | Yes |
| 5 | 222,874 | 12,556 | No | Yes |
| 6 | 329,064 | 18,541 | No | **No** |
| 10 (current) | 753,856-753,864 | 42,481 | No | No |

Scaling is **sharply non-linear**, not accidental bloat: cost jumps
~11× between `n=3` (10,491) and `n=4` (116,672), then increases by a
near-constant ~106,200 Steps per additional episode from `n=4` onward.
This is consistent with the Q-learning mechanic itself: episode 0
always goes straight (bootstrap, dies quickly); episodes 1-3 use a
still-sparse Q-table and likely also die quickly; by episode 4 the
learned policy survives measurably longer (toward the 200-step
per-episode cap) and the snake's body grows, making `is_danger`'s
`contains()` sequence scan (`snake_learning.sm:64-68`) more expensive
per step. **The cost is an emergent property of what the benchmark
demonstrates (a Q-learning policy improving over episodes), not
incidental overshoot.**

Fitting `VerifiedLocal` requires `n_episodes <= 3` - a 70% cut from the
authored 10, and since `greedy_action` only activates from episode 1
onward, an `n=3` run would demonstrate only 2 episodes of actual learned
behavior. This would necessarily invalidate both golden assertions
(`total_score == 8`, `total_steps == 1417`), which were authored
specifically for `n=10` in the file's single, deliberate origin commit
(§10).

**REJECT** as the sole remedy. It preserves neither the deliberately-chosen
episode count nor the golden regression values, and it does not fix the
underlying mismatch (a static parameter picked once, five months before
enforcement existed to check it) so much as re-author the benchmark
around today's budget - the opposite of "narrowest contract-preserving
remedy."

## 21. Candidate B — explicit higher context

Route `snake_learning` through `KernelBound` via a new CLI/library
selection surface. Falsified by §16: `KernelBound`'s own published
budget is itself insufficient (753,864 > 250,000 Steps, 3.0× short;
42,481 > 32,768 Calls, 1.3× short) - the numeric problem is not solved by
switching contexts, only by *also* raising `KernelBound`'s own values,
which is its own separate, unauthorized quota-baseline change and
carries the same single-benchmark-justification weakness as raising
`VerifiedLocal` directly (§25). Additionally, `KernelBound`'s only real
current usage is consistently tied to the Prometheus gate/rule-engine
host-integration surface, with zero production implementors - repurposing
it for a plain CLI script benchmark would be a semantic overload, not a
clean reuse.

**REJECT** as the sole remedy - both the specific named existing context
fails numerically and, independently, would be a doubtful semantic fit
even if the numbers worked.

## 22. Candidate C — explicit quota override

Preserve the workload and its `VerifiedLocal` label, but execute the
full-completion guarantee under an explicit custom `RuntimeQuotas`
override (`ExecutionConfig::new`, already supported since `#1762`).
§13's minimal-raise result shows only two fields (`max_steps`,
`max_calls`) need to move, both with a wide, deterministic, exactly-measured
margin. This does not require raising `KernelBound` or `VerifiedLocal`'s
own published values, does not require any CLI surface, and matches
provenance semantics already frozen by `#1762`.

The open question is *where* this override should live: exposing it as
a raw CLI quota-override flag is a materially larger, precedent-setting
API-surface decision (§11 confirms zero such surface exists today for
*any* program) that this one benchmark does not by itself justify.
Applying it narrowly, inside the test suite only (a Rust `#[test]`
calling library execution functions directly, never through `smc run`),
requires no CLI change at all and keeps every ordinary user's `smc run`
behavior - including running this exact file - completely unchanged.

**ACCEPT, narrowly** - as a test-local mechanism only, not as a new CLI
capability. See §27/§29 for the combined mechanic with Candidate E.

## 23. Candidate D — raise VerifiedLocal

Increase `RuntimeQuotas::verified_local().max_steps` (and/or
`max_calls`) globally. §19 confirms no other test in the workspace
depends on the exact `100_000`/`16_384` values, so the immediate blast
radius is low - but **low blast radius is not the same as sufficient
justification**. This checkpoint's own governing instruction requires
repo-wide evidence for a global widening, and explicitly disqualifies
one benchmark's requirement as sufficient (§25). `VerifiedLocal` is the
*only* CLI-reachable, documented default execution budget
(`docs/spec/quotas.md:126`) for every ordinary local `smc run` - widening
it changes the resource contract for every such program, not just this
one, on the strength of a single artifact's cost profile.

**REJECT.** No repo-wide evidence exists that `100_000`/`16_384` is
wrong for ordinary `VerifiedLocal` programs in general; only this one
artifact's measured cost is known, and it is not representative (§20's
scaling data shows its cost is driven by an emergent, learned-behavior
property specific to this Q-learning simulation, not a generic "typical
programs need more Steps" finding).

## 24. Candidate E — split test contract

Stop requiring `snake_learning`'s full stress-execution completion under
the same test that proves language acceptance and bytecode verification.
§18 confirms `check`/`compile`/`verify` never touch runtime quotas at
all - they can be unignored with **zero** risk of the `QuotaExceeded`
failure recurring, immediately restoring coverage that does not exist
today (the whole function is currently ignored). The `run`-to-completion
guarantee is preserved, not deleted, by giving it its own dedicated test
under an explicit, correctly-scoped envelope (Candidate C).

Default `smc run examples/benchmarks/snake_learning.sm`, invoked by an
ordinary user from the CLI, **continues to fail under `VerifiedLocal`
exactly as it does today** - this decision does not make that
`QuotaExceeded` error go away for ordinary CLI usage, and is not
intended to. The remedy is scoped to test coverage: proving the
benchmark's language/bytecode correctness (restored immediately) and its
full-completion correctness (via a dedicated, explicitly-labeled test),
without changing what any user observes running `smc run` on any file,
including this one.

**ACCEPT**, combined with Candidate C for the completion guarantee - see
§27.

## 25. Decision matrix

Legend: STRONG / ACCEPTABLE / WEAK / REJECT.

| Criterion | A (reduce) | B (KernelBound) | C (custom override, test-local) | D (raise VerifiedLocal) | E (split test) |
|---|---|---|---|---|---|
| Contract correctness | WEAK | REJECT (numerically insufficient) | STRONG | WEAK (unjustified global claim) | STRONG |
| Evidence fit | WEAK (contradicts authored intent, §10) | REJECT (§16) | STRONG (§13,§17) | REJECT (§19,§23) | STRONG (§18) |
| Blast radius | ACCEPTABLE (isolated to this file) | REJECT (would require also raising KernelBound) | STRONG (test-local only) | WEAK (global, unjustified) | STRONG (test-only) |
| User-facing API impact | ACCEPTABLE (no API change) | WEAK (would need new CLI surface) | STRONG (none) | WEAK (silently widens default budget for everyone) | STRONG (none) |
| Provenance clarity | ACCEPTABLE | ACCEPTABLE (if numbers worked) | STRONG (#1762 mechanism, explicit) | ACCEPTABLE | STRONG |
| Determinism | ACCEPTABLE | N/A (fails before completing) | STRONG (confirmed §14) | ACCEPTABLE | STRONG |
| Resource-bound integrity | ACCEPTABLE | REJECT (still fails) | STRONG (fail-closed preserved for default CLI) | WEAK (weakens the meaning of "VerifiedLocal" as a bound) | STRONG |
| Benchmark fidelity | REJECT (guts 10-episode demonstration, invalidates goldens) | REJECT (still fails) | STRONG (workload untouched) | STRONG (workload untouched) | STRONG (workload untouched) |
| Implementation scope | ACCEPTABLE (small diff, high semantic cost) | WEAK (needs new CLI + raised KernelBound) | STRONG (test-only) | WEAK (touches a published global constant) | STRONG (test-only) |
| Long-term architectural cleanliness | WEAK (couples benchmark authorship to current enforcement limits) | WEAK (overloads KernelBound's implicit meaning) | STRONG | WEAK | STRONG |
| Reversibility | ACCEPTABLE | ACCEPTABLE | STRONG (test-local, trivially revertable) | WEAK (a published contract number, harder to walk back) | STRONG |
| Risk of hiding a real workload problem | WEAK (masks the actual cost by shrinking the workload to fit) | REJECT (doesn't even hide it - still fails) | STRONG (cost stays fully visible and measured) | WEAK (raises the ceiling to hide the specific cost, with no general justification) | STRONG (makes the cost explicit and tested) |

## 26. Falsification results

- **H1** (ordinary example, should fit VerifiedLocal): intent-supported
  (§10 - author never mentions resource budgets) but empirically
  falsified as currently authored (§13,§15) - the mismatch was never
  checked until enforcement existed, not evidence the intent was wrong.
- **H2** (intentionally a stress benchmark needing higher budget):
  falsified - founding-intent evidence (§9,§10) frames it as a
  feature-completeness demo, and even a deliberately higher published
  context (`KernelBound`) still doesn't fit (§16).
- **H3** (KernelBound sufficient): **falsified** - needs 3.0× its Steps
  budget and 1.3× its Calls budget (§16).
- **H4** (VerifiedLocal baseline globally too small): not supported -
  zero repo-wide evidence beyond this one artifact (§19,§23); explicitly
  insufficient per this checkpoint's own evidentiary bar.
- **H5** (workload drifted beyond intended scale): falsified as
  "drift" specifically - the file is byte-for-byte unchanged since its
  single origin commit (§10) - though the underlying premise (an
  unmeasured cost/budget mismatch) is confirmed, just not via drift.
- **H6** (test conflates distinct guarantees): **confirmed** - `check`/
  `compile`/`verify` never touch runtime quotas; the four-verb bundling
  was inherited generically, not designed around this benchmark (§10,
  §18).

## 27. Selected decision

**SELECTED MODEL: composite of Candidate E (split test contract) and
Candidate C (explicit, test-local quota override) - no other candidate
alone or combination survives falsification.**

- **snake_learning classification**: an unchanged, deliberately-authored,
  fully-deterministic feature-completeness demonstration whose true
  execution cost (753,864 Steps / 42,481 Calls) was never measured
  against a real budget until `#1759`'s enforcement went live. Not a
  performance stress benchmark in intent; not a "shrink-to-fit" example
  in practice either, since its cost is an emergent function of the
  Q-learning behavior it exists to demonstrate.
- **required execution envelope**: an explicit custom envelope
  (`VerifiedLocal` context, `max_steps >= ~1,500,000`, `max_calls >=
  ~90,000`, every other quota field left at `VerifiedLocal`'s default),
  constructed and used **only** inside the test suite via the
  already-supported `ExecutionConfig::new` API.
- **default `smc run` behavior**: **unchanged** - continues to use
  `VerifiedLocal` with its current published values for every program,
  including this one run from the CLI by an ordinary user, who will
  continue to see the current, honest `QuotaExceeded` failure.
- **VerifiedLocal baseline**: unchanged.
- **KernelBound baseline**: unchanged.
- **benchmark workload**: unchanged - no parameter, no source line of
  `snake_learning.sm` is touched.
- **CLI surface**: unchanged - no new flag, option, or environment
  variable.
- **ignored test**: future implementation splits it into (a) an
  unignored `check`/`compile`/`verify` test (restoring coverage that
  exists nowhere today, zero risk since neither touches runtime quotas),
  and (b) a new, separately-named test that runs `snake_learning.sm` to
  completion under the explicit custom envelope above and asserts the
  exact golden values (`total_score == 8`, `total_steps == 1417`),
  proving completion is real rather than merely "no longer failing."
- **provenance**: unaffected for default CLI usage (nothing changes
  there); the new test does not route through the CLI/audit-trail path
  at all, so no audit-record change is implicated - if a future,
  separately-authorized change ever surfaced this envelope through an
  audited path, `#1762`'s existing rule already covers it correctly.
- **fallback**: NONE.
- **automatic quota escalation**: FORBIDDEN.
- **hidden benchmark exception**: FORBIDDEN - no production code (CLI or
  VM) is aware of `snake_learning.sm`'s existence or behaves differently
  for it; the entire remedy lives in test-suite code that already has
  unrestricted access to construct any `ExecutionConfig` it wants,
  exactly as `tests/ssf04_effect_quota.rs` already does for a different
  quota dimension.

## 28. Rejected alternatives

- **A (reduce workload)**: rejected because fitting `VerifiedLocal`
  requires cutting to `n_episodes <= 3` (§20), a 70% reduction that
  would leave only 2 episodes of actual learned (`greedy_action`)
  behavior, invalidate both authored golden values, and re-author a
  benchmark whose exact shape was deliberately fixed in its single
  origin commit - substituting "shrink until it fits today's enforcement"
  for the real, separate, harder question of what envelope this workload
  actually needs.
- **B (KernelBound)**: rejected because `KernelBound`'s own published
  budget is itself 3.0× (Steps) and 1.3× (Calls) short of what the
  unmodified workload needs (§16) - selecting it alone does not solve
  the problem - and because its only real usage is semantically tied to
  the Prometheus gate/rule-engine surface, not general "more budget for
  any script."
- **D (raise VerifiedLocal)**: rejected because this checkpoint found no
  repo-wide evidence that `100_000`/`16_384` is wrong for `VerifiedLocal`
  programs in general - only one artifact's cost is known, and its cost
  is driven by an emergent property specific to this Q-learning
  simulation (§20), not evidence of a generally-mis-sized baseline. A
  global, published resource-contract change is a materially larger
  action than what one benchmark's evidence supports.

## 29. Exact future implementation mechanic

Binding on the future, separately-authorized implementation checkpoint;
nothing here is left as an open choice.

- In `tests/snake_learning_benchmark.rs`: remove the single bundled,
  `#[ignore]`d `snake_learning_passes_check_run_compile_verify` test.
  Replace with:
  - An **unignored** test exercising `smc check`, `smc compile`, and
    `smc verify` (via the existing `check_run_compile_verify`-style CLI
    helper minus the `run` step, or a narrowed equivalent) against
    `examples/benchmarks/snake_learning.sm` - proving language
    acceptance and bytecode verification, at zero quota risk.
  - A **new**, separately-named test (e.g.
    `snake_learning_completes_under_explicit_high_budget_envelope`) that
    reads/compiles/verifies the same file, constructs
    `ExecutionConfig::new(ExecutionContext::VerifiedLocal, quotas)` with
    `quotas.max_steps` and `quotas.max_calls` raised per §22's measured
    margin, executes to completion via an existing verified-execution
    entrypoint (e.g. `run_verified_entry_semcode_with_config` or
    equivalent), and asserts the exact golden invariants already present
    in the source (`total_score == 8`, `total_steps == 1417`,
    non-empty Q-table). Does not assert an exact Steps/Calls count as a
    pass/fail gate (§30) - only that completion succeeds and the golden
    program output is correct.
- No change to `crates/smc-cli/src/app.rs`, `crates/sm-runtime-core/src/lib.rs`,
  `crates/sm-vm/src/semcode_vm.rs`, or `examples/benchmarks/snake_learning.sm`.
- No change to any golden snapshot.
- No CLI documentation change required, since no CLI behavior changes;
  an optional doc note in `examples/benchmarks/README.md` explaining
  that this file's full completion is validated by a dedicated
  high-budget test rather than the CLI default may be added at the
  implementing engineer's discretion, but is not required by this
  decision.

## 30. Required regression tests

- The restored `check`/`compile`/`verify` test must pass unconditionally
  (no quota dependency).
- The new completion test must assert the exact golden values
  (`total_score == 8`, `total_steps == 1417`), not merely "did not
  error" - protecting against a future silent behavior change being
  masked by a generously-sized envelope.
- The new completion test's envelope must not assert an exact
  Steps/Calls count as a hard boundary (that would reintroduce the same
  fragility this decision is correcting) - it must simply provide
  enough headroom (§22) for the already-measured, deterministic cost.
- No test may re-introduce a hardcoded exact-boundary assertion against
  `VerifiedLocal`'s or `KernelBound`'s published values for this
  benchmark.

## 31. Public documentation consequence

None required. `docs/spec/quotas.md`, `docs/spec/vm.md`, and
`docs/spec/cli.md` remain accurate as-is: `VerifiedLocal` is still the
correct, unchanged default for `smc run`; nothing about the published
context/quota contract changes. An optional, non-required
`examples/benchmarks/README.md` note is available to the implementing
checkpoint (§29).

## 32. Audit/provenance consequence

None. The new completion test does not execute through the CLI or any
audited path; default `smc run` provenance behavior for every program,
including this one, is unchanged. If a future change ever surfaced this
custom envelope through an audited path, `#1762`'s existing "effective
quotas are recorded exactly as constructed" rule already covers it
correctly with no further design work.

## 33. Compatibility consequence

None. No public API, CLI surface, quota baseline, or golden snapshot
changes. This decision's own implementation is additive to the test
suite only.

## 34. AC4 / Lane 5 consequence

`#1763` already satisfied AC4.d/AC4.e; those conclusions are not
reopened. `#1902` is confirmed to be a residual workload/envelope
consistency issue, not an enforcement defect:

- **AC4.a** (active quota enforcement): **NOT AFFECTED** - Steps/Calls
  enforcement is working exactly as designed; it correctly caught a
  real, always-present violation.
- **AC4.b** (inactive/deferred quota vocabulary): NOT AFFECTED.
- **AC4.c** (execution-envelope provenance): NOT AFFECTED - the existing
  `#1762` mechanism already correctly covers whatever envelope this
  decision's implementation ends up using.
- **AC4.d** (failure taxonomy): NOT AFFECTED - `QuotaExceeded` already
  fires exactly as `#1763` froze it.
- **AC4.e** (documentation accuracy): NOT AFFECTED - no active doc
  requires correction as a result of this decision.

## 35. #1579 consequence

SSF-08 umbrella `#1579` remains OPEN. This document does not modify,
close, or update any acceptance checkbox on `#1579`, and does not claim
SSF-08 is complete. Final `#1579` reconciliation happens only after
`#1902`'s implementation lands and is post-merge qualified.

## 36. Exit gate

- exact target `RuntimeTrap`/`RuntimeError`/quota/CLI shape: N/A (no
  production code changes in this decision or its implementation)
- exact test-suite mechanic: frozen (§29)
- exact envelope values and margin: frozen (§22's 753,864/42,481
  measured cost, 1,500,000/90,000 proposed limits, ~49.7%/52.8%
  headroom)
- payload/behavior preservation: confirmed zero change to default CLI
  behavior, benchmark source, or any published quota value
- documentation consequence: none required (§31)
- compatibility consequence: none (§33)
- AC4/Lane 5 consequence: none of AC4.a-e affected (§34)
- `#1579` consequence: remains OPEN, untouched (§35)

**Nothing above is left open.** A future implementation checkpoint may
proceed directly from this document without a further design pass.

## 37. Unresolved questions

NONE.

**CONTRACT FROZEN = YES. READY TO IMPLEMENT = YES.**

**Wait for owner GO before implementation.**
