# Semantic Hub Adapter Contract v0

Status: draft v0
Owner crate: `semantic-hub` (contract) / `semantic-hub-turbovec` (reference adapter)

This document defines what a Hub v0 tool adapter may and may not do: the
worker lifecycle it is driven through, the restricted context it receives
per invocation, the failure-containment behavior it is subject to, and the
audit/provenance obligations it carries. `vector.turbovec` (crate
`semantic-hub-turbovec`) is the only adapter registered against Hub v0
today and is used here as the concrete reference; nothing below is specific
to vector search. See `hub_api_v0.md` for the envelope, fault taxonomy, and
admission order this contract assumes.

## 1. Scope

An adapter implements the `HubTool` trait
(`semantic_hub::runtime::HubTool`) and is registered into a `Hub` via
`Hub::register_tool`. Everything past that point -- admission, dispatch,
failure containment, audit -- is driven by the Hub, not the adapter. An
adapter's only obligations are: implement `HubTool` correctly, and do not
attempt to reach outside the restricted context it is given (Section 7).

## 2. Registration

`Hub::register_tool(tool: Box<dyn HubTool>)` clones the tool's own
`descriptor()`, runs `HubToolDescriptor::validate` against it
(`hub_api_v0.md` Section 6), and rejects:

- an invalid descriptor (`RegistryError::DescriptorInvalid`)
- an exact duplicate `HubToolId` already registered with an identical
  descriptor (`RegistryError::DuplicateToolId`)
- a *different* descriptor re-using an already-registered `HubToolId`
  (`RegistryError::ConflictingDescriptor`) -- v0 has no replace-in-place or
  upgrade lifecycle; changing a tool's descriptor requires a new process

Registration is static: there is no dynamic loading and no filesystem
scanning for adapters. A `ToolRegistry` only ever contains what the host
process linked in and registered explicitly, in a `BTreeMap<HubToolId, _>`
that iterates in ascending `HubToolId` order (this is what makes `smc hub
tools` output deterministic).

## 3. Worker lifecycle

Every registered tool gets a `HubWorkerHealth` the adapter never sees or
touches directly. Nine states:

```text
Registered   just registered, has never been dispatched to
Starting     lifecycle transition in progress (entered automatically)
Ready        accepts dispatch
Busy         currently executing one invocation
Degraded     accepts dispatch, but has an outstanding failure count
Restarting   lifecycle transition back toward Ready
Quarantined  does NOT accept dispatch; terminal except for manual recovery
Disabled     does NOT accept dispatch; administratively turned off
Stopped      does NOT accept dispatch; terminal
```

`.accepts_dispatch()` is `true` only for `Ready` and `Degraded`.

Legal transition table (`can_transition_to`; anything not listed is illegal
and rejected as `IllegalWorkerTransition { from, to }`):

```text
Registered  -> Starting, Disabled
Starting    -> Ready, Degraded, Quarantined
Ready       -> Busy, Degraded, Disabled, Stopped
Busy        -> Ready, Degraded, Quarantined, Stopped
Degraded    -> Ready, Restarting, Quarantined, Disabled
Restarting  -> Ready, Degraded, Quarantined
Quarantined -> Disabled, Restarting
Disabled    -> Starting, Stopped
```

A freshly registered worker starts `Registered` and does not accept
dispatch. `Hub::dispatch` lazily starts it on its first-ever dispatch
(`Registered -> Starting -> Ready`, both transitions applied inline) rather
than requiring a separate explicit start call; every dispatch after that
goes through `mark_busy()` before `handle()` is called and `mark_ready()`
(or a supervision escalation -- Section 4) afterward. All of these
transitions are applied by the Hub via `HubWorkerHealth`; the adapter has
no access to this state machine at all.

## 4. Supervision policy

`HubSupervisionPolicy { restart_policy, max_restarts_before_quarantine, max_protocol_violations_before_quarantine }`.
`HubRestartPolicy`: `Never`, `OnCrash`, `OnTransientFailure`, `Always`,
`ManualOnly`. `.conservative_default()` = `OnCrash` / `3` / `1` -- every
registered v0 tool currently uses this default.

`report_crash()`: increments `crash_count`; escalates to `Quarantined` once
`crash_count >= max_restarts_before_quarantine` **or**
`restart_policy == Never`; otherwise returns to `Degraded`.

`report_protocol_violation()`: increments `protocol_violation_count`;
escalates to `Quarantined` once `protocol_violation_count >=
max_protocol_violations_before_quarantine`. Under the conservative default
(threshold `1`) a **single** protocol violation quarantines the worker
immediately -- deliberately stricter than crash tolerance, because a tool
that violates the wire contract is untrustworthy in a way a transient
crash is not.

Neither a tool-declared failure (`Ok(Err(HubToolError))`) nor an
output-budget rejection counts against either counter -- both return the
worker straight to `Ready`. Only a panic (Section 10) or a `validate_reply`
failure (Section 11) is treated as adapter misbehavior.

Once `Quarantined`, a worker accepts no further dispatch
(`ToolQuarantined`) until an explicit `disable()` or `restart()` call --
there is no silent auto-recovery.

## 5. State ownership

An adapter owns exactly its own private mutable state behind `&mut self`
in `HubTool::handle` -- e.g. `TurboVecAdapter` holds only a `ScopedStorage`
root path; there is no other cached mutable state, since each Hub CLI
invocation is a fresh process and the on-disk index file is the real
source of truth between invocations.

The Hub never exposes to an adapter:

- the `ToolRegistry` (no way to look up or affect another tool's
  descriptor or worker state)
- the `HubAuditTrail` (no way to read or forge audit history)
- any other tool's `Box<dyn HubTool>` instance or private state
- its own `HubWorkerHealth` for the current tool (state transitions are
  driven entirely by the Hub around the `handle()` call, never by the
  adapter)

## 6. The `HubTool` trait

```text
trait HubTool: Send {
    fn descriptor(&self) -> &HubToolDescriptor;

    fn handle(
        &mut self,
        operation_id: &HubOperationId,
        payload: &[u8],
        context: &RestrictedHubContext,
    ) -> Result<Vec<u8>, HubToolError>;

    fn validate_reply(
        &self,
        operation_id: &HubOperationId,
        payload: &[u8],
    ) -> Result<(), String> { Ok(()) }
}
```

`descriptor()` returns the same static shape validated at registration.
`handle()` is where the adapter's actual operation logic runs; a returned
`Err(HubToolError { code, message })` is wrapped as
`HubFault::ToolDeclaredFailure` by the Hub -- the adapter declares its own
operation-level failures without needing to know the Hub's fault taxonomy.
`validate_reply()` defaults to `Ok(())`; an adapter may override it to add
a structural self-check on its own reply bytes (e.g. `vector.turbovec`
checks that every reply is valid JSON) -- failure here is classified
`ProtocolViolation`, never conflated with a tool-declared operation
failure.

## 7. Restricted context

```text
struct RestrictedHubContext<'a> {
    resource_budget: &'a HubResourceBudget,
    capability_context: &'a HubCapabilitySet,
    deadline: Option<Instant>,
}
```

Exactly these three fields, both budget and capability references shared
(`&'a`, never `&'a mut`). `context.deadline_exceeded()` is a convenience
that compares `Instant::now()` against `deadline` when present.

What `RestrictedHubContext` does **not** give the adapter:

- no reference to the `ToolRegistry` or any other tool's descriptor/state
- no reference to the `HubAuditTrail`
- no way to widen its own `resource_budget` or `capability_context` --
  both are shared references to values the Hub built from the admitted
  request; there is no setter, no interior mutability, and no path back to
  the `HubRequest` that produced them
- no ambient access to Semantic Core, the verifier, the VM, or any
  `prom-*` crate (Section 15)

## 8. Immutable resource budget

The budget passed for one invocation cannot be raised by the adapter that
receives it: `resource_budget: &'a HubResourceBudget` is a shared
reference, not `&'a mut HubResourceBudget`, and `HubResourceBudget` itself
exposes no interior mutability. An adapter can read
`context.resource_budget` to size its own work (e.g. `vector.turbovec`
reads `index_item_count` and `result_count` to bound batch/result sizes
before calling into `turbovec`), but has no mechanism to request more than
admission already granted for this call. Any change to the effective
budget must happen in a **different**, separately admitted `HubRequest`.

## 9. Cancellation and timeout

`already_cancelled` is checked once, in `admit()`, before the registry is
even consulted (`hub_api_v0.md` Section 9) -- a cancelled request never
reaches an adapter at all. The in-process `Hub::invoke` currently always
constructs its `AdmissionAmbient` with `already_cancelled: false`; the
field exists in the contract for an embedder to wire a real cancellation
signal into, but no such signal is wired in v0.

`deadline: Option<Instant>` is combined with a deadline derived from the
admitted `request.resource_budget.wall_time_millis`
(`started + Duration::from_millis(wall_time_millis)`) -- the tighter of
the two wins. This combined, *effective* deadline is checked exactly once,
at the very top of `Hub::invoke`, before admission runs -- already passed
produces `DeadlineExceeded` without ever calling `admit()`. The same
effective deadline is then threaded into `RestrictedHubContext` and made
available to the adapter via `context.deadline_exceeded()`, but the Hub
itself never calls that method again once dispatch has started: there is
no mid-call preemption. This is an honest limitation, not an oversight --
Hub v0's CLI dispatch path is synchronous and single-threaded, so nothing
could preempt a running `handle()` call even if the Hub tried (doing so
would require detached background execution with shared-ownership worker
state, deferred to a future execution mode). An adapter that wants to
honor a deadline inside a long-running operation must poll
`context.deadline_exceeded()` itself between internal steps; the Hub does
not do this on the adapter's behalf. Concretely: `wall_time_millis` is
enforced at dispatch entry (a request whose budget is already effectively
exhausted is rejected before the adapter is ever called), not enforced by
stopping an in-flight native call once started.

## 10. Panic handling

`Hub::dispatch` calls the adapter through
`panic::catch_unwind(AssertUnwindSafe(|| worker.handle(...)))`.
`AssertUnwindSafe` is required because `&mut dyn HubTool` is not
`UnwindSafe` by default; the Hub accepts this because a caught panic's
adapter-internal invariants after the fact are the adapter's own
responsibility, not the Hub's.

On a caught panic, the payload is converted to a message by
`panic_message`: downcast to `&str` first, then `String`, else a fixed
fallback string (`"worker panicked with a non-string payload"`) -- this
covers the two payload shapes Rust's `panic!`/`assert!` machinery actually
produces, plus a safe default for anything else. The resulting
`HubFault::WorkerPanicked(message)` is reported as `HubReplyStatus::Crashed`,
and `HubWorkerHealth::report_crash()` is called (Section 4) -- the panic is
contained inside the Hub process, never propagated as a process crash, and
the worker either degrades or quarantines according to supervision policy.
A second, unrelated invocation on the same tool after a caught panic is
expected to still complete normally (pinned by
`worker_panic_is_contained_and_reported_as_crashed`).

## 11. Protocol violations

A protocol violation is a malformed **reply shape**, never a declared
operation failure. It is raised only by `validate_reply()` returning
`Err`, after `handle()` itself returned `Ok(bytes)` and after the
output-bytes budget check has already passed (Section 12).
`Hub::dispatch` maps this to `HubFault::ProtocolViolation(reason)`
(reported as `HubReplyStatus::ToolFailed`), and calls
`HubWorkerHealth::report_protocol_violation()` -- under the conservative
default policy this quarantines the worker on the very first occurrence
(Section 4), visible to the *next* invocation, not just the current one.

## 12. Output validation

On `handle()` returning `Ok(bytes)`, `Hub::dispatch` runs two checks in
this fixed order:

1. `bytes.len() as u64 > context.resource_budget.output_bytes` ->
   `HubFault::OutputRejected`, worker returns to `Ready` (not a health
   penalty -- an oversized reply is treated as bad output, not adapter
   misbehavior)
2. `worker.validate_reply(operation_id, &bytes)` -> on `Err`, classified
   `ProtocolViolation` (Section 11)

The output-bytes budget check always runs **before** `validate_reply`: a
reply that is both oversized and structurally invalid is reported as
`OutputRejected`, not `ProtocolViolation`.

## 13. Audit and provenance responsibilities

An adapter's descriptor supplies the provenance facts the Hub cannot
determine on its own:

- `tool_version: HubToolVersion` -- the adapter's own semantic version
- `adapter_provenance: String` -- free-text identity of the adapter and
  its own pinned dependencies (e.g. `vector.turbovec`'s
  `"semantic-hub-turbovec {version}; turbovec {version} (MIT, {source url})"`)
- `determinism: HubDeterminismClass`, declared **per operation** on
  `HubOperationDescriptor`, from real qualification evidence -- never
  inferred from the operation's name

The Hub, not the adapter, always computes:

- `input_digest` / `output_digest` (`HubDigest::of`, FNV-1a-64 plus byte
  length) over the exact request payload and exact reply bytes
- the full `HubAuditRecord` (all 20 canonical fields -- see
  `hub_api_v0.md` Section 10) and its append to the `HubAuditTrail`

This happens in `record_and_reply`, unconditionally, for every outcome
(success, rejection, tool failure, crash) -- an adapter cannot suppress,
alter, or bypass its own audit record; it never sees the record at all.

## 14. Future execution-mode compatibility

`HubTool` and `HubExecutionMode` are mode-neutral by construction:
`HubExecutionMode::Subprocess`, `::Wasm`, and `::Remote` already exist as
descriptor values precisely so that adding subprocess or WASM execution
later does not require changing the trait, the envelope, or the fault
taxonomy. Nothing in the current contract assumes in-process execution is
permanent. That said, only `InProcess` has an actual dispatch
implementation in Hub v0 -- `Hub::dispatch` calls `worker.handle()`
in-process, in the same thread, with no subprocess or WASM host bridge
existing yet. A descriptor declaring `Subprocess`, `Wasm`, or `Remote`
today would validate and register, but the Hub has no runtime code path
that executes it any differently from `InProcess`.

## 15. Forbidden adapter behavior

An adapter must not:

- access Semantic Core, the verifier, the VM, or any `prom-*` crate
  internals -- this is structurally impossible for a well-formed adapter,
  since `semantic-hub`'s own `Cargo.toml` has an empty `[dependencies]`
  table (no path dependency on any other in-repo crate) and an adapter
  crate such as `semantic-hub-turbovec` only depends on `semantic-hub`
  plus its own external libraries (`turbovec`, `serde`, `serde_json`)
- grant itself a capability it was not admitted with -- `capability_context`
  is a shared reference built by the Hub from the admitted request
  (Section 7); there is no setter
- increase its own resource budget -- `resource_budget` is likewise a
  shared reference with no interior mutability (Section 8)
- mutate Semantic project source or state -- `SemanticStateMutation` and
  `ProjectMutation` are both sensitive capabilities (`hub_api_v0.md`
  Section 4) and can never be satisfied by admission, regardless of what a
  request declares

These are not conventions an adapter author must remember to follow; each
one is enforced by the type shapes and the deny-by-default capability
model described above, not by adapter-side discipline.
