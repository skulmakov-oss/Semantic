# SemCode Specification

Status: draft v0
Format owner: `sm-format`
Producer: `sm-ir` (lowering and emission over the `sm-format` contract)
Current producer facade: `sm-emit`
Admission owner: `sm-verify`
Execution consumer: `sm-vm`

## Purpose

SemCode is the binary contract between the Semantic producer pipeline and the
Semantic VM.

Ownership rule:

- `sm-format` owns the SemCode header, opcode, capability, and structural-limit contract
  (`local_format`, `semcode_format`, `semcode_decode`); dependency direction is
  `sm-ir -> sm-format`, never the reverse (#1715)
- `sm-ir` is a producer: it selects a header from emitted usage, re-exports the
  `sm-format` surface for compatibility, and must not emit an artifact the
  `sm-format` decoder rejects; structural maximums are read from
  `sm_format::semcode_decode`, never copied (#1728)
- `sm-emit` exposes producer-facing entrypoints over that contract and is not a second format owner
- `sm-emit` must re-export the canonical format surface rather than maintain a forked local copy

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

### Top-Level Artifact Framing

An artifact is the 8-byte header magic followed by its top-level content:

- header revision `>= 23` (`SEMCOD22` and newer,
  `SEMCODE_ADT_DESCRIPTOR_MIN_REVISION`): the magic, then exactly one
  mandatory `ADT0` descriptor section, then the function envelopes running
  to the end of the artifact;
- header revision `<= 22` (`SEMCOD21` and older): the magic, then the
  function envelopes running to the end of the artifact. These artifacts
  have no descriptor section and no ADT descriptor authority.

`ADT0` presence is derived from the header revision alone, on both the
encode and decode side. It is never content-sniffed: under a revision
`<= 22` the bytes after the magic are always read as a function envelope,
even if they happen to spell `ADT0` (such an artifact fails the legacy
function-envelope grammar), and under a revision `>= 23` a missing `ADT0`
section is a decode rejection. A second `ADT0` section immediately after the
first is rejected explicitly.

### `ADT0` Descriptor Section

All integers are unsigned little-endian; all names are UTF-8 prefixed by
their `u16` byte length.

```text
ADT0 section   := tag:"ADT0"(4 bytes) descriptor_count:u16 descriptor*
descriptor     := name_len:u16 name:utf8[name_len]
                  variant_count:u16 variant*
variant        := name_len:u16 name:utf8[name_len] payload_arity:u16
```

Canonical-form rules, each a decode rejection when violated:

- descriptor names are non-empty, unique, and appear in strictly ascending
  raw UTF-8 byte order;
- variant names are non-empty and unique within their descriptor; variants
  keep declaration order, and a variant's discriminant (tag) is its index in
  that order;
- the built-in descriptors `Option = [None/0, Some/1]` and
  `Result = [Ok/1, Err/1]` (variant/payload arity) are always present,
  exactly in that canonical form; a source enum may not use either name;
- the section is read exactly as encoded: the decoder never sorts, adds,
  drops or rewrites a descriptor, and never repairs a missing or
  non-canonical built-in (the artifact's own table is the only descriptor
  authority; decoding never calls `AdtDescriptorTable::with_builtins`).

ADT identity is the canonical type name. A descriptor's position in the
table is only a lookup ordinal: it is not identity and is not stable across
artifacts.

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
- `SEMCOD22`

- `SEMCOD15`
- `SEMCOD16`
- `SEMCOD17`
- `SEMCOD20`

(The four entries above were emitted and admitted before they were listed
here; PB-05 #1714 closes that documentation gap. Their semantics are under
`## Current Header Semantics`.)

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
- `SEMCOD22`: epoch `0`, revision `23`

Header responsibilities:

- identify the format family
- identify the supported epoch and revision
- select, through the 8-byte magic alone, the fixed capability envelope of
  that revision (`header_spec_from_magic` in `sm-format`); no capability
  bitset is serialized in the artifact (#1727)

## Version Policy

Compatibility rules:

1. A producer must emit exactly one supported SemCode header variant.
2. A verifier must reject artifacts with unknown or unsupported headers.
3. A VM must not silently reinterpret an unsupported header as a supported one.
4. Any incompatible binary layout or meaning change requires a version bump.

Discipline rules:

- existing admitted header families remain fixed once they ship on `main`;
  a change to what their artifacts mean is made only through the
  `## Backward Compatibility Rule` (see its recorded `SEMCOD22` change)
- capability widening stays additive in the current baseline and must not
  repurpose existing bits
- release-facing documents must distinguish the published stable line from the
  wider admitted line on current `main`
- SemCode header selection remains derived from actual emitted usage, not from
  policy permission alone: the producer picks the lowest revision whose
  envelope covers what was emitted, and the envelope may be wider than the
  exact set of capabilities used (envelope model, #1727)

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

`SEMCOD15`

- epoch `0`, revision `16`
- promoted contract used when emitted program usage requires the deterministic
  PRNG family (`RngSeed`, `RngNextI32`)
- envelope: the `SEMCOD14` set plus `CAP_PRNG`

`SEMCOD16`

- epoch `0`, revision `17`
- promoted contract used when emitted program usage calls `print`
- envelope: the `SEMCOD15` set plus `CAP_STDOUT`

`SEMCOD17`

- epoch `0`, revision `18`
- promoted contract used when emitted program usage calls an application
  builtin (args, stdin, stdout/stderr write, path inspection, filesystem
  read/write, duration time)
- envelope: the `SEMCOD16` set plus `CAP_ARGS_READ`, `CAP_STDIN_READ_TEXT`,
  `CAP_STDOUT_WRITE`, `CAP_STDERR_WRITE`, `CAP_PATH_INSPECT`, `CAP_FS_READ`,
  `CAP_FS_WRITE`, `CAP_TIME_DURATION`

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

`SEMCOD20`

- epoch `0`, revision `21` (`SEMCODE_OWNERSHIP_ANCHOR_MIN_REVISION`)
- promoted contract used when any ownership `Borrow` event carries an
  activation site or any `Write` event carries a write site
- carries forward the `SEMCOD19` envelope unchanged; adds the `OWN0`
  activation-mode (Borrow) and execution-mode (Write) tag bytes, whose
  unknown values are hard structural rejections

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

`SEMCOD22`

- ADT descriptor contract (SSF-09 D2-2), revision `23`
  (`SEMCODE_ADT_DESCRIPTOR_MIN_REVISION`)
- every artifact carries the mandatory `ADT0` descriptor section between the
  magic and the function envelopes (see `### Top-Level Artifact Framing` and
  `### ADT0 Descriptor Section` above)
- the artifact's `ADT0` table is the only ADT descriptor authority for its
  `MAKE_ADT`, `ADT_TAG`, and `ADT_GET` instructions; the verifier checks each
  of them against that table (see `verifier.md`)
- uses the fixed-width 8-byte header magic form `SEMCOD22`
- inherits the `SEMCOD21` capability set unchanged; adds no capability bit
  and does not change the function-envelope, `DBG0`, `OWN0`, or `SIG0` layout
- is the floor of every artifact the current compiler emits (see
  `## Opcode Vocabulary And Header Identity`)

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

The descriptor-dependent ADT opcodes `MAKE_ADT`, `ADT_TAG`, and `ADT_GET`
remain baseline (revision `1`) in `Opcode::minimum_semcode_revision()`, but
are subject to a separate, independent rule: they are admitted only under a
header that carries the `ADT0` descriptor section (revision `>= 23`). Under
any older header they are rejected with `AdtRequiresDescriptorHeader`, not
`OpcodeRequiresNewerHeader` (see `docs/spec/verifier.md` and
`## Backward Compatibility Rule`).

Important rule:

- header selection is derived from actual emitted usage, not from profile
  permission alone, above a fixed floor: every artifact the current compiler
  emits carries the mandatory `ADT0` section, so it is emitted under
  `SEMCOD22` (revision `23`) or newer regardless of which opcodes it uses; a
  program that needs a newer revision still promotes above that floor

That means:

- a profile may allow `f64`
- if the program does not actually use the `f64` family, that family does not
  raise the header; the program is still emitted at the `SEMCOD22` floor

## Capability Contract

The current capability contract is an envelope selected by the SemCode header
magic and verified against actual opcode usage: every opcode must be covered
by the envelope of the artifact's revision. The artifact does not serialize a
capability bitset, and the envelope is not required to equal the exact set of
capabilities used (#1727).

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
- SemCode header records the revision, and therefore the envelope, chosen for
  what was actually produced
- verifier proves that opcode usage matches the emitted capability contract

## Structural Contract

Current SemCode admission validates:

- header magic and supported version
- at revision `>= 23`, presence and canonical validity of the single `ADT0`
  descriptor section (see `### ADT0 Descriptor Section`)
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

### Recorded Change: `SEMCOD22` ADT Descriptor Authority (SSF-09 D2-2)

`SEMCOD22` (revision `23`) is a header and section layout change, and it is
an intentional verifier interpretation change for existing artifacts:

- a `SEMCODE0`..`SEMCOD21` artifact containing `MAKE_ADT`, `ADT_TAG`, or
  `ADT_GET` was previously verifier-admissible without any descriptor
  authority; it is now rejected with `AdtRequiresDescriptorHeader`. No
  descriptor table is inferred from its instructions, reconstructed from its
  strings, or synthesized from the built-ins for it;
- `SEMCODE0`..`SEMCOD21` artifacts without those opcodes keep their existing
  contract, and all legacy artifacts remain structurally decodable exactly
  as before (descriptor-less, no `ADT0` sniffing);
- historical bytes are never silently upgraded: decoding a legacy artifact
  does not add a descriptor section or change its header;
- the only supported migration for an affected artifact is recompilation
  with the current toolchain, which emits `SEMCOD22` with its `ADT0` table.

The follow-up above is recorded in `docs/roadmap/compatibility_statement.md`,
`docs/roadmap/v1_readiness.md`, and `docs/spec/verifier.md`, with verifier,
VM, and golden fixture coverage in the same change.

## No Silent Mutation Rule

The following are forbidden without a documented version change:

- repurposing an existing capability bit
- changing the meaning of an existing header family
- changing section interpretation while keeping the same public version

## Consumer Rule

`sm-vm` may consume SemCode on the standard execution route only through a
verified admission path.

Any raw or testing-only path must not redefine the public SemCode contract.
