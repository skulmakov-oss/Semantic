# Semantic UI Implementation Gate

Status: Draft
Track: POST-UI / H-series
Purpose: define the gate for moving from UI boundary documentation into implementation

## 1. Goal

This document defines when Semantic UI implementation may begin and what constraints every implementation PR must follow.

H-series documentation has established the semantic boundaries required to avoid architecture drift.

Implementation may begin only through small, boundary-backed PRs.

## 2. Gate statement

The UI documentation phase is considered sufficient after this gate is merged.

The next phase is implementation.

However, implementation is allowed only if it preserves the admitted boundary stack:

```text
visual doctrine
  -> visual token boundary
  -> layout primitive boundary
  -> component admission boundary
  -> interaction/input semantic boundary
  -> focus/selection semantic boundary
  -> semantic action boundary
  -> effect request / UI capability boundary
  -> trace/audit visual boundary
  -> error/denial/quarantine visual boundary
  -> recovery/rollback visual boundary
  -> renderer transcript / presentation status boundary
  -> Workbench UI consumption boundary
  -> simulation/snapshot UI boundary
```

## 3. Required PR declaration

Every future UI implementation PR must declare:

1. touched layer;
2. boundary documents used;
3. explicitly out-of-scope layers;
4. authority ownership;
5. trace/audit relationship if any;
6. renderer/native relationship if any;
7. Workbench relationship if any;
8. tests added or updated.

A PR without boundary declaration is not admitted.

## 4. Implementation order

Preferred implementation order:

```text
I1 boundary registry scaffold
I2 visual token type scaffold
I3 layout primitive type scaffold
I4 component metadata scaffold
I5 interaction intent descriptor scaffold
I6 focus/selection state scaffold
I7 semantic action descriptor scaffold
I8 renderer transcript type scaffold
I9 Workbench consumption map scaffold
```

Renderer pixels come later.

## 5. First allowed implementation PR

The first allowed implementation PR:

```text
PR-UI-I1 — feat(ui): add UI boundary registry scaffold
```

Allowed:

* a tiny docs-backed registry;
* names/paths of admitted boundary documents;
* tests that ensure all required boundary docs exist;
* no runtime behavior;
* no renderer;
* no Workbench UI;
* no components.

Forbidden:

* rendering;
* `wgpu` / `pixels` / `softbuffer`;
* native surface ownership;
* buttons/widgets;
* action execution;
* effect execution;
* command palette;
* snapshot/replay engine.

## 6. Renderer implementation gate

Renderer implementation is not admitted by H15.

Before renderer code, the project must have at minimum:

* visual token scaffold;
* layout primitive scaffold;
* renderer transcript scaffold;
* presentation status scaffold;
* tests proving draw staging is not presentation;
* explicit renderer admission PR.

Renderer must not be the first implementation step.

## 7. Workbench implementation gate

Workbench UI implementation is not admitted directly by H15.

Before Workbench UI code, the project must have at minimum:

* Workbench consumption map;
* source-of-truth rules;
* stale/snapshot/simulation mode handling or explicit non-support;
* action/effect admission path if commands are added.

Workbench must consume core contracts, not define them.

## 8. Component implementation gate

Component implementation is not admitted directly by H15.

Before components:

* visual token scaffold must exist;
* layout primitive scaffold must exist;
* component metadata/admission scaffold must exist;
* each component must cite semantic purpose and boundary.

No arbitrary widget library is admitted.

## 9. Interaction/action implementation gate

Interaction/action implementation must preserve:

```text
native event
  -> InputEvent
  -> interaction intent
  -> admission
  -> semantic action
  -> trace/effect if applicable
```

No callback may directly perform semantic effects.

Workbench shortcuts must not bypass this chain.

## 10. Effect/capability implementation gate

Effect/capability implementation must preserve:

```text
semantic UI action
  -> effect request
  -> UI capability admission
  -> runtime capability mapping if admitted
  -> prepared effect
  -> commit boundary
```

UI capability display is not capability grant.

## 11. Trace/audit implementation gate

Trace/audit UI implementation must preserve:

```text
trace/audit record
  -> visual projection
  -> inspection UI
```

Visual trace is not source of truth.

Renderer transcript is not audit authority by default.

## 12. Simulation/snapshot implementation gate

Simulation/snapshot implementation must preserve:

```text
live source
  -> snapshot / replay / simulation / preview
  -> explicit mode label
  -> no authority unless re-admitted
```

Non-live views must not look authoritative.

## 13. Required tests

Every implementation PR must include at least one of:

* contract test;
* compile-time shape test;
* docs index test;
* snapshot test;
* no-op/fail-closed test.

No implementation PR should land without a test unless it is docs-only.

## 14. Stop conditions

Stop implementation if a PR requires:

* bypassing admission;
* treating renderer output as semantic authority;
* treating Workbench command as semantic action;
* treating trace view as audit authority;
* using visual state as capability grant;
* treating simulation/snapshot as live state;
* hiding denial/failure/quarantine;
* adding global UI state without ownership.

If a stop condition appears, write a boundary correction PR first.

## 15. Documentation freeze

After H15, the UI boundary docs are frozen for initial implementation.

Allowed docs changes after freeze:

* typo fixes;
* link fixes;
* boundary correction PRs caused by implementation discovery;
* implementation-specific docs for admitted code.

Not allowed:

* adding new H-series boundary docs without explicit reason;
* expanding architecture instead of implementing;
* delaying code with speculative UI doctrine.

## 16. Current decision

H15 closes the H-series boundary phase.

Next step:

```text
PR-UI-I1 — feat(ui): add UI boundary registry scaffold
```

This starts code carefully, without renderer, without Workbench UI, without components, and without effectful behavior.
