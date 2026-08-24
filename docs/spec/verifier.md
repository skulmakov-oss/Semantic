# Verifier Specification

Status: draft v0
Admission owner: `sm-verify`

## Purpose

This document defines the current SemCode admission contract before standard VM
execution.

The verifier is a public admission layer.
It is not an internal VM detail and it is not an optimizer.

## Public Surface

Current verifier surface is centered on:

- `verify_semcode`
- `VerifiedProgram`
- `VerifiedFunction`
- `RejectReport`
- `VerifiedSemCode`
- `VerifiedEntrySemCode`
- `EntryResolutionError`

Canonical token-producing admission path:

- `verify_semcode_token` — admits against the default `VerifiedLocal` resource
  quota profile.
- `verify_semcode_token_with_quotas` — identical admission logic, but the
  caller supplies the `RuntimeQuotas` profile to admit against (e.g.
  `KernelBound`). `verify_semcode_token` is a thin wrapper over this with
  `RuntimeQuotas::verified_local()`. Rejection diagnostics report the budget
  and usage from whichever profile was supplied, not a fixed profile name —
  callers should not assume a rejection implies `VerifiedLocal` limits.

Compatibility / legacy admission surface:

- `verify_semcode`

## Verification Scope

Current SemCode verification checks include:

- header validity
- supported version validity
- function and section integrity (the number of function definitions in an
  artifact is a static structural bound owned by `sm-format`
  (`MAX_FUNCTIONS`), independent of any runtime resource quota; the verifier
  does not re-derive or narrow it from a quota profile)
- canonical (unambiguous) instruction framing
- opcode validity
- opcode/header-revision consistency
- operand shape validity
- jump-target validity
- reachable control-flow closure (no end-of-stream fallthrough)
- string and debug reference validity
- register-budget validity (against the caller-selected `RuntimeQuotas`
  profile; see Public Surface)
- program-wide runtime symbol-table budget validity (the number of *distinct*
  strings the VM will intern across every function, deduplicated by exact
  string value program-wide - not a per-function string-table entry count;
  also checked against the caller-selected `RuntimeQuotas` profile)
- call-target validity
- capability consistency with actual opcode usage

## Opcode Admission Matrix

The verifier treats opcode admission as part of SemCode admission.

This matrix documents the current public admission boundary. It uses operation
families rather than binary opcode numbers so that internal encodings can change
without turning this document into a stable bytecode ISA promise.

| Operation family | Admission status | Capability / boundary | Notes |
| --- | --- | --- | --- |
| Ordinary source-derived families: control flow, data movement, literal loading, arithmetic, comparison, calls, and returns | Admitted when produced by supported SemCode and consistent with the emitted contract | No extra capability beyond the artifact contract | Baseline verifier-admitted surface; this document intentionally does not stabilize binary opcode numbers. |
| `SEQUENCE_LEN` | Admitted only when the emitted contract carries `CAP_SEQUENCE_ITERATION` and the header family supports the sequence-iteration contract | Capability-gated | Built-in sequence lowering opcode for the admitted `Sequence(T)` iteration slice. |
| Effect-oriented host-boundary families such as `GateRead`, `GateWrite`, and `PulseEmit` | Admitted only when the emitted contract matches the required capability envelope | Capability-gated / host-boundary | These opcodes do not define capability policy semantics by themselves. |
| Ownership transport payloads admitted through `OWN0` | Admitted structurally only when the ownership transport slice is present and well formed | Header and capability consistency required | This covers the currently documented tuple-only and direct record-field ownership transport slices. |
| Unknown, unsupported, or malformed opcode encodings | Rejected | N/A | Rejection must happen before a successful VM execution path. |
| Opcode streams that fail operand, jump-target, reachable-control-flow, call-target, closure-function-target, register-budget, string-reference, or section-integrity checks | Rejected | N/A | Direct calls may resolve to declared functions or admitted builtins; closure targets must resolve to declared functions. These are verifier admission failures, not successful runtime executions. |

Current ownership-specific structural checks for ownership transport include:

- `OWN0` section presence and layout validity when ownership transport is used
- admitted ownership event kind validity
- tuple `AccessPath` payload validity under `SEMCOD11`
- direct record-field `AccessPath` payload validity under `SEMCOD12`
- structural admission for valid `Borrow(Field)` and `Write(Field)` payloads
- header/capability consistency for ownership transport

## Canonical Operand Value Domains

Operand shape validity is not only "a byte is present" — for fields with a
fixed value domain, the byte must also be the canonical encoding for that
domain. The verifier enforces:

- boolean literal operands (`LOAD_BOOL`): must be `0` (false) or `1` (true)
- quad literal operands (`LOAD_Q`): must be `0..=3` (the four-value quad
  domain)
- presence-flag operands (`CALL` and `CLOSURE_CALL` destination-present,
  `RET` source-present): must be `0` or `1`

A byte outside the canonical domain for its field is rejected at admission
(`OperandOutOfBounds`), not normalized downstream. This narrows the
previously admitted surface: byte values outside these domains that an
earlier verifier accepted are no longer admissible.

## Opcode/Header-Revision Consistency

Header identity constrains the executable opcode vocabulary (see
`docs/spec/semcode.md`). For every decoded instruction, the verifier
requires:

```text
artifact_header_revision >= opcode.minimum_semcode_revision()
```

before `VerifiedSemCode` is issued. `Opcode::minimum_semcode_revision` (in
`sm-format`) is the single, format-owned authority for this comparison -
`sm-verify` uses it directly for admission, and `sm-ir`'s header selection
uses the same underlying opcode/feature relationship (through its own IR-
level promotion predicates) to choose a header whose revision actually
covers everything a program emits. There is no separate, independently-
maintained opcode-to-revision table anywhere else.

A violation rejects with `AmbiguousInstructionFraming`'s sibling code,
`VerificationCode::OpcodeRequiresNewerHeader` - deliberately distinct from
`CapabilityViolation`. The two checks address different failure modes:

- `CapabilityViolation`: the artifact's header does not carry a capability
  bit an opcode requires. Because each header revision's capability set is
  fixed and cumulative, this check already transitively enforces a minimum
  header for any capability-gated opcode.
- `OpcodeRequiresNewerHeader`: the opcode requires no missing capability at
  all - it is structurally valid and every capability it needs (if any) is
  present - but the artifact's header revision predates the revision whose
  contract actually admits this opcode's semantics. This is a version-
  identity gap, not a capability gap, and only opcodes with no capability
  bit at all need this check to be independently load-bearing (see #1732 /
  FA-05-002).

`Opcode::minimum_semcode_revision` assigns a minimum revision explicitly to
every `Opcode` variant. Variants established as baseline are explicitly
assigned revision `1` (`SEMCODE0`). An opcode family with a *provable* later
introduction revision - backed by an actual repository decision record, not
by commit date alone - is explicitly assigned that later revision. The match
is intentionally exhaustive and has no wildcard/default revision arm, so a
new `Opcode` variant cannot acquire a revision policy implicitly. This
function must not imply stronger historical knowledge than the repository has
actually established. Currently the only family assigned a non-baseline
minimum revision is `QTruth` (revision `19`, `SEMCOD18`).

## Canonical Instruction Framing

A canonical SemCode function encoding must have exactly one unambiguous
structural interpretation (see `docs/spec/semcode.md`).

The `DBG0` debug-section tag is recognized by sniffing a fixed 4-byte
sequence, and its first byte (`0x44`) is also `TupleGet`'s opcode byte, so a
producer-emitted instruction stream can coincidentally spell the same bytes
as `DBG0` framing. A byte sequence that is simultaneously valid as `DBG0`
metadata and as a complete instruction stream is non-canonical.

The verifier detects this by checking, whenever a `DBG0` section was
recognized, whether the same bytes - read from immediately after the string
table, with no metadata-section recognition at all - would also form a
complete, well-formed instruction stream to the end of the function's code.
If both readings are valid, admission fails closed with
`AmbiguousInstructionFraming` rather than silently keeping the `DBG0`
reading.

This is a purely STRUCTURAL question, deliberately kept separate from
SEMANTIC admission:

- structural framing: opcode recognition, operand byte shape, and
  presence/count-controlled byte lengths, evaluated to determine only
  whether a complete instruction stream exists at all
- semantic admission: canonical literal value domains (`LOAD_Q`,
  `LOAD_BOOL`), canonical presence-flag domains (`CALL`, `CLOSURE_CALL`,
  `RET`), and canonical arity/cardinality domains (`MAKE_TUPLE` arity
  `>= 2`, `MAKE_RECORD` slot count `>= 1`), which remain a separate, later
  concern applied only to whichever single reading admission actually
  accepts

The alternative reading only needs to be structurally complete to count as
a genuine competing interpretation - a non-canonical operand value (for
example an out-of-domain literal byte) does not make an otherwise
shape-complete instruction reading any less structurally real, and does not
exempt an artifact from the ambiguity check. Gating ambiguity detection on
today's semantic admission policy would make the one-canonical-
interpretation invariant depend on that policy instead of being a
decoder-level fact, and would silently keep the `DBG0` reading whenever the
competing instruction reading merely contained a non-canonical literal.

This check reuses the verifier's own operand-shape decoder for the
alternative reading - the same function, same opcode-shape match, that
semantic admission uses, with canonical-domain enforcement turned off -
rather than a second, independently-maintained opcode-shape table.

`OWN0`'s tag byte (`0x4F`) is not a currently valid opcode, so this specific
ambiguity does not apply to ownership-section recognition.

## Reachable Control-Flow Closure

For every function, the verifier reuses the instruction boundaries, decoded
next offsets, and explicit jump targets collected by its normal instruction
walk. Starting at instruction offset zero, it admits only reachable successors
with these VM-derived rules:

- `RET` is terminal and has no successor
- `JMP` has only its explicit jump target
- `JMP_IF` has its explicit jump target and the decoded next instruction
- every other opcode, including `CALL` and `CLOSURE_CALL`, has the decoded next
  instruction as its successor

The two call forms are fallthrough operations structurally because the VM saves
the decoded next PC before entering the callee and resumes there after `RET`.

Every reachable successor must be an instruction start. A successor equal to
the end of the executable instruction stream is rejected with
`VerificationCode::ReachableFunctionFallthrough`; an empty executable stream is
the same defect at entry offset zero. Existing explicit jump-target range and
instruction-boundary checks remain independently enforced as
`InvalidJumpTarget`.

This proves structural control-flow closure, not program termination. A closed
infinite loop is admissible, and an ordinary trailing instruction may fall
through if it is unreachable from function entry. The verifier does not require
the final encoded instruction to be `RET`.

## Debug Reference Validity

A `DBG0` debug symbol's `pc` must reference a decoded instruction boundary,
not merely fall within the executable byte range. The verifier reuses the
same `instr_starts` set collected during its normal instruction walk (the
same authority the jump-target boundary check above already relies on) - it
does not run a second decoder or infer boundaries from opcode-byte scanning.
A `pc` that is numerically less than the instruction stream length but lands
inside a decoded instruction's operand bytes is rejected with
`VerificationCode::InvalidDebugSection`, the same code already used for an
out-of-range `pc`. This proves only that a debug reference denotes a real
instruction boundary; it does not claim source-line fidelity, debugger
correctness, or any stronger metadata semantics than that structural fact.

## Contract Rule

Standard execution uses the chain:

`emit SemCode -> verify_semcode -> (optional) require_entry -> execute`

Important rule:

- VM execution does not replace SemCode admission
- a valid producer path does not waive verifier admission
- entry resolution failure (`EntryResolutionError`) is distinct from verifier rejection (`RejectReport`); missing an entrypoint does not make the artifact itself invalid.

## Separation Rule

`sm-verify` must not become:

- a source parser
- a semantic runtime
- a VM executor
- a general optimizer

It is allowed to reject malformed or contract-inconsistent bytecode only.

Current ownership rule:

- verifier admits ownership payload structurally only
- verifier does not evaluate borrow overlap, release timing, or runtime alias
  policy

## Reject Model

Verifier rejection must preserve:

- the failing verification code
- enough function or offset context to debug the failure
- deterministic diagnostics for the same input artifact

`ReachableFunctionFallthrough` specifically means that control flow reachable
from function entry can advance to the end of the executable instruction
stream without first reaching an admitted terminal instruction.

## Callable Arity Enforcement

For every `Opcode::Call` site whose callee resolves to a known internal
function, the verifier checks the call's `argc` against the callee's decoded
canonical signature (see [`semcode.md`](semcode.md#callable-signature-sig0)),
rejecting a mismatch with `CallArgumentCountMismatch`. This is arity only:

- `sm-verify` cannot prove a `CALL` argument register's runtime family
  statically - registers are untyped storage - so it never attempts to; that
  half of the contract is the VM's responsibility, enforced immediately
  before `push_frame` (see [`vm.md`](vm.md#callable-runtime-family-enforcement))
- caller-supplied `argc` never defines callee arity; only the callee's own
  decoded signature does
- when the callee's artifact predates canonical signatures
  (`signature: None`), there is no contract to check and this enforcement is
  a no-op - unchanged behavior for pre-#1773 artifacts
- a missing `SIG0` section on a header that requires one
  (`SEMCODE_SIGNATURE_MIN_REVISION` or newer) is not a policy check at this
  layer at all: it is a structural decode rejection
  (`InvalidSignatureSection`) before admission ever reaches this check,
  since `SIG0` presence is derived deterministically from the header
  revision, never sniffed

Calls to builtins are out of scope for this check; only calls that resolve
to a known internal function (with a decoded canonical signature to check
against) are covered.

## Verified Execution Rule

The standard `.smc` execution route must require `sm-verify` admission.

Helpers that bypass verification may exist for lower-level testing, but they
must not redefine the public execution contract.

`verify_semcode_token` is the canonical admission boundary for token-first
execution flows. `verify_semcode` remains a compatibility / legacy admission
API and should not be described as the preferred canonical boundary for new
trusted public routes unless a document explicitly distinguishes the two.

## Review Rule

Changes to the verifier require review if they alter:

- what SemCode is considered admissible
- the meaning of an existing verification code
- the structure or deterministic order of reject diagnostics

Required follow-up:

1. update this specification
2. update verifier tests
3. update compatibility or golden tests if public behavior changed
