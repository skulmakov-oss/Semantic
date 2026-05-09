# Semantic UI Visual Design Doctrine

Status: Draft
Track: POST-UI / H-series
Purpose: define the visual doctrine before renderer and visual implementation

## 1. Core principle

Semantic UI is not decoration.

Semantic UI is the visible form of Semantic architecture.

The interface must express:

- semantic state;
- capability admission;
- lifecycle transitions;
- traceability;
- controlled effects;
- deterministic reference behavior;
- native/runtime boundaries.

Visual design must follow system meaning.

```text
semantic state
  -> admission
  -> lifecycle
  -> trace
  -> visual grammar
  -> renderer
```

The renderer must serve this doctrine.
The doctrine must not be retrofitted to whatever the renderer happens to support.

## 2. Target character

Semantic UI should feel like:

```text
a precise industrial control surface
a semantic machine cockpit
a laboratory-grade reasoning instrument
an architectural map of controlled execution
```

It should not feel like:

```text
a generic dashboard
a web admin panel
a toy sci-fi interface
a neon cyberpunk skin
a random node editor
a decorative IDE clone
```

## 3. Design values

| Value | Meaning |
| --- | --- |
| semantic-first | visual state follows semantic state |
| deterministic | same state should produce the same visual interpretation |
| industrial clarity | every element has purpose |
| traceability | actions expose cause, admission, result, and trace |
| restraint | visual emphasis is rare and meaningful |
| depth without noise | hierarchy is visible without decoration clutter |
| fail-closed visibility | denied/invalid states are visible, not hidden |
| platform discipline | native surface must not dictate architecture |

## 4. Visual hierarchy

The UI must reflect system hierarchy:

```text
System
  -> Module
    -> State
      -> Transition
        -> Effect
          -> Trace
```

Visual hierarchy may use:

- panels;
- layers;
- timelines;
- state cards;
- graph edges;
- admission gates;
- trace lanes;
- capability badges;
- lifecycle indicators.

The hierarchy must not be arbitrary.
Every visible grouping should correspond to ownership, state, capability, or trace.

## 5. State visual language

The UI must distinguish system states clearly.

| Semantic / runtime state | Visual intent |
| --- | --- |
| ready | stable, quiet, low emphasis |
| running | active, controlled emphasis |
| admitted | clear positive confirmation |
| denied | strong but not noisy refusal |
| unknown | subdued, unresolved state |
| conflict | visible tension or boundary warning |
| closed | inert, final, non-interactive |
| failed | explicit failure, trace required |
| quarantined | isolated but inspectable |

No state should rely on color alone.
Shape, placement, label, and trace context must also carry meaning.

## 6. Color doctrine

Color is semantic, not decorative.

Allowed use:

```text
status
capability
admission
risk
conflict
trace category
focus
```

Forbidden use:

```text
random accenting
ornamental gradients
decorative neon
theme-first color choices
color without semantic meaning
```

Recommended palette direction:

| Role | Direction |
| --- | --- |
| base | graphite / dark neutral / restrained light neutral |
| surface | layered matte panels |
| active | single controlled accent |
| warning | amber / controlled hazard |
| denial/error | strict red, used sparingly |
| success/admitted | stable green or cool white |
| unknown | muted blue/gray |
| conflict | high-contrast boundary treatment |

Exact colors are not fixed in H1.
They belong to a later visual token PR.

## 7. Motion doctrine

Motion must represent state change.

Allowed motion:

- admission granted;
- admission denied;
- lifecycle transition;
- trace committed;
- effect prepared;
- rollback;
- conflict isolation;
- focus shift;
- module activation.

Forbidden motion:

- decorative animation;
- constant idle motion without meaning;
- motion that hides state;
- motion that makes deterministic state feel random;
- animation that implies progress where none exists.

Motion must be interruptible, explainable, and non-essential for understanding.

## 8. Layout doctrine

Layout must expose architecture.

Preferred layout qualities:

- grid-based;
- modular;
- inspectable;
- stable under state changes;
- clear ownership boundaries;
- dense but not cluttered;
- readable in long sessions.

Forbidden layout qualities:

- floating widgets with unclear ownership;
- dashboard tiles without semantic grouping;
- deep nesting without trace path;
- decorative panels without function;
- hidden effects behind visual affordances.

## 9. Typography doctrine

Typography must support inspection and control.

Text should distinguish:

- object name;
- semantic type;
- state;
- capability;
- error;
- trace;
- explanation;
- operator action.

Typography must not become ornamental.
Readable technical density is preferred over marketing-style visual weight.

## 10. Trace-first interface

Semantic UI must make actions inspectable.

For important actions, the UI should be able to answer:

1. What action was requested?
2. Which capability was required?
3. Was admission granted or denied?
4. What lifecycle state was active?
5. What state changed?
6. What trace was produced?
7. What effect was prepared or committed?
8. What failed, if anything?

A beautiful UI that cannot explain action causality is not Semantic UI.

## 11. Capability-aware visual grammar

Capability and admission must be visible as first-class UI concepts.

Examples:

```text
disabled because capability missing
available because capability admitted
blocked because lifecycle state invalid
accepted because admission metadata matches
quarantined because conflict boundary triggered
```

The UI must not hide capability failures behind generic disabled controls.

## 12. Renderer relationship

Renderer implementation must serve this doctrine.

Renderer admission is defined separately in:

```text
docs/architecture/ui_renderer_admission_boundary.md
```

Renderer must not introduce visual behavior that contradicts:

- semantic-first state;
- traceability;
- capability admission;
- lifecycle discipline;
- fail-closed behavior;
- platform-neutral runtime boundaries.

The renderer is not the owner of visual meaning.
The renderer is an execution layer for admitted visual grammar.

## 13. Native backend relationship

Native backend is a platform path, not a visual authority.

```text
NativeBackendWinitApp
  -> native facade
  -> event loop/window ownership
```

It must not decide visual doctrine.

Native backend may expose:

- window lifecycle;
- event facts;
- draw staging facts;
- renderer transcript facts after renderer admission.

But visual meaning remains owned by Semantic UI doctrine and higher UI contracts.

## 14. Forbidden styles

Semantic UI must avoid:

- generic SaaS dashboard look;
- toy cyberpunk;
- random glow effects;
- “AI assistant chat skin” aesthetic;
- decorative node graph chaos;
- IDE clone without semantic identity;
- corporate gray admin panel;
- skeuomorphic industrial cosplay;
- overly playful motion;
- hidden side effects.

Industrial inspiration is allowed.
Industrial imitation is not the goal.

## 15. Design references by principle, not by copying

Allowed inspirations by principle:

| Source family | Extracted principle |
| --- | --- |
| industrial control rooms | functional density and status clarity |
| laboratory instruments | precision and restraint |
| avionics | state visibility and fail-closed controls |
| architectural diagrams | hierarchy and structure |
| high-end product design | material discipline and calm surfaces |
| IDEs | technical density and inspection |

Do not copy brand-specific style.

## 16. Acceptance criteria for future visual implementation

A future visual implementation is admissible only if it can show:

- semantic state hierarchy;
- lifecycle state;
- admission status;
- capability status;
- trace path;
- draw/render transcript distinction;
- failure/denial states;
- no hidden side effects;
- no renderer-owned semantics.

## 17. Current decision

Visual implementation is not admitted in H1.

H1 only defines doctrine.

Current admitted UI implementation remains:

```text
prom-ui-runtime
  -> lifecycle/session/backend contracts

prom-ui-backend-native
  -> native facade
  -> transcript boundaries
  -> draw staging/accounting
```

Renderer and visual implementation must follow this doctrine in later PRs.

## Workbench UI consumption boundary

Workbench must consume Semantic UI doctrine, not redefine it.

Workbench UI consumption is defined separately in:

```text
docs/architecture/ui_workbench_consumption_boundary.md
```

Workbench views must not become source of truth.

## Visual token system boundary

The visual token system is defined separately in:

```text
docs/architecture/ui_visual_token_system_boundary.md
```

Visual tokens are the reusable vocabulary of Semantic UI visual meaning.

Tokens must follow this doctrine.

Tokens must not introduce arbitrary theme values or renderer-owned meaning.

## Layout primitive boundary

The layout primitive system is defined separately in:

```text
docs/architecture/ui_layout_primitive_boundary.md
```

Layout primitives are the spatial grammar of Semantic UI.

They must follow this doctrine and consume admitted visual tokens.

Layout primitives must not introduce arbitrary widgets or renderer-owned layout meaning.

## Component admission boundary

The component admission boundary is defined separately in:

```text
docs/architecture/ui_component_admission_boundary.md
```

Components are reusable semantic UI units.

They must follow this doctrine, consume admitted visual tokens, and compose admitted layout primitives.

Components must not introduce arbitrary widgets or renderer-owned component meaning.

## Error, denial, and quarantine doctrine

Error, denial, conflict, and quarantine states must be explicit and inspectable.

They are defined separately in:

```text
docs/architecture/ui_error_denial_quarantine_visual_boundary.md
```

Visual refusal must not become hidden no-op.
