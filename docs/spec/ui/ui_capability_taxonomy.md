# UI Capability Taxonomy

Status: Draft
Track: POST-UI
Depends on:
- `host_runtime_effect_path_boundary.md`
- `ui_effect_envelope_v0.md`
Scope: UI capability policy vocabulary only
Implementation: out of scope

Related:

- `README.md`
- `../../architecture/ui_host_runtime_effect_boundary.md`
- `../../architecture/ui_full_effect_trace_ladder.md`

## 1. Purpose

`I52` fixes the UI capability vocabulary used by `UiEffectEnvelope.policy`.

`I50` defined where the UI effect travels.
`I51` defined what the UI effect envelope contains.
`I52` defines what right the envelope must carry in order to be admitted.

## 2. Core rule

```text
No UI-visible host effect may be admitted without an explicit UI capability.
```

Short form:

```text
No capability -> no UI effect.
```

## 3. Capability kinds v0

The admitted v0 UI capability set is:

- `UiWindowCreate`
- `UiWindowClose`
- `UiEventRead`
- `UiFrameBegin`
- `UiDrawSubmit`
- `UiFrameEnd`

## 4. Relationship to effect IDs

| UiEffectId | Required capability | Budget class | Minimum audit |
| --- | --- | --- | --- |
| `WindowCreate` | `UiWindowCreate` | `WindowLifecycle` | `IntentOutcome` |
| `WindowClose` | `UiWindowClose` | `WindowLifecycle` | `IntentOutcome` |
| `PollEvents` | `UiEventRead` | `EventRead` | `Trace` |
| `BeginFrame` | `UiFrameBegin` | `FrameLifecycle` | `Trace` |
| `SubmitDrawCommands` | `UiDrawSubmit` | `DrawSubmission` | `Trace` |
| `EndFrame` | `UiFrameEnd` | `FrameLifecycle` | `Trace` |

If the declared capability does not match the required capability for the
effect, the envelope is invalid or denied depending on the validation stage.

## 5. Capability table v0

| Capability | Direction | Host-visible | External input | Purpose |
| --- | --- | ---: | ---: | --- |
| `UiWindowCreate` | `HostVisibleWrite` | yes | no | create a window |
| `UiWindowClose` | `HostVisibleWrite` | yes | no | close a window |
| `UiEventRead` | `Read` | no | yes | read normalized input events |
| `UiFrameBegin` | `SessionWrite` | yes | no | begin a frame scope |
| `UiDrawSubmit` | `HostVisibleWrite` | yes | no | submit bounded draw batches |
| `UiFrameEnd` | `SessionWrite` | yes | no | end a frame scope |

## 6. Direction model

```text
UiCapabilityDirection =
  Read
  Write
  SessionWrite
  HostVisibleWrite
```

Rules:

- `Read` means the effect consumes external input.
- `Write` means the effect mutates admitted runtime state.
- `SessionWrite` means the effect mutates session-scoped UI state.
- `HostVisibleWrite` means the effect changes host-visible UI state.

## 7. Scope model

Capability scope is a scoped permission, not a global flag.

### 7.1 Session scope

```text
UiCapabilitySessionScope =
  SessionLocal
```

Rules:

- UI capabilities are valid only inside the current runtime session.
- capability must not survive process restart;
- capability must not become global;
- capability must not be shared across unrelated sessions.

### 7.2 Window scope

```text
UiCapabilityWindowScope =
  AnyWindowCreatedBySession
  SpecificWindow(window_id)
  NoWindowYet
```

| Capability | Scope |
| --- | --- |
| `UiWindowCreate` | `NoWindowYet` or session-level |
| `UiWindowClose` | `SpecificWindow(window_id)` or session-owned |
| `UiEventRead` | `SpecificWindow(window_id)` or session-owned |
| `UiFrameBegin` | `SpecificWindow(window_id)` |
| `UiDrawSubmit` | `SpecificWindow(window_id)` |
| `UiFrameEnd` | `SpecificWindow(window_id)` |

### 7.3 Frame scope

```text
UiCapabilityFrameScope =
  NoFrame
  ActiveFrame(frame_id)
```

Rules:

- `SubmitDrawCommands` requires `ActiveFrame(frame_id)`.
- `EndFrame` requires `ActiveFrame(frame_id)`.
- `BeginFrame` requires `NoFrame` for the target window.

## 8. Budget mapping

The admitted budget classes are:

- `WindowLifecycle`
- `EventRead`
- `FrameLifecycle`
- `DrawSubmission`

| Capability | Budget class | Budget reason |
| --- | --- | --- |
| `UiWindowCreate` | `WindowLifecycle` | limit window spam |
| `UiWindowClose` | `WindowLifecycle` | limit churn |
| `UiEventRead` | `EventRead` | limit event polling |
| `UiFrameBegin` | `FrameLifecycle` | limit frame spam |
| `UiDrawSubmit` | `DrawSubmission` | limit draw batch size |
| `UiFrameEnd` | `FrameLifecycle` | limit lifecycle churn |

Unbounded UI loops are forbidden:

- unbounded window create/close;
- unbounded event polling;
- unbounded draw command submission;
- unbounded frame begin/end churn.

## 9. Audit mapping

The admitted audit classes are:

- `Trace`
- `IntentOutcome`
- `Sensitive`

| Capability | Minimum audit | Reason |
| --- | --- | --- |
| `UiWindowCreate` | `IntentOutcome` | host-visible new resource |
| `UiWindowClose` | `IntentOutcome` | host-visible resource destruction |
| `UiEventRead` | `Trace` | external input boundary |
| `UiFrameBegin` | `Trace` | lifecycle trace |
| `UiDrawSubmit` | `Trace` | visible output trace |
| `UiFrameEnd` | `Trace` | present/finalize trace |

Reserved future sensitive classes:

- clipboard;
- file picker;
- drag/drop;
- raw pointer capture;
- IME/text input.

## 10. Denial model

Capability denial categories are:

- `MissingCapability`
- `CapabilityScopeMismatch`
- `CapabilityDirectionMismatch`
- `CapabilityExpired`
- `CapabilityAttenuated`
- `CapabilityNotAdmittedForEffect`

Capability denial may be represented in `UiEffectResult` as `CapabilityDenied`
with a more specific subcode.

## 11. Attenuation rules

Capabilities may become narrower, not broader.

Allowed attenuation examples:

- `AnyWindowCreatedBySession` -> `SpecificWindow(window_id)`
- `DrawSubmission` count limit `1000` -> `100`
- `EventRead` max events `64` -> `16`
- `Trace` audit -> `IntentOutcome` audit if stricter

Forbidden attenuation examples:

- `SpecificWindow(window_id)` -> `AnyWindowCreatedBySession`
- `EventRead` -> `WindowCreate`
- `Trace` audit -> `None`
- `DrawSubmission` limit `100` -> unlimited
- session-local -> global

Delegation may only reduce authority.

## 12. Delegation rules

Semantic programs cannot mint or delegate UI capabilities in v0.

Runtime may attenuate effective capability internally.

## 13. Forbidden bypasses

Forbidden bypasses include:

- VM creates UI effect without capability metadata;
- envelope declares a weaker or different capability than the effect requires;
- `prom-ui-runtime` dispatches before capability admission;
- platform adapter performs effect based on local policy;
- draw submission bypasses `UiDrawSubmit`;
- event polling bypasses `UiEventRead`;
- frame lifecycle bypasses `UiFrameBegin` / `UiFrameEnd`;
- `WindowId` from one session is reused in another session;
- raw OS handle is treated as a capability;
- debug or dev mode bypasses capability checks.

`Raw OS handle` is not a capability.
`WindowId` is not a capability.
`FrameId` is not a capability.

## 14. Reserved capabilities

The following are not admitted in v0:

- `UiClipboardRead`
- `UiClipboardWrite`
- `UiFilePickerOpen`
- `UiDragDropRead`
- `UiTextInputRead`
- `UiRawPointerCapture`
- `UiGamepadRead`
- `UiAudioOutput`
- `UiGpuDeviceCreate`
- `UiShaderCompile`
- `UiTextureUpload`

These are sensitive, platform-heavy, or renderer/backend-specific.

## 15. Extension policy

A new UI capability requires all of the following:

- effect mapping;
- budget class;
- audit class;
- scope model;
- denial behavior;
- attenuation rules;
- reserved/non-reserved classification.

No new UI capability without policy metadata.

## 16. Out of scope

This document does not add:

- Rust enum definitions;
- `prom-cap` changes;
- `prom-ui-runtime` code;
- ABI widening;
- VM changes;
- verifier changes;
- new opcodes;
- runtime enforcement implementation;
- renderer implementation;
- window backend implementation;
- widget/layout framework;
- tests beyond docs/link checks.

## 17. Acceptance checklist

- `docs/spec/ui/ui_capability_taxonomy.md` exists;
- `docs/spec/ui/README.md` links it;
- `docs/spec/index.md` links it;
- the document references `I50` and `I51`;
- capability kinds v0 are defined;
- effect-to-capability mapping is explicit;
- direction model is defined;
- scope model is defined;
- budget mapping is defined;
- audit mapping is defined;
- denial model is defined;
- attenuation rules are defined;
- program-level delegation is out of scope;
- forbidden bypasses are listed;
- reserved capabilities are listed;
- extension policy is defined;
- no code changes;
- no ABI widening;
- no VM/runtime changes.
