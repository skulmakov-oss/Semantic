# ProjectionBundle v0 Contract

Status: NORMATIVE CONTRACT FREEZE
Track: UI-DNA2-8A (logical contract), 8B/8C (bounded codec + loader
delivered under Issue #1543; see §17)
Scope: logical contract for all future stages, plus one bounded
crate-private codec/loader for the UI-DNA2-10 reference contour only
General Level 4: NOT CLAIMED
Gate D: OPEN WITH LIMITS for the bounded UI-DNA2-10 reference contour only
Production promotion: PROMOTE WITH LIMITS, bounded to that same contour
(`docs/roadmap/post_ui/ui_dna2_implementation_roadmap.md` UI-DNA2-11);
general or unrestricted promotion is not authorized

## 1. Purpose and scope

This document freezes the ProjectionBundle v0 logical contract and the
boundaries between its future processing stages. It does not define Rust
types, a stable public API, or a final wire format.

ProjectionBundle carries projection artifacts.
ProjectionBundle does not own Semantic truth.

This contract does not claim reader, parser, validator, verifier,
cryptographic verifier, loader, filesystem-loading, runtime-loading,
activation, shell-integration, production-readiness, or security-proof
evidence.

UI-DNA2-8A is documentation-only. A later implementation requires a new
owner authorization, a new harness task, and resolution of every blocking
decision named in this document.

## 2. Prior Evidence Reconciliation

Existing ProjectionBundle evidence remains useful within these exact
boundaries:

| Evidence | Classification |
| --- | --- |
| `docs/spec/ui/projection_bundle_delivery.md` | Directional delivery and trust-boundary evidence; not the final parser or wire contract. |
| `docs/spec/ui/projection_bundle_reader_parser_entry_gate.md` | Active pre-implementation gate. |
| `docs/spec/ui/projection_bundle_reader_parser_basis.md` | Reader/parser evidence basis; does not claim general Level 4. |
| `docs/roadmap/post_ui/projection_bundle_level4_evidence_matrix.md` | Promotion tracker; not a promotion claim. |
| `docs/roadmap/post_ui/projection_bundle_fixture_inventory.md` | Fixture evidence inventory; not parser qualification. |
| Historical fixture readers and draft tools | Evidence-only; not a production parser, loader, or runtime. |

UI-DNA2-8A neither discards prior evidence nor promotes that evidence into
implementation truth. Material conflicts and undecided ownership questions
remain explicit in [Unresolved Decisions Blocking Implementation](#16-unresolved-decisions-blocking-implementation).

## 3. Governing stage model

The logical processing model is:

```text
bounded input
  → framing / preflight
  → parsing
  → structural validation
  → cross-artifact validation
  → compatibility validation
  → trust verification
  → inert loading
  ─────────────────────────────
  → activation
```

The separation before activation is normative.

```text
framing != parsing
parse success != structural validity
structural validity != cross-artifact compatibility
compatibility != trust verification
trust verification != activation
inert loading != activation
activation != production promotion
```

The conceptual stage boundaries are:

| Stage | Conceptual input | Conceptual output |
| --- | --- | --- |
| Bounded input | Caller-supplied bytes and caller-supplied limits | A bounded input view or a deterministic resource rejection |
| Framing / preflight | Bounded input view | Framing facts sufficient for parsing, without allocating beyond limits |
| Parsing | Successfully framed input | An in-memory syntactic representation carrying fields and trust metadata |
| Structural validation | Parsed representation | A structurally valid logical bundle representation with every required artifact class present |
| Cross-artifact validation | Structurally valid representation | Present artifacts whose identities, references, and internal links agree |
| Compatibility validation | Cross-artifact-valid representation plus supported compatibility context | A compatibility-valid representation |
| Trust verification | Compatibility-valid representation plus caller-selected trust context | A representation with evaluated trust evidence |
| Inert loading | Successfully parsed, validated, and verified in-memory representation | An inert ProjectionBundle representation with no effects |
| Activation | Inert bundle plus separately governed admission and runtime context | Outside v0 implementation authorization |

No stage output grants authority to skip a later stage.

## 4. Authority and ownership

```text
bundle identity != authority
bundle version != compatibility proof
reference possession != referenced truth
hash transport != hash verification
signature transport != signature verification
parse success != trust
verification success != admission
verification success != activation
inert bundle != active UI
loader success != runtime application
bundle activation != production promotion
```

Ownership remains:

```text
Semantic owns meaning.
Projection owns presentation intent.
UI IR owns structure.
Binding Graph owns deterministic dependency mapping.
Action IR owns affordance routing.
Shell owns rendering behavior.
Renderer owns pixels.
Verifier / admission owns semantic admission decisions.
```

ProjectionBundle aggregates evidence and projection artifacts. It does not
transfer any owner's authority to the bundle, its producer, its parser, or
its consumer.

## 5. Logical identity contract

Every identity field uses exact equality. No identity field confers authority.
UI-DNA2-8A does not select a UUID syntax, hash algorithm, signature algorithm,
or public Rust representation.

| Logical field | Canonical bundle identity participation | Classification | Equality and authority rule |
| --- | --- | --- | --- |
| bundle identity | Yes | trust-relevant identifier | Exact equality; identity is not authority |
| bundle version | Yes | compatibility-relevant | Exact equality; version is not compatibility proof |
| projection identity | Yes | compatibility-relevant | Exact equality; projection identity is not Semantic truth |
| contract version | Yes | compatibility-relevant | Exact equality; unsupported versions fail closed |
| role dictionary identity/version | Yes | compatibility-relevant | Exact equality against the selected compatible dictionary |
| renderer profile identity | Yes | compatibility-relevant | Exact equality against the selected supported profile |
| artifact identity | Yes, for every included artifact | compatibility- and trust-relevant | Exact equality; duplicate identity is rejected |
| artifact digest | Yes when a digest is present or required by policy | trust-relevant | Exact equality after verification; transport is not verification |
| source/provenance references | Yes | informational and trust-relevant | Exact reference equality; possession is not referenced truth |
| compiler identity | Yes | informational and trust-relevant | Exact equality; identity alone is not compiler trust |
| compatibility declaration | Yes | compatibility-relevant | Exact declared values; declaration is not compatibility proof |
| safety/criticality classification | Yes | compatibility- and trust-relevant | Exact equality; classification is constrained by separately owned policy |

Canonical bundle identity is the deterministic identity of the complete
logical field set and artifact inventory defined by this contract. The
canonical encoding that will make that identity reproducible remains
unresolved.

## 6. Artifact composition

Presence policy is explicit:

| Artifact class | v0 presence | Logical relationship |
| --- | --- | --- |
| Static UI IR | Required | Carries the structural projection artifact; it remains non-authoritative |
| Binding Graph | Required | Carries deterministic dependency declarations; it does not read live Semantic state |
| Action IR | Required | Carries affordance routes; inclusion does not authorize execution or admission |
| Role Dictionary | Required | Supplies exact role identity/version compatibility context |
| Denial/recovery projection metadata | Optional | When present, must reference existing bundle identities exactly |
| Task Projection metadata | Optional | When present, remains inert and non-authoritative |
| Freshness/connectivity projection metadata | Optional | When present, carries projection evidence, not connection truth |
| Accessibility metadata | Required | Keeps non-visual interpretation and accessibility in the projection contract |
| Diagnostic/provenance metadata | Required container | Provenance entries required by policy are exact; diagnostic entries may be empty |

No artifact class may be silently substituted for another. Missing required
artifacts, duplicate artifact identities, or cross-artifact mismatches are
rejections.

Action IR inclusion does not authorize ActionIntent execution.
ProjectionBundle processing does not authorize ProjectionPatch application.

## 7. Representation policy

The following representations are separate:

- logical canonical representation: the deterministic field and artifact
  model frozen here;
- input serialization: the future external byte or text form;
- canonical encoding: the future deterministic encoding used for exact
  identity and equality;
- runtime in-memory representation: a future crate-private inert value.

Existing accepted evidence does not select a final serialization.

```text
FINAL SERIALIZATION = UNRESOLVED
PARSER IMPLEMENTATION = BLOCKED
```

UI-DNA2-8B cannot begin until an owner decision selects:

- the final serialization and framing boundary;
- the canonical encoding and exact equality rule;
- embedding versus exact-reference policy per artifact class;
- trust metadata representation;
- source-coordinate semantics supported by that serialization;
- numeric resource limits or the contract for caller-supplied values;
- compatibility-version negotiation policy.

JSON, YAML, TOML, binary, or another encoding must not be selected merely for
implementation convenience.

Under the owner authorization in Issue #1543, a concrete representation
was implemented for the bounded UI-DNA2-10 contour
(`crates/prom-ui/src/projection_bundle.rs`), the same way prior PRs
resolved analogous "unresolved" language for other artifact classes (e.g.
Static UI IR Artifact V1 in #1511). This resolves, for that contour only:

- **final serialization / canonical encoding:** length-prefixed binary
  framing (magic, schema/contract version, a fixed section table);
  little-endian fixed-width integers, length-prefixed byte strings,
  ascending-sorted collections, proven by re-encode-and-compare equality;
- **embedding:** every artifact class is embedded in-line (no
  content-addressed form);
- **trust metadata:** still **not selected** — no digest/signature field
  exists and no algorithm was chosen (§11, §16 items 4-5 remain
  genuinely unresolved);
- **source coordinates:** the existing `SourceRef`/`SourceSpan` model;
- **resource limits:** caller-supplied (`ProjectionBundleLimits`), no
  frozen defaults;
- **compatibility negotiation:** exact-equality only, no range/fallback.

This is bounded to UI-DNA2-10, not a general Level 4/5 readiness claim
(§13).

## 8. Validation policy

Future conforming stages must fail closed:

| Condition | Owning rejection class |
| --- | --- |
| missing required field | Structural rejection |
| unknown field | Syntax or structural rejection according to the selected serialization policy |
| duplicate field | Syntax or structural rejection according to the selected serialization policy |
| malformed field | Syntax rejection |
| unsupported contract version | Compatibility rejection |
| unsupported bundle version | Compatibility rejection |
| incompatible role dictionary | Compatibility rejection |
| incompatible renderer profile | Compatibility rejection |
| missing required artifact | Structural rejection |
| duplicate artifact identity | Structural rejection |
| artifact digest mismatch | Trust-verification rejection |
| cross-artifact reference mismatch | Cross-artifact rejection |
| placeholder hash or signature value | Trust-verification rejection |
| unsupported safety classification | Compatibility or trust-verification rejection according to the separately owned policy |

The rejection domains are distinct:

- syntax rejection means the selected serialization cannot produce a parsed
  representation;
- structural rejection means the parsed representation violates the logical
  shape;
- cross-artifact rejection means artifact identities or references disagree;
- compatibility rejection means declared versions, profiles, or dictionaries
  are unsupported;
- trust-verification rejection means transported trust evidence is absent,
  placeholder, inconsistent, or unverified;
- activation rejection is a future boundary and is not implemented here.

## 9. Determinism and diagnostics

A future implementation must provide:

- one stable primary diagnostic per rejected input;
- deterministic stage precedence;
- deterministic ordering for diagnostics within a stage;
- exact source-coordinate preservation where the selected serialization
  supports coordinates;
- no timestamps;
- no absolute paths;
- no host-dependent strings;
- no OS-dependent ordering;
- no nondeterministic map iteration;
- no trust inference from placeholder values.

Required precedence is:

1. input/resource preflight;
2. framing and syntax;
3. structural validation;
4. cross-artifact validation;
5. compatibility validation;
6. trust verification;
7. inert-load validation.

`docs/ERROR_CODES.md` exists and was inspected. It is the current
human-facing Semantic diagnostic catalog.

Its Maintenance section currently names `src/bin/smc.rs` as the source
catalog location. That path is stale at this repository baseline. The root
binary is a thin CLI wrapper and does not own the diagnostic registry.

The authoritative source registry is the `diagnostic_catalog` function owned
by the canonical core diagnostics module. `smc-cli` consumes that registry
through its import in `crates/smc-cli/src/lib.rs`.

The defining core module is already governed by the repository's
explicit legacy-compatibility perimeter. UI-DNA2-8A records the current
ownership boundary without propagating compatibility-only crate names into
this new UI contract.

UI-DNA2-8A does not repair the stale general maintenance instruction because
`docs/ERROR_CODES.md` is outside this PR's authorized scope.

The current human-facing and source catalogs do not define a
ProjectionBundle-specific diagnostic namespace. UI-DNA2-8A therefore
does not add, allocate, reserve, or freeze ProjectionBundle error codes.

Any future ProjectionBundle diagnostic namespace requires a separate
owner decision and an explicit collision audit against both the
then-current human-facing catalog and the authoritative source registry
before implementation.

## 10. Resource-bound model

Future implementations require caller-supplied limits for:

- maximum input bytes;
- maximum manifest fields;
- maximum string bytes;
- maximum artifact count;
- maximum artifact bytes;
- maximum total decoded bytes;
- maximum reference count;
- maximum diagnostic count;
- maximum nesting/depth where the selected serialization permits nesting;
- maximum provenance entries.

Rules:

- limits are caller supplied;
- overflow is rejection;
- input/resource preflight occurs before unbounded allocation;
- every limit failure is deterministic;
- limit failure does not partially load a bundle;
- checked arithmetic is required;
- numeric defaults are not frozen without repository evidence.

Every unresolved numeric value remains an implementation prerequisite, not an
implicit host default.

## 11. Trust-verification boundary

A future trust verifier may evaluate:

- exact artifact digests;
- a bundle digest;
- signatures and signature metadata;
- compiler-identity trust evidence;
- other explicitly trust-owned evidence selected by a separately authorized
  trust policy.

Compatibility validation, not trust verification, owns supported contract and
bundle versions, Role Dictionary identity/version compatibility, renderer
profile compatibility, and other compatibility-policy checks.

Cross-artifact validation, not trust verification, owns agreement of artifact
identities, references, and internal links.

Structural validation owns presence of required artifact classes.

A separately owned safety-classification policy must assign each check to
exactly one stage. The same condition must not be evaluated as both a
compatibility rejection and a trust-verification rejection for the same
input.

The parser transports trust metadata.
The validator checks structure.
The verifier evaluates trust evidence.
None of these stages grants Semantic admission authority.

UI-DNA2-8A does not select a digest or signature algorithm. Algorithm
ownership remains unresolved until an accepted repository contract assigns
that decision.

## 12. Inert-loader boundary

The future pure in-memory inert-loader contract is:

```text
input:
  successfully parsed, validated and verified in-memory representation

output:
  inert ProjectionBundle representation

effects:
  none

filesystem authority:
  none

runtime authority:
  none

UI mutation:
  none

activation:
  none
```

```text
inert loader != filesystem loader
inert loader != runtime loader
inert loader != activation
inert loader != shell mutation
inert loader != ProjectionPatch application
```

## 13. Claim-level separation

```text
fixture reader != production parser
parser != validator
validator != verifier
verifier != loader
loader != activation
activation != production promotion
```

UI-DNA2-8A freezes the logical contract only.
General Level 4 remains not claimed.
UI-DNA2-8B is not authorized.
UI-DNA2-8C is not authorized.

## 14. Explicit non-goals

This contract does not authorize:

- Rust implementation;
- a reader, parser, validator, verifier, or cryptographic verifier;
- a public API;
- final serialization or canonical encoding;
- a filesystem loader or runtime loader;
- shell-player integration;
- Action IR execution or admission;
- ProjectionPatch construction or application;
- UI mutation;
- Gate D transition;
- production promotion;
- a security-proof claim.

## 15. Follow-on decomposition

These contours are descriptive only:

- UI-DNA2-8B: bounded crate-private parser / validator / verifier
  qualification;
- UI-DNA2-8C: bounded pure in-memory inert-loader qualification;
- later activation contour: separately owned and separately gated.

Every future slice requires new owner authorization and a new harness task.
Roadmap order alone is not authorization.

## 16. Unresolved Decisions Blocking Implementation

This section originally blocked all implementation. Under the Issue #1543
authorization, items 1-3 and 6-9 were resolved by the bounded
implementation in §7/§17. Items 4, 5, and 10 remain genuinely unresolved —
no crypto crate exists in this workspace, and the bounded contour
implements no safety-classification policy.

| # | Item | Status |
| --- | --- | --- |
| 1 | Final serialization | Resolved for the bounded contour (§7): length-prefixed binary framing |
| 2 | Canonical encoding | Resolved for the bounded contour (§7) |
| 3 | Artifact embedding vs. exact references | Resolved: every class embedded in-line |
| 4 | Digest algorithm ownership | **Unresolved** — no algorithm selected |
| 5 | Signature algorithm ownership | **Unresolved** — no algorithm selected |
| 6 | Numeric resource defaults | Resolved: caller-supplied only, no frozen defaults |
| 7 | Compatibility-version negotiation | Resolved for the bounded contour: exact-equality only |
| 8 | Critical/pinned bundle policy | **Unresolved** — not implemented |
| 9 | Unknown/duplicate field policy | Resolved: both are Structural rejections (§8) |
| 10 | Safety-classification policy | **Unresolved** — not implemented |

For general, non-bounded implementation: final serialization, general
parser implementation, and cryptographic trust all remain unresolved/blocked.

## 17. Final status

Logical contract frozen by UI-DNA2-8A. For the bounded UI-DNA2-10
reference contour only (Issue #1543): parser, validator (structural,
cross-artifact, compatibility), and inert loader are implemented
crate-private/pure in-memory; the verifier implements deterministic
self-consistency, not cryptographic trust; activation runs through the
bounded Gate D policy; `ProjectionPatch` application is unchanged,
governed by the existing Shell Player contract.

Gate D = OPEN WITH LIMITS and production promotion = PROMOTE WITH LIMITS,
both bounded to this one contour (roadmap UI-DNA2-11). General Level 4/5
production reader/parser is not claimed. No further implementation slice
is currently authorized.
