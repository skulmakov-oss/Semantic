# Core Trust Freeze Checklist

Status:
  DRAFT / FREEZE PLANNING CHECKLIST

Core Trust Freeze is **not** declared complete by this document.
This document maps readiness gates and blockers from the current PCC baseline.

This checklist is derived from `docs/roadmap/pcc/practical_core_matrix.md` and
the existing CTF policy lane. It keeps freeze planning separate from PCC
qualification and does not widen release claims.

## 1. Freeze Candidate Contour

The following areas are eligible for the freeze candidate contour based on the
current evidence set:

- SemCode format authority:
  - `sm-format` owns SemCode format and decode.
  - `sm-verify` owns structural admission.
  - `sm-vm` consumes verified SemCode and does not own the binary contract.
- VM trust boundary:
  - public execution remains verifier-first;
  - verified execution is the canonical route;
  - direct VM helpers remain internal or test-only unless explicitly admitted.
- Verifier path:
  - verifier consumes SemCode and runtime vocabulary, not construction-layer
    SemCode internals;
  - verified execution is the public route.
- Runtime ownership conservative contour:
  - record fields;
  - tuple indexes;
  - ADT payloads;
  - static sequence indexes;
  - dynamic sequence fallback to the parent sequence path.
- CLI / toolchain public path:
  - selected `check / compile / verify / run / run-smc` canonical fixtures are
    qualified in the PCC CLI matrix.
- 7hell:
  - the current gate passes and preserves the public qualification path.

These items are freeze-candidate only insofar as the matrix and closeout docs
provide evidence. `READY` in the PCC matrix does **not** mean Core Trust Freeze
is complete.

## 2. Freeze Blockers

The following are blockers if they are pulled into the freeze scope before
being separately qualified:

- any public claim wording that implies freeze or stability too early;
- any promoted `PARTIAL` practical-core row that is treated as `READY` without
  additional evidence;
- any raw-execution compatibility perimeter that would allow unchecked public
  execution;
- any `UNKNOWN` surface such as Logos / System / Entity / Law if it is brought
  into the freeze contour without its own evidence;
- contract or schema runtime semantics that remain `PARTIAL`;
- exports surface if treated as freeze-critical without further qualification;
- no_std qualification if the evidence base is still limited;
- any hidden dependency-boundary change that is not reflected in the trust
  policies or guard tests.

## 3. Explicitly Deferred / Non-Blocking

The following are deferred and must not block a narrow Core Trust Freeze contour
if they stay outside the freeze scope:

- `SequenceIndexDynamic`;
- symbolic dynamic sequence ownership;
- range ownership;
- iterator ownership;
- advanced alias reasoning;
- runtime dynamic-index equality;
- Logos / System / Entity / Law surface qualification, unless explicitly
  included in scope;
- full contract runtime semantics, if still `PARTIAL`;
- full language completion;
- UI / Workbench expansion.

Deferred does **not** mean ready. Deferred means outside this freeze contour.

## 4. Freeze Gate Checklist

| Gate | Required evidence | Current status | Blocker? | Notes |
| --- | --- | --- | --- | --- |
| `cargo fmt --check` | formatting clean on the current baseline | PASS | No | Current PCC baseline has passed formatting in the freeze-planning work. |
| `cargo test --workspace --all-features` | workspace stays green on the accepted PCC baseline | PASS | No | Baseline matrix and closeout docs were validated with workspace tests. |
| `7hell` gate | `powershell -ExecutionPolicy Bypass -File .\tools\7hell\run.ps1` passes | PASS | No | Public qualification gate remains green. |
| Trust boundary dependency guards | verifier-first route, explicit capability boundary, no unchecked public execution | PASS | No | Supported by CTF verifier-first and project-root trust policies. |
| SemCode format authority ownership | `sm-format` owns format/decode; verifier admits; VM executes verified bytes | PASS | No | SemCode authority split is already documented. |
| Verified execution path | public route is `SemCode -> verify -> verified VM run` | PASS | No | Direct VM helpers stay non-public or test-only. |
| Runtime ownership contour | record, tuple, ADT, static sequence, dynamic sequence fallback are qualified | PASS | No | Conservative contour is closed and documented. |
| PCC matrix consistency | status labels remain `READY` / `CONSERVATIVE` / `PARTIAL` / `DEFERRED` / `UNKNOWN` / `OUT OF SCOPE` | PASS | No | The practical core matrix is the current baseline. |
| Public API / snapshot guard, where applicable | public-surface widening is intentional and reviewed | PARTIAL | Conditional | Already relevant for some slices; not every freeze candidate needs a snapshot update. |
| No release claim widening | docs avoid claiming freeze or stability beyond evidence | PASS | No | Required for all freeze-planning docs. |
| Dirty-file isolation | pre-existing local artifacts stay uncommitted | PASS | No | Local artifact dirs remain outside the commit scope. |

## 5. Allowed Claims

Allowed claims after the current PCC work:

- Practical core has a qualified baseline.
- Runtime ownership has a conservative qualified contour.
- SemCode format authority is split into `sm-format`.
- Dynamic sequence ownership is safe but conservative.
- Some areas are `READY`, some `CONSERVATIVE`, some `PARTIAL`, some
  `DEFERRED`.

## 6. Forbidden Claims

Forbidden claims:

- Core Trust Freeze complete;
- full language complete;
- full symbolic alias precision;
- full dynamic sequence precision;
- iterator / range ownership complete;
- all contracts fully runtime-qualified;
- Logos / System / Entity / Law fully qualified;
- stable release readiness;
- full no_std qualification beyond available evidence.

## 7. Next Recommended Slices

Recommended next slices are docs/checklist-hardening only:

- `CTF-1`:
  final trust-boundary guard audit
- `CTF-2`:
  raw execution compatibility perimeter audit
- `CTF-3`:
  public claim wording audit
- `CTF-4`:
  no_std qualification audit

Do not recommend immediate language expansion as part of freeze planning.

## 8. Final Position

Core Trust Freeze is **not** complete.

The current PCC baseline is mapped into:

- freeze candidate contour;
- blockers;
- deferred / non-blocking areas;
- required gates;
- allowed claims;
- forbidden claims.

This document prepares the next gate-hardening phase without widening claims.
