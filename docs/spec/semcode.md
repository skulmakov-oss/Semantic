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
optional tagged `DBG0` debug section, then an optional tagged `OWN0`
ownership section, then the instruction stream running to the end of the
code block. `DBG0` and `OWN0` are recognized by sniffing a fixed 4-byte tag
immediately after the preceding section - there is no explicit
presence-flag or length-prefixed section table.

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

`SEMCOD15`, `SEMCOD16`, and `SEMCOD17` are also currently emitted/admitted
by the toolchain but are not yet documented in this section; that is a
pre-existing documentation gap, not part of the #1732 repair.

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
- call-target validity
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

Execution semantics for admitted ownership payload are specified separately in
`runtime_ownership.md`.

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
