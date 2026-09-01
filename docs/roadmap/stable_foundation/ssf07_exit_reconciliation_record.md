# SSF-07 Exit / Reconciliation Record

Status: SSF-07 formal exit evidence
Normative contract: `docs/spec/foundation_source_profile_v1.md`
Dependency map: `docs/roadmap/stable_foundation/stable_foundation_dependency_map.md`
Umbrella: #1578 (SSF-07), findings filed under #1617 (18-module Phase-A audit)

This is the durable record for why SSF-07 (#1578, "Type and abstraction
closure") is accepted as exited into SSF-08. It exists because #1578 was
closed once, prematurely, on 2026-08-30 — directly contradicting its own most
recent comment ("SSF-07 remains open; this does not close #1578") — while
`stable_foundation_dependency_map.md` and `.harness/current.task.yaml` both
continued to say SSF-07 was **Active**. That closure was reverted (#1578
reopened) once the contradiction was found; this record, plus the dependency
map and harness phase switch landing in the same PR, is what makes the exit
real rather than a second premature closure.

## What SSF-07 owned

Per the dependency map: numeric/text/collections/closures/generics/traits/
pattern bounds — "ordinary programs without undocumented workarounds." SSF-07
does not own every FA-02-prefixed finding; `FA-02` is a module identifier
(the sm-front frontend audit module from #1617), not an SSF phase number. A
finding filed under `FA-02` can still belong to an earlier phase (SSF-01,
SSF-02, SSF-03) or a later one (SSF-08, SSF-12) if that is where its true
acceptance criteria live. Treating every `FA-02` finding as automatically
"SSF-07's problem" is exactly the drift this record exists to avoid.

## Residual repairs closed under this reconciliation

Four findings were identified as SSF-07's own residual, DIRECT gaps and
repaired:

| Finding | Issue | PR | Merge commit | Summary |
|---|---|---|---|---|
| FA-02-001 | #1633 | #1874 | `6959195a` | Non-function trait bounds were parsed then silently discarded; `parse_type_params` now rejects a bound on a non-function type parameter at parse time. |
| FA-02-007 | #1639 | #1875 | `39989470` | Empty `match` statement wildcard (`_ => {}`) was conflated with an absent wildcard; `Stmt::Match.default` is now `Option<Vec<StmtId>>`, mirroring `MatchExpr.default`. |
| FA-02-014 | #1646 | #1876 | `5fe9bdb1` | `Self` outside a trait/impl silently resolved to `Type::Record("Self")`; `parse_type`'s `Self` branch now rejects deterministically outside `self_type_scope`. |
| FA-02-038 | #1861 | #1877 | `437ea872` | `ensure_storage_type_supported` ended in a fail-open `_ => Ok(())` and was never called at all for record/ADT field storage; rewritten exhaustive (no wildcard arm) and wired into both declaration-validation sites, with end-to-end proof (typecheck + IR-opcode + VM-execution) for every admitted composite in both aggregate positions. |

## Lifecycle correction (not a repair)

| Finding | Issue | Resolution |
|---|---|---|
| FA-04-011 | #1717 | Never closed after its repair landed. Already resolved by merged PR #1873 (`17d4b7dc`): `ensure_function_is_ir_concrete` deterministically rejects every generic function declaration at the IR boundary instead of silently erasing `type_params` for unreferenced type parameters. `docs/roadmap/language_maturity/generics_full_scope.md` was corrected by that PR; IR monomorphisation itself remains honestly classified as Roadmap. Closed here as housekeeping — no new code. |

## Classification of remaining open `FA-02` findings

SSF-07's own module (`FA-02`) has 20 open findings as of this record. None of
them are DIRECT SSF-07 blockers; each is classified below using the
DIRECT / CROSS / RETURN / DEFER scheme (DIRECT = this SSF's own exit gate;
CROSS = lives elsewhere but this SSF can't close without it; RETURN = truly
belongs to a different phase, handed back rather than fixed under the wrong
name; DEFER = explicitly outside the current Stable Foundation contour,
tracked, not blocking).

**RETURN — SSF-01 (`#1572`, language contract, already Completed):**
- #1636 (FA-02-004) — duplicate function parameter names accepted, later
  collapse in `ScopeEnv`.
- #1652 (FA-02-020) — untyped `let _ = expr` can skip RHS typechecking.
- #1654 (FA-02-022) — `qtruth_*` user functions admitted then replaced by
  intrinsic lowering.
- #1721 (FA-04-015, sm-ir module but same owning phase) — sm-ir bare-name
  intrinsic lowering can override an admitted user function before `FnTable`
  resolution.

**RETURN — SSF-02 (`#1573`, Rust-like/Logos coherence, already Completed:
Model B):**
- #1644 (FA-02-012) — duplicate Logos `System` declarations silently use
  last-write-wins semantics.
- #1645 (FA-02-013) — legacy Logos `Import` declarations are accepted then
  discarded.

Both are Logos-syntax-specific, not canonical Rust-like grammar. Model B
(decided and frozen at SSF-02, see `rustlike_logos_coherence_decision.md`)
already treats Logos as a separate, explicitly non-canonical, experimental
profile from the language SSF-07 qualifies — these two findings are gaps in
that experimental profile's own declaration handling, not in the
Rust-like contract SSF-07 owns.

**RETURN — SSF-03 (`#1574`, standard library, already Completed):**
- #1655 (FA-02-023) — language helper names are special-cased before
  `FnTable` lookup without a matching declaration reservation.

**CROSS / RETURN — SSF-08 (`#1579`, ownership and memory positioning, next
active phase):**
- #1656–#1664 (FA-02-024 through FA-02-032, nine findings) — the ownership-
  state cluster: `if`/`loop`/`match` statement and expression forms that
  discard or fail to propagate ownership state changes, plus `ScopeEnv`
  ownership APIs failing open on a missing binding. All nine share one root
  cause (`ScopeEnv` clone / local state transition never joined back) and
  belong to SSF-08's own acceptance gate ("Ownership Position A/B, value
  paths, frames, host ownership, quotas"). SSF-08's own governing document
  requires a Position A vs. B architecture decision before touching any
  individual finding in this cluster — that decision is SSF-08's first step,
  not something this record or SSF-07 can or should make.

**RETURN — SSF-12 (`#1583`, qualification and promotion verdict):**
- #1666 (FA-02-034) — sm-front's documented `alloc`/`no_std` build contract
  is not currently satisfied. Conditional: a blocker only if `no_std` stays
  in the supported target matrix by the time SSF-12 runs; if Stable
  Foundation formally excludes it, this narrows the contract instead.

**DEFER — explicitly outside the current contour ("Phase B"), not blocking
any SSF phase's closure:**
- #1640, #1641, #1642, #1643 (FA-02-008 through FA-02-011) — `qvec`
  dimension/delimiter parsing gaps. `QVec` is documented reserved and
  not-yet-promoted-to-executable throughout the current Foundation source
  profile (see e.g. `ensure_storage_type_supported`'s own `QVec` arm,
  #1861); these are real gaps but sit inside a surface no phase currently
  depends on being complete.

This accounts for all 20 currently-open `FA-02` findings plus the one
same-owning-phase `FA-04` cross-finding (#1721) already named above. Other
FA modules (FA-03 Logos-semantic adapter, FA-09 frame/host boundary, and so
on) are out of `FA-02`'s — and therefore SSF-07's — scope entirely; their
findings are tracked against their own owning phases and are not re-audited
by this record.

## Acceptance

With the four DIRECT residuals repaired, the one lifecycle correction closed,
and every other open `FA-02` finding classified as belonging to a different
phase (RETURN) or explicitly out of contour (DEFER) rather than silently
absorbed or silently dropped, SSF-07's exit gate — "ordinary programs without
undocumented workarounds," bounded to the numeric/text/collections/closures/
generics/traits/pattern abstractions the frozen Stable Foundation target
already selected — is met. SSF-07 is accepted as **Completed**; SSF-08
becomes **Active**. See `stable_foundation_dependency_map.md` and
`.harness/current.task.yaml` for the phase-state change this record
accompanies.
