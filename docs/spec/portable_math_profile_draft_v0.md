# Host-Math-Free Admission Profile — Design Draft v0

Status: OWNER-APPROVED DESIGN / NOT_IMPLEMENTED / NOT_PUBLISHED_STABLE.

Owner decisions recorded here: Q1–Q9 (2026-11-08), revised by the S14-B2 reconciliation
(2026-11-08): Q1 control-flow reachability, Q9 split into Q9a and Q9b (architecture V+), the
optimizer monotonicity obligation, the `HOST_MATH_FREE@draft1` identity and its limited guarantee;
and by S14-B2-R1 (2026-11-08): admission split into the Host Policy Gate (Q7) and the Executable
Host Math Analysis (Q1, Q9a).

This document records the approved design of a future host admission profile. It is not a live
contract: no verifier, VM, compiler, optimizer, issuer, admission API, host policy, header,
capability, opcode, quota, effect or error code implements or allocates anything described here.
It does not change `semantic.foundation.std/0.1` (`foundation_stdlib_v0.md`),
`semantic.compiler.u32/0.1` or any published contour. Implementing it requires a separate checkpoint
with explicit owner GO (§16).

The key words MUST, MUST NOT and MAY state requirements on a future implementation and on future
qualification evidence. They describe no current behaviour.

## 1. Current behaviour this design is bounded by

| # | Fact on `main` | Authority |
|---|---|---|
| F1 | `sin`, `cos`, `tan`, `pow` execute host libm (`f64::sin/cos/tan/powf`) | `crates/sm-vm/src/semcode_vm.rs` `try_eval_builtin_call` |
| F2 | `sqrt` and `abs` execute `f64::sqrt` / `f64::abs` | same |
| F3 | `CAP_F64_MATH` is a SemCode header capability required by every math builtin **and** by ordinary `LoadF64` and `f64` arithmetic | `crates/sm-verify/src/lib.rs`, `crates/sm-format` |
| F4 | `Call` resolves user-first: a program function of the callee name is entered before any builtin is tried; a name that is neither is an unknown function | `semcode_vm.rs` `Opcode::Call` |
| F5 | All six math names are `UserShadowable` | `crates/sm-front/src/lib.rs` `BUILTIN_NAMESPACE` |
| F6 | `std.math` selects `sqrt`/`abs`; `sqrt` promises bit-exact results only for `x >= 0.0` and only "is NaN" otherwise; `sin cos tan pow` are Deferred | `foundation_stdlib_v0.md` |
| F7 | Duplicate function names are rejected by the verifier (`DuplicateFunction`) | `sm-verify` |
| F8 | `crates/sm-admission` is a compile-only boundary with no authority, issuer, verifier or VM API; the semantic-admission error taxonomy is open | `crates/sm-admission/src/lib.rs`, `docs/architecture/q3_10_d6_r5_normative_reconciliation.md` |
| F9 | The verifier computes per-function control-flow successors (`Ret`: none; `Jmp`: its target; `JmpIf`: its target and the next instruction; every other opcode: the next instruction) and the set of offsets reachable from offset 0. It does not reject CFG-unreachable instructions, but it checks the call target of every decoded `Call` and `MakeClosure`, reachable or not: a `Call` must name a program function or a builtin known to the verifier, and a `MakeClosure` must name a program function | `sm-verify` `verify_reachable_control_flow`, `UnknownCallTarget`, test `verifier_rejects_builtin_closure_target` |
| F10 | The only call-like opcodes are `Call` and `ClosureCall`. Inside the VM a closure value is created only by `MakeClosure`; an embedder can still pass a closure value as an entry argument | `crates/sm-format/src/local_format.rs` `Opcode`; `semcode_vm.rs` |
| F11 | The verifier's instruction decoder, successor table and reachable-offset set are internal; no public API exposes them | `sm-verify` (`VerifiedSemCode` exposes function envelopes only) |
| F12 | From SemCode revision `SEMCODE_SIGNATURE_MIN_REVISION` (header V19) each function may carry a canonical signature: one coarse family per parameter (`Quad`, `Bool`, `Text`, `Sequence`, `Map`, `Closure`, `I32`, `U32`, `Fx`, `F64`, `Tuple`, `Record`, `Adt`, `Unit`, `Bytes`). Element and field types of containers are not recorded | `crates/sm-format/src/local_format.rs` `CallableSignature`, `CallableValueFamily` |
| F13 | No guest effect can move a callable across the guest boundary: `GateWrite` and `StateUpdate` pass values through the host ABI, which accepts only scalar values (closures, tuples, records, ADTs, sequences, maps, text and bytes are rejected at run time); `PulseEmit` and `EventPost` carry only a signal name; the host ABI never yields a closure; the VM keeps no state shared between executions. The entry parameters and the entry result are the only remaining channels | `semcode_vm.rs` `value_to_abi`, `value_from_abi`, `Opcode::PulseEmit`, `Opcode::EventPost` |
| F14 | No public VM API invokes a closure value. `ClosureValue` has public fields (`function_name`, `captures`), so an embedder can construct one and pass it as an entry argument, or run a returned closure's target function as a new entry. `ClosureCall` passes a closure's captures as the leading ordinary parameters of its target function (`captures` followed by the argument), so captures reach a function only through its declared parameters | `semcode_vm.rs` `ClosureValue`, `Opcode::ClosureCall`, `run_verified_function_semcode_with_args` |
| F15 | Every frame, including the entry frame, is admitted by `push_frame_in`, which checks each actual argument's runtime family against the callee's canonical signature (exact family equality) before the callee's first instruction. Without a canonical signature this check is skipped | `semcode_vm.rs` `validate_call_arguments`, `push_frame`, `push_frame_in` |
| F16 | No canonical profile-request representation exists: the SemCode header carries only `magic`, `epoch`, `rev` and `capabilities`; the known sections are `ADT0`, `DBG0`, `EVR0`, `OWN0` and `SIG0`; no capability bit, compiler pragma, VM or admission API expresses a profile request | `crates/sm-format/src/local_format.rs` `SemcodeHeaderSpec`, section tags, `CAP_*`; `crates/sm-format/src/semcode_decode.rs` |

Because of F3, `CAP_F64_MATH` is a format bit and MUST NOT be used as a portability or admission
decision. Clearing or setting it proves nothing about reachable host math.

## 2. Profiles and the guarantee

- `LOCAL_CANONICAL_COMPAT` — today's behaviour exactly: whatever the verifier admits runs with host
  libm. No cross-OS or cross-ISA bit identity is promised. It remains the default and is unchanged.
- `HOST_MATH_FREE@draft1` — proposed. Its only promise is:

  > No potentially reachable prohibited host-math builtin from the admitted verified entry, under
  > the conservative call-graph analysis of §4.

  The prohibited set `D` of draft1 is the set of builtin identities
  `{abs, cos, pow, sin, sqrt, tan}`. A user function that shadows one of these names (F4, F5) is a
  program function, not a member of `D`; its body is analysed like any other function.

`PORTABLE_DETERMINISTIC@draft1` is the historical design name of this profile. It MUST NOT be
promoted, published or used as a portability guarantee. The non-guarantees of §12 are part of the
profile definition.

No silent downgrade from a strict host grant to `LOCAL_CANONICAL_COMPAT` exists, and no fallback
implementation of any builtin is introduced.

## 3. Admission order (fail-closed)

1. Verifier: the artifact must be valid SemCode. A malformed artifact is a verifier rejection, never
   a profile error.
2. Entry validation: a missing entry is rejected as today.
3. Profile admission: decided once, for the selected entry, before any guest instruction runs. No
   print, effect, quota consumption or other observable behaviour precedes a denial. A lazy denial
   at the first host-math `Call` is forbidden. It has two gates, in this order:
   - 3a. **Host Policy Gate** (Q7, §9). It selects and authorizes the profile from the trusted host
     configuration and, where one exists, the canonical profile-request input. Its result is the
     **policy verdict**: `AUTHORIZED(<profile>)` or `REFUSE(profile_request_exceeds_grant)`.
   - 3b. **Executable Host Math Analysis** (Q1, Q9a, §4–§6) under the profile selected at 3a. Its
     result is the **mathematical safety verdict**: `ADMIT`, or `REFUSE` with `May` and `Indet`
     (§5).

   Admission succeeds only if both gates accept. 3b runs only after 3a authorized a profile.
4. Execution.

A step runs only if every earlier step accepted. A later step never overrides an earlier rejection.

## 4. Reachability (Q1, Q2, Q8)

### 4.1 Q1 — conservative control-flow reachability

The analysis input is a verifier-admitted artifact `A`, a validated entry `e` and the trust context
of §6.1. The analysis MUST compute the following, and nothing weaker.

1. **Intra-function CFG.** For each program function `f`, `Reach(f)` is the set of instruction
   offsets reachable from offset 0 under the successor rule of F9. Both successors of every `JmpIf`
   are reachable; conditions, constants and loop bounds are never evaluated. `Reach(f)` MUST be
   identical to the verifier's reachable-offset set for `f`.
2. **CFG-unreachable code is ignored.** An instruction whose offset is not in `Reach(f)` contributes
   no edge and no hazard. This includes instructions after an unconditional `Ret` or `Jmp` that no
   reachable edge enters.
3. **Resolution.** A `Call` at a reachable offset with callee name `n` resolves exactly as the VM
   resolves it (F4): to the program function `n` if `A` contains one, otherwise to the builtin
   identity `builtin:n`. A divergence between the analysis, the verifier and the VM resolution rule
   is `INDETERMINATE(resolution_mismatch)`.
4. **Closures (C0, closed world).** A `MakeClosure` at a reachable offset of a reachable function
   contributes its target function to the closure-target set. If any reachable function contains a
   `ClosureCall` at a reachable offset, every function in the closure-target set is reachable. A
   `MakeClosure` at a CFG-unreachable offset contributes nothing.
5. **Fixed point.** `Reach(A, e)` is the least set of program functions that contains `e` and is
   closed under rules 3 and 4. It is computed to a fixed point; recursion terminates because the
   function set is finite.
6. **Unknown origins refuse.** The analysis MUST return `INDETERMINATE(<reason>)` (§5) for a
   callable whose origin lies outside the closed world. This covers:
   - an entry parameter whose type family can carry a callable, because an embedder can supply
     closure values (F10);
   - an entry whose parameter families the analysis cannot determine, including an artifact without
     canonical signatures (F12).

   A family can carry a callable if it is `Closure`, `Tuple`, `Record`, `Adt`, `Sequence` or `Map`.
   SemCode signatures record no element or field types (F12), so no container family can be proved
   callable-free in draft1. Rule 6 also covers, within the functions of `Reach(A, e)` at reachable
   offsets, a `ClosureCall` or other callable target the analysis cannot resolve or bound, and an
   opcode the analyser version does not support, and it covers an exceeded analysis resource limit.
   A resource limit MUST be a deterministic bound fixed in the trust context and measured over
   `Reach(A, e)` only. Non-executable sections and functions outside `Reach(A, e)` never trigger
   rule 6. `INDETERMINATE` is a refusal.

   An entry result that carries a closure needs no refusal: a closure value can run only as a new
   entry invocation of its target function (F14), and that entry is admitted on its own (Q8).

   Rule 6 judges actual entry arguments through their declared families because the VM rejects any
   argument whose runtime family differs from the signature before the entry runs (F15); without a
   signature that check is absent, which is why such an artifact is `INDETERMINATE`.

   The closed world rests on F13 and F14: entry parameters are the only channel by which a callable
   can enter the guest, and a callable leaving it through the entry result runs only as a new entry.
   If a future host ABI, effect or VM feature can carry a callable, the analysis MUST treat that
   channel as an unknown origin until this draft is revised.
7. **Unrelated functions do not refuse.** A function outside `Reach(A, e)` MUST NOT affect the
   result, whatever it contains. A function reachable only from a CFG-unreachable call site is
   outside `Reach(A, e)`.

### 4.2 Q2 — authority

The graph MUST be built by trusted code over verifier-admitted bytes, sharing the verifier's
decoder. Because of F11 this requires a future verifier-owned API, under its own GO.
Decoder-equivalence tests validate that sharing; they are not an alternative authority and do not
license a separate decoder. The analysis must be reconciled with the Q3.10 `sm-admission` gateway
design. Scanning source text, comments, debug symbols, string contents or raw bytes is not an
authority and is not a substitute. The S11-C research analyser is not promoted.

### 4.3 Q8 — scope

The result is per verified entry, keyed to the complete identity of §11. A result is never reused
across entries or artifacts.

## 5. Hazards and the admission result (Q3, Q4)

For a verified artifact `A` and a validated entry `e`:

- `Sites(A, e)` is the set of `Call` instructions at reachable offsets of the functions in
  `Reach(A, e)`.
- `May(A, e)` is the set of `b` in `D` such that some site in `Sites(A, e)` resolves to `builtin:b`.
- `Indet(A, e)` is the set of `INDETERMINATE` reasons raised by §4.1.
- `Hazards(A, e)` is `UNKNOWN` if `Indet(A, e)` is non-empty, and `May(A, e)` otherwise.

`UNKNOWN` stands for "any prohibited builtin may be reachable". It is never treated as harmless.

**Admission.** `HOST_MATH_FREE@draft1` admits `(A, e)` if and only if `Hazards(A, e)` is the empty
set. Otherwise it refuses.

**Profile dependence of step 3b.** The mathematical safety verdict depends on the profile selected
at step 3a. Under `HOST_MATH_FREE@draft1` it is the admission rule above. Under
`LOCAL_CANONICAL_COMPAT` step 3b imposes no restriction: its verdict is `ADMIT` without host-math
analysis, which preserves today's behaviour (§2).

**Q4 — result.** The result is deterministic. It reports the entry, the profile identifier and
version, the sorted set `May(A, e)` and the sorted set `Indet(A, e)`, and the verdict. A witness
path is reported only when one was actually derived; C0 never invents one. Under
`LOCAL_CANONICAL_COMPAT` no analysis runs, so `May` and `Indet` are reported as not applicable,
never as empty sets: an empty set would claim an absence of hazards that was not established.

**Q3 — diagnostic.** A refusal is a distinct admission-failure class, separate from verifier
diagnostics (`V00xx`) and runtime traps (`R000x`). The symbol `E_PROFILE_HOST_MATH` is a placeholder
used in design tables only. Its name and number are **not allocated** (OPEN, §14). The reason names
used in design tables (`callable_entry_abi`, `closure_target`, `resolution_mismatch`,
`unsupported_opcode`, `analysis_limit`) are provisional and not allocated either.

## 6. Determinism of the verdict (Q9a, Q9b)

### 6.1 Q9a — metadata invariance (strict)

Q9a applies to the mathematical safety verdict of the Executable Host Math Analysis (step 3b of
§3). Given the same verified executable graph, the same selected profile and the same trust
assumptions, that verdict MUST be identical. The Host Policy Gate (step 3a) is governed by Q7 (§9).
Neither gate reads debug data, comments, strings or unrelated metadata.

The analysis input is restricted to the executable projection `Exec(A)`: for each function, its
name, its verifier-decoded canonical signature (parameter families, F12) where the artifact carries
one, and its verifier-decoded instruction stream, in which every string-table index is replaced by
the referenced name when it is a `Call` or `MakeClosure` target and by an opaque placeholder
otherwise. String-table numbering, debug sections and debug symbols, comments, string literals used
as data, and any string-table entry not referenced as a call or closure target are outside
`Exec(A)`.

The trust context `T` is the profile selected at step 3a, the exact profile version, the entry
symbol, the closed-world and entry ABI assumptions, the analysis resource limits, the verifier and
decoder identity, and the analyser identity.

For two verifier-admitted artifacts `A1` and `A2`, if `Exec(A1) = Exec(A2)` and the trust context is
the same, the mathematical safety verdicts MUST be identical: verdict, `May`, `Indet` and any
witness. In particular, adding, removing or changing debug metadata MUST NOT change that verdict.

A cache miss caused by a changed artifact hash (§11) is not a verdict change. Re-analysis after such
a miss MUST produce the identical result.

### 6.2 Q9b — per-artifact soundness (V+)

Each verified artifact receives its own sound admission decision. If `(A, e)` is admitted under
`HOST_MATH_FREE@draft1` with trust context `T`, then no execution of `A` from `e` under the
closed-world and entry-ABI assumptions of `T` executes a `Call` that resolves to a member of `D`.

`O0` and `O1` artifacts of the same source MAY receive different verdicts when their executable
graphs differ. The analysis decides each artifact from that artifact alone. It MUST NOT consult
another artifact, an optimization level, compiler provenance or an optimizer qualification record,
and it MUST NOT rely on optimizer correctness instead of checking the final verified artifact.

The earlier draft rule that the same source at `O0`, `O1` and with debug symbols always receives the
same verdict is superseded: Q9a covers the debug-symbol part, and Q9b replaces the `O0`/`O1` part.

## 7. Optimizer monotonicity (qualification obligation)

This is a requirement on a future optimizer qualification, not an admission rule and not a property
claimed for the current optimizer.

**Comparable pair.** `(A0, A1, e)` is comparable when `A0` and `A1` are compiled from the same
source by the same compiler revision at `O0` and `O1`, under the same language semantics and the
same profile assumptions (closed world, entry ABI), both are verifier-admitted, and both contain the
validated entry `e`.

**Order.** On the values of `Hazards`, `X ⊑ Y` holds when `Y` is `UNKNOWN`, or when both are sets
and `X ⊆ Y`. `UNKNOWN ⊑ Y` holds only when `Y` is `UNKNOWN`.

**Obligation OM.** An optimizer revision satisfies OM only if `Hazards(A1, e) ⊑ Hazards(A0, e)`
holds for every comparable pair. Written with the convention that `UNKNOWN` contains every hazard,
this is `Hazards(O1) ⊆ Hazards(O0)`. A pair whose `O1` artifact fails verification or lacks the
entry is not comparable and is outside OM.

Consequences when OM holds:

- O1 introduces no hazard outside the abstract `O0` hazard bound, and never turns a determinate `O0`
  result into `UNKNOWN`. When `O0` is determinate, O1 introduces no potentially reachable member of
  `D` that is absent at `O0`. When `O0` is `UNKNOWN`, OM says nothing more about `O1`.
- If `O0` is admitted, `O1` is admitted.
- An `O0` refusal MAY become an `O1` admission when O1 removed the hazard from the executable graph.
- The comparison uses resolved builtin identities. A user function named like a builtin is not
  `builtin:<name>`, and raw names or instruction counts are not compared.

**Qualification.** OM is checked by differential testing of every pair in a qualification corpus.
Passing that corpus is evidence for the tested revision and corpus only; it does not establish OM
for every program, and a counterexample outside the corpus is still an OM violation. A violation
fails the qualification of that optimizer revision. It never weakens admission: the per-artifact
check of §6.2 still refuses any `O1` artifact with a hazard.

### 7.1 Proposed optimizer obligation OV — PROPOSED, NOT APPROVED, not normative

OV is recorded here as a separate proposal for owner decision. It is not part of OM, Q9a or Q9b, and
nothing in this draft depends on it.

> For the same source, compiler revision and entry: if the `O0` artifact is verifier-admitted and
> contains the entry `e`, the `O1` artifact is also verifier-admitted and contains `e`.

Without OV, a pair whose `O1` artifact fails verification or lacks the entry is simply not
comparable under §7.

## 8. `sqrt` and `abs` (Q5, Q6)

- **Q5.** `sqrt` is prohibited in draft1. The NaN bit policy is **unresolved** (OPEN, §14).
  Candidate approaches, none chosen: a pinned canonical quiet NaN, a trap on invalid input, or
  static domain exclusion.
- **Q6.** `abs` is also prohibited in draft1. `abs` is the first candidate for a later draft, after
  qualification under G-F64 (§12.2). `sqrt` follows only after a Q5 decision.
- Single-machine evidence shows non-portability only and is not a portability qualification.

## 9. Host Policy Gate — profile selection (Q7)

Q7 governs the Host Policy Gate (step 3a of §3): profile selection and authorization.

- **Authority.** Only trusted host configuration selects and grants a profile. A guest request
  cannot increase authority: a request stronger than the host grant is refused at 3a
  (`REFUSE(profile_request_exceeds_grant)`), never upgraded. A guest request cannot downgrade a
  mandatory host policy: on a host that grants `HOST_MATH_FREE@draft1`, the granted profile is
  selected whatever the request says, and the host does not silently fall back to
  `LOCAL_CANONICAL_COMPAT`.
- **Inputs.** The gate reads the trusted host configuration and, only if it exists, one explicitly
  defined profile-request input. Ordinary debug data, comments, strings and unrelated metadata MUST
  NOT influence either gate. No other part of the artifact is a request.
- **Representation.** No canonical profile-request representation exists today (F16). It is
  `NOT_IMPLEMENTED`. This draft creates no header field, section, opcode, capability or API for it.
  Until one is defined under its own owner GO, the gate has no guest request input, and its policy
  verdict depends on the host configuration alone.
- **Separation.** The request is never part of the trust context `T` of §6.1 and never an input of
  the Executable Host Math Analysis. 3a has its own result, reported separately from 3b.

## 10. Relation to existing trusted boundaries

The analysis adds no authority. The verifier remains the only authority on SemCode validity, entry
validation remains as today, and the VM keeps its user-first resolution and its runtime checks. The
profile can only refuse an artifact that the earlier steps accepted; it can never admit an artifact
that an earlier step rejected.

## 11. Proof and cache identity

The cache holds mathematical safety verdicts of step 3b under `HOST_MATH_FREE@draft1` only; the Host
Policy Gate (step 3a) is recomputed for every admission, and the unconditional
`LOCAL_CANONICAL_COMPAT` verdict needs no cache entry. A cached result may be reused only if every
field matches:

- artifact SHA-256;
- verified code digest;
- entry symbol;
- verifier and decoder identity and version;
- admission analyser identity and version;
- profile selected at step 3a (fixed by the host grant);
- exact profile version (`draft1`);
- backend algorithm identifier or host-refused marker per builtin;
- analysis assumptions (closed world, entry ABI) and analysis resource limits;
- cache schema version.

Any mismatch is a miss. A partial, stale or unverifiable proof fails closed. A key over `Exec(A)`
alone, which would share results between debug variants, is not authorized in draft1.

**Optimizer identity is not an admission key.** The verdict is a function of the verified artifact
(§6.2), and the artifact SHA-256 already distinguishes `O0` from `O1` output. Adding the optimizer
identity would not change any verdict. The optimizer identity belongs to the OM qualification record
(§7): compiler and optimizer revision, the pass pipeline, the analyser identity, the profile
version, and the digest of every corpus source.

## 12. Explicit non-guarantees and G-F64

### 12.1 What `HOST_MATH_FREE@draft1` does not promise

The profile does not claim, and no document may present it as claiming:

- cross-OS bitwise equality of `f64` results;
- cross-ISA determinism of NaN sign or payload bits;
- portability of signed zero, subnormal handling or the floating-point environment (rounding mode,
  flush-to-zero, denormals-are-zero);
- equality between `f64` values folded on the compiling host and values computed on the executing
  host;
- a portable libm or any replacement implementation of a builtin;
- that any of the policy vectors has passed. None has been executed.

### 12.2 G-F64

`G-F64` is a separate, future qualification gate for any claim about `f64` bit portability, and for
any builtin leaving `D`. It MUST include:

- at least two operating systems and at least two ISAs, including x86-64 and AArch64;
- pinned toolchain identities;
- bitwise comparison of finite values, infinities, signed zero, subnormals and NaN-producing
  operations;
- `O0` against `O1` comparison;
- compiling-host against executing-host comparison;
- explicit floating-point environment assumptions;
- a separate normative NaN policy.

G-F64 is not part of this draft's implementation and does not block development on any single
platform.

## 13. Threat coverage

| Threat | Required outcome |
|---|---|
| canonical profile request stronger than the grant on a local-only host (representation `NOT_IMPLEMENTED`, F16) | 3a `REFUSE(profile_request_exceeds_grant)`; 3b not reached |
| canonical profile request for a downgrade on a strict host | 3a `AUTHORIZED(HOST_MATH_FREE@draft1)`; no downgrade; 3b decides |
| debug data, comment, string or unrelated metadata claiming a profile | no effect on either gate |
| user function shadowing a builtin name | analysed body; admit if the body is clean |
| shadowing body that reaches `sqrt`/`tan` | refuse |
| two functions with the same name | verifier rejection |
| host math only in a dead branch (`if false`) | refuse; conditions are not evaluated |
| host math only after an unconditional `return` | admit; CFG-unreachable |
| host math only in an unreachable function | admit |
| closure reaching host math, including through a record, ADT, sequence or map | refuse (C0) |
| host-injected or unbounded callable | `INDETERMINATE`, refuse |
| entry parameter of a container family that could hold a closure | `INDETERMINATE`, refuse (F12) |
| entry result carrying a closure | no refusal; the closure runs only as a separately admitted entry (F14) |
| `MakeClosure` naming a builtin | verifier rejection |
| malformed bytecode or relabelled header | verifier rejection |
| stale, forged or cross-artifact proof | cache miss, re-analyse |
| missing entry | entry rejection |
| effect before host math | refuse before the first instruction |
| `"sin"` in a string, comment or debug symbol | no effect on the verdict (Q9a) |
| debug-only metadata change | identical verdict (Q9a) |
| `O0` and `O1` with different executable graphs | each decided on its own artifact (Q9b) |
| optimizer introduces a prohibited builtin | the `O1` artifact is refused; OM qualification fails |
| `CAP_F64_MATH` cleared to claim no math | not evidence (F3) |
| recursion hiding `pow` | fixed point, refuse |

The policy vectors that specify these outcomes are design expectations only. No vector has been
executed; none may be reported as a passing test until an implementation exists.

## 14. Open owner decisions

- Q3: the name and number of the admission-failure code.
- Q5: the `sqrt` NaN policy.
- The admission entry point and its reconciliation with Q3.10 `sm-admission`.
- The canonical profile-request representation (F16, §9): `NOT_IMPLEMENTED`; it needs its own
  owner GO.
- The proposed optimizer obligation OV (§7.1).
- The qualification matrix of G-F64 (operating systems, ISAs, toolchains).

## 15. Non-goals

Changing the default profile; any portable libm or replacement implementation; new verifier,
admission, issuer or host APIs; new headers, capability bits, opcodes, quotas or effects; error code
allocation; optimizer changes; Harness or CI changes; any claim that `sqrt` or `abs` is
bit-portable.

## 16. Future implementation checkpoint

A separate checkpoint, under its own owner GO, must at minimum provide:

- a verifier-owned, decoder-consistent API for instructions, successors and reachable offsets (F11);
- unit tests for every policy vector;
- mutation tests whose mutants compile and are killed by real failing tests;
- decoder-equivalence tests against the verifier;
- metadata-invariance differential tests (Q9a);
- `O0`/`O1` per-artifact tests and an OM differential suite with a qualification record (Q9b, §7);
- a proof that no guest instruction runs before a denial;
- cache-identity miss tests for every field of §11;
- G-F64 qualification before any builtin leaves `D`.
