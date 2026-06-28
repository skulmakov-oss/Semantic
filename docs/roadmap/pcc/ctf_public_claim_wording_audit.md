# CTF Public Claim Wording Audit

Status:
  DRAFT / AUDIT ONLY

Core Trust Freeze is **not** declared complete by this document.
This audit inventories public-facing wording so practical qualification is not
misread as freeze, stability, or release readiness.

Basis:

- [README](../../README.md)
- [Core Trust Freeze Checklist](core_trust_freeze_checklist.md)
- [PCC Practical Core Matrix](practical_core_matrix.md)
- [Raw Execution Compatibility Inventory](raw_execution_compatibility_inventory.md)
- [CTF Raw Execution Compatibility Perimeter Audit](ctf_raw_execution_compatibility_perimeter_audit.md)
- [Semantic VM Specification](../../spec/vm.md)
- [Semantic Verifier Specification](../../spec/verifier.md)
- [Semantic UI DNA](../../dna/SEMANTIC_UI_DNA.md)

## 1. Executive Summary

The repository has a strong practical baseline, but several public-facing docs
use wording that can be read more strongly than the evidence warrants if the
reader skips the status model.

Safe overall claim today:

- the project has a qualified practical core baseline;
- the canonical trusted execution route is verifier-first and token-first;
- runtime ownership is conservative where sequence dynamics remain deferred;
- Core Trust Freeze is still a planning / guard-hardened track, not a declared
  complete state.

Main public-claim risk:

- phrases like `stable`, `frozen`, `complete`, `trusted execution`, or
  `production-ready` can be read as broader promises than the audit baseline
  actually supports unless they are carefully scoped.

## 2. Public Claim Audit Matrix

| File | Claim / wording area | Current reading | Allowed? | Risk | Recommended action |
| --- | --- | --- | --- | --- | --- |
| `README.md` | `post-v1 contract-stabilized platform in controlled expansion` | Strong but still bounded by the later statement that `main` is not automatically stable | Yes, with context | MEDIUM | Keep the status-model sentence adjacent; do not let the phrase stand alone as a release promise. |
| `README.md` | `The repository main may contain work that is newer than the currently published stable line` | Honest status framing | Yes | LOW | None. |
| `README.md` | `The stable public contract lives in docs/spec/*` | Allowed if read as current contract source, not broad stable-release promise | Yes, with context | LOW | Keep tied to `docs/roadmap/public_status_model.md`. |
| `README.md` | `Runtime ownership slice remains intentionally narrow and frozen` | Safe for the currently documented narrow slice, but easy to overread as full ownership completion | Yes, with context | MEDIUM | Keep the unsupported list visible; avoid adding “complete” wording nearby. |
| `README.md` | `current main should be read as an active development line, not as a blanket stable-release promise` | Explicitly corrective | Yes | LOW | None. |
| `README.md` | `no silent stable-claim widening` | Good guard wording | Yes | LOW | None. |
| `docs/roadmap/pcc/core_trust_freeze_checklist.md` | `Core Trust Freeze is not declared complete` | Correct freeze-planning stance | Yes | LOW | None. |
| `docs/roadmap/pcc/core_trust_freeze_checklist.md` | `freeze candidate contour` / `freeze blockers` / `deferred / non-blocking` | Good planning language | Yes | LOW | None. |
| `docs/roadmap/pcc/practical_core_matrix.md` | `READY / CONSERVATIVE / PARTIAL / DEFERRED / UNKNOWN / OUT OF SCOPE` | Good classification vocabulary, but easy to flatten into “all ready” if cited selectively | Yes, with context | MEDIUM | Keep the matrix as the baseline reference and avoid cherry-picked “READY” claims outside the table. |
| `docs/roadmap/pcc/practical_core_matrix.md` | `Core Trust Freeze is not declared complete` | Correct | Yes | LOW | None. |
| `docs/roadmap/pcc/raw_execution_compatibility_inventory.md` | canonical / compatibility / raw / tooling classification | Correct trust inventory, but helper names can still mislead if quoted without class labels | Yes, with context | MEDIUM | Preserve trust-class labels alongside the helper names wherever the inventory is referenced. |
| `docs/roadmap/pcc/ctf_raw_execution_compatibility_perimeter_audit.md` | verifier-first vs raw/compatibility perimeter distinction | Correct | Yes | LOW | None. |
| `docs/spec/vm.md` | `canonical trusted execution route is verifier-first and token-first` | Correct after wording hardening | Yes | LOW | None. |
| `docs/spec/vm.md` | `verified compatibility helpers` / `lower-level helpers` | Correct, but can be misread if the qualification context is omitted | Yes, with context | MEDIUM | Keep the canonical/trusted/compatibility split explicit. |
| `docs/spec/verifier.md` | `verify_semcode_token` canonical admission boundary; `verify_semcode` compatibility / legacy surface | Correct | Yes | LOW | None. |
| `docs/architecture/adt_payload_ownership_paths.md` | `stable paths into ADTs` | Safe within the ownership-path explanation, but must not be generalized to full language readiness | Yes, with context | MEDIUM | Keep it localized to ADT payload ownership-path behavior. |
| `docs/workbench/*` | `operator surface`, `not authority`, `read-only`, `non-authoritative` | Good authority boundary wording | Yes | LOW | None. |
| `docs/workbench/*` | `stable`, `production-ready`, `release-ready` in roadmap / readiness tables | Usually guarded, but a few lines can be read as user-facing promise if quoted out of context | Mostly yes, with context | MEDIUM | Ensure roadmap tables keep their status labels and evidence references visible. |
| `docs/status/feature_maturity_matrix.md` | feature rows labeled `qualified limited release` / `not published stable` / `stable release surface` | Strong baseline vocabulary, but the file is easy to misread as blanket product release status if read selectively | Yes, with context | MEDIUM | Preserve the note that stable-release status is separate from qualified coverage. |
| `docs/NO_STD.md` and README no_std section | `no_std` / `alloc` boundaries | Honest boundary wording, but not a full workspace claim | Yes, with context | MEDIUM | Keep the narrow-scope wording; do not promote to full-workspace no_std readiness. |

## 3. Allowed Vs Forbidden Claims

| Topic | Allowed claim | Forbidden claim | Reason / evidence |
| --- | --- | --- | --- |
| Core Trust Freeze | Freeze planning / freeze candidate contour / checklist / not declared complete | Core Trust Freeze complete / frozen trusted core | The CTF docs explicitly say freeze is not complete. |
| Practical Core Matrix | qualified baseline with READY / CONSERVATIVE / PARTIAL / DEFERRED / UNKNOWN / OUT OF SCOPE | everything is ready / complete | The matrix intentionally distinguishes those statuses. |
| Runtime ownership | conservative runtime ownership contour; static indexes precise; dynamic fallback to parent seq | full symbolic alias precision; full dynamic sequence precision; range / iterator ownership complete | Sequence symbolic/range/iterator remain deferred. |
| Dynamic sequence ownership | `seq[i] -> seq` conservative fallback | precise dynamic index ownership; dynamic equality by default | The accepted CTF sequence wave chooses conservative fallback. |
| Execution route | verifier-first / token-first canonical route | raw bytes are trusted without admission | `vm` and verifier specs now distinguish canonical route from helpers. |
| Raw helpers | raw lower-level helper; compatibility surface; noncanonical execution helper | trusted execution route; canonical execution route | The inventory and wording hardening explicitly label them noncanonical. |
| Verified compatibility helpers | compatibility helper around admitted execution | primary canonical route for new trusted public paths | They remain admitted wrappers, not the preferred token-first boundary. |
| CLI public route | verifier-first public route; canonical toolchain route | byte-first trusted route | README and CLI docs position `smc check/compile/verify/run/run-smc` as verifier-first. |
| `no_std` | narrow core-library no_std smoke / core boundary | full workspace no_std qualification | README and docs call out the limited scope. |
| UI / Workbench | operator surface; tooling shell; non-authoritative | compiler truth / verifier truth / VM truth / release authority | Semantic UI DNA forbids authority transfer. |

## 4. Wording Risks By Area

### README / top-level

- `post-v1 contract-stabilized platform in controlled expansion` is acceptable,
  but it should always stay adjacent to the `main is not automatically stable`
  rule.
- `stable public contract` should remain tied to the canonical spec bundle and
  public status model, not read as “all implemented behavior is stable”.
- `frozen` language around runtime ownership is safe only when the narrow slice
  and unsupported cases stay visible.

### PCC / CTF docs

- `READY` in the practical matrix is not the same as “Core Trust Freeze
  complete”.
- `Core Trust Freeze checklist` is a planning overlay, not a release
  declaration.
- `raw / compatibility` inventory should keep trust-class labels attached to
  helper names.

### Spec docs

- `verify_semcode_token` is the canonical admission boundary.
- `verify_semcode` and `run_verified_semcode*` are compatibility or legacy
  surfaces and should not be left unqualified when discussed alongside the
  canonical route.
- `run_semcode*` must remain clearly lower-level and noncanonical.

### Workbench / UI docs

- `operator surface`, `not authority`, and `read-only` are correct.
- Any wording that sounds like “stable UI truth” or “release truth” should stay
  scoped to a roadmap view, not the core contract.

## 5. Constraints And Observations

- `README.md` is the most visible public-facing page and already contains
  multiple status guards; the main risk is selective quotation, not immediate
  falsehood.
- The strongest overclaim risk is not code behavior; it is the possibility that
  a reader sees one status phrase without the surrounding status model.
- The execution perimeter wording is now materially better after CTF-2b, but the
  README and roadmap pages still need the status model to be read together.

## 6. Follow-Up Recommendations

Recommended next slices:

- `CTF-3a`:
  apply tiny wording hardening to the README or other top-level docs if any
  sentence still reads too strongly when isolated.
- `CTF-3b`:
  align roadmap / status wording with the PCC matrix labels and the CTF
  checklist language.
- `CTF-4`:
  no_std qualification audit.
- `CTF-5`:
  UI / Workbench authority wording audit, if future doc review shows any drift.

Do not treat the presence of qualified wording as a claim of freeze or release
readiness.

## 7. Final Verdict

The repository’s public wording is broadly disciplined, but it still relies on
context to prevent overreading.

Allowed claims today:

- qualified practical core baseline;
- verifier-first canonical execution route;
- conservative runtime ownership contour;
- explicit freeze planning in progress.

Forbidden claims today:

- Core Trust Freeze complete;
- stable / production / release-ready blanket promise;
- full language completion;
- full symbolic alias precision;
- full dynamic sequence precision.

The right next step is wording hardening, not scope expansion.

