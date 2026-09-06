# SemCode Specification

Status: draft v0
Current format owner: `sm-ir`
Current producer facade: `sm-emit`
Admission owner: `sm-verify`
Execution consumer: `sm-vm`

## Purpose

SemCode is the binary contract between the Semantic producer pipeline and the
Semantic VM.

Ownership rule:

- `sm-ir` owns the SemCode header, opcode, and capability contract in the current `v1` baseline
- `sm-emit` exposes producer-facing entrypoints over that contract and is not a second format owner
- `sm-emit` must re-export the canonical format surface from `sm-ir` rather than maintain a forked local copy

Standard execution rule:

`frontend -> semantics -> lowering -> IR passes -> emit -> verify -> execute`

SemCode is the downstream binary contract after IR passes and before
verifier-admitted VM execution.

The VM is not the primary structural admission gate.
`sm-verify` is the required admission stage for standard SemCode execution.

## Canonical Structural Framing

A canonical SemCode function encoding must have exactly one unambiguous
structural interpretation.

Per function, the code block is: a length-delimited string table, then an
optional tagged `DBG0` debug section, then an `OWN0` ownership section
(structurally optional below `SEMCOD11`, the header revision that first
requires per-function ownership-path metadata; content-sniffed - not
enforced present - at `SEMCOD11` through `SEMCOD18`; deterministically
mandatory at `SEMCODE_SIGNATURE_MIN_REVISION` or newer), then (at
`SEMCODE_SIGNATURE_MIN_REVISION` or newer) a tagged `SIG0`
callable-signature section, then the instruction stream running to the end
of the code block. `DBG0` and `OWN0` are recognized by sniffing a fixed
4-byte tag immediately after the preceding section - there is no explicit
presence-flag or length-prefixed section table.

Admission at `SEMCOD11` through `SEMCOD18` only proves that *some* function
in the artifact has `OWN0` (`sm-verify`'s program-wide
`.any(has_ownership_section)` check) - a specific function omitting it is
not independently rejected at those revisions. This is a pre-existing gap
that predates #1773 and is out of scope for it; only
`SEMCODE_SIGNATURE_MIN_REVISION` and newer closes it per function, at
decode time, deterministically from the header revision alone,
independent of whether any other function in the same artifact has one
(see [`## Callable Signature (SIG0)`](#callable-signature-sig0)). `SIG0`
itself is never content-sniffed at all - its presence is derived the same
deterministic way.

A byte sequence that is simultaneously valid as `DBG0` debug-section framing
and as executable instruction framing is non-canonical. "Executable
instruction framing" here is a structural question - opcode recognition and
operand byte shape - independent of whether the resulting operand values
are themselves semantically canonical; a competing reading is not exempted
from this rule merely because it also contains a non-canonical literal.
Admission (`sm-verify`) rejects such an artifact rather than silently
choosing one reading, because doing so could hide otherwise-invalid
instruction content (e.g. a register reference outside the verified-local
budget) inside what gets reclassified as metadata. See #1731 and
`docs/spec/verifier.md`.

`OWN0`'s tag byte (`0x4F`) is not a currently valid opcode, so it cannot
collide with the start of an instruction the way `DBG0`'s tag byte (`0x44`
= `TupleGet`) can; this ambiguity is specific to `DBG0`, not a general
property of the tagged-section scheme.

## Versioned Header Family

Current supported header family:

- `SEMCODE0`
- `SEMCODE1`
- `SEMCODE2`
- `SEMCODE3`
- `SEMCODE4`
- `SEMCODE5`
- `SEMCODE6`
- `SEMCODE7`
- `SEMCODE8`
- `SEMCODE9`
- `SEMCOD10`
- `SEMCOD11`
- `SEMCOD12`
- `SEMCOD13`
- `SEMCOD14`
- `SEMCOD18`
- `SEMCOD19`
- `SEMCOD21`

`SEMCOD15`, `SEMCOD16`, `SEMCOD17`, and `SEMCOD20` are also currently
emitted/admitted by the toolchain but are not yet documented in this
section; that is a pre-existing documentation gap, not part of the #1732 or
#1718 repairs.

Observed runtime support in the current toolchain:

- `SEMCODE0`: epoch `0`, revision `1`
- `SEMCODE1`: epoch `0`, revision `2`
- `SEMCODE2`: epoch `0`, revision `3`
- `SEMCODE3`: epoch `0`, revision `4`
- `SEMCODE4`: epoch `0`, revision `5`
- `SEMCODE5`: epoch `0`, revision `6`
- `SEMCODE6`: epoch `0`, revision `7`
- `SEMCODE7`: epoch `0`, revision `8`
- `SEMCODE8`: epoch `0`, revision `9`
- `SEMCODE9`: epoch `0`, revision `10`
- `SEMCOD10`: epoch `0`, revision `11`
- `SEMCOD11`: epoch `0`, revision `12`
- `SEMCOD12`: epoch `0`, revision `13`
- `SEMCOD13`: epoch `0`, revision `14`
- `SEMCOD14`: epoch `0`, revision `15`
- `SEMCOD18`: epoch `0`, revision `19`
- `SEMCOD19`: epoch `0`, revision `20`
- `SEMCOD21`: epoch `0`, revision `22`

Header responsibilities:

- identify the format family
- identify the supported epoch and revision
- carry the emitted capability bitset for the produced artifact

## Version Policy

Compatibility rules:

1. A producer must emit exactly one supported SemCode header variant.
2. A verifier must reject artifacts with unknown or unsupported headers.
3. A VM must not silently reinterpret an unsupported header as a supported one.
4. Any incompatible binary layout or meaning change requires a version bump.

Discipline rules:

- existing admitted header families remain fixed once they ship on `main`
- capability widening stays additive in the current baseline and must not
  repurpose existing bits
- release-facing documents must distinguish the published stable line from the
  wider admitted line on current `main`
- SemCode header selection remains derived from actual emitted usage, not from
  policy permission alone

## Current Header Semantics

`SEMCODE0`

- baseline SemCode contract
- does not imply floating-point math capability

`SEMCODE1`

- promoted contract used when emitted program usage requires the `f64` math
  family
- carries the stronger capability envelope required by that produced artifact

`SEMCODE2`

- promoted contract used when emitted program usage requires the canonical `fx`
  value family
- extends the supported opcode/header family without changing standard
  admit-then-execute rules

`SEMCODE3`

- promoted contract used when emitted program usage requires canonical plain
  `fx` arithmetic
- keeps the earlier `SEMCODE2` fixed-point value/equality contract intact for
  older artifacts

`SEMCODE4`

- promoted contract used when emitted program usage requires admitted
  post-stable `StateQuery` host calls
- keeps `SEMCODE0..3` fixed for older artifacts that do not use the widened
  host-call family

`SEMCODE5`

- promoted contract used when emitted program usage requires admitted
  post-stable `StateUpdate` host calls
- keeps `SEMCODE0..4` fixed for older artifacts that do not use the widened
  write-side host-call family

`SEMCODE6`

- promoted contract used when emitted program usage requires admitted
  post-stable `EventPost` host calls
- keeps `SEMCODE0..5` fixed for older artifacts that do not use the widened
  event-side host-call family

`SEMCODE7`

- promoted contract used when emitted program usage requires admitted
  post-stable `ClockRead` host calls
- keeps `SEMCODE0..6` fixed for older artifacts that do not use the widened
  clock-query host-call family

`SEMCODE8`

- promoted contract used when emitted program usage requires the canonical text
  value carrier for admitted literal/equality programs
- keeps `SEMCODE0..7` fixed for older artifacts that do not use executable
  text values

`SEMCODE9`

- promoted contract used when emitted program usage requires the canonical
  ordered sequence carrier for the admitted `M8.3` first-wave surface
- keeps `SEMCODE0..8` fixed for older artifacts that do not use executable
  sequence values

`SEMCOD10`

- promoted contract used when emitted program usage requires the canonical
  first-wave closure carrier and direct invocation path for admitted `M8.4`
  closure values
- keeps `SEMCODE0..9` fixed for older artifacts that do not use executable
  closure values
- uses the fixed-width 8-byte header magic form `SEMCOD10` rather than
  `SEMCODE10`

`SEMCOD11`

- promoted contract used when emitted program usage requires tuple-only
  ownership path metadata transport for lowered borrow/write events
- keeps `SEMCODE0..10` fixed for older artifacts that do not use executable
  ownership-path metadata
- uses the fixed-width 8-byte header magic form `SEMCOD11`
- adds the tagged function-local ownership section `OWN0` after the optional
  `DBG0` section and before the instruction stream
- encodes each ownership event deterministically as:
  - event kind (`Borrow` or `Write`)
  - root `SymbolId` as little-endian `u32`
  - ordered tuple-only path components as `TupleIndex(u16)`
- does not claim record, ADT payload, schema, or release/lifetime transport
  beyond the current frame-local tuple slice

`SEMCOD12`

- promoted contract used when emitted program usage requires direct
  record-field ownership path transport
- keeps `SEMCOD11` fixed for tuple-only ownership-path artifacts
- uses the fixed-width 8-byte header magic form `SEMCOD12`
- keeps the tagged function-local ownership section `OWN0`
- extends the ownership-path component vocabulary with:
  - `Field(SymbolId)` encoded as component kind + little-endian `u32`
- transports direct record-field `Borrow` and `Write` paths deterministically
- requires `CAP_OWNERSHIP_FIELD_PATHS` when direct record-field components are
  present
- does not claim ADT payload, schema, or release/lifetime transport beyond the
  current frame-local tuple+record slice

`SEMCOD13`

- promoted contract used when emitted program usage requires executable
  first-wave built-in iterable loops over `Sequence(T)`
- keeps `SEMCOD12` fixed for artifacts that do not use the widened sequence
  iteration primitive
- uses the fixed-width 8-byte header magic form `SEMCOD13`
- adds the deterministic execution opcode `SEQUENCE_LEN` for built-in
  sequence-loop lowering
- requires `CAP_SEQUENCE_ITERATION` when `SEQUENCE_LEN` is present
- does not claim executable user-defined `Iterable` impl dispatch, ADT payload
  iteration, schema iteration, or non-frame-local iterator state

`SEMCOD14`

- promoted contract used when emitted program usage requires the deterministic
  functional `Map(K, V)` empty/get/set/contains operations
- keeps `SEMCOD13` fixed for artifacts that do not use `Map(K, V)`
- uses the fixed-width 8-byte header magic form `SEMCOD14`
- adds the deterministic execution opcodes `MAP_EMPTY`, `MAP_CONTAINS`,
  `MAP_GET`, and `MAP_SET`
- requires `CAP_MAP_VALUES` when any of those opcodes is present
- does not claim mutable in-place map update, iteration, or non-frame-local
  map state beyond the admitted functional empty/get/set/contains contour

`SEMCOD18`

- promoted contract used when emitted program usage requires the `QTruth`
  Belnap truth-table opcode family (`QTruthAnd`, `QTruthOr`, `QTruthNot`,
  `QTruthImpl`)
- keeps `SEMCODE0..17` fixed for older artifacts; `QTruth` is not admitted
  under any older header (see #1732 / FA-05-002 and
  `## Opcode Vocabulary And Header Identity` below)
- carries forward the same capability envelope as `SEMCOD17` unchanged - no
  new capability bit is introduced; the gap this closes is a missing
  version-identity gate, not a missing capability
- does not claim any change to the existing lattice `QAnd`/`QOr`/`QNot`/
  `QImpl` opcodes, which remain baseline and unaffected

`SEMCOD19`

- promoted contract used unconditionally by the current emitter for every
  compiled artifact (#1773 / FA-09-005), independent of which opcodes the
  program actually uses - every function envelope under this revision
  carries a canonical callable-signature record, so the revision floor
  applies uniformly rather than being promoted per-opcode like the
  revisions above
- carries forward the same capability envelope as `SEMCOD18` unchanged - no
  new capability bit is introduced; the gap this closes is a missing
  version-identity gate (every function's signature is now structurally
  present and provable), not a missing capability
- keeps `SEMCODE0..18` fixed for older artifacts: an artifact under any
  older header structurally cannot carry a `SIG0` section at all, and its
  functions decode with `signature: None` - canonical typed callable
  execution then has no contract to prove for that artifact and cannot
  offer the same trusted-callable guarantee (see
  [`verifier.md`](verifier.md#callable-arity-enforcement) and
  [`vm.md`](vm.md#callable-runtime-family-enforcement))

### Callable Signature (`SIG0`)

Every function envelope under `SEMCOD19` or newer carries a `SIG0` section,
placed immediately after the (also now-mandatory) `OWN0` section and before
the instruction stream:

- 4-byte tag `SIG0`
- `u16` little-endian parameter count
- one family-tag byte per parameter, in declaration order

The parameter count and the number of family-tag bytes are the same field by
construction - there is no separate, independently-desyncable count. Each
family tag is one of the 14 executable runtime families (`Quad`, `Bool`,
`Text`, `Sequence`, `Map`, `Closure`, `I32`, `U32`, `Fx`, `F64`, `Tuple`,
`Record`, `Adt`, `Unit`); tag `0` is deliberately never assigned, so a
zero-initialized or truncated buffer never decodes as a valid family. A
malformed, truncated, or unknown-tag `SIG0` section is a deterministic
decode rejection.

Unlike `DBG0`/`OWN0`, `SIG0` presence is never content-sniffed - it is
derived purely from the artifact's header revision
(`SEMCODE_SIGNATURE_MIN_REVISION`), on both the encode and decode side. This
is a deliberate difference: sniffing would reopen the `TupleGet`/`DBG0` byte
collision class (#1731) for a new tag, and a mandatory, revision-derived
section has no ambiguous alternative reading to defend against.

This signature originates at the function's typed source definition and
survives unchanged through IR and SemCode emission - see
[`ir.md`](ir.md#current-ir-shapes) for where it is derived, and
[`verifier.md`](verifier.md#callable-arity-enforcement) /
[`vm.md`](vm.md#callable-runtime-family-enforcement) for how it is enforced
at a callee before execution.

`SEMCOD21`

- promoted contract used when emitted program usage requires ownership path
  transport for the `SequenceIndexStatic` component (`Borrow` and `Write`
  both) or the `AdtPayload` component in a `Borrow` event (#1718 /
  FA-04-012; see
  `docs/roadmap/stable_foundation/ssf08_1718_path_family_contract_decision.md`)
- keeps `SEMCODE0..20` fixed for older artifacts that do not use these
  ownership path families: an artifact under any older header that carries a
  `SequenceIndexStatic` component, or an `AdtPayload` component in a
  `Borrow` event, is rejected at decode/verify - the header never had
  authority over these families, and no older header's meaning changes
  retroactively
- uses the fixed-width 8-byte header magic form `SEMCOD21`
- adds two new capability bits, `CAP_OWNERSHIP_SEQUENCE_PATHS` and
  `CAP_OWNERSHIP_ADT_BORROW_PATHS` (see `## Capability Contract` below);
  neither widens `CAP_OWNERSHIP_PATHS`'s or `CAP_OWNERSHIP_FIELD_PATHS`'s
  own scope, which remain exactly tuple-only and tuple+record as `SEMCOD11`
  and `SEMCOD12` already defined them
- does **not** change the `OWN0` section's wire layout, the `SIG0` floor, or
  the Borrow-activation/Write-execution-mode grammar `SEMCOD11`
  (`CAP_OWNERSHIP_PATHS`) and the rev21 anchor grammar
  (`SEMCODE_OWNERSHIP_ANCHOR_MIN_REVISION`) already established - this
  revision answers only "which path components may this header's `OWN0`
  section carry," not "how are events/anchors encoded"; those two questions
  are deliberately kept orthogonal
- does **not** admit `Write(AdtPayload)` under any circumstance: an
  `AdtPayload` component inside a `Write` event is rejected unconditionally,
  at every header revision including `SEMCOD21` itself, regardless of
  capability - this is not a missing-capability gap a future header could
  close by inheritance; promoting `Write(AdtPayload)` requires a new,
  separately authorized contract change, never an incidental relaxation of
  this revision's own grammar

## Opcode Vocabulary And Header Identity

SemCode header identity constrains the executable opcode vocabulary. Every
`Opcode` variant is explicitly bound to a minimum SemCode header revision by
`Opcode::minimum_semcode_revision()`. Variants established as baseline are
explicitly assigned revision `1` (`SEMCODE0`); a family with repository-backed
evidence for a later semantic introduction is explicitly assigned that later
revision. The mapping is exhaustive and has no wildcard/default revision arm,
so adding a new `Opcode` variant requires an explicit revision-policy decision
at compile time.

An opcode introduced after a header revision is non-canonical under an older
header and must be rejected before `VerifiedSemCode` is issued, even if that
opcode is structurally well-formed and requires no missing capability.

This is a distinct concern from the capability contract above: most
opcodes that gained new semantics after the baseline also gained a
capability bit, and since each header's capability set is fixed and
cumulative per revision, the capability check already transitively enforces
their minimum header. The opcode-vocabulary/header-identity invariant is
only independently load-bearing for an opcode family that carries no
capability bit at all - currently only `QTruth` (see #1732 / FA-05-002 for
the full audit and rationale). See `docs/spec/verifier.md` for the
enforcement mechanism.

Important rule:

- header selection is derived from actual emitted usage, not from profile
  permission alone

That means:

- a profile may allow `f64`
- if the program does not actually use the `f64` family, the producer may still
  emit `SEMCODE0`

## Capability Contract

The current capability contract is carried by the SemCode header and verified
against actual opcode usage.

Current canonical capability families:

- `CAP_F64_MATH`
- `CAP_FX_VALUES`
- `CAP_FX_MATH`
- `CAP_GATE_SURFACE`
- `CAP_STATE_QUERY`
- `CAP_STATE_UPDATE`
- `CAP_EVENT_POST`
- `CAP_CLOCK_READ`
- `CAP_TEXT_VALUES`
- `CAP_SEQUENCE_VALUES`
- `CAP_SEQUENCE_ITERATION`
- `CAP_CLOSURE_VALUES`
- `CAP_OWNERSHIP_PATHS`
- `CAP_OWNERSHIP_FIELD_PATHS`
- `CAP_OWNERSHIP_SEQUENCE_PATHS`
- `CAP_OWNERSHIP_ADT_BORROW_PATHS`
- `CAP_MAP_VALUES`
- `CAP_DEBUG_SYMBOLS`

Contract rule:

- profile policy constrains what may be produced
- SemCode header records what was actually produced
- verifier proves that opcode usage matches the emitted capability contract

## Structural Contract

Current SemCode admission validates:

- header magic and supported version
- section and function-layout integrity
- opcode validity against the public opcode admission matrix in `verifier.md`
- opcode/header-revision consistency (see
  `## Opcode Vocabulary And Header Identity`)
- operand shape validity
- jump-target validity
- reachable control-flow closure: every successor reachable from function
  entry is another instruction boundary or an admitted terminal condition;
  end-of-stream fallthrough is not admissible
- executable-target validity: direct calls resolve to declared functions or
  admitted builtins, while closures resolve only to declared functions
- register-budget validity against the runtime contract
- string and debug reference validity
- capability consistency between actual usage and emitted contract

Current ownership-specific structural admission for `SEMCOD11` validates:

- `OWN0` section layout
- admitted ownership event kinds
- tuple-only path component kinds under `SEMCOD11`
- deterministic root/component payload shape
- capability/header consistency for ownership transport

Current `SEMCOD12` format extension in this slice:

- producer transport may encode direct record-field `Borrow` and `Write` paths
  in `OWN0`
- verifier admits direct record-field ownership payload structurally
- VM consumes admitted direct record-field ownership payload for frame-local
  borrow tracking and overlap enforcement
- ownership execution semantics remain specified separately in
  `runtime_ownership.md`

Current `SEMCOD21` format extension in this slice (#1718):

- producer transport may encode `SequenceIndexStatic` paths (`Borrow` and
  `Write` both) and `AdtPayload` paths in `Borrow` events only, in `OWN0`
- requires `CAP_OWNERSHIP_SEQUENCE_PATHS` for any `SequenceIndexStatic`
  component and `CAP_OWNERSHIP_ADT_BORROW_PATHS` for any `AdtPayload`
  component in a `Borrow` event; an artifact under a header lacking the
  relevant bit is rejected at decode, independent of the verifier's own
  separate capability-consistency check
- an `AdtPayload` component in a `Write` event is rejected unconditionally,
  under `SEMCOD21` and every other header, regardless of capability
- verifier admits `SequenceIndexStatic` and `Borrow`-side `AdtPayload`
  ownership payload structurally, on the same terms as `SEMCOD11`/`SEMCOD12`
  path kinds, and independently re-derives the same capability requirement
  from decoded content rather than trusting decode alone
- VM consumes admitted `SequenceIndexStatic`/`AdtPayload` ownership payload
  for frame-local borrow tracking and overlap enforcement, using the same
  `AccessPath`/overlap machinery already used for tuple/record paths
- ownership execution semantics remain specified separately in
  `runtime_ownership.md`

Execution semantics for admitted ownership payload are specified separately in
`runtime_ownership.md`.

### Offset Arithmetic Must Stay Inside The Result Model

Every cursor/length computation in the `sm-format` decoder
(`local_format.rs`'s low-level readers, and `semcode_decode.rs`'s function
`code_len` check and its `DBG0`/`OWN0` section tag-sniffs) that could
produce an out-of-bounds slice uses `checked_add`, never a raw `+`. A fully
attacker-controlled length field (function `code_len`, or a per-string
`len` consumed from the string table) combined with an already-advanced
cursor must never be able to wrap past `usize::MAX` and produce a false
in-bounds result - on any target width, including 32-bit, where a `u32`
length field can realistically overflow `usize` arithmetic. Loop trip
counts (string/debug-symbol/ownership-path counts, and ownership
component counts) never participate in this cursor arithmetic themselves -
each loop iteration's individual field read is independently bounds-checked
- so an oversized count cannot overflow anything; it only causes however
many extra `read_*` calls the loop makes, each still subject to the same
checked arithmetic.

For the function `code_len` check specifically, an overflow is always
treated as "the claimed length cannot possibly fit" and rejected with the
same structural decode error (`DecodeError::TruncatedFunction`) the
ordinary bounds check already produces; it is never silently wrapped,
saturated, or ignored. The `DBG0`/`OWN0` tag-sniffs use the identical
checked-arithmetic pattern, but for a different purpose: they are a
lookahead probe for an *optional* section, not an accept/reject gate. A
failed probe - whether from overflow, an ordinary out-of-bounds lookahead,
or (the common case) simply because the function has no debug/ownership
section - means "section absent," and decoding proceeds normally; it does
not, by itself, produce a decode error. Genuine corruption of a section
that IS present (a truncated count or entry once the tag has matched) is
still caught deterministically by the ordinary `read_*` calls inside that
section's parsing, same as everywhere else in this file. The checked
arithmetic's job in the tag-sniff is narrower than in the `code_len` check:
only to prevent the lookahead read itself from panicking, not to gate
whether the artifact is accepted.

Diagnostic-only offset values reported inside an already-failed read's
error message (i.e. values that do not themselves gate acceptance) may
saturate instead, since no accept/reject decision depends on them.

## Backward Compatibility Rule

The following changes require a SemCode version review:

- header layout change
- section layout change
- opcode encoding change
- capability bit meaning change
- verifier interpretation change that alters what previously valid artifacts
  mean

Required follow-up:

1. update this specification
2. update `docs/roadmap/compatibility_statement.md`
3. update `docs/roadmap/v1_readiness.md`
4. update verifier compatibility tests
5. update VM compatibility tests
6. update golden or compatibility fixtures if public behavior changed

## No Silent Mutation Rule

The following are forbidden without a documented version change:

- repurposing an existing capability bit
- changing the meaning of an existing header family
- changing section interpretation while keeping the same public version

## Consumer Rule

`sm-vm` may consume SemCode on the standard execution route only through a
verified admission path.

Any raw or testing-only path must not redefine the public SemCode contract.
