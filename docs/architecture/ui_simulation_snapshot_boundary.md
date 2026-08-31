# Semantic UI Simulation and Snapshot Boundary

Status: Draft
Track: POST-UI / H-series
Purpose: define simulation, snapshot, replay, preview, and stale-state boundaries before implementation

## 1. Goal

This document defines the boundary between live UI state, snapshots, simulations, previews, replays, and stale projections.

The project must distinguish:

```text
snapshot != live state
simulation != runtime state
preview != admitted action
replay != current trace
stale view != authority
```

A non-live view must never look authoritative.

Simulation and snapshot UI must be explicit, inspectable, and separated from live runtime authority.

## 2. Relationship to Workbench boundary

Simulation and snapshot views are especially relevant to Workbench:

```text
docs/architecture/ui_workbench_consumption_boundary.md
```

Workbench may later consume:

* live state;
* snapshots;
* trace replays;
* simulated state;
* previewed actions;
* stale/disconnected projections.

But Workbench must clearly label those modes.

Workbench view is not source of truth.

## 3. Layer separation

| Mode         | Meaning                                           | Authority                     |
| ------------ | ------------------------------------------------- | ----------------------------- |
| live         | current admitted runtime/UI state                 | live runtime/source boundary  |
| snapshot     | captured state at a point in time                 | historical/static only        |
| replay       | playback of prior trace/event sequence            | historical/trace-derived only |
| simulation   | hypothetical state evolution                      | sandbox/hypothesis only       |
| preview      | possible result before admission/commit           | non-authoritative             |
| stale        | previously valid view no longer confirmed current | no current authority          |
| disconnected | no active source connection                       | no live authority             |

This preserves:

```text
Live is not snapshot.
Snapshot is not live.
Simulation is not runtime.
Preview is not action.
Replay is not current trace.
Stale is not authority.
```

## 4. Live state definition

Live state is the currently authoritative state from an admitted source.

Live state may include:

* current UI state;
* current capability state;
* current lifecycle state;
* current renderer transcript state;
* current trace projection;
* current Workbench-connected view.

Live state must have a source/provenance.

A view must not claim to be live unless the source is current and admitted.

## 5. Snapshot definition

Snapshot is captured state from a specific point.

A snapshot must carry:

* capture time or logical epoch if available;
* source identity;
* scope;
* whether it is complete or partial;
* whether it is safe for inspection only;
* relation to trace/audit if available.

Snapshot does not authorize current action.

## 6. Replay definition

Replay is playback of prior trace, event, or state sequence.

Replay may help inspect causality.

Replay must not be treated as current trace.

Replay must show:

* replay source;
* replay range;
* replay cursor;
* original trace reference;
* whether replay is complete or filtered;
* whether effects are simulated or disabled.

Replay must not re-execute effects unless a separate admitted boundary exists.

## 7. Simulation definition

Simulation is hypothetical state evolution.

Simulation may be based on:

* proposed action;
* hypothetical capability;
* sandbox state;
* replay input;
* altered target;
* renderer/presentation assumption.

Simulation must be visibly non-live.

Simulation must not mutate runtime state without explicit admission.

## 8. Preview definition

Preview is a projected possible result before admission or commit.

Examples:

```text
preview selection result
preview layout change
preview effect request
preview renderer frame
preview rollback path
preview recovery path
```

Preview is not action.

Preview is not effect.

Preview is not commitment.

A preview must not imply admission.

## 9. Stale state definition

Stale state is a view that was once based on a source but is no longer confirmed current.

Stale state may occur when:

* runtime disconnected;
* source epoch changed;
* trace moved forward;
* capability state changed;
* renderer state changed;
* Workbench lost connection;
* snapshot was opened from history.

Stale state must be labeled.

Stale state must not allow effectful actions without freshness checks and admission.

## 10. Disconnected state definition

Disconnected state means the UI no longer has a live source.

Disconnected views may remain inspectable.

They must not appear live.

Disconnected view must show:

* source lost;
* last known state if available;
* time/epoch if available;
* unavailable actions;
* safe inspection options.

## 11. Authority boundary

Only admitted live sources may provide authority.

The following are not authority by default:

```text
snapshot
simulation
preview
replay
stale view
disconnected view
Workbench local cache
renderer output
visual trace projection
```

A non-live projection may inform the user.

It must not authorize action or effect.

## 12. Admission boundary

Any action from non-live context requires explicit admission.

Examples:

```text
snapshot -> inspect only
snapshot -> restore request -> admission
simulation -> commit proposal -> admission
preview -> semantic action request -> admission
replay -> rollback request -> admission
stale view -> refresh required before effect
```

No non-live view may directly produce effectful operations.

## 13. Trace/audit relationship

Snapshot, replay, simulation, and preview may display trace/audit references.

But they must not become trace/audit authority.

Replay must preserve original trace meaning.

Simulation must not invent audit records.

Preview must not imply audit success.

If a simulated or previewed path becomes real, it must produce its own admitted trace/audit path.

## 14. Semantic action relationship

A previewed action is not an admitted action.

A simulated action is not an admitted action.

A replayed action is not current action.

Future implementation must distinguish:

```text
action.previewed
action.simulated
action.replayed
action.admitted
action.executed
```

H14 does not implement these states.

## 15. Effect/capability relationship

A simulated or previewed capability state is not a capability grant.

Examples:

```text
preview capability available
simulation assumes capability
snapshot shows capability was available
replay shows capability was admitted
```

None of these prove current capability availability.

Effectful operations from non-live views require re-admission.

## 16. Renderer transcript relationship

Renderer replay or preview must distinguish:

```text
staged frame snapshot
render preview
presentation replay
current presented frame
```

A rendered preview is not frame presentation.

A replayed presented frame is not current presentation.

Renderer transcript from replay/snapshot is not current renderer authority.

## 17. Workbench relationship

Workbench may support snapshot, replay, preview, simulation, and stale views.

Workbench must clearly distinguish modes:

```text
LIVE
SNAPSHOT
REPLAY
SIMULATION
PREVIEW
STALE
DISCONNECTED
```

Workbench must not allow these modes to visually masquerade as live state.

Workbench-local cache must not become source of truth.

## 18. Visual grammar requirements

Future visual implementation must distinguish:

| Mode         | Must not look like  | Required visibility          |
| ------------ | ------------------- | ---------------------------- |
| live         | snapshot/simulation | source/current status        |
| snapshot     | live                | capture source/time/epoch    |
| replay       | current trace       | replay cursor/source         |
| simulation   | runtime state       | hypothetical marker          |
| preview      | admitted action     | preview/non-committed marker |
| stale        | live authority      | stale marker/refresh path    |
| disconnected | live                | source unavailable state     |

Visual representation must not rely on color alone.

## 19. Forbidden shortcuts

The system must not:

* treat snapshot as live state;
* treat simulation as runtime state;
* treat preview as admitted action;
* treat replay as current trace;
* treat stale view as authority;
* allow effectful action from stale state without admission;
* allow Workbench local cache to become source of truth;
* hide disconnected state;
* show simulated capability as granted capability;
* show previewed effect as prepared or committed effect;
* show replayed frame as current presented frame.

## 20. Required admission rule

A future simulation/snapshot UI implementation PR must define:

1. mode name;
2. source identity;
3. authority level;
4. stale/freshness behavior;
5. allowed actions;
6. forbidden actions;
7. trace/audit relationship;
8. capability/effect relationship;
9. visual marker requirements;
10. Workbench relationship if applicable;
11. tests/snapshots where applicable.

No snapshot/simulation mode should be added only because it is convenient for debugging.

## 21. Future implementation shape

H14 does not mandate implementation.

Possible future shapes:

```text
docs/spec/ui_simulation_snapshot.md
crates/prom-ui-snapshot/
crates/prom-ui-simulation/
crates/prom-ui-replay/
historical TS/Tauri snapshot/replay views (retired, see docs/history/workbench_ts_tauri_legacy.md)
historical TS/Tauri simulation sandbox (retired, see docs/history/workbench_ts_tauri_legacy.md)
```

Any implementation must preserve:

```text
live source
  -> snapshot / replay / simulation / preview
  -> explicit mode label
  -> no authority unless re-admitted
```

## 22. Current decision

Simulation and snapshot UI handling is not implemented in H14.

H14 only defines the boundary.

Current admitted visual/interaction/action architecture:

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

Not yet admitted:

```text
snapshot structs
simulation structs
replay engine
preview action model
stale-state model
freshness checker
Workbench simulation view
Workbench replay timeline
snapshot-to-action bridge
simulation-to-effect bridge
```
