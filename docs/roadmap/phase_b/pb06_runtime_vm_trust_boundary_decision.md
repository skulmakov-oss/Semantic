# PB-06 — Runtime / VM Trust Boundary & Admission Freshness Decision

Umbrella: #1617 · Base: `a36237f950634594934e80fe12377193113f5700`
Issues: #1764, #1765, #1766, #1767, #1769, #1776, #1777
C1 `89641da8237f4fcefb50cf1958a50e4d4003aea7` / `v1.2.0` are not touched.
Out of scope: PB-07 (`prom-*`, #1778–#1787) and #1987.

Principle: absence of authority, evidence, host state or fresh provenance is
never converted into success.

## Re-adjudication on current main (`a36237f9`)

| Issue | Reproduction on base | Classification |
|---|---|---|
| #1764 | `sm-runtime-core::hello_observation_route::route_hello_observation_to_sink` performs admission, controlled-text policy, forbidden-output policy and sink dispatch; its only callers are four root test harnesses | STILL_REPRODUCED |
| #1765 | the same route accepts `HelloObservationRouteInput { admitted: true, .. }` from any caller | STILL_REPRODUCED |
| #1766 | the VM mints indices with an unchecked `self.sequence_index += 1` (panics in debug, wraps in release); `HelloObservationSinkError::NondeterministicOrder` is produced only by one test harness's own sink | STILL_REPRODUCED |
| #1767 | `sm_runtime_core_lib.txt` lists only `pub mod hello_observation_route;` / `pub mod hello_observation_sink;`, none of their items | STILL_REPRODUCED |
| #1769 | `run_verified_semcode` of a program with `GateRead` and no host returns `Ok(())` through `LegacyVmHost`'s synthetic value | STILL_REPRODUCED |
| #1776 | after a dependency's content is tampered with (pin unchanged), a second public `resolve_package_import_path` on the same thread returns `Ok`; after a declared dependency is deleted, a second `admit_package_entry_module` returns `Ok(Some(..))` | STILL_REPRODUCED |
| #1777 | `module_graph_fingerprint` is unchanged after renaming a declared-but-unimported dependency's package (admission becomes invalid) and after editing that dependency's source; `cmd_watch` neither re-runs nor resets its caches | STILL_REPRODUCED |

Prior work: SSF-06's existing test (DL-023) covers only an edit to the *root*
manifest. That manifest governs a visited module, so the edit already changes
the source fingerprint. The dependency's own manifest and its content were
never part of the watch signal.

## Answers

1. **What remains in `sm-runtime-core`?** Vocabulary only: `HelloObservationEvent`, `HelloObservationClass`, `HelloObservationSequenceIndex`, `HelloObservationSink` and `HelloObservationSinkError { Unavailable, Denied }`. `hello_observation_route` is deleted.
2. **Who owns controlled observation route semantics?** `sm-vm`. `HelloObservationRuntime::record_controlled_text_observation` already owns the forbidden-marker policy and event construction for every controlled `print`. The provisional Hello-only rule (accept exactly `"Hello, World!"`) is not moved into the VM. It survives only in a test-only harness helper (`tests/support/`), which belongs to no crate API.
3. **How is verifier admission bound to canonical observation execution?** Structurally: `sm-verify` → `VerifiedSemCode` → `require_entry` → `VerifiedEntrySemCode` → `sm-vm` (`run_verified_entry_semcode_collecting_hello_observations`). The only way to reach verified execution is to hold the verifier's token.
4. **Why is `admitted: bool` forbidden?** Any caller can write `true`. A value with no unforgeable link to verification is not evidence. It is deleted, and no replacement flag, string or token is added to runtime-core.
5. **Who owns observation sequence generation?** `sm-vm`. `HelloObservationRuntime` mints `0, 1, 2, …` per execution, in both collect and discard modes.
6. **Does the sink trait enforce order?** No. It consumes events already ordered by the VM and is not a second sequence authority. `NondeterministicOrder` is removed because no production code produced or consumed it. The one harness that checked order in its own sink now asserts it directly.
7. **What happens on sequence-index overflow?** Progression uses `checked_add`. When it would overflow, execution fails with `RuntimeError::Trap(RuntimeTrap::ArithmeticOverflow)` before any event is built, so the index never wraps to 0 and no index is emitted twice. The behavior is identical in every build profile.
8. **What does "no host" mean?** No host authority. Each host/effect operation fails as `HostAbi(AbiError { call, kind: Unavailable })`. There are no synthetic reads, no hash-derived state, no zero clock and no successful no-op writes.
9. **Which public VM entry points use the no-host bridge?** Every public `run_*` without a `_with_host` / `_with_application_host` parameter: `run_semcode*`, `run_semcode_collecting_hello_observations*`, `run_verified_semcode*`, `run_verified_entry_semcode*` (including the collecting and profile variants) and `run_verified_function_semcode_with_args*`. All of them reach one of two internal executors, and each executor constructs `UnavailableVmHost`.
10. **Which calls fail with `HostAbi(Unavailable)`?** `GateRead`, `GateWrite`, `PulseEmit`, `StateQuery`, `StateUpdate`, `EventPost` and `ClockRead`, each carrying its own `HostCallId`. The application calls (`ArgsRead` … `TimeDuration`) already failed this way on base.
11. **Are synthetic host semantics kept anywhere?** No. `LegacyVmHost` and its hash-derived state fallback are deleted. Every existing `sm-vm` unit test passes against `UnavailableVmHost`, so no test-only synthetic host is needed. Host behavior in tests goes through the explicit `_with_host` APIs.
12. **How do raw and verified routes differ?** Raw routes (`run_semcode*`) skip the verifier by design and are documented as raw/diagnostic. Verified routes require `sm-verify` evidence. Both have the same no-host behavior. Being raw does not grant host authority.
13. **What owns package admission cache lifetime?** The admission pass, not the thread. The `package_manifest` caches are valid only inside one pass.
14. **What is one admission pass?** The dynamic extent of the outermost package-aware public call on a thread. Entering at depth 0 clears both caches; nested package-aware calls inside it reuse them; leaving drops the depth. An RAII guard keeps the depth correct even on unwind.
15. **Which public operations begin a pass?** `admit_package_entry_module` and `resolve_package_import_path`, which every production path calls (bundle walks, canonical check, the LSP and CLI providers, incremental graph reads), plus the orchestrators that call them in loops. Those orchestrators open the pass first so the walk shares one pass. The orchestrators only affect performance; correctness comes from the two entry functions.
16. **What does the watch source fingerprint cover?** Unchanged `module_graph_fingerprint`: the imported module graph (sources, edges, exports) plus each visited module's governing manifest.
17. **What does the watch trust fingerprint cover?** The full declared dependency graph reachable from the root's governing manifest, imported or not: every manifest's bytes, its identity and declared edges (via the manifest), and every dependency's module-root content (relative path + bytes). Traversal uses sorted paths and canonical-path dedup, hashed with the crate's FNV-1a. Modification times are never used.
18. **How are declared-but-unimported dependencies represented?** As manifest nodes in the trust fingerprint, independent of source imports.
19. **How are pinned dependency mutations detected?** Content edits change the dependency's content hash in the trust fingerprint, so watch starts a fresh pass. In that pass the pin cache is empty and the real pin check runs again.
20. **Which behavior intentionally differs from C1?**
    - No-host effect opcodes now fail as `HostAbi(Unavailable)` instead of returning synthetic success.
    - Sequence overflow now traps instead of wrapping.
    - `hello_observation_route` and `NondeterministicOrder` are removed from the public API.
    - Repeated public package calls on one thread re-validate.
    - `smc watch` rebuilds when any declared dependency's manifest or content changes.

## Public API changes

| Change | Class |
|---|---|
| `sm_runtime_core::hello_observation_route` removed | authority removal (#1764/#1765) |
| `HelloObservationSinkError::NondeterministicOrder` removed | de-authorized dead variant (#1766) |
| `hello_observation_sink.rs` snapshot added | guard expansion (#1767) |
| `reset_*_cache` kept public; documented as unnecessary for correctness | compatibility |

## Non-goals

- No `prom-*` production change.
- No #1987 work.
- No package format, registry, lockfile or solver change.
- No new runtime error taxonomy.
- No change to the C1/v1.2.0 evidence.
