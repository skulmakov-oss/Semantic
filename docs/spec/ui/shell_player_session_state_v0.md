# Shell Player Session & Local State Contract v0

Status: NORMATIVE CONTRACT FREEZE
Track: UI-DNA2-9B
Implementation: NOT AUTHORIZED

This contract preserves the UI-DNA2-9A1 ownership boundary:

```text
Shell Player owns local projection playback.
Shell Player does not own Semantic truth.

local shell state != Semantic truth
patch application != Semantic mutation
hit-test result != action authorization
ActionIntent candidate != admitted action
draw material != pixels
shell transition != backend event loop
```

## 1. Purpose

This document defines the deterministic conceptual model for:

- activated shell-session input;
- local Shell Player state;
- session lifecycle;
- transition stimulus;
- transition evaluation;
- transition result;
- resource accounting;
- diagnostics.

It makes a later implementation contract possible without defining Rust types,
module layout, public APIs, or implementation algorithms.

## 2. Activated session input contract

`ActivatedShellSessionContext` is the conceptual read-only input for one
activated Shell Player session. It contains or carries:

- bundle identity;
- bundle contract version;
- activation decision identity;
- activation scope;
- session identity;
- initial viewport context;
- deterministic resource limits;
- caller-owned session metadata.

Normative rules:

- the activated context is caller supplied;
- the activated context is read only;
- the activated context does not grant Semantic authority;
- the activated context does not grant capability authority;
- the activated context does not imply production promotion;
- Shell Player does not create the activation decision;
- Shell Player does not validate bundle trust;
- Shell Player does not load the bundle.

The deterministic resource-limit set is supplied exactly once through
`ActivatedShellSessionContext`. It is the sole normative limit source and is
immutable for the lifetime of the activated session. A transition does not
accept a second independent resource-limit set. Changing resource limits
requires a new caller-supplied activated session context and does not mutate an
existing session in place.

The exact Rust representation of `ActivatedShellSessionContext` remains
unresolved and unauthorized.

## 3. Session lifecycle

The conceptual lifecycle has exactly four states:

- `Created`;
- `Active`;
- `Suspended`;
- `Closed`.

Allowed lifecycle transitions are:

| From | To |
| --- | --- |
| `Created` | `Active` |
| `Created` | `Closed` |
| `Active` | `Suspended` |
| `Active` | `Closed` |
| `Suspended` | `Active` |
| `Suspended` | `Closed` |
| `Closed` | no further lifecycle transition |

Normative rules:

- `Closed` is terminal;
- a closed session cannot consume interaction or patch input;
- suspended sessions preserve local state but do not process interaction;
- lifecycle transitions are caller initiated;
- lifecycle state is local runtime state, not Semantic truth;
- invalid lifecycle transitions fail deterministically.

## 4. Local state domains

`ShellLocalState` is the conceptual state owned by one Shell Player session.
Every owned domain is local, non-authoritative, reconstructible,
session-scoped, and not Semantic truth.

| Domain | Local | Non-authoritative | Reconstructible | Session-scoped | Not Semantic truth |
| --- | --- | --- | --- | --- | --- |
| lifecycle state | yes | yes | yes | yes | yes |
| focus state | yes | yes | yes | yes | yes |
| hover state | yes | yes | yes | yes | yes |
| pressed state | yes | yes | yes | yes | yes |
| pointer-capture state | yes | yes | yes | yes | yes |
| projection replay cursor | yes | yes | yes | yes | yes |
| local projected-value cache | yes | yes | yes | yes | yes |
| local invalidation state | yes | yes | yes | yes | yes |
| local damage bookkeeping | yes | yes | yes | yes | yes |
| local hit-test realization | yes | yes | yes | yes | yes |
| local accessibility realization | yes | yes | yes | yes | yes |
| viewport-local realization state | yes | yes | yes | yes | yes |
| deterministic resource counters | yes | yes | yes | yes | yes |
| diagnostic counters | yes | yes | yes | yes | yes |

Local state cannot become:

- task truth;
- freshness truth;
- connectivity truth;
- capability policy;
- admission state;
- action acceptance;
- bundle trust;
- backend state;
- renderer pixel state.

## 5. Stable identities

All local references derive from caller-supplied or bundle-owned stable
identities.

The following identity sources are forbidden:

- memory addresses;
- filesystem paths;
- wall-clock timestamps;
- random UUIDs;
- OS handles;
- native window handles;
- host thread identifiers;
- map iteration order.

A local target identity does not imply that the target is authorized or
semantically valid outside the active projection.

## 6. Transition stimulus model

`ShellTransitionInput` is the conceptual input envelope for one transition.
Each envelope contains exactly one primary stimulus class:

- `LifecycleCommand`;
- `NormalizedInteraction`;
- `OrderedProjectionPatchBatch`;
- `ViewportContextChange`;
- `ExplicitNoOp`.

Common transition inputs are:

- previous `ShellLocalState`;
- `ActivatedShellSessionContext`;
- the primary stimulus.

A transition must not acquire additional data from the host. Hidden reads are
not permitted.

## 7. Evaluation order

Every transition is evaluated in this deterministic order:

1. validate session identity;
2. validate lifecycle eligibility;
3. validate the outer transition envelope and primary stimulus class;
4. validate input-side resource bounds;
5. validate stable target identities;
6. validate replay-cursor compatibility where applicable;
7. calculate the candidate next state and candidate outputs without committing;
8. validate candidate invariants and candidate-state/output resource bounds;
9. commit the complete candidate state or preserve the previous state;
10. publish the disposition and already validated bounded outputs, then apply
    the immutable diagnostic emission cap to the stable logical diagnostic
    sequence.

No partial local-state commit is permitted.

Stages 1 and 2 perform only bounded session and lifecycle checks.

Stage 3 validates only the fixed outer envelope and primary stimulus
discriminant required to identify the stimulus class. It does not traverse
patch operations, target collections, route collections, or other
variable-length semantic contents.

Stage 4 validates every resource bound that can reject the supplied input
before per-element processing begins:

- maximum patches per transition;
- maximum transition stimulus bytes.

Patch count and transition stimulus byte length must be available through
bounded structural metadata or another representation-independent bounded
preflight mechanism. This contract does not select a Rust representation,
serialized format, or counting algorithm.

Stable-target validation and replay-cursor compatibility traversal do not begin
until stage 4 succeeds.

Stage 8 validates limits that depend on the calculated candidate state or
candidate outputs:

- maximum active nodes;
- maximum focusable nodes;
- maximum hit-test entries;
- maximum accessibility nodes;
- maximum draw commands per transition;
- maximum projected text bytes;
- maximum local session-state bytes;
- maximum projected-value cache entries;
- maximum invalidation entries;
- maximum damage regions.

No candidate state is committed until stage 8 succeeds. Failure at stage 4 or
stage 8 produces `Rejected` and preserves the complete previous
`ShellLocalState`.

After stage-4 rejection, no target, replay, candidate-state, draw,
accessibility, hit-test, focus, or `ActionIntent` processing occurs. Any
diagnostic produced by stage-4 rejection remains subject to the immutable
stage-10 diagnostic emission cap.

Before stage 10 emission, the transition has determined its disposition and
complete logical diagnostic sequence in stable diagnostic order. Stage 10
applies maximum diagnostics per transition to that ordered sequence.
Diagnostic emission bounding is output shaping only. It is not transition
validation, state authorization, or a reason to commit a candidate after
another resource bound failed.

This contract does not define `Atomic` versus `OrderedPartial` semantics inside
a `ProjectionPatch` batch. Patch-batch transaction and rollback semantics
remain a separate future contract.

### 7.1 Replay-cursor compatibility

Replay-cursor compatibility is evaluated only for an
`OrderedProjectionPatchBatch` after stages 1 through 5 have succeeded.

Stage 5 owns stable-target validation. Stage 6 assumes that stage 5 has
succeeded and does not repeat, bypass, weaken, or reinterpret target
validation.

The Shell Player replay cursor is local, reconstructible session state. It is
not Semantic truth, authority, admission evidence, patch-application evidence,
or a renderer/backend coordinate.

`ProjectionReplayCursor` has the conceptual states:

- `Uninitialized` — no outer patch-batch replay coordinate has been
  established for this local session;
- `At(n)` — `n` is the currently established outer patch-batch replay
  coordinate for this local session.

This contract defines only compatibility with the established coordinate. It
does not define cursor advancement, cursor reset, persistence, restoration, or
the operation that establishes `At(n)`.

`OrderedProjectionPatchBatch.sequence_no` is the Shell Player outer batch
sequence coordinate. It does not reinterpret, replace, validate, or expose the
internal `ProjectionPatchSequence` values owned by the Projection Patch model.

Compatibility is determined as follows:

| Patch count | Previous cursor | Incoming sequence | Result |
| --- | --- | --- | --- |
| `0` | any cursor | any `u64` | Not applicable |
| greater than `0` | `Uninitialized` | any `u64` | Compatible |
| greater than `0` | `At(n)` | `n.checked_add(1) == Some(sequence_no)` | Compatible |
| greater than `0` | `At(n)` | any other value | Mismatch |
| greater than `0` | `At(u64::MAX)` | any value | Mismatch |

A zero-patch batch does not participate in replay compatibility and does not
establish or consume a replay coordinate.

Sequence arithmetic never wraps. `u64::MAX + 1` is not sequence zero and is
not a compatible successor.

Duplicate, lower, skipped, wrapped, and otherwise non-successor values all map
to the single stable diagnostic class:

```text
SPV0_REPLAY_CURSOR_MISMATCH
```

The diagnostic belongs to evaluation stage 6.

Compatibility evaluation is read-only. It does not mutate the previous cursor,
calculate a candidate cursor, traverse patch operations, validate stable
targets, apply a patch, or commit local state.

A compatible result means only that stage 6 succeeded. It does not mean that
stages 7 through 9 will succeed, that the patch batch will be applied, or that
the cursor will advance.

If any of stages 1 through 5 rejects the transition, stage 6 is not evaluated.
The diagnostic and preservation rules of the earlier rejecting stage retain
precedence.

Cursor advancement and the exact commit rule that may establish a new `At(n)`
remain separately unauthorized.

## 8. Transition disposition

The transition disposition is exactly one of:

| Disposition | Meaning |
| --- | --- |
| `Applied` | A complete valid next state was committed. |
| `NoChange` | The input was valid but produced no observable local-state change. |
| `Rejected` | No state change was committed. |

This contract does not define `PartiallyApplied`. Partial patch-batch semantics
remain unresolved.

## 9. Transition outputs

A successful transition may produce:

- next `ShellLocalState`;
- focus realization;
- hit-test realization;
- accessibility realization;
- backend-neutral draw/session material;
- an optional `ActionIntent` candidate;
- deterministic diagnostics;
- a resource-accounting result.

Normative non-authority rules:

```text
focus realization != Semantic focus truth
hit-test realization != authorization
ActionIntent candidate != admission
draw/session material != pixels
diagnostic output != production evidence
```

## 10. Resource contract

Caller-supplied deterministic limits use these categories:

- maximum active nodes;
- maximum focusable nodes;
- maximum hit-test entries;
- maximum accessibility nodes;
- maximum patches per transition;
- maximum draw commands per transition;
- maximum diagnostics per transition;
- maximum projected text bytes;
- maximum local session-state bytes;
- maximum transition stimulus bytes;
- maximum projected-value cache entries;
- maximum invalidation entries;
- maximum damage regions.

This contract assigns no default numeric values.

Normative rules:

- limits are caller supplied exactly once through
  `ActivatedShellSessionContext`;
- limits grant no authority;
- limits are immutable for the lifetime of the activated session;
- Shell Player does not invent or widen limits;
- all stimulus, input-side, candidate-state, and candidate-output limits are
  checked before the stage 9 state commit;
- exhaustion of an input-side, candidate-state, or candidate-output limit
  yields `Rejected` and preserves the previous state;
- state and candidate resource-limit exhaustion never causes partial commit;
- limit-exhaustion diagnostics are deterministic.

Maximum diagnostics per transition is a deterministic diagnostic emission cap
applied at stage 10. It does not affect `Applied`, `NoChange`, or `Rejected`,
cannot convert a rejected transition into a committed transition, and cannot
cause rollback after a valid candidate state has been committed. A zero cap
emits no diagnostics. If the logical diagnostic count exceeds the cap, only
the stable prefix up to the cap is emitted. Truncation does not generate
another diagnostic and therefore cannot recurse.

Resource accounting may conceptually distinguish the logical diagnostic count
from the emitted diagnostic count. This contract does not define Rust fields
or serialization for those counts.

## 11. Diagnostic namespace

The reserved diagnostic prefix is `SPV0_`.

The diagnostic classes are:

| Code | Class |
| --- | --- |
| `SPV0_SESSION_MISMATCH` | The transition context and previous state do not identify the same session. |
| `SPV0_INVALID_LIFECYCLE` | The requested lifecycle transition is not allowed. |
| `SPV0_SESSION_CLOSED` | Input was presented to a closed session. |
| `SPV0_SESSION_SUSPENDED` | Interaction input was presented to a suspended session. |
| `SPV0_INVALID_STIMULUS` | The primary stimulus shape or class is invalid. |
| `SPV0_INVALID_TARGET` | A stable target identity is invalid for the active projection. |
| `SPV0_REPLAY_CURSOR_MISMATCH` | stage-6 outer batch sequence incompatibility with the established local replay cursor. |
| `SPV0_RESOURCE_LIMIT_EXCEEDED` | A caller-supplied deterministic limit would be exceeded. |
| `SPV0_STATE_INVARIANT_VIOLATION` | The candidate next state violates a frozen invariant. |

Every diagnostic has:

- a stable code;
- a stable stage;
- a stable primary coordinate where applicable;
- deterministic ordering;
- no host-specific paths;
- no memory addresses;
- no nondeterministic debug formatting.

This contract does not define implementation-specific Rust error enums.

## 12. Determinism

For identical:

- `ActivatedShellSessionContext`;
- previous `ShellLocalState`;
- `ShellTransitionInput`;

the transition produces identical:

- disposition;
- next local state;
- outputs;
- diagnostics;
- resource accounting.

Transition behavior must not depend on:

- system clock;
- randomness;
- filesystem;
- network;
- host locale;
- OS ordering;
- thread scheduling;
- ambient process state;
- backend polling;
- live Semantic reads.

## 13. Explicitly unresolved after UI-DNA2-9B

The following remain unresolved:

- replay cursor advancement rule;
- replay cursor establishment/restore rule;
- cursor persistence representation;
- integration with ProjectionPatch internal sequences;
- `ProjectionPatch` batch transaction model;
- `Atomic` versus `OrderedPartial` patch semantics;
- rollback representation;
- unknown-target patch handling;
- unknown-operation patch handling;
- patch mutation algorithm;
- focus traversal algorithm;
- pointer-capture algorithm;
- hit-test coordinate model;
- accessibility encoding;
- draw-command encoding;
- layout algorithm;
- `ActionIntent` route-emission algorithm;
- Rust representations;
- module layout;
- public APIs.

These decisions must not be silently solved by UI-DNA2-9B.

## 14. Explicit non-goals

- no Rust code;
- no `ShellSession` struct;
- no `shell_player` module;
- no `ProjectionPatch` application;
- no bundle parser;
- no bundle validator;
- no bundle verifier;
- no inert loader;
- no bundle activation implementation;
- no `ActionIntent` admission;
- no renderer integration;
- no backend integration;
- no event loop;
- no Workbench;
- no Semantic Studio;
- no Gate D movement;
- no production promotion.

## 15. Final status

```text
Shell Player session input contract = FROZEN
Shell Player lifecycle contract = FROZEN
Shell Player local-state domains = FROZEN
Shell transition envelope = FROZEN
transition disposition model = FROZEN
resource-limit categories = FROZEN
diagnostic namespace = FROZEN
replay-cursor compatibility relation = FROZEN

replay-cursor compatibility implementation = NOT AUTHORIZED
replay-cursor advancement = NOT AUTHORIZED
ProjectionPatch application = NOT AUTHORIZED
Shell Player Rust implementation = NOT AUTHORIZED
bundle activation = NOT AUTHORIZED
renderer integration = NOT AUTHORIZED
backend integration = NOT AUTHORIZED
Gate D = CLOSED
production promotion = NOT AUTHORIZED
NEXT AUTHORIZED IMPLEMENTATION SLICE = NONE
```
