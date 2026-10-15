# Runtime Quotas Specification

Status: draft v0
Model owner: `sm-runtime-core`
Enforcement owner: `sm-vm` (runtime-resident quotas); `sm-verify`
(`SymbolTable`, checked statically at admission - see "Ownership Rule")

## Purpose

Runtime quotas define the bounded execution contract for Semantic programs.

Quota model rule:

- `sm-runtime-core` defines quota taxonomy and baseline profiles
- `sm-vm` enforces quotas during execution, except `SymbolTable`, which
  `sm-verify` enforces statically, pre-execution, at admission
- higher integration layers may choose context-specific quota envelopes, but
  must not weaken the core safety contract silently - meaning divergence
  from the baseline must be explicit in provenance (recorded, not hidden),
  not that a weaker envelope is itself forbidden; see "Execution Envelope
  Provenance Rule" below (#1762, FA-08-004)

## Quota Taxonomy

Current quota kinds:

- `Steps`
- `Calls`
- `StackDepth`
- `Frames`
- `Registers`
- `SymbolTable`
- `EffectCalls`

Current quota descriptor fields:

- `max_steps`
- `max_calls`
- `max_stack_depth`
- `max_frames`
- `max_registers`
- `max_symbol_table`
- `max_effect_calls`

`RuntimeQuotas` also carries a field that is not part of this taxonomy -
`max_debug_symbols_per_function` - see "Verifier Admission Compatibility
Note" below.

## Current Baseline Profiles

### `verified_local`

- `max_steps = 100000`
- `max_calls = 16384`
- `max_stack_depth = 256`
- `max_frames = 256`
- `max_registers = 4096`
- `max_symbol_table = 16384`
- `max_effect_calls = 1024`

### `pure_compute`

- `max_steps = 100000`
- `max_calls = 16384`
- `max_stack_depth = 256`
- `max_frames = 256`
- `max_registers = 4096`
- `max_symbol_table = 16384`
- `max_effect_calls = 0`

### `kernel_bound`

- `max_steps = 250000`
- `max_calls = 32768`
- `max_stack_depth = 256`
- `max_frames = 256`
- `max_registers = 8192`
- `max_symbol_table = 16384`
- `max_effect_calls = 4096`

## Verifier Admission Compatibility Note

`RuntimeQuotas` also carries `max_debug_symbols_per_function`:

- `verified_local = 8192`
- `pure_compute = 4096`
- `kernel_bound = 16384`

This field:

- is a verifier-side per-function debug-metadata admission limit, enforced
  by `sm-verify::verify_function_code` against a decoded function's
  `debug_symbols.len()`
- is **not** a `QuotaKind` member and is not listed in the quota taxonomy
  above
- is **not** charged by `sm-vm` and has no runtime charge point
- is **not** surfaced as `RuntimeError::QuotaExceeded` - a rejection under
  this limit is a static admission failure
  (`VerificationCode::ResourceLimitExceeded`), not a runtime quota
  exhaustion
- remains a field on `RuntimeQuotas` only to preserve the existing
  per-profile admission plumbing (#1760, FA-08-002); it is intentionally
  kept distinct from the runtime quota taxonomy above rather than merged
  into it

`sm-format` independently enforces a fixed, profile-independent decode-time
structural cap, `MAX_DEBUG_SYMBOLS_PER_FUNCTION = 8192`, before this
verifier-layer check ever runs. `verified_local` (8192) and `kernel_bound`
(16384, which cannot widen the decoder's own fixed 8192 cap) make this
verifier-layer check dead code in practice; `pure_compute` (4096) is
strictly tighter than the decoder's 8192 cap, so this check has real,
distinct authority only for `pure_compute`.

## Context Mapping

Current `ExecutionContext -> RuntimeQuotas` mapping:

- `PureCompute -> pure_compute`
- `VerifiedLocal -> verified_local`
- `RuleExecution -> verified_local`
- `KernelBound -> kernel_bound`

Contract rule:

- context selection is explicit through `ExecutionConfig`
- default execution for standard verified runs is `VerifiedLocal`

## Trusted-Compiler Envelope

Status: SHF-R (#2030). It is an experimental contract candidate for
explicitly selected local execution, exposed only by application-mode
`smc run --envelope trusted-compiler` (see `docs/spec/cli.md`).

### Quotas

`RuntimeQuotas::trusted_compiler(max_steps, max_frames)` starts from
`verified_local()` and applies only explicit overrides:

| Field | Without overrides | Override |
|---|---|---|
| `max_steps` | 100000 | `--max-steps`, 1 ..= `TRUSTED_COMPILER_MAX_STEPS` (200000000) |
| `max_frames` | 256 | `--max-frames`, 1 ..= `TRUSTED_COMPILER_MAX_FRAMES` (1024) |
| `max_stack_depth` | 256 | follows `--max-frames` |
| `max_calls` | 16384 | P3 candidate: `--max-calls`, 1 ..= `TRUSTED_COMPILER_MAX_CALLS` (200000000), no greater than effective `max_steps` |
| `max_effect_calls` | 1024 | not overridable |
| `max_registers`, `max_symbol_table`, `max_debug_symbols_per_function` | as `verified_local` | not overridable |

The existing two-argument Rust constructor remains unchanged. The separate
`RuntimeQuotas::with_trusted_compiler_calls(value)` method applies the explicit
Calls override to the selected quota object, rejects zero/above 200000000 and
values above its effective Steps, and leaves every other field unchanged.
`CallsOverrideError` is separate from the existing `QuotaOverrideError`, so
existing exhaustive matches and the old constructor signature are preserved.
The CLI permits this method only for an explicitly selected `trusted-compiler`
envelope. Custom `ExecutionConfig` construction remains supported as before.

Omitted Calls stays exactly 16384; a Steps override alone never increases it.
The explicit-override relation is not imposed on existing configurations with
omitted Calls and a Steps budget below 16384.

- The context stays `ExecutionContext::VerifiedLocal`. This is a custom
  envelope under the "Execution Envelope Provenance Rule" below.
- The admission-relevant fields never change, so verifier admission of the
  same bytes is identical with and without the envelope.
- The CLI reports the effective quotas in `--metrics json`.

### Non-claims

Quota values bound counts, not cost:

- `Steps` counts decoded opcodes. A single step may clone a composite value
  (`Sequence`, `Tuple`, `Record`, `Map`, `Text`) whose size is unbounded,
  because register and variable reads copy values. CPU time and allocation
  traffic are therefore not bounded by `max_steps`.
  - SHF-R R0 measured `push`, `len`, indexing, field reads, record update
    and tuple passing at a constant step count but with wall time growing
    linearly in the size of the touched value.
- Frames are heap-allocated, and each frame may hold up to `max_registers`
  values of unbounded size. No quota accounts for bytes; there is no
  aggregate heap bound.
- No quota bounds wall-clock time.

Raising `max_steps` or `max_frames` is therefore not a memory-safety or
time-safety measure. Larger maxima require, at minimum:

- an aggregate heap or allocation bound, or a proven bound on per-step cost;
- Windows and Linux measurements at the proposed limit;
- a revision of this contract.

### Historical compiler-workload blocker and P3 candidate

`Calls` counts every admitted non-root call over the whole execution, not
the stack depth.

The figures below are informative, not normative. They come from the SHF-R
R0 measurements of MiniLang workloads on Windows x64 at base
`327c0b6d9b70c73d0f674d0499e7b5803856e07e` (#2030 evidence).

- The SHF-R measurements on MiniLang workloads issue `Call` opcodes at about
  6.5-7 % of steps. Its lexer makes about 9 calls per source byte.
- Under the trusted-compiler envelope with `--max-steps 200000000`, the
  lexer stops with `Calls limit=16384 used=16385` on inputs of 2880 bytes
  and above. Its practical ceiling is about 1.8 KB of source, independent of
  `max_steps`.

Raising `Steps` alone therefore does not remove this blocker. P3 proposes
an opt-in override, retaining 16384 as the default. This is a reviewable
experimental contract candidate, not a released/stable resource guarantee.

### P3 accounting rationale and alternatives

Every admitted non-root frame is charged once in `sm-vm::push_frame_in`.
On the supported trusted SemCode execution routes it is reached from one
`Call` or `ClosureCall` dispatch. The common loop charges that instruction's
Step first; target/signature/frame/stack/register checks precede the Calls
charge. Root entry is exempt; inline builtins spend Steps without Calls.
Direct, recursive and closure invocations share these charge points. Thus
stored charged `calls <= steps`; a Steps failure prevents a later Calls
charge, and a rejected Calls increment is not committed. Overflow fails
closed. This is a count relationship, not a bound on cost per call.

The proposed 200000000 Calls cap reuses the existing experimental maximum
Steps count: no permitted explicit Calls budget exceeds the permitted Steps
count. It does not assert that running that many calls is operationally safe
on all hosts. A lower fixed cap (for example the research alternative 2000000)
would provide a stricter operational ceiling, but available toy/partial
compiler workloads do not justify a universal compiler-size threshold.
Allowing Calls above Steps would configure an unreachable budget. Removing
Calls charging or redefining it as depth would break the existing contract.
These alternatives and the numeric cap remain subject to final OWNER
contract review; implementation qualification does not approve a release.

Higher Calls does not establish an aggregate heap limit or wall-clock limit,
does not bound retained values across calls, and grants no capabilities or
external-caller trust. Frames/StackDepth still bound simultaneously live
frames; they do not bound bytes per frame. Other resource and semantic
failures can still terminate a compiler workload.

## Execution Envelope Provenance Rule

Frozen and implemented (#1762, FA-08-004,
`docs/roadmap/stable_foundation/ssf08_1762_execution_envelope_provenance_decision.md`):

- `ExecutionContext` is a construction-time baseline/default selector (via
  `ExecutionConfig::for_context`) and an audit-class label. It is **not**
  proof, by itself, of which `RuntimeQuotas` values actually governed a
  given execution.
- The `RuntimeQuotas` values carried in `ExecutionConfig.quotas` are the
  actual effective authority for the quota/profile dimensions
  `RuntimeQuotas` represents. This does **not** extend to
  `VerificationLimits` (an orthogonal verifier envelope),
  `sm-format`'s structural decode-time caps, verifier rules unrelated to
  `RuntimeQuotas`, capability policy, or any other independent admission
  authority.
- `ExecutionConfig::new(context, quotas)` remains supported: custom
  envelopes (a `context` paired with `quotas` that diverge from
  `for_context`'s canonical mapping) are permitted, with no
  equality-with-baseline validation and no stricter-than-baseline
  restriction.
- Because custom envelopes are permitted, `prom_runtime::RuntimeSessionDescriptor`
  and `prom_audit::AuditSessionMetadata` record the **effective**
  `RuntimeQuotas` actually used - copied from the same `ExecutionConfig`
  passed to session construction, never re-derived from `context` - so
  that two sessions sharing the same `ExecutionContext` but running under
  different `RuntimeQuotas` remain distinguishable in provenance. See
  `docs/spec/audit.md`'s "Effective Quota Provenance Rule" for the archive
  wire-format consequence.

## Enforcement Rule

Quota enforcement happens at runtime for the resources the VM cannot prove
statically.

Current enforced areas include:

- opcode-dispatch count (`Steps`)
- admitted non-root call count (`Calls`)
- frame count
- effective stack depth
- register growth
- effect-call budget

Quota exhaustion is not a single caller-visible channel; the real,
current split is:

- `Steps`, `Calls`, `Frames`, `Registers`, `EffectCalls` exhaustion
  produces `RuntimeError::QuotaExceeded(QuotaExceeded { kind, limit,
  used })`, with the payload preserved and caller-visible.
- `StackDepth` exhaustion is an existing compatibility exception: it is
  remapped to `RuntimeError::StackOverflow` before reaching a caller,
  which does **not** carry the `QuotaExceeded { kind, limit, used }`
  payload - the stack limit is still governed by the shared runtime
  quota contract (`max_stack_depth`), but its exhaustion is not
  caller-visible as a `QuotaExceeded` occurrence.
- `SymbolTable` exhaustion is not an `sm-vm` runtime `QuotaExceeded`
  occurrence at all - it is a static `sm-verify` admission rejection,
  before execution begins (see "Ownership Rule").

`Steps`/`Calls` semantics (charge point, root-frame exemption, exhaustion
timing, and `usize::MAX` overflow discipline) are frozen in
`docs/roadmap/stable_foundation/ssf08_1759_steps_calls_contract_decision.md`.
`Steps` / `Calls` / `EffectCalls` execution counters use overflow-safe
fail-closed charging (a shared `checked_add`-based primitive): ordinary
usage reports the exact attempted value, and the single unrepresentable
case (the increment itself would overflow `usize::MAX`) fails closed with
`used` SATURATED at `usize::MAX` for reporting only - never a silent wrap,
never continued execution.

## Determinism Rule

Quota behavior must be deterministic for the same:

- verified bytecode
- execution config
- runtime entry path

The VM must not:

- continue execution after quota exhaustion
- downgrade quota failure to a warning
- hide which quota kind was exceeded for `Steps`, `Calls`, `Frames`,
  `Registers`, or `EffectCalls` - the `QuotaExceeded { kind, limit, used }`
  payload for these five is caller-visible in full. `StackDepth` is the
  existing, documented compatibility exception to this rule (see
  "Enforcement Rule"); `SymbolTable` is not a runtime `QuotaExceeded`
  occurrence at all.

## Ownership Rule

The following ownership split is mandatory:

- quota taxonomy: `sm-runtime-core`
- quota enforcement: `sm-vm`, except `SymbolTable`, which `sm-verify`
  enforces statically at admission, before execution begins
- session-level quota selection: higher orchestration layers

## Version Review Rule

The following changes require quota contract review:

- changing a quota kind meaning
- adding a new quota kind
- changing a baseline profile value in a user-visible execution path
- changing error-reporting semantics for quota exhaustion

Required follow-up:

1. update this specification
2. update quota-related tests
3. update user-facing reporting if the surfaced contract changed
