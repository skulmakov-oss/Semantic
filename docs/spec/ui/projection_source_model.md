# Projection Source Model

Status: draft spec
Track: POST-UI / Intent-Driven Projection
Scope type: documentation only
Depends on:
- docs/dna/SEMANTIC_UI_DNA.md
- docs/dna/SEMANTIC_UI_DNA_v2.md
- docs/roadmap/post_ui/intent_driven_projection_roadmap.md
Related:
- #1310
- #1327
- #1328
- #1329
- #1330
- docs/spec/ui/projection_source_grammar_v0.md

The projection source model describes presentation intent for Semantic-owned UI projection.

It does not define semantic truth, verifier admission, VM behavior, runtime authority, renderer behavior, or production UI wiring.

The source model does not itself define parser grammar.

The bounded structural grammar contract is specified separately in
projection_source_grammar_v0.md.

Parser implementation remains a separate, unauthorized slice.

## Source Role Representation Boundary

Projection Source AST stores authored role identifiers as owned source text.

RoleDictionary retains canonical static `RoleId` values.

Owned source text is resolved through RoleDictionary during Projection Source
semantic validation and lowering.

A syntactically valid unknown role remains representable in the source AST and
fails through `PS_UNKNOWN_ROLE`.

Source-role storage does not grant role validity, authority, runtime behavior,
renderer ownership, or admission.

```text
source role name != canonical RoleId
source storage != semantic acceptance
unknown source role != parser failure
known role resolution != authority
```

## Source Size and Provenance Representability

Projection Source provenance uses UTF-8 byte offsets represented by the landed
`SourceSpan` `u32` fields. The normative representability limit is
`u32::MAX = 4_294_967_295` source bytes. This model does not widen `SourceSpan`
to `u64`, `usize`, character offsets or line/column coordinates.

For a future parser receiving `&str`, source-size preflight occurs before
parser ownership. A source longer than `u32::MAX` bytes is classified as the
future input-domain error `ProjectionSourceInputError::SourceTooLarge`; it
produces no SourceSpan, PSP diagnostic, tokenization, AST or PS validation.
The conceptual error carries the caller-supplied `SourceId`, actual UTF-8 byte
length and maximum accepted byte length. The concrete API is not implemented
by this model correction.

The limit is a representability ceiling, not a recommended operational file
size. A future host or loader may impose a smaller memory, quota, transport or
sandbox limit, but that host resource rejection remains outside Grammar v0.

The normative representability ceiling is `u32::MAX` bytes on every platform.
On platforms where `usize` can represent values greater than `u32::MAX`, an
`&str` longer than `u32::MAX` bytes is rejected as `SourceTooLarge` before
lexical processing. On platforms where `usize` cannot represent a value
greater than `u32::MAX`, every constructible `&str` is within the
representability ceiling. The grammar contract does not vary with pointer
width; practical allocation capacity on a 32-bit platform is a separate
matter.

```text
normative maximum != usize::MAX
normative maximum != platform-dependent maximum
32-bit platform behavior != different grammar contract
64-bit platform capability != wider SourceSpan
```

After preflight succeeds, every parser position is in
`0..=input_byte_length` and therefore in `0..=u32::MAX`. Surface and node
declaration endpoints, token endpoints, zero-width missing-token positions,
duplicate-declaration keyword spans, offending-character or offending-token
spans, the EOF position and `PSP_UNEXPECTED_EOF` all use representable
`SourceSpan` endpoints. Representability is not validity and does not permit
unchecked conversion. Existing half-open span shapes remain unchanged.

```text
source byte length != character count
source-size acceptance != syntax validity
source-size acceptance != parse success
source-size acceptance != semantic validity
parse success != semantic validation success
semantic-validation success != runtime activation
input rejection != parser diagnostic
input rejection != runtime admission
provenance representability != authority
```

Source-size policy specified != parser implementation
source-size policy specified != parser qualification
source-size policy specified != runtime source loading
source-size policy specified != WP2C completion

The P1 source-size contract is landed; it did not itself authorize WP2C-P2.
Grammar v0 separately defines the normative
WP2C-P2 clause-context diagnostic contract. Specification of that contract is
not parser or lexer implementation, does not resolve WP2C-P3, and does not
authorize runtime loading or activation. The P2 contract is not treated as
landed-main evidence until merge and ledger rebaseline.

WP2C-P3 remains unresolved and unauthorized.
The Projection Source parser and lexer remain unimplemented and unauthorized.

No source-size check grants capability, performs admission, loads files,
allocates runtime buffers, activates a ProjectionBundle, mutates shell state,
selects a renderer, opens Gate D or claims production readiness.

## 1. Purpose

The projection source model exists to prevent the bad workflow:

```text
.sm + hand-written Rust UI glue
```

The intended workflow is:

```text
.sm owns meaning.
Projection source owns presentation intent.
Compiler emits UI IR.
Shell renders UI IR.
Semantic admission remains authoritative.
```

Projection source is a way to describe what should be projected without turning Semantic source into layout code.

## 2. Source File Posture

`.proj.sm` is the preferred v0 working name for projection source files.
The name is not a parser commitment until a grammar spec is approved.

A projection source is a companion to Semantic source.
It does not replace `.sm`.
It does not embed semantic business logic.
It does not override verifier or admission behavior.

### Inline projection rule

Inline projection inside `.sm` is forbidden in v0 unless separately approved by governance.

Semantic meaning must not be polluted by renderer, layout, or presentation lifecycle concerns.

## 3. Allowed Projection Intent

Projection source may express intent about:

- surfaces;
- semantic roles;
- state bindings;
- action affordances from admitted / action-offer sources;
- evidence outlets;
- denial outlets;
- recovery outlets;
- task projection contracts;
- freshness / connectivity display intent;
- accessibility labels and focus intent;
- priority / criticality hints;
- viewer-relative visibility policy.

These are intent declarations, not widgets.

Projection source may describe what a surface should present, what it should expose, and how it should be interpreted by non-visual and visual channels.

## 4. Forbidden Content

Projection source must not contain:

- business logic;
- verifier rules;
- admission rules;
- VM / runtime behavior;
- host effects;
- network calls;
- file system effects;
- renderer backend selection;
- absolute pixels;
- CSS-like layout;
- manual colors / fonts / themes;
- animation implementation;
- hand-written Rust UI glue;
- production shell wiring;
- dependency declarations;
- unsafe escape hatches.

Projection source is presentation intent, not a hidden implementation language.

## 5. Minimal Role Vocabulary

The following draft roles seed the projection vocabulary:

| Role | Meaning | Allowed use | Forbidden interpretation |
| --- | --- | --- | --- |
| `AppSurface` | top-level projected application surface | root surface container | renderer backend or app runtime |
| `Panel` | grouped projected region | structural partition | absolute layout object |
| `Section` | named area within a surface | logical grouping | business logic boundary |
| `FieldGroup` | related readouts or inputs | clustered form-like projection | widget toolkit ownership |
| `TextReadout` | textual projected value | labels, status, prose | semantic source of truth |
| `NumericReadout` | numeric projected value | counters, totals, results | computation engine |
| `StateBadge` | compact state indicator | status, mode, freshness | admission authority |
| `EvidencePanel` | evidence or trace surface | diagnostics, provenance, audit projection | audit authority |
| `DenialOutlet` | denial or refusal surface | projected denials, reasons, recovery hints | policy engine |
| `RecoveryOutlet` | recovery / retry projection | resume, retry, acknowledge | recovery implementation |
| `TaskPanel` | task-oriented projection area | progress, task state, controls | task engine |
| `ActionSlot` | location for an action affordance | button region or command affordance | direct effect execution |
| `SafeAction` | low-risk admitted action affordance | accepted control surface | bypass around capability checks |
| `GuardedAction` | constrained action affordance | capability-gated control | silent repeat or unchecked effect |
| `DangerAction` | high-risk action affordance | explicit risky control | implicit confirmation bypass |
| `ConnectivityBadge` | freshness or connection indicator | connected / stale / offline display | networking implementation |
| `List` | projected collection container | ordered item set | storage engine |
| `ListItem` | projected collection item | one stable item projection | anonymous visual fragment |

Roles are semantic projection roles, not renderer widgets.
A role must remain interpretable by non-visual surfaces such as CLI, logs, voice UI, or evidence reports.

## 6. Binding Model Boundary

Projection bindings observe Semantic state.
They do not mutate Semantic state directly.
Mutation attempts must become structured `ActionIntent` candidates and pass admission.

Binding categories include:

- state binding;
- evidence binding;
- action offer binding;
- task binding;
- connectivity binding.

Projection bindings describe relationships and visibility, not mutation behavior or implementation control flow.

## 7. Action Affordance Boundary

Projection source may declare where action affordances appear.
It must not invent actions.
It must not bypass `ActionOffers`, capability checks, or admission.

```text
UI proposes.
Semantic disposes.
Shell shows.
```

Action affordance declarations describe route and presentation, not execution authority.

## 8. Accessibility as Projection Contract

Accessibility is part of the source model, not renderer polish.

Accessibility declarations should include:

- human-readable label;
- role description;
- focus order intent;
- criticality;
- denial / recovery discoverability;
- non-visual interpretation;
- evidence provenance where practical.

Accessibility intent must remain visible in the projection contract even when the final renderer differs.

## 9. Quad-State Preservation

Projection source must preserve:

- `N` — unknown
- `F` — false
- `T` — true
- `S` — conflict

Projection source must not flatten Quad-state into boolean visibility, success/failure, or generic disabled UI.

Unknown remains unknown.
Conflict remains conflict.
Denial is not the same as false.

## 10. Non-Normative Example

Non-normative sketch — not parser syntax.
This sketch remains non-normative and may contain concepts outside grammar v0.

```text
projection CalculatorView for CalculatorState {
  surface main role AppSurface {
    readout display role NumericReadout bind result
    action add role SafeAction from ActionOffers.add
    evidence outlet EvidencePanel
    denial outlet DenialOutlet
  }
}
```

This sketch only illustrates intent flow and role placement.
It is not final grammar, not an implementation, and not a parser commitment.

## 11. Acceptance Criteria

The spec is acceptable when:

- it defines `.proj.sm` as preferred working name;
- it explains relation to `.sm`;
- it blocks inline projection in v0;
- it defines allowed projection intent;
- it defines forbidden content;
- it includes draft role vocabulary;
- it preserves Semantic authority;
- it preserves Quad-state meaning;
- it treats accessibility as contract;
- it does not implement parser grammar;
- it does not claim production readiness.
