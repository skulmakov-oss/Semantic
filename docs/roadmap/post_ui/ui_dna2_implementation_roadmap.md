# UI-DNA2 implementation roadmap

Status: DIRECTIONAL RESEARCH ROADMAP
Repository evidence baseline: `94ae4a4ed187f589264160e794f6ebb45de1261d`
Live tracker: `#1489` (closed — see §9)

## Document role

This is a directional roadmap, not a canonical authority. It tracks
research direction, phase decomposition, and landed evidence. Canonical
ownership and forbidden-boundary decisions live in the accepted ownership
and compatibility documents; this roadmap doesn't override them, and
roadmap position alone never authorizes implementation work.

```text
roadmap direction != canonical truth
roadmap phase != automatic authorization
implementation evidence != production promotion
```

## 1. Governing formula

```text
Meaning → Projection intent → Static UI IR → Binding Graph + Action IR
  → ProjectionBundle → Patch-driven shell → Renderer
```

Ownership: Semantic owns meaning; Projection owns presentation intent; UI
IR owns structure; Shell owns local projection playback; Renderer owns
pixels.

## 2. Prerequisites (complete)

- `#1488` — UI DNA v2 / `prom-ui` reconciliation.
- Ownership decisions D01-D11 approved and preserved.
- `docs/roadmap/post_ui/ui_dna2_prom_ui_reconciliation.md` and
  `..._ownership_and_compatibility_freeze.md` landed.

## 3. Merged PRs

| PR | Merge commit | Landed |
| --- | --- | --- |
| #1490 | `5c81047e` | Projection Source AST, Role Dictionary, contract primitives, Static UI IR foundation, WP2 qualification |
| #1491 | `f5e02b4a` | Binding Graph / Action IR contract foundation, `ActionIntent` transport, WP3 qualification |
| #1492-1496 | `48b243e0`-`5bbda874` | `prom-refs` authority-free reference wrappers; exact `CapabilityRef` lookup in `prom-cap` |
| #1497 | `f28d8d37` | Projection Patch contract foundation, WP4A qualification |
| #1498 | `5f954969` | Roadmap rebaseline; Gate D closure preserved |
| #1500 | `fa6215bd` | Projection Source Grammar v0 normative contract |
| #1507-1508 | `0ac0d980`, `184103ec` | Grammar v0 parser/scanner (all 12 `PSP_*` diagnostics); parser-to-compiler frontend |
| #1513-1515 | `57a9332b`-`64bbaa63` | Binding Graph observation/dirty v0 contract, engine, and caller-supplied Semantic observation adapter — all crate-private |
| #1516-1518 | `b6c151f0`-`547e00c9` | Denial/recovery/freshness v0, Task Projection v0, P2 corrective qualification |
| #1519 | `d395e570` | ProjectionBundle v0 logical contract freeze (documentation-only) |
| #1520-1521 | `b514c51e`, `c71242d0` | Shell Player v0 ownership/stage-boundary freeze and closeout |
| #1522-1523 | `6219e67e`, `3e229a82` | Rust 1.97.1 CI baseline |
| #1524-1536 | `0eede939`-`d1797a1a` | Shell Player session/local-state contract, lifecycle evaluator, `ShellSession` owner, replay-cursor representation and compatibility contract, stage-5 stable-target boundary, prepared-handoff contract, `CollectionAnchor` declaration contract — each crate-private or documentation-only, landed incrementally |
| #1539 | `94ae4a4e` | Programmatic `CollectionAnchor` declaration qualification, 20 tests, CI 8/8 |
| #1541 | `8d29c19c` | Shell Player transition pipeline complete: `PreparedProjectionPatchTargets`/`PreparedActiveProjectionTargets`, the first public `prom-ui::shell_bridge` surface, `ActiveProjectionTargetCatalog`, stage-4/5/6 evaluation and orchestration, patch application, replay-cursor advancement, atomic commit, `--shell-player-demo`. Gate D still closed. |
| #1544 | `f9727312` | End-to-end pipeline (closes #1543): textual `collection_anchor` syntax; `ProjectionBundle v0` codec with an 8-stage verifier, golden vectors, and a negative-test matrix (trust = self-consistency, not cryptographic); Gate D **OPEN WITH LIMITS** for `--ui-dna2-reference`; `ReferenceContourAdmission` reusing the existing admission chain; glyphon text rendering; bounded hit-test/focus/accessibility; the `--ui-dna2-reference` app; and a fix to `UiBackendAdapter::run_event_loop` so real input reaches every native backend consumer (it silently didn't before, including #1541's demo). CI 16/16, verified with real OS input. |
| #1545 | `142d1ea1` | Roadmap and spec cleanup recording #1544's evidence and the UI-DNA2-11 decision; 2 P2 review findings fixed pre-merge (see §9). |

## 4. Phase status

| Phase | Status | Remaining work |
| --- | --- | --- |
| UI-DNA2-0 — Reconciliation | Complete | — |
| UI-DNA2-1 — Ownership/contract freeze | Complete | Changes need a separate owner decision |
| UI-DNA2-2 — Projection source frontend | Parser/frontend qualified, crate-private | Public API and runtime loading unauthorized |
| UI-DNA2-3 — Static UI IR | Artifact V1 qualified, crate-private | Public codec API and loaders unauthorized |
| UI-DNA2-4 — Binding Graph | Foundation + dirty-engine + observation adapter landed | Live Semantic reads/subscriptions absent — **not complete** |
| UI-DNA2-5 — Action IR integration | Contract foundation landed generally; bounded admission adapter landed via #1544 for `--ui-dna2-reference` only | General admission integration beyond that one contour remains unapproved |
| UI-DNA2-6 — Patch model/runtime | Foundation + replay-order complete | Application deferred to UI-DNA2-9 (done there) |
| UI-DNA2-7 — Denial/recovery/task/freshness | v0 + P2 qualification landed | Task Projection application, admission execution, runtime integration remain unauthorized |
| UI-DNA2-8 — ProjectionBundle | 8A contract freeze, 8B parser/validator/verifier, 8C inert loader — all landed via #1544, bounded to `--ui-dna2-reference` | Digest/signature trust algorithm unresolved; general Level 4/5 reader not claimed |
| UI-DNA2-9 — Shell Player integration | Stages 1-9 landed via #1541; Gate D activation, hit-test/focus/accessibility, text rendering landed via #1544 for the bounded contour | General admission/dispatch/UI wiring, full accessibility-tree integration, general layout beyond the bounded contour |
| UI-DNA2-10 — End-to-end reference | **Complete** (#1544) | `--ui-dna2-reference`, verified with real OS input reaching hit-test → admission → commit (screenshots for admitted + denied clicks) |
| UI-DNA2-11 — Promotion decision | **Decided** (#1544/#1545): PROMOTE WITH LIMITS | See §9 for the decision and its exclusions |

UI-DNA2-4, the general slice of UI-DNA2-5, and UI-DNA2-7's remaining work
are genuinely incomplete and were never in scope of #1543/#1544/#1545 —
that work required only what the one bounded reference application needed.

## 5. Gate D

`prom-cap` owns capability lookup; `prom-refs` is a neutral,
zero-dependency reference-value crate. The pre-#1543 Gate D0
reference/lookup subtrack (D0B-D0E) is complete and separate from Gate D
proper.

**Gate D = OPEN WITH LIMITS**, bounded strictly to `--ui-dna2-reference`.
The policy (`docs/spec/ui/gate_d_activation_policy_v0.md`) is fail-closed,
has fixed non-caller-configurable resource limits, and authorizes exactly
one function (`activate_projection_bundle_v0_gate_d`) for exactly that
contour. Gate D remains closed for everything else — general admission,
dispatch, revocation, or UI wiring.

## 6. Governance boundaries

Workbench and Semantic Studio remain governed by `#675`. `ui-shell-kit`
remains governed by `#1310` and experimental. Historical R12/Aldente
structures are evidence only unless separately revived.

```text
parser != validator != verifier != loader != activation != production promotion
lookup != authority
reference != referenced truth
```

**Review rule:** any PR that changes Gate D status or records/amends a
production-promotion decision (a UI-DNA2-11-style entry) must request at
least one reviewer before merging, even if the change is documentation-only.
#1544/#1545 self-merged this exact kind of change without one — see the
#1545-cleanup PR description for that gap; this rule exists so it doesn't
repeat.

## 7. Acceptance criteria

- [x] `#1488` reconciliation complete; ownership/compatibility frozen (D01-D11).
- [x] Projection Source AST, Static UI IR, Binding Graph, Action IR contract foundations landed.
- [x] `prom-refs` authority-free values; public API guarded; lookup non-authoritative (`prom-cap`).
- [x] Projection Patch contract + replay-order model landed; patch application landed through #1541 for the `SetBindingValue`/`SetNodeAvailability`/`CollectionInsert`/`CollectionUpdate`/`CollectionRemove`/`CollectionMove` family.
- [x] Denial/recovery/freshness v0 and Task Projection v0 landed (#1516-1518); application and runtime integration remain unauthorized.
- [x] ProjectionBundle v0: contract freeze (#1519), then parser/validator/verifier/inert-loader (#1544, bounded to `--ui-dna2-reference`); digest/signature trust unresolved.
- [x] Shell Player v0 boundary, session/lifecycle contract, and full stages 1-9 transition pipeline landed (#1520-1541); reused unmodified by #1544.
- [x] `CollectionAnchor`: programmatic representation (#1539), then textual Grammar v0 syntax and frontend integration (#1544).
- [x] Gate D **OPEN WITH LIMITS** for `--ui-dna2-reference` (#1544); general Gate D remains closed.
- [x] Real text rendering, bounded hit-test/focus/pointer-capture, and deterministic accessibility evidence landed (#1544); full OS accessibility-tree integration not implemented.
- [x] UI-DNA2-10 end-to-end reference app landed and verified with real OS input (#1544).
- [x] UI-DNA2-11 promotion decision recorded: PROMOTE WITH LIMITS (#1544/#1545, see §9).
- [ ] Action IR admission integration beyond the bounded reference contour is separately approved and qualified.

## 8. Roadmap-scope authorizations, consumed and closed

`SHELL-PLAYER-END-TO-END-TURBO` (#1541) and `UI-DNA2-END-TO-END-GATE-D-TURBO`
(#1543/#1544) were each scoped, consumed, and closed. #1541 covered the
Shell Player transition pipeline, public bridge, runtime catalog, and
native demo. #1544 covered textual `CollectionAnchor` syntax, the
ProjectionBundle codec, bounded Gate D activation, `ReferenceContourAdmission`,
native rendering, bounded hit-test/focus/accessibility, the UI-DNA2-10 app,
the UI-DNA2-11 decision, and the `run_event_loop` input-delivery fix.
Neither authorizes general admission/dispatch integration, general Gate D
movement, full accessibility-tree integration, or unrestricted production
promotion.

## 9. Definition of Done

### UI-DNA2-11 production promotion decision (#1544/#1545)

**Decision: PROMOTE WITH LIMITS**, for the `--ui-dna2-reference` contour
only — its Gate D policy, its ProjectionBundle parser/validator/verifier
(self-consistency trust, not cryptographic), its `ReferenceContourAdmission`
reuse of the existing admission evidence chain, and its native rendering/
hit-test/focus/accessibility path.

**Excluded from this decision:** general or unrestricted Gate D; any
cryptographic trust claim (no digest/signature algorithm exists in this
repository); a general Level 4/5 ProjectionBundle reader; general
admission/dispatch integration; full OS accessibility-tree (`accesskit`)
integration; general layout beyond this contour's deterministic
vertical-stack model; any critical, safety-relevant, or unrestricted
production use.

Rationale: every arrow in the pipeline is real, tested code, verified with
real OS-level input reaching hit-testing, admission, and commit — not
simulated. The verification and admission mechanisms are honest about
their own limits (self-consistency, not cryptography; one fixed
granted-action set, not general capability evaluation). That combination —
genuine end-to-end evidence plus explicitly-stated limits — is what
`PROMOTE WITH LIMITS` means here.

### Closure basis for #1489

#1543 required only that (a) Gate D not remain fully closed, (b)
UI-DNA2-10 be started and complete, and (c) the promotion outcome be
resolved — and required "the roadmap" to be reconciled, not every phase
completed. All three conditions hold. UI-DNA2-4 (live Binding Graph
reads), the general slice of UI-DNA2-5, and UI-DNA2-7's remaining runtime
integration are honestly recorded above as incomplete and out of scope —
not silently dropped — which is what §4 already states. On that basis
#1489 closed as completed via #1545; that closure did not, and does not,
claim those other phases are done.

Should that remaining work resume, track it as new, separately-scoped
work against this roadmap's current baseline.
