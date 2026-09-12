# SSF-09 Diagnostic Authority Decision

## Status

**ARCHITECTURE / CONTRACT DECISION ONLY. NO PRODUCTION BEHAVIOR CHANGE.**

Baseline SHA: `4fa3331452c6730ab380c00b5c9ead35e1433ad2` (`main`, confirmed
via `git fetch origin && git rev-parse origin/main` immediately before
this checkpoint began; unmoved throughout).
Umbrella: SSF-09 / `#1580`, confirmed OPEN.
Precondition: `.harness/current.task.yaml` confirmed `active_phase:
SSF-09` / `issue: 1580` (activated by the SSF09-E0 governance transition,
merged `#1917`).

This document freezes ONE canonical diagnostic authority - who owns
source-surface selection, diagnostic identity, file identity, and source
range - before any individual diagnostic defect (`#1670`, `#1697`,
`#1698`, `#1699`, `#1704`/`E0243`) is repaired, and before any canonical
carrier, versioned schema, or LSP code is written. It does not implement
any of that. It does not touch `#1580`'s lifecycle.

## Scope

In scope: the four authority questions, the canonical-carrier boundary
(internal model vs. external schema), the verifier/runtime source-mapping
law, the dependency-owner analysis, and the exact future repair
sequencing. Out of scope, explicitly forbidden by this checkpoint: any
Rust change, any test-behavior change, any repair of `#1670`/`#1697`/
`#1698`/`#1699`, any carrier implementation, any JSON schema
implementation, any LSP/editor code, any issue-lifecycle transition, any
`.harness/current.task.yaml` scope widening.

## Evidence inventory

All claims below trace to direct reads of this exact baseline SHA, plus
the SSF09-E0-predecessor entry reconnaissance (four independent read-only
research passes: diagnostic surface + dataflow, schema/span/formatter
reality, editor/LSP surface + authority map, public-claim + issue
reconstruction), re-confirmed at specific points where this decision
needed more precision than that reconnaissance's own summary level.

## Decision A - Source Surface Authority

**CORRECTION NOTE (owner review round)**: the first version of this
section conflated two distinct types - it claimed `ParserProfile` itself
carries an explicit RustLike/Logos/Auto selection and that
`check_source_with_profile` is the sole dispatch point globally. Fresh
re-reading of the actual code shows both claims are wrong. This section
is rewritten below from direct evidence.

**CURRENT STATE - two distinct types, not one**:

- `sm_front::CompileProfile` (`crates/sm-front/src/lib.rs:1629-1634`) is
  the ONLY explicit surface-selector type in the codebase:
  `enum CompileProfile { Auto, RustLike, Logos }`. `smc`'s `compile`,
  `dump-ir`, `hash-ir`, and `hash-smc` subcommands parse a caller-facing
  `--profile auto|rust|logos` flag directly into this type
  (`crates/smc-cli/src/app.rs:2588-2590`).
- `sm_profile::ParserProfile` (`crates/sm-profile/src/lib.rs:119-125`) is
  a completely separate type with **no** surface-selection field at all:
  `identity: String, version: ProfileVersion, abi: AbiProfile,
  compatibility: CompatibilityMode, features: FeaturePolicy,
  capabilities: CapabilityExpectations, aliases: BTreeMap<String,
  String>`. It carries admission/policy configuration (which language
  features and capabilities are permitted, ABI/compatibility mode,
  alias normalization) that applies **identically regardless of which
  surface was selected**. `docs/architecture/dependency_boundary_rules.md`
  already lists `` `ParserProfile` outside `sm-profile` is architectural
  debt `` as a standing "Immediate debt marker" - independent
  corroboration that `ParserProfile` is not meant to carry
  surface-selection responsibility.

**CURRENT STATE - three independent dispatch paths, no single authority**:

1. `smc check` (the command `#1670` was filed against) exposes **no**
   `--profile` flag at all (`crates/smc-cli/src/app.rs:460-465`'s usage
   string has no `--profile`) - `CompileProfile` never enters this path.
   `cmd_check` first calls `check_file_with_provider_and_profile`
   (`crates/sm-sema/src/std_adapters.rs:134-171`), a **strict,
   Logos-only** multi-module project loader
   (`load_module_recursive` calls `parse_logos_program_with_profile`
   with a real `Err`/`E0239` on any non-Logos or invalid module, no
   fallback, `std_adapters.rs:264-272`). On **any** error from that
   whole multi-module load - including a real cross-module import
   error unrelated to grammar choice - `cmd_check` discards it via
   `.or_else(|_| check_source_with_profile(&src, &parser_profile))`
   (`app.rs:589`) and retries as a **single-file**, non-project check.
2. `check_source_with_profile` (`std_adapters.rs:92-101`) is that
   single-file fallback. It takes only a `&ParserProfile`, no
   `CompileProfile`, and does its own implicit probe:
   `if let Ok(logos) = parse_logos_program_with_profile(...) { .. }`
   with no `else` - a genuine Logos parse/policy **failure** is silently
   discarded (not just an empty-but-successful parse) and execution
   falls through to the RustLike grammar on the same input regardless
   (`#1670`, FA-03-001). This is also the exact function where the
   RustLike-path position loss (`#1698`) is hardcoded
   (`SourceMark::default()`), which is why the repair DAG (§9) still
   sequences `#1670` immediately before `#1698`/`#1699`. `#1670`'s own
   filed text names a **second**, distinct defect in this same
   discriminator: "an import-only Logos parse can also fall through
   because the surface discriminator only checks System/Entity/Law
   presence." Confirmed directly against the parser
   (`crates/sm-front/src/parser.rs:2954-3005`,
   `parse_logos_program`): an `import`/`pulse`/`profile` directive is
   recognized as valid Logos syntax (it requires legacy-compatibility
   mode via `require_legacy_compatibility` and is consumed without
   error) but its content is discarded - the loop skips to the next
   newline without populating `out.system`/`out.entities`/`out.laws`.
   A source file containing **only** import directives therefore parses
   as `Ok(LogosProgram { system: None, entities: vec![], laws: vec![] })`
   - a **successful** Logos parse that the `system.is_some() ||
   !entities.is_empty() || !laws.is_empty()` discriminator cannot
   distinguish from a genuinely non-Logos empty result, so it is
   misclassified as "not really Logos" and falls through to RustLike.
   This is a distinct failure mode from the swallowed-`Err` case above:
   the parse here does not fail at all, it succeeds and is
   under-recognized.
3. `smc compile`/`dump-ir`/`hash-ir`/`hash-smc` route through
   `sm_ir::legacy_lowering::compile_program_to_ir_with_options_and_profile`
   (`crates/sm-ir/src/legacy_lowering.rs:1176-1220`), which **does**
   take an explicit `profile: CompileProfile` alongside
   `parser_profile: &ParserProfile`. **This function probes Logos
   unconditionally, for every profile value, including explicit
   `RustLike`** - `let logos_detected = parse_logos_program_with_profile(input,
   parser_profile).map(...).unwrap_or(false);` (`legacy_lowering.rs:1201-1203`)
   runs before any branch on `profile`; the preceding `match profile { .. }`
   only early-returns on disabled compile-time features, not on which
   surface was explicitly requested. Two separate defects follow from
   this one call site:
   - A genuine Logos parse/policy **failure** collapses to `false` via
     `unwrap_or`, identical in shape to path 2's swallow. For `Auto`,
     this means execution falls through to RustLike lowering on the
     same input without surfacing the real Logos failure - a second,
     independent instance of path 2's defect class, in a different
     function, not covered by `#1670`'s filed scope.
   - For explicit `RustLike`, the Logos parser is invoked at all,
     unconditionally, before `profile` is ever consulted for this
     purpose. `logos_detected`'s *value* happens not to change
     `RustLike`'s control flow today (the two `if` branches that consult
     it below only match `Logos` or `Auto`), but the *invocation itself*
     - a real attempt to parse the input as Logos, with any resulting
     error silently discarded - already happens under an explicit
     `RustLike` request. This is a distinct, present-tense violation of
     the "explicit RustLike MUST NOT silently probe Logos" invariant
     below, independent of whether the probe's result currently drives
     any decision.

No single function, and no single type, is the source-surface authority
today. `CompileProfile` is the only type that is ever an *explicit,
caller-facing* selector, but the one command `#1670` was filed against
(`smc check`) does not expose it, and the one path that does expose it
(`compile`/IR path) still contains its own unaddressed instance of the
same swallow-and-fall-through defect.

**DECISION**: `sm_front::CompileProfile` (`Auto`/`RustLike`/`Logos`)
becomes the canonical source-surface selector for **every** entry point,
including `smc check`, which has none today - this is a forward
decision about what should become authoritative, not a description of
an already-unified reality. `ParserProfile` remains a separate,
surface-independent admission/policy configuration, applied identically
under whichever surface `CompileProfile` selects; it must never be
extended with, or treated as carrying, surface-selection responsibility.

**Surface-classification law (three outcomes, not two)**: a two-state
"empty-but-valid vs. policy-invalid" split is insufficient - it is
exactly what lets the import-only-Logos case above stay misclassified,
because an import-only parse is neither an error nor genuinely empty of
Logos content. Any future classifier (wherever `Auto` or an implicit
probe is implemented) MUST distinguish three outcomes, semantically, not
just as an implementation detail:

**CORRECTION NOTE (owner review round 4)**: the first version of this
law defined AUTHORITATIVE FAILURE as triggering once a surface "has
already been established **or is being classified**" and parsing/policy
rejects it. That wording is dangerously broad: it can be read as making
*any* candidate-probe rejection authoritative merely because a probe was
attempted, which would make an ordinary `RustLike` program's Logos probe
- e.g. `fn main() { return; }`, which fails immediately at the first
token with `parse_logos_program`'s generic "expected Logos declaration"
error, having established zero positive Logos evidence - block `RustLike`
from ever being tried. That is a *worse* regression than `#1670` itself,
not a fix for it. The law is also inconsistent with this document's own
"Fail-closed rules" (below), which already forbids turning ambiguous
input into a guessed source surface - the prior law's "NO SURFACE MATCH"
outcome listed "syntactically ambiguous under every profile-eligible
grammar" as an example that permits trying another surface, which is
exactly that forbidden guess. Both are corrected below by requiring
**positive evidence before a rejection can be authoritative**, and by
separating ambiguity into its own outcome instead of folding it into "no
match, try the next one."

- **NO SURFACE CLAIM** - no candidate surface has yet produced sufficient
  surface-specific *positive* evidence of ownership for this input. A
  candidate probe failing with **no prior positive evidence** for that
  surface is NOT automatically authoritative merely because the probe
  was attempted - `fn main() { return; }` failing a Logos probe at the
  first token, having matched none of `system`/`entity`/`law`/`import`/
  `pulse`/`profile`, is exactly this case. Another candidate surface MAY
  be evaluated.
- **UNIQUE POSITIVE SURFACE CLAIM** - **CORRECTION NOTE (owner review
  round 5)**: naming this outcome plain "POSITIVE SURFACE CLAIM" left a
  loophole - it could be read as licensing a classifier that stops at
  the first candidate producing positive evidence without ever checking
  whether a second candidate also would, which would make the
  AMBIGUOUS/CONFLICTING CLAIMS outcome below unreachable by
  construction ("first positive wins" can never observe a conflict it
  never looked for). Renamed to make explicit that reaching this outcome
  requires the evidence to be *unique* among candidates, not merely
  present in one - a candidate surface has produced sufficient
  surface-specific evidence to establish ownership of this input, **and
  no other candidate surface independently produces such evidence for
  the same input** (that joint condition is exactly what separates this
  outcome from AMBIGUOUS/CONFLICTING CLAIMS below; a classifier
  satisfies this law only if it can actually detect the conflicting
  case, however it chooses to do so - short-circuiting on the first hit
  without any conflict check does not). For Logos, this MUST include
  every supported Logos-only declaration form, including import-only
  Logos source - a successful parse that only consumed `import`/`pulse`/
  `profile` directives is a positive Logos claim, not an empty result,
  even though today's `system`/`entities`/`laws`-only discriminator
  cannot see it. Reaching this outcome ends classification: no other
  surface may be attempted, and precisely two sub-outcomes follow -
  **success** (parsing/policy subsequently accepts the input under the
  owning surface), or **AUTHORITATIVE FAILURE** (parsing/policy
  subsequently rejects it under the owning surface; that failure is
  preserved as the outcome for this input, and it alone - never a bare
  probe rejection with no prior positive evidence - is what the
  swallowed-`Err` half of `#1670`, and path 3's `unwrap_or(false)`,
  currently destroy).
- **AMBIGUOUS / CONFLICTING CLAIMS** - more than one candidate surface
  independently produces sufficient positive evidence for the same
  input, or the classifier cannot deterministically resolve which
  surface owns it. This outcome must remain genuinely reachable by the
  classifier, not defined away by an implementation that stops
  evaluating after the first candidate satisfies UNIQUE POSITIVE
  SURFACE CLAIM's evidence threshold without checking uniqueness. This
  is itself a deterministic classification error, reported as such -
  **never** resolved by picking a "winner" through evaluation order,
  fallback, or any other implicit tie-break. Folding ambiguity into "no
  match, try the next candidate" (an earlier draft's error) is precisely
  the guessed-source-surface outcome the "Fail-closed
  rules" section below already forbids.

This decision freezes the semantic distinction only, not the concrete
classifier implementation (e.g., whether it is expressed as an enum,
which exact declaration forms count toward a positive claim beyond
`system`/`entities`/`laws`/imports, or how it composes with policy
checks) - that is implementation work, out of scope here.

**INVARIANT** (fail-closed law, applies to every entry point and every
current implementation of "Auto" uniformly - paths 1-3 above each
violate at least one clause of it today, and path 3 alone violates two
different clauses at once):

- Explicit `RustLike`: MUST NOT silently probe Logos. **Currently
  violated by path 3**: `compile_program_to_ir_with_options_and_profile`
  invokes `parse_logos_program_with_profile` unconditionally, including
  when the caller explicitly requested `RustLike`, before `profile` is
  consulted for this purpose at all. This holds regardless of whether
  the probe's result currently changes `RustLike`'s outcome - the
  invariant is about not performing the probe under an explicit
  single-grammar request, not merely about not acting on it.
- Explicit `Logos`: MUST NOT silently probe RustLike.
- `Auto`: may perform canonical surface classification using the law
  below (NO SURFACE CLAIM / UNIQUE POSITIVE SURFACE CLAIM (success or
  AUTHORITATIVE FAILURE) / AMBIGUOUS-CONFLICTING CLAIMS), but only a
  **NO SURFACE CLAIM** outcome permits evaluating another surface; a
  **UNIQUE POSITIVE SURFACE CLAIM** ends classification (its
  AUTHORITATIVE FAILURE sub-outcome MUST NOT be discarded in favor of
  another grammar); and **AMBIGUOUS/CONFLICTING CLAIMS** MUST NOT be
  resolved by picking a winner via fallback or evaluation order - it is
  reported as a classification error, and the classifier must be capable
  of reaching this outcome (see the correction under "UNIQUE POSITIVE
  SURFACE CLAIM" above - a first-match-wins implementation that never
  checks for a second candidate cannot satisfy this). This applies
  equally to a whole-project multi-module load failure (path 1's
  `.or_else`) and to a single-file grammar-probe failure (paths 2 and 3's
  `unwrap_or`/`if let Ok`) - "try a different mechanism after a real
  failure" is the same defect whether the discarded failure came from a
  directory walk or a single parse call, **provided that failure
  followed positive evidence** - a bare probe rejection with no prior
  positive evidence is NO SURFACE CLAIM, not an authoritative failure,
  and trying another candidate after it is correct, not a violation.
  **Currently violated by path 3's `Auto` branch** via the same
  `unwrap_or(false)` call site named above, and **by paths 2 and 3's
  shared discriminator** which cannot distinguish a UNIQUE POSITIVE
  SURFACE CLAIM (import-only Logos) from NO SURFACE CLAIM (genuinely
  empty input) - see path 2's evidence above and `#1670`'s own filed
  text.

**WHY**: Fixing `#1670` alone, in `check_source_with_profile` alone,
without freezing this invariant first, would leave the *general* rule
undefined for path 1's project-level fallback and path 3's independent
swallow - both of which this evidence pass found are real, present, and
currently un-filed. Freezing "first classified failure is authoritative,
no silent grammar-hopping, no silent mechanism-hopping" as one law
covering all three paths is what lets `#1704`'s catalog and any future
entry point be judged against the same standard, instead of accumulating
a fourth ad hoc partial fix.

**REJECTED ALTERNATIVES**:
- *Try every profile/mechanism and pick the "best" result heuristically*
  - rejected: reintroduces exactly the guessed-success-over-real-failure
  pattern the invariant forbids, and "best" has no principled definition
  (multiple grammars/mechanisms can independently "succeed" on
  adversarial input).
- *Make RustLike the sole grammar, deprecate Logos auto-detection
  entirely* - rejected: out of scope for a diagnostic-authority decision;
  this is a language-surface decision belonging to SSF-01/02 (already
  closed, Model B), not SSF-09.
- *Treat `ParserProfile` as the surface selector by adding
  Auto/RustLike/Logos fields to it* - rejected: `CompileProfile` already
  exists and is already the caller-facing selector on three of `smc`'s
  five relevant subcommands; duplicating that responsibility into
  `ParserProfile` would create two competing selector types and directly
  contradicts the standing debt marker that `ParserProfile` outside
  `sm-profile` is already architectural debt.

**IMPLEMENTATION CONSEQUENCE** (not performed here): `smc check` needs an
explicit or defaulted `CompileProfile` wired into its call path (it has
none today); `check_source_with_profile` must classify the Logos
attempt's outcome using the law above (NO SURFACE CLAIM / POSITIVE
SURFACE CLAIM-with-success-or-AUTHORITATIVE-FAILURE / AMBIGUOUS-
CONFLICTING CLAIMS) - not the two-state split an earlier draft of this
decision used, and not the broader "or is being classified" framing a
later draft used before this correction - before deciding whether to
attempt RustLike; that classification must recognize import-only Logos
source as a positive claim, not as "empty," and must NOT treat a bare
probe rejection with no prior positive evidence as authoritative (an
ordinary `RustLike` program failing a Logos probe at the first token
must still allow `RustLike` to be attempted);
`compile_program_to_ir_with_options_and_profile` must stop invoking
`parse_logos_program_with_profile` at all when `profile` is explicitly
`RustLike` (not merely stop acting on its result), and its `Auto` branch
needs the same classification instead of `unwrap_or(false)`; and
`cmd_check`'s `.or_else` must stop discarding a real multi-module load
failure in favor of a silently-narrower single-file check. Whether these
are one filed issue or several is an implementation-sequencing question,
not a decision-authority question - out of scope here; see "Durable
tracking for newly discovered defects" below for proposed issue text
awaiting owner approval.

**TEST CONSEQUENCE** (not performed here): a regression per path proving
that a genuinely invalid (not merely empty) grammar-specific input is
rejected with a diagnostic attributed to the surface that actually failed,
and never silently succeeds under a different surface or a narrower
mechanism - covering `check_source_with_profile`,
`compile_program_to_ir_with_options_and_profile`'s `Auto` branch, and
`cmd_check`'s multi-module-to-single-file fallback independently. Plus
two additional, distinct regressions: (1) for explicit `RustLike`,
`compile_program_to_ir_with_options_and_profile(input, CompileProfile::RustLike,
..)` must never invoke the Logos parser at all for any input, verified
directly (not merely that its result is ignored); (2) a source file
containing only `import` directives and no `system`/`entity`/`law`
declaration must be classified as a positive Logos surface claim and
must never be reinterpreted as RustLike - the exact case `#1670` itself
names and the two-state classification would still miss; (3) an ordinary,
valid `RustLike` program (e.g. `fn main() { return; }`) must still
compile successfully under `Auto` even though a Logos probe against it
fails immediately with no prior positive evidence - proving the
classifier does not treat every candidate-probe rejection as
authoritative; (4) an input that independently satisfies both grammars'
positive-claim conditions (or otherwise cannot be deterministically
resolved) must be rejected with an explicit classification-ambiguity
diagnostic, never silently resolved to whichever grammar was tried
first.

## Decision B - Diagnostic Identity Authority

**CURRENT STATE**: four independent, non-unified diagnostic types exist,
one per canonical stage: `sm_front::FrontendError` (parse/typecheck/
lowering, `crates/sm-front/src/types.rs:861-903`; `pos` is additionally
overloaded to mean an IR instruction index at one lowering call site,
`crates/sm-ir/src/legacy_lowering.rs:1290` - a real internal
inconsistency, not just a cross-crate one), `sm_sema::SemanticDiagnostic`/
`SemanticError` (`crates/sm-sema/src/std_adapters.rs:52-85`, its own
independent `DiagLevel` enum converted to `ton618_core`'s only at render
time), `sm_verify::VerificationDiagnostic`/`RejectReport`
(`crates/sm-verify/src/lib.rs:88-240`, the richest vocabulary - 25-variant
`VerificationCode` - but zero source-mapping field), and
`sm_vm::RuntimeError`/`RuntimeTrap` (`crates/sm-vm/src/semcode_vm.rs`,
composes `RejectReport` by direct embedding - the one already-real
cross-type link, `RuntimeError::VerifierRejected(RejectReport)`).
`ton618_core::diagnostics::Diagnostic<M>` (`crates/ton618-core/src/
diagnostics.rs:1-12`) is the type that *reads* as the apparent shared
carrier but is never constructed anywhere in the repository - dead code,
not evidence of an already-working unification. The stable-code registry
(`diagnostic_catalog()`, same file) is itself already stale: `sm-sema`
constructs `"E0243"` in production
(`crates/sm-sema/src/std_adapters.rs:226`) with no matching catalog
entry, so `smc explain E0243` fails today for a code the compiler
actually emits (`#1704`'s own class, a fresh instance found by the E0
predecessor reconnaissance).

**DECISION**: Each originating canonical stage continues to own its own
diagnostic **semantic identity** - `sm-front` for parse/typecheck/
lowering, `sm-sema` for semantic-analysis/module findings, `sm-verify`
for admission rejection, `sm-vm` for runtime failure. This decision does
**not** unify `FrontendError`/`SemanticDiagnostic`/`VerificationDiagnostic`/
`RuntimeError` into one Rust type, and does not require it. What it does
require: every one of these types' code/severity/family, once assigned by
its originating stage, is preserved unchanged by every adapter/
presentation layer downstream. Adapters may attach proven context
(a real span once one exists, a real file identity), render, serialize,
add related locations, or add transport metadata - they may not silently
replace the originating code, change family, heuristically upgrade/
downgrade severity, flatten an authoritative diagnostic into a generic
substitute (the exact anti-pattern `render_diag`'s `SourceMark::default()`
calls already commit today), or invent new semantic meaning. Where a
higher-level diagnostic intentionally wraps a lower-level one
(`RuntimeError::VerifierRejected(RejectReport)` is the existing, correct
example), the wrapped diagnostic is retained as a structured cause, never
destroyed - `RuntimeError`'s existing shape already gets this right and
is the pattern to generalize, not to change.

The diagnostic-code registry (`diagnostic_catalog()`) is placed under the
same authority as the codes it lists: it is a **derived index over
production-construction sites**, not an independently hand-maintained
list. A code that is ever constructed in production and absent from the
registry, or vice versa, is a drift defect (`#1704`/`E0243`'s class) by
this decision's own definition, and a future qualification gate (§11)
must be able to detect it mechanically rather than by manual audit.

**Ownership scope of this rule (owner review round 4 clarification)**:
`diagnostic_catalog()` currently lives in `crates/ton618-core/src/
diagnostics.rs` - the same crate the "Dependency-owner analysis" section
disqualifies from any *new* ownership role. This rule does **not**
conflict with that disqualification, because it does not assign
`ton618-core` a new role: `#1704`'s repair, as scoped here, is
**qualification/coherence of the existing catalog in its existing
location** - making the current function mechanically accurate against
current production call sites - not a decision that `ton618-core` is,
or becomes, the *canonical* registry owner for the future internal
diagnostic model (§6). When that future model is eventually built, its
own registry component's physical location is a question for the
carrier/governance checkpoint (see "Dependency-owner analysis"), not
answered by this rule and not assumed to be `ton618-core` merely because
today's stopgap function happens to live there. This is why the repair
DAG (§9) can leave `#1704` sequenced *before* the carrier-governance
checkpoint without reordering: a qualification-only fix to an existing
utility function's accuracy does not require the future owner to be
selected first, whereas building the actual canonical registry would.

**INVARIANT**: A diagnostic's originating semantic identity
(code/severity/family) is immutable once assigned; only structured
context (span, file, related locations, notes) may be added downstream,
never invented.

**WHY**: Unifying the four types into one Rust struct now, before the
frontend/semantic layers even preserve accurate positions (Decision D),
would freeze a rich external shape around fields most of the pipeline
cannot yet populate honestly. Preserving per-stage ownership while
freezing the *preservation rule* lets each stage's existing evidence
(`VerificationCode`'s 25 variants, `RuntimeTrap`'s already-frozen 4-variant
taxonomy from `#1763`) stay authoritative without a disruptive rewrite,
while still closing the actual defect class (`render_diag` discarding
real data) that motivated this decision.

**REJECTED ALTERNATIVES**:
- *Force one shared Rust enum/struct across all four stages now* -
  rejected: would either lose real per-stage structure (e.g.
  `VerificationCode`'s variant richness, `RuntimeTrap`'s already-frozen
  taxonomy) or become an unwieldy union type; also violates the
  canonical-carrier boundary this decision separately freezes (§6) by
  conflating "internal model" with "what every stage's Rust type must
  look like."
- *Let each adapter re-derive severity/family from the message string
  heuristically* - rejected: this is exactly the `.kind()` pattern
  already found dead in production (`sm_front::types::FrontendError::kind()`,
  `crates/sm-front/src/types.rs:887-893`, consulted only in test
  assertions) - a heuristic reconstruction of semantic meaning that
  the governing invariant forbids for source-surface identity applies
  equally here.

**IMPLEMENTATION CONSEQUENCE** (not performed here): no Rust type changes;
future repair of `#1704`/`E0243` adds a mechanical check (build script or
test) that every string literal diagnostic code constructed in production
across `sm-front`/`sm-sema`/`sm-verify` appears in the registry and vice
versa.

**TEST CONSEQUENCE** (not performed here): a registry-completeness test
in the same spirit as `tests/public_api_contracts.rs`'s drift guard.

## Decision C - File Identity Authority

**CURRENT STATE**: `file_id` (`ton618_core::source::FileId = u32`,
`crates/ton618-core/src/source.rs:1-17`) is hardcoded to `0` at every
construction site in the lexer (`crates/sm-front/src/lexer.rs:16`) and
every `SourceMark` built in `sm-sema` (`crates/sm-sema/src/
std_adapters.rs:321,344,424,492,1407`, all literal `file_id: 0`). Real
multi-file identity, when present at all, is glued on as a **string
prefix** at the presentation layer (`format!("{}: {}", module_path.display(),
e.diag.message)`, `std_adapters.rs:155-156`) or hardcoded to the literal
string `"<input>"` for the throwaway caret-rendering source map
(`std_adapters.rs:817-820`), regardless of the real path. Separately, the
canonical project/module authority already exists and is real: import
resolution canonicalizes filesystem paths
(`crates/smc-cli/src/package_manifest.rs::resolve_package_import_path`,
using `Path::canonicalize()` against the importer's own resolved path)
and project-root detection (`resolve_project_root_check_entry*`) is the
authority every `smc` subcommand already routes through. This is a real,
working authority - it is simply never connected to diagnostic file
identity today. The canonical package/module authority is richer than a
bare path: `admit_package_entry_module(entry: &Path) -> Result<Option<PackageModuleAdmission>,
PackageModuleAdmissionError>` (`package_manifest.rs:627-...`) returns a
structured `PackageModuleAdmission { manifest_path: String, package_name:
String, module_path: String }` when an entry is admitted into a package
context, and `Option::None` (not an error) when no enclosing manifest
exists - the exact "rootless standalone file" case this decision must
resolve. Separately, dependency packages are real, structurally distinct
subtrees: each dependency resolves to its own `package_root`/
`dependency_root` with its own manifest (`package_manifest.rs:1908-1911`
and surrounding), addressed by `package_name`/alias, not merely
flattened into the top-level project's file tree. This matters directly
for external identity: two different dependency packages (or a
dependency and the top-level project) can each contain a module at the
same *relative* path (e.g. `src/lib.sm`) - a bare "project-root-relative
path," without saying relative to *which* package's root, does not
distinguish them.

**DECISION**: File identity has two distinct representations, not one:

- **INTERNAL identity**: an opaque, cheap-to-compare identifier (a
  `FileId`-shaped value, or the existing type widened as needed) suitable
  for compiler-internal data structures - comparable, hashable, does not
  need to be human-legible.
- **EXTERNAL stable identity**: **CORRECTION NOTE (owner review round 5)**:
  two successive drafts of this bullet each froze a concrete wire form
  too early and each turned out collision-prone or unstable. Round 4
  corrected a CWD-relative path (unstable: the same file invoked from two
  different working directories earned two different strings) to a
  canonicalized file name - but a bare file name is itself collision-prone
  (`/a/foo.sm` and `/b/foo.sm`, or two different dependency packages each
  containing `src/lib.sm`, would collide on the same external identity
  for genuinely different sources), and a display-legible name is not the
  same claim as a collision-free identity. This decision now freezes the
  **law** external identity must satisfy, not a premature concrete
  serialization:

  External source identity MUST be:
  - derived from the canonical project/package/module admission
    authority (`admit_package_entry_module`/`PackageModuleAdmission`/
    `resolve_package_import_path`) - never a second, diagnostic-only
    resolver;
  - **collision-free within one admitted compilation/project graph** -
    two distinct admitted sources (whether in the same package, in
    different dependency packages, or a mix) MUST NOT share an external
    identity. For a package-admitted module, this means the identity is
    scoped by *which package* the module belongs to
    (`PackageModuleAdmission.package_name`) in addition to its
    in-package `module_path`, not a bare path assumed unique
    project-wide;
  - **deterministic independent of invocation CWD** - the same admitted
    logical source resolves to the same identity regardless of the
    directory a command was invoked from;
  - reproducible for the same admitted logical source across
    invocations/machines/CI (ruling out absolute filesystem paths, per
    "Rejected Alternatives" below);
  - **absent** when the canonical authority cannot prove such an
    identity - never a fabricated substitute.

  A concrete exact serialization (e.g. whether package-scoped identity is
  literally `package_name` + `module_path` joined by some separator, or a
  structured pair) is **not frozen here** - that is wire-format work for
  the future carrier/external-schema checkpoint (§6), constrained only by
  the law above. A file name or any other human-legible label MAY exist
  separately as a `display_name` for presentation, but a `display_name`
  is explicitly **not** the canonical identity and must never be used as
  one - a basename is display data, not identity, exactly like a
  `file_id = 0` or a fabricated `"<input>"` string is forbidden as a
  stand-in for absence elsewhere in this decision.

  For a **rootless standalone source** (`admit_package_entry_module`
  returns `None` - no enclosing manifest), the most honest resolution
  consistent with the law above is: **external identity is absent**.
  Inventing a basename-based identity for this case would violate the
  collision-freedom requirement (two different standalone files can
  trivially share a name) for no real benefit - `admit_package_entry_module`
  already models "no admitted identity" as `None`, and this decision
  follows that signal rather than working around it. A file name MAY
  still be surfaced as a separate `display_name` in this case, distinct
  from - and never presented as - the canonical identity. An imported
  module's external identity is the package-scoped identity already
  established when it was admitted and loaded - not re-derived
  independently at the diagnostic layer. Stdin/synthetic/virtual sources
  (none currently exist as a compiler input path) have no frozen
  representation here - out of scope until such an input path is added.

**INVARIANT**: If the canonical project/module authority cannot prove a
file's identity for a given diagnostic, that diagnostic's file identity
is **absent**, never `file_id: 0` presented as if it were meaningful,
never a fabricated `"<input>"` placeholder presented as if it were a real
path.

**WHY**: `file_id = 0` today is not "unknown file" and "the first file"
made to look the same by coincidence - it is the *only* value ever
constructed, so it carries zero information regardless of what it might
theoretically mean. Any future multi-file diagnostic UI (aggregated
`smc check` output across a project, or a future LSP's per-file
`publishDiagnostics`) is currently unbuildable without inventing ad hoc
string-parsing over the `"{path}: {message}"` convention `std_adapters.rs:155-156`
already uses informally. Deriving external identity from the *existing*
project/module authority - rather than building a second, diagnostic-only
resolver - is what keeps this decision inside the governing invariant's
"no second parser/typechecker/verifier/project-model authority" boundary
extended to project structure itself.

**REJECTED ALTERNATIVES**:
- *Build a dedicated diagnostic-file-identity resolver independent of
  `package_manifest.rs`* - rejected: exactly the kind of duplicate
  authority (`prom-ui`'s ~20 independent diagnostic structs are the
  cautionary precedent the reconnaissance already flagged) this whole
  checkpoint exists to prevent.
- *Use an absolute filesystem path as external identity* - rejected: not
  reproducible across machines/CI, and already contradicted by the
  project model's own preference for root-relative paths elsewhere
  (`docs/roadmap/.../semantic_stable_foundation_matrix.md`'s "Source/module
  root resolution" row).
- *Use a URI (`file://...`) now, anticipating LSP* - rejected explicitly
  by this checkpoint's own instruction not to choose based on LSP
  convenience; URI conversion is a presentation-adapter concern (§6/§7),
  not the canonical identity itself.
- *Use a bare project-root-relative path, assumed unique project-wide*
  (round 4's framing) - rejected (round 5): does not account for
  dependency packages, which are structurally distinct subtrees that can
  each contain a module at the same relative path; collision-prone
  across package boundaries.
- *Use a canonicalized file name for rootless standalone files* (round
  4's correction) - rejected (round 5): fixes CWD-instability but is
  itself collision-prone (two different standalone files can share a
  name), and a display-legible name is not the same claim as a
  collision-free identity; superseded by treating this case as identity-
  absent with an optional separate `display_name`.

**IMPLEMENTATION CONSEQUENCE** (not performed here): thread
`PackageModuleAdmission`'s `package_name` + `module_path` (available at
the point `admit_package_entry_module`/`check_file_with_provider`
already resolve them) into the diagnostic construction path as a single,
package-scoped external identity, replacing every `file_id: 0`/
string-prefix/`"<input>"` site; for a rootless single-file invocation
(`admit_package_entry_module` returns `None`), represent external
identity as absent, with the canonicalized file name available
separately only as a `display_name`, never conflated with identity; the
exact serialization of the package-scoped identity (e.g. how
`package_name` and `module_path` combine into one string or structured
value) is deferred to the future carrier/external-schema checkpoint, not
frozen here.

**TEST CONSEQUENCE** (not performed here): positive tests for an
ordinary project file and an imported module (real, resolvable,
package-scoped identity); a collision test proving two modules at the
same relative path in two different admitted packages (or a package and
its dependency) resolve to two *different* external identities; a
collision test proving two different rootless standalone files sharing a
file name do **not** resolve to the same identity (both must be absent,
or otherwise provably distinct - never accidentally equal); a test that
the same single file, invoked from two different working directories,
never produces two different identities (whether that identity is
present or absent, it must not depend on CWD); and a deterministic-
absence test for a source whose identity cannot be proven at all (no
invented `0`/`"<input>"`/basename fallback).

## Decision D - Source Range Authority

**CURRENT STATE**: four incompatible position representations coexist
today with no interconversion: (1) `ton618_core::SourceMark{line,col:u32,
file_id}` (point only) plus a `Span{start,end}` **range** type that is
defined in the same crate but used exclusively by `prom-ui`, never by the
language compiler itself (`crates/ton618-core/src/source.rs:7-17`,
zero uses outside `prom-ui`); (2) a parallel raw byte `pos: usize` on
`Token`/`FrontendError` (`crates/sm-front/src/types.rs:867-870,906-910`),
never merged with `SourceMark`; (3) `prom-ui`'s own private
`SourceSpan{start:u32,end:u32}`+opaque `SourceId` with no line/col concept
at all (`crates/prom-ui/src/contract_primitives.rs:47-106`, a different
DSL, out of this decision's scope but a real architectural precedent of
what happens absent one frozen law); (4) the compiled-bytecode
`DecodedDebugSymbol{pc,line,col:u16}` (`crates/sm-format/src/
semcode_decode.rs:23-25`), a fourth shape with a `col` width mismatch
(`u16` vs. `SourceMark`'s `u32`). The lexer already tracks raw **byte**
offsets internally before collapsing to line/col
(`crates/sm-front/src/lexer.rs:402,478`: `line_start` is a running byte
offset, `col = (start + 1)` where `start` is a byte index within the
current line) - byte-offset tracking is not a new capability this
decision would have to invent, it already exists internally and is
simply discarded rather than preserved as a range. Position is
additionally zeroed outright (not merely coarse) for the majority of
error classes today (`SourceMark::default()`, `pos: 0` - see Decisions A
and C's evidence) - a separate defect from the *representation* question
this section answers, already covered by Decisions A/B's preservation
rule.

**CORRECTION NOTE (owner review round 4)**: the first version of this
section said the canonical byte range "extend[s] `ton618_core::Span`'s
existing shape (already a range type...)." Two independent problems:
(1) this reassigns `ton618-core` a new canonical-ownership role
("extend its shape" makes it the future carrier of the canonical range
type), which the "Dependency-owner analysis" section elsewhere in this
same document already establishes `ton618-core` is disqualified from -
a closed governance track forbids assigning it any new ownership role,
diagnostic carrier or otherwise, and that conclusion must hold
consistently across every section, not just the one that states it
explicitly. (2) It also mischaracterizes the actual current shape:
`ton618_core::Span` is `{ start: SourceMark, end: SourceMark }`, and
`SourceMark` is `{ line: u32, col: u32, file_id: FileId }`
(`crates/ton618-core/src/source.rs:1-17`) - two line/column/file-id
*points*, not a pair of byte offsets. "Extending its existing shape" to
mean byte offsets would not be an extension, it would be a different
representation entirely. Both issues are corrected below by separating
what this decision actually freezes (semantics) from what it explicitly
does not freeze (a concrete Rust type or its owning crate).

**DECISION**: The canonical **internal** source-range *semantics* are:
**UTF-8 byte offsets, zero-based, half-open `[start, end)`**, using a
real range (two offsets) rather than a single point. This decision
freezes those semantics only - it does **not** freeze a concrete Rust
type or a physical owning crate for them; per "Dependency-owner
analysis," that selection belongs to the future carrier/governance
checkpoint, exactly like the internal canonical diagnostic model's own
carrier selection. `ton618_core::Span`/`SourceMark` are cited elsewhere
in this document only as **current/legacy representation evidence**
(what exists today and why it is inadequate - see "CURRENT STATE"
above), never as the future implementation target. Line/column is a
**derived presentation view** computed from the canonical byte range
plus the source text, never itself the canonical identity - this
directly reverses today's lexer behavior (byte offset computed first,
then immediately collapsed to line/col and discarded), not merely
documents it. Concrete rules: an empty range is a valid zero-width
`[n, n)`, not a special case requiring its own type; an EOF-anchored
diagnostic uses `[len, len)` on the source's own byte length; a
diagnostic with no provable location (a synthetic/whole-program-level
finding) carries **no range at all**, not a `[0, 0)` placeholder
presented as if it meant something; invalid UTF-8 is not a range-law
concern (the compiler's own source-loading boundary already rejects
non-UTF-8 input via `fs::read_to_string`'s `Result::Err`, confirmed
panic-free, before any range is ever computed) and CRLF is not a
range-law concern either (byte offsets are representation-agnostic to
line-ending style; only the line/col *presentation* view needs a
documented CRLF convention, which this decision does not need to fix
since it is a derived view, not the canonical identity).

Downstream artifact/execution coordinates are explicitly **not** source
ranges and must never be presented as one without a validated mapping:
`DecodedDebugSymbol{pc,line,col:u16}` is source-mapping *evidence*
embedded in a compiled artifact, not itself the canonical source range;
`VerificationDiagnostic.offset` (`crates/sm-verify/src/lib.rs:88-225`) is
a bytecode-byte-offset **artifact coordinate**; a VM's runtime program
counter is an **execution coordinate**. A verifier- or runtime-originated
diagnostic receives a canonical source range **only** when validated
provenance proves, for that specific diagnostic instance, **all** of:
(a) which source the range belongs to (a resolvable file identity per
Decision C), (b) that the mapping corresponds to the actual compiled
source revision (not a stale or mismatched build), and (c) a concrete
byte `[start, end)` pair against that source's bytes. **`DecodedDebugSymbol`'s
existing `{pc, line, col}` shape does not, by itself, satisfy this** - it
carries no file identity and no byte offsets, only a program counter and
a presentation-level line/column point; a successful lookup against it
proves at most "this PC maps to this line/column," which is not the same
claim as a proven canonical byte range. If a future architecture decides
that a `DecodedDebugSymbol` lookup can license a *derived* zero-width
byte anchor (e.g. by re-deriving a byte offset from its line/column
against a known-matching source text) or some other reduced-but-honest
range, that is a **new, explicit rule** requiring its own decision - not
assumed here, and not implied by "the mapping exists as data." Today,
`DebugSymbol` is validated for structural well-formedness at VM load
(`crates/sm-vm/src/semcode_vm.rs:1668-1684`) but is exhaustively never
consulted at any of the eight production `RuntimeError::Trap(...)`
construction sites - so under this decision's own rule, every current
runtime trap correctly has **no** source range today, not a fabricated
`offset 0`/`line 0` standing in for one, and closing that gap requires
both performing the lookup *and* resolving the file-identity/byte-range
insufficiency named above - neither is future implementation performed
here.

**INVARIANT**: A source range is either a real, provably-mapped
`[start, end)` byte pair against a specific source's actual bytes, or it
is absent. `offset 0`, `line 0`, `pc 0`, or any other artifact/execution
coordinate is never substituted for an absent source range.

**WHY**: Byte offsets are the representation the lexer already computes
internally before throwing it away, so adopting them as canonical costs
no new instrumentation, only *preserving* what already exists one line
later than it currently survives. Half-open ranges compose cleanly
(concatenation, zero-width insertion points, "end of file" without a
special "one past the last valid index" off-by-one convention) and match
the convention `prom-ui`'s own already-existing (if unrelated-language)
`SourceSpan` independently converged on. Deriving line/col from the byte
range rather than storing both avoids exactly the representation drift
already visible between `SourceMark`'s `u32` column and
`DecodedDebugSymbol`'s `u16` column - one stored canonical form, computed
views, never two independently-maintained "same" values that can silently
diverge.

**REJECTED ALTERNATIVES**:
- *Keep `SourceMark` (line/col) as canonical, treat byte offsets as
  derived* - rejected: line/col is not stable under CRLF-vs-LF
  normalization, tab width, or multi-byte UTF-8 sequences without an
  explicit codepoint-counting convention this decision would then also
  have to freeze; byte offsets require none of that to be well-defined,
  and the lexer already computes them first regardless.
- *UTF-16 code-unit offsets (LSP's own convention) as canonical* -
  rejected explicitly per this checkpoint's own instruction not to choose
  based on LSP convenience; a UTF-16 conversion belongs at a future LSP
  adapter boundary (§6/§7), converting *from* the canonical UTF-8 byte
  range, not replacing it.
- *Inclusive end offsets* - rejected: half-open composes correctly for
  zero-width diagnostics (e.g. "expected token here") without a special
  case, which an inclusive convention cannot represent without a signed
  or `Option`-wrapped end.

**IMPLEMENTATION CONSEQUENCE** (not performed here): the lexer preserves
`(start_byte, end_byte)` per token instead of only `pos`/`mark`;
`FrontendError` and every downstream diagnostic gain a real, `(start,
end)`-range-shaped optional field (concrete type/owner not frozen here -
see "DECISION" above) instead of/alongside the current point `pos`;
`sm-verify`/`sm-vm` gain the actual debug-symbol lookup at trap sites -
but populating a range requires *more* than a successful
`DecodedDebugSymbol` lookup, per the corrected mapping law above: file
identity and a genuine byte `[start, end)` must also be established
(whether by extending what a debug symbol carries, or by a separately
frozen derivation rule), or the range stays absent even when the
existing `{pc, line, col}` lookup itself succeeds.

**TEST CONSEQUENCE** (not performed here): positive range-preservation
tests per error class (mirroring Decision C's per-class tests); an
explicit test that a runtime trap with no available debug symbols
produces a diagnostic with an absent range, not a fabricated one; and an
explicit test that a *successful* `DecodedDebugSymbol` lookup alone
(today's `{pc, line, col}` shape, with no file identity or byte offsets
added) still produces an absent canonical range, proving the
implementation does not conflate "PC mapped to a line/column" with "byte
range proven."

## Canonical carrier boundary

Not implemented here. This decision freezes only what the future carrier
**must represent**, and separates two boundaries that must never collapse
into one Rust type:

- **INTERNAL CANONICAL DIAGNOSTIC MODEL**: whatever in-process
  representation each stage's diagnostic is converted into for
  cross-stage/cross-crate consumption (rendering, aggregation). Evolves
  under ordinary Rust/workspace-internal API-compatibility rules (the
  same `tests/public_api_contracts.rs` discipline already used
  throughout SSF-08). At minimum represents: diagnostic identity/code
  (Decision B), severity, phase/family, message, file identity (Decision
  C, optional/absent-capable), source range (Decision D, optional/
  absent-capable), structured related locations, structured notes, an
  optional structured fix/proposal, and an optional structured cause
  (for the `RuntimeError::VerifierRejected(RejectReport)`-style wrapping
  case).
- **EXTERNAL VERSIONED MACHINE SCHEMA**: the future stable, versioned,
  machine-readable contract (`#1580`'s own AC1) that external tooling
  (a future LSP, CI integrations) actually parses. Requires explicit
  version negotiation and a compatibility policy before it is frozen -
  none exists yet, and this decision does not draft one.

These are two different documents/types with two different change-control
regimes, not one Rust struct serialized two ways: the internal model may
gain a field for a new stage's need without that being an external
schema-breaking change; the external schema's own versioning is a
separate, later decision.

## Dependency-owner analysis

Reconstructed from each relevant crate's actual `Cargo.toml`
`[dependencies]` block at this exact baseline (not inferred, not assumed):

| Crate | Depends on (direct) | Depended on by |
|---|---|---|
| `ton618-core` | *(none)* | `sm-front`, `sm-sema`, `smc-cli` |
| `sm-format` | *(none)* | `sm-ir`, `sm-emit`, `sm-verify`, `sm-vm` |
| `sm-runtime-core` | *(none)* | `sm-verify`, `sm-vm` |
| `sm-profile` | *(none - only optional external `serde`/`serde_json`)* | `sm-front`, `sm-sema`, `sm-ir` |
| `sm-front` | `ton618-core`, `sm-profile` | `sm-sema`, `sm-ir`, `smc-cli` |
| `sm-sema` | `ton618-core`, `sm-front`, `sm-profile` | `smc-cli` |
| `sm-ir` | `sm-front`, `sm-profile`, `sm-format` | `sm-emit`, `smc-cli` |
| `sm-emit` | `sm-format`, `sm-ir` | `smc-cli` |
| `sm-verify` | `sm-runtime-core`, `sm-format` | `sm-vm`, `smc-cli` |
| `sm-vm` | `sm-runtime-core`, `sm-verify`, `sm-format`, `prom-*` | `smc-cli` |
| `smc-cli` | (all of the above) | *(top of graph)* |

**Load-bearing finding**: `ton618-core`, `sm-format`, `sm-runtime-core`,
and `sm-profile` are four *separate*, non-overlapping zero-dependency
leaf crates. `sm-front`/`sm-sema` reach `ton618-core` and `sm-profile`;
`sm-verify`/`sm-vm` reach only `sm-format`/`sm-runtime-core`. **Neither
branch currently depends on any of the other branch's leaf crates** -
`ton618-core`'s existing `Diagnostic<M>`/`SourceMark`/`Span` types, and
`sm-profile` itself, are structurally unreachable from `sm-verify`/
`sm-vm` today, which is an independent, architectural reason (not merely
"it's dead code") why the current
`ton618_core::diagnostics::Diagnostic<M>` cannot yet be a real cross-stage
canonical carrier even in principle, regardless of whether anyone ever
constructs it.

**CORRECTION NOTE (owner review round 2)**: the prior version of this
section named `ton618-core` as "SELECTED" future carrier owner, then
separately flagged an "open question" about whether that selection is
consistent with the TON618 perimeter's own ownership policy. Freezing a
selection while simultaneously flagging that its legality is unresolved
is a contradiction, not a decision. Re-reading the perimeter's own
closure record settles the question directly rather than leaving it
open: `docs/roadmap/language_maturity/ton618_compatibility_perimeter_scope.md`
is `Status: completed post-stable closure track`. Its "Explicit
Non-Goals" list includes, verbatim, "moving canonical ownership back
from `sm-*` crates into TON618-named paths." Its "Why This Exists"
section states plainly: "canonical public ownership for CLI, frontend,
IR, SemCode, VM, and profile contracts already lives in the `sm-*`
owners." Its "Close-Out Reading" states: "Any future change to \[the
TON618 perimeter's\] behavior or ownership would require a new
explicitly scoped follow-up track rather than incremental drift." This
is a closed, frozen governance record, not merely a harness path list -
`ton618-core` cannot be assigned a new ownership role (diagnostic
carrier or otherwise) without first reopening that specific track, which
this decision does not do and is not scoped to do. The candidate table
and selection below are re-run under two constraints instead of one.

**CORRECTION NOTE (owner review round 3)**: the round-2 candidate table
covered `ton618-core`, `sm-runtime-core`, and `sm-format` only, which
made "no existing crate satisfies" an unproven assertion rather than an
exhaustive conclusion - it omitted `sm-profile` (an existing `sm-*`,
zero-dependency, Construction-zone crate that passes the dependency-graph
test) and the non-leaf Construction crates entirely. The table below
evaluates every architecturally-relevant existing crate under three
independent criteria, so the "no existing crate satisfies" verdict is
now evidence-derived rather than an artifact of an incomplete candidate
list.

**Candidate owners for the future internal canonical model**, classified
under three independent criteria: **(A)** dependency-graph correctness
(can `sm-verify`/`sm-vm` reach it without a cycle, and without violating
`dependency_boundary_rules.md`'s "Construction -> Execution" flow
direction), **(B)** ownership/naming-policy consistency (does assigning
it this role conflict with an existing, frozen ownership record), and
**(C)** semantic responsibility cohesion (does the crate's existing
charter make diagnostic-carrier ownership a natural extension or a
grab-bag):

| Candidate | (A) Dependency-safe | (B) Ownership-policy consistent | (C) Responsibility cohesion | Verdict |
|---|---|---|---|---|
| `sm-profile` | YES - zero dependencies (only optional external `serde`/`serde_json`), so `sm-verify`/`sm-vm` could depend on it without a cycle | YES - already `sm-*`-named, already Construction-zone, no frozen record restricts its role | **NO** - its existing, sole charter is admission/policy configuration (`ParserProfile`: identity/version/abi/compatibility/features/capabilities/aliases); bolting on diagnostic-carrier ownership bundles two unrelated concerns (language-admission policy and diagnostic identity/representation) into one crate, which `dependency_boundary_rules.md`'s own per-crate single-responsibility pattern (lexer/AST in `sm-front`, optimizer/SemCode format in `sm-ir`, CLI contract in `smc-cli`) argues against | REJECTED (fails C) |
| `sm-front` | **NO** - not a zero-dependency leaf (depends on `ton618-core`, `sm-profile`); `sm-verify`/`sm-vm` depending on it would pull in the entire lexer/parser/AST surface for one diagnostic type | N/A - fails (A) | N/A - already has a real, unrelated charter (lexer/AST/source-level type-check, per `dependency_boundary_rules.md`) | REJECTED (fails A; also `dependency_boundary_rules.md`'s "execution crates must not reach back into parser/sema internals" forbids the direction outright) |
| `sm-sema` | **NO** - depends on `sm-front`, same transitive-weight problem, one layer further | N/A - fails (A) | N/A - already has a real, unrelated charter (semantic analysis) | REJECTED (fails A, same rule as `sm-front`) |
| `sm-ir` | **NO** - depends on `sm-front`, `sm-profile`, `sm-format`; same transitive-weight and layering problem | N/A - fails (A) | N/A - already owns the optimizer and SemCode format contract per `dependency_boundary_rules.md` | REJECTED (fails A) |
| `sm-emit` | **NO** - depends on `sm-format`, `sm-ir` | N/A - fails (A) | N/A - already a producer-facing SemCode facade | REJECTED (fails A) |
| `sm-format` | trivially yes for `sm-verify`/`sm-vm`/`sm-ir`/`sm-emit` (already a dependency) but **NO** for `sm-front`/`sm-sema` | N/A - fails (A) for the frontend-facing half | N/A - a SemCode binary-format crate; diagnostic identity is not a bytecode-format concern | REJECTED (fails A and C) |
| `sm-runtime-core` | trivially yes for `sm-verify`/`sm-vm` (already a dependency) but **NO** for `sm-front`/`sm-sema` (would require Construction-zone crates to depend backward into the Execution zone) | N/A - fails (A); `dependency_boundary_rules.md`'s own "Allowed flow: Construction -> Execution -> Integration" and "construction crates must not depend on VM/runtime state" rules independently forbid this direction | N/A - a runtime/quota/trap-taxonomy crate (`#1759`-`#1763`'s own home) | REJECTED (fails A and C) |
| `ton618-core` | YES - zero dependencies itself, so any crate (including `sm-verify`/`sm-vm`) can add it as a dependency without creating a cycle | **NO** - `ton618_compatibility_perimeter_scope.md` closes this crate's role as retained-non-owning and lists moving canonical ownership into TON618-named paths as an explicit non-goal; assigning it a new diagnostic-carrier ownership role would reopen a track that document declares closed | N/A - fails (B) regardless | **DISQUALIFIED** (fails B) |
| A brand-new `sm-*`-named Construction-zone leaf crate | YES - zero dependencies by construction, and `sm-verify`/`sm-vm` (Execution) depending on it follows the documented "Construction -> Execution" allowed flow directly | YES - `dependency_boundary_rules.md`'s own zone model already places canonical Construction-zone ownership in `sm-*`-named crates, and the perimeter-scope document's own stated principle is that canonical ownership belongs in `sm-*` owners, not TON618-named paths | YES - a purpose-built crate with no pre-existing, unrelated charter has no cohesion conflict by construction | **NO EXISTING CRATE SATISFIES ALL THREE CRITERIA - A NEW CRATE IS THE LEADING ARCHITECTURAL DIRECTION, NOT A FROZEN SELECTION** |

**No future carrier owner is selected by this decision.** Every existing
crate fails at least one criterion: the zero-dependency leaves
(`ton618-core`, `sm-format`, `sm-runtime-core`, `sm-profile`) each fail
either (B) or (C); every non-leaf Construction crate (`sm-front`,
`sm-sema`, `sm-ir`, `sm-emit`) fails (A) outright, both on raw dependency
weight and on `dependency_boundary_rules.md`'s explicit "execution
crates must not reach back into parser/sema internals" rule. A new
`sm-*`-named, zero-dependency, Construction-zone leaf crate is the
best-evidenced *direction* - it is dependency-safe, ownership-policy
consistent, and free of any cohesion conflict by construction - but
naming, exact scope, and creation of such a crate is itself a decision
this checkpoint is not scoped to make unilaterally (it would need its
own `allowed_paths` entry and its own `dependency_changes` authorization,
exactly like the disqualified `ton618-core` path would have). This is
deliberately left as:

```
CARRIER OWNER SELECTION:                                  NOT YET VALID
DEPENDENCY GRAPH ANALYSIS:                                VALID
ton618-core TECHNICALLY POSSIBLE (DEPENDENCY GRAPH ONLY): YES
ton618-core ARCHITECTURALLY AUTHORIZED:                   NO
LEADING CANDIDATE DIRECTION:                              new sm-*-named Construction-zone leaf crate
CANDIDATE DIRECTION FROZEN:                               NO - requires a separate governance/ownership checkpoint
```

**A separate governance/ownership checkpoint - not this decision - must
select and authorize the final carrier owner**, informed by this
dependency-and-policy analysis, before any carrier-implementation PR is
opened. That checkpoint's own future governance-authorization request
should evaluate a new `sm-*` leaf crate as the leading candidate,
propose its exact name and scope for owner approval, and add the
resulting path to `.harness/current.task.yaml`'s `allowed_paths` with
`dependency_changes` enabled for that specific edge - none of which this
PR performs.

## Verifier/runtime source-mapping law

Stated fully under Decision D; restated here as the standalone rule
future qualification (§11) must check: a verifier- or runtime-originated
diagnostic's source range is present **if and only if** validated
provenance proves file identity, source-revision correspondence, and a
genuine byte `[start, end)` for that specific diagnostic instance - not
merely that *some* debug-symbol lookup succeeded. No mapping attempted:
no range. Mapping attempted and failed (out-of-bounds, missing table):
no range. A `DecodedDebugSymbol` lookup succeeding on its own (today's
`{pc, line, col}` shape) is **insufficient** and must still produce no
range, because it carries neither file identity nor byte offsets. Only
when the fuller provenance above is actually established does a real
mapped range exist. Never a placeholder in any other case.

## Presentation/LSP boundary

No editor/LSP node may own semantic truth (restated from the governing
invariant, made concrete for the authority graph, §7).

**CORRECTION NOTE (owner review round 6)**: the first version of this
section said a future LSP adapter "converts the canonical
project-root-relative file identity to a `file://` URI." That
representation is no longer Decision C's contract - Decision C (as
corrected in round 5) freezes a package/module-authority-derived law
with an unfrozen concrete serialization, and explicitly allows canonical
identity to be **absent** for a rootless standalone source. "Convert the
canonical identity to a URI" silently assumed canonical identity is
always a filesystem-shaped string, which is exactly the premature
wire-format assumption round 5 removed. Corrected by separating two
concepts this section previously conflated:

- **CANONICAL LOGICAL SOURCE IDENTITY** - the compiler/project authority
  from Decision C. May be present (derived from
  `PackageModuleAdmission`) or **absent** (a rootless standalone source
  with no provable identity). This is compiler-side semantic truth; an
  LSP adapter never invents it.
- **TRANSPORT / DOCUMENT LOCATOR** - editor/session routing metadata
  (e.g. the `textDocument.uri` an LSP client supplies with a request or
  an open-document notification). This is protocol plumbing supplied by
  the *client*, not compiler semantic authority, and it exists
  independently of whether canonical logical identity is present or
  absent for that document.

Rules for a future LSP adapter, given this separation:

- It MAY route a diagnostic back to the transport/document locator the
  client supplied for that document (`textDocument.uri`) - this is
  ordinary protocol routing, not a semantic claim.
- It MUST NOT promote that locator into canonical compiler identity, and
  MUST NOT claim it was "derived from" canonical identity when canonical
  identity is absent for that source (the rootless-standalone case).
  Presenting a client-supplied URI as if it were the compiler's own
  proven identity would be exactly the kind of guessed/fabricated
  authority the "Fail-closed rules" section forbids.
- For a package-admitted source, a canonical-identity-to-filesystem (and
  from there, to URI) mapping may be derived only through the existing
  canonical project/package/module authority - never a second,
  LSP-specific path resolver.
- A rootless standalone document with no canonical logical identity
  remains fully routable and diagnosable through its transport locator
  alone; identity being absent is not a reason to withhold diagnostics,
  only a reason not to fabricate an identity that was not proven.

Converting the canonical UTF-8 byte range to LSP's UTF-16 code-unit
convention (a pure, lossless-in-the-safe-direction numeric conversion,
not a reinterpretation) and relaying the canonical diagnostic identity
(code/severity/family) unchanged remain as originally stated; caching
keyed by canonical identity is only available when that identity is
present, and must fall back to a transport-locator-scoped cache
otherwise - it may never recompute a diagnostic itself. This mirrors,
and is bound by, the same rule `.agents/skills/semantic-ui-boundary-guard/SKILL.md`
already enforces for the unrelated UI-DNA2 presentation layer ("must NOT
rewrite or alter verifier diagnostics") - a precedent this decision
generalizes rather than invents. None of this freezes the external
schema or an LSP implementation - both remain future work.

## Fail-closed rules

**CORRECTION NOTE (owner review round 6)**: the first version of this
section consolidated all four decisions' fail-closed behavior into one
claim - "the correct representation is a deterministic, explicit
'absent'" - as if every authority's failure mode resolves the same way.
That is no longer accurate: Decision A's AMBIGUOUS/CONFLICTING CLAIMS
outcome is a **reported classification error**, not an absent surface,
and Decision B's rule is about **preserving** an already-assigned
identity, not producing an absent one. The common law across all four is
narrower and more precise than "always absent": **never guess or
fabricate authority** - what satisfies that law differs by which
authority is missing or in conflict:

- **Source surface (Decision A)**: an authoritative failure is preserved
  as an error attributed to the surface that owns it, never discarded in
  favor of another surface. Ambiguous or conflicting claims are reported
  as a **deterministic classification error**, never resolved by
  guessing a winner. Neither case produces an "absent" surface - both
  produce an explicit, attributed failure.
- **Diagnostic semantic identity (Decision B)**: code/severity/family are
  preserved unchanged once originating-assigned; never invented, never
  silently replaced by an adapter. There is no "absent" form of this rule
  - a diagnostic that exists always has a real originating identity: the
  rule forbids overwriting it, not producing an empty one.
- **File identity (Decision C)**: unprovable is **absent** - `file_id =
  0`/`"<input>"`/a collision-prone basename forbidden as a stand-in.
- **Source range (Decision D)**: unprovable is **absent** - `[0,0)`/
  `pc 0`/`offset 0` forbidden as a stand-in.

"Absent" is the correct outcome specifically for Decisions C and D
(identity/location data that may simply not exist for a given input).
For Decision A, the correct outcome is a reported error, not an absence
- a source file that fails to classify is not "sourceless," it is
unclassifiable, and that fact must be surfaced, not silently swallowed
into nothing. For Decision B, the correct outcome is preservation, not
absence. The rule that generalizes across all four is **never guess or
fabricate a value in place of the real authority** - never a default
value presented as if it were meaningful, and never a silent retry under
a different authority (Decision A's grammar-hopping case) presented as
if it were the original authority succeeding.

## Rejected alternatives (cross-cutting)

- *Defer all four authority questions until the canonical carrier is
  implemented, deciding them implicitly by whatever the first
  implementation PR happens to do* - rejected: this is precisely the
  "build a nicer-looking policy after the fact" anti-pattern this whole
  engagement has repeatedly found and corrected (most recently in
  `#1762`'s own corrective round); freezing the authority first is what
  lets `#1670`/`#1697`/`#1698`/`#1699`/`#1704` all be judged against one
  standard instead of four ad hoc fixes.
- *Adopt Position-B-style "expand scope now, converge later" for
  diagnostics* - rejected by the same reasoning `ssf08_ownership_position_decision.md`
  already applied to ownership: no evidence requires a broader claim than
  what is being frozen here, and a narrower, evidence-backed contract is
  preferred over a more ambitious one this checkpoint cannot yet qualify.

## Dependency-owner analysis addendum: implementation sequencing note

**CORRECTION NOTE (owner review round 2)**: this note previously assumed
`ton618-core` as the carrier owner and only flagged a governance-path
gap. Per the correction above, `ton618-core` is disqualified outright by
the TON618 perimeter's own closed governance track, not merely
ungoverned. The *first* piece of actual carrier work (not authorized by
this decision) is therefore not "add `ton618-core` to `allowed_paths`" -
it is a separate governance/ownership checkpoint that selects and
authorizes the actual owner (the leading candidate being a new `sm-*`
Construction-zone leaf crate), adds that crate's path to
`.harness/current.task.yaml`'s `allowed_paths`, and enables
`dependency_changes` for the resulting edge into `sm-verify`/`sm-vm`.
Only after that checkpoint can the dependency edge itself - a small,
dependency-graph-safe one-line `Cargo.toml` change regardless of which
crate is finally selected - be added.

## Non-goals

- No canonical carrier implementation, no JSON schema, no LSP/editor code
  (all explicitly forbidden by this checkpoint, restated from its own
  governing brief).
- No unification of `FrontendError`/`SemanticDiagnostic`/
  `VerificationDiagnostic`/`RuntimeError` into one Rust type (Decision B).
- No change to `RuntimeTrap`'s already-frozen 4-variant taxonomy (`#1763`,
  SSF-08) or to `RuntimeQuotas`/`ExecutionConfig` (`#1759`-`#1762`,
  SSF-08) - this decision's file-identity/source-range authority applies
  *to* those existing types' diagnostics, it does not reopen them.
- No repair of `#1670`, `#1697`, `#1698`, `#1699`, or `#1704`/`E0243` in
  this checkpoint.
- No `.harness/current.task.yaml` scope widening in this PR, and no
  carrier owner is frozen by this PR. `ton618-core` is disqualified as a
  carrier owner by the TON618 perimeter's own closed governance track
  (`ton618_compatibility_perimeter_scope.md`); a new `sm-*`
  Construction-zone leaf crate is the leading candidate direction but is
  not itself authorized, named, or created here. A separate governance/
  ownership checkpoint must select the final owner, add its path to
  `allowed_paths`, and enable `dependency_changes` for the resulting
  edge before any carrier-implementation PR - see the correction under
  "Dependency-owner analysis" above. This document does not perform, and
  does not claim to have already obtained, that authorization.
- No new crate is created by this PR.

## Qualification requirements (for the future implementation checkpoint)

- Every one of Decision A-D's "TEST CONSEQUENCE" items.
- The registry-completeness mechanical check (Decision B).
- Confirmation that `sm-verify`/`sm-vm` gaining a dependency on whichever
  crate the future governance/ownership checkpoint selects does not
  introduce a build cycle (`cargo tree` / a full workspace build is
  sufficient evidence, no new tooling required).
- Re-confirmation that `#1670`'s fix does not change any *admitted*
  program's compilation result - only failure-classification behavior for
  currently-misclassified inputs.
- A test proving `compile_program_to_ir_with_options_and_profile(input,
  CompileProfile::RustLike, ..)` never invokes the Logos parser, for any
  input (see Decision A's TEST CONSEQUENCE).

## Residual/open issues

Unaffected by this decision, tracked separately: `#1885` (`sm-front`
API-drift-guard gap, INDEPENDENT per the SSF09-E0 reconnaissance), `#1778`
(ABI value-canonicality, INDEPENDENT), `#1617` (broad platform audit,
INDEPENDENT), `#1716` (`sm-ir` API-drift-guard gap, same class as
`#1885`), `#1768` (quota-baseline pinning, a qualification gap unrelated
to diagnostics). None of these overlaps this decision's own scope.

## Required repair sequencing

The proposed DAG from this checkpoint's own governing brief is confirmed
correct against the evidence gathered above - no reordering required:

```
Diagnostic Authority Decision (this document)
        |
        v
#1670 source-surface fail-open repair
        |
        v
#1697 file/module identity preservation     (Decision C)
#1698 source-position preservation          (Decision D)
#1699 imported-module position preservation (Decisions C+D)
        |
        v
#1704 + E0243 catalog coherence             (Decision B)
        |
        v
carrier-owner governance/ownership checkpoint (selects and authorizes
                                                the owner; NOT ton618-core -
                                                see "Dependency-owner analysis")
        |
        v
canonical internal diagnostic carrier       (§6 - Decisions B/C/D combined,
                                              built in the crate that
                                              checkpoint selects)
        |
        v
versioned external machine schema           (§6, separate document/versioning policy)
        |
        v
CLI structured output
        |
        v
diagnostics-only LSP
```

Confirmed order-preserving reason for each edge: `#1670` must land before
`#1698`/`#1699` because they currently share one function
(`check_source_with_profile`) whose control flow `#1670`'s fix itself
restructures - repairing position-preservation first would mean repairing
it twice. `#1697`/`#1698`/`#1699` must land before catalog coherence
because a position/identity fix can itself introduce or retire
diagnostic-code call sites the catalog check would otherwise need to
track through churn. Catalog coherence must land before the carrier
because the carrier's Decision-B-mandated code/severity/family fields are
only as trustworthy as the registry they are validated against. The
governance/ownership checkpoint must land before the carrier
implementation itself because the carrier cannot be built in a crate
that has not yet been selected and authorized - see "Dependency-owner
analysis" above for why `ton618-core` cannot fill that role. The
carrier should still land before the external schema, but not because
"the schema versions the carrier's shape" - §6 explicitly forbids that
framing (internal model and external schema are two independently
versioned artifacts, never one struct serialized two ways). The real
reason is authority and convergence: the external schema is a separately
versioned *projection* of canonical diagnostic semantics, and it cannot
honestly project semantics (Decision B's code/severity/family, Decision
C's file identity, Decision D's source range) that the internal carrier
has not yet settled. Building the schema first would mean guessing at a
projection of a model that does not exist yet, then reconciling drift
after the fact - the same "build a nicer-looking policy after the fact"
anti-pattern the cross-cutting rejected alternatives above already name.

**Scope note (owner review round 6)**: this DAG covers the
currently-filed issues only. Decision A's evidence pass found three
same-defect-class instances that are not covered by `#1670`'s filed
scope: `cmd_check`'s `.or_else` discarding a real multi-module load
failure in favor of a narrower single-file check; `compile_program_to_ir_with_options_and_profile`'s
`Auto` branch swallowing a Logos parse/policy failure via
`unwrap_or(false)`; and that same function's unconditional Logos-parser
invocation even under an explicit `RustLike` request. All three must be
judged against Decision A's invariant when repaired. **Owner has
approved these as two separate, post-merge-only issues** (see "Durable
tracking for newly discovered defects" below for the exact drafted text)
- not folded into `#1670`, and not created by this document or before
`#1918` merges.

## Durable tracking for newly discovered defects

Proposed for owner approval - **not created**. Two candidate issues,
kept separate because they sit in different crates with independent
repair and qualification surfaces:

---

**Proposed issue A**

> **Title**: `smc check` discards a real multi-module project failure in
> favor of a narrower single-file fallback
>
> **Body**: `cmd_check` (`crates/smc-cli/src/app.rs:588-589`) calls
> `check_file_with_provider_and_profile` first - a strict, Logos-only
> multi-module project loader (`crates/sm-sema/src/std_adapters.rs:134-171`,
> `load_module_recursive` returns a real `Err`/`E0239` on any non-Logos
> or otherwise invalid module, with no fallback internally). On **any**
> error from that whole multi-module load - including a genuine
> cross-module import error unrelated to grammar choice - `cmd_check`
> discards it via `.or_else(|_| check_source_with_profile(&src,
> &parser_profile))` and silently retries as a single-file,
> non-project check of only the root source. For a multi-file project
> whose entry point is not valid standalone Logos, this means `smc
> check <project-root>` can silently validate only the entry file in
> isolation, never surfacing the discarded project-level error and
> never checking imported modules against it. Found during the SSF09-E1
> Diagnostic Authority Decision's Source Surface Authority evidence
> pass (`docs/roadmap/stable_foundation/ssf09_diagnostic_authority_decision.md`,
> Decision A, path 1); not covered by `#1670`'s filed scope. Repair
> must follow that decision's fail-closed invariant: an authoritative
> multi-module load failure must never be discarded in favor of a
> narrower mechanism.

---

**Proposed issue B**

> **Title**: `compile_program_to_ir_with_options_and_profile` probes
> Logos unconditionally (including under explicit `RustLike`) and
> swallows a real Logos failure under `Auto`
>
> **Body**: `crates/sm-ir/src/legacy_lowering.rs:1201-1220` computes
> `logos_detected` via `parse_logos_program_with_profile(input,
> parser_profile).map(...).unwrap_or(false)` **unconditionally**, for
> every `CompileProfile` value, before `profile` is consulted for this
> purpose. Two independent defects follow: (1) under `CompileProfile::Auto`,
> a genuine Logos parse/policy failure collapses to `false` via
> `unwrap_or`, and execution silently falls through to RustLike
> lowering on the same input without surfacing the real failure - the
> same defect class as `#1670`, in a different function; (2) under
> explicit `CompileProfile::RustLike`, the Logos parser is invoked at
> all, with any error discarded, before `profile` is ever branched on -
> a real probe of the non-selected grammar under an explicit
> single-grammar request, independent of whether its result currently
> changes `RustLike`'s outcome. Found during the SSF09-E1 Diagnostic
> Authority Decision's Source Surface Authority evidence pass
> (`docs/roadmap/stable_foundation/ssf09_diagnostic_authority_decision.md`,
> Decision A, path 3); not covered by `#1670`'s filed scope. Repair must
> follow that decision's fail-closed invariant: explicit `RustLike` must
> not invoke the Logos parser at all, and `Auto`'s classification must
> distinguish a real failure from "no Logos content" instead of
> collapsing both to `false`.

---

**Owner decision recorded**: approved as two separate issues (different
crate/path/root-cause/repair-surface for each - `smc-cli`'s project-vs.
-single-file authority loss for A, `sm-ir`'s `CompileProfile` dispatch
and grammar-probe swallow for B - not folded into `#1670`, which is
`sm-sema`-scoped). **Neither is created by this PR or before it merges.**
After `#1918` merges, create both using the issue text above verbatim
(or as refined at creation time) and record their issue numbers in the
SSF-09 progress evidence (e.g. a follow-up comment on `#1580`), rather
than reopening this document or its exact HEAD solely to add issue
numbers.

## Exit gate

**CONTRACT FROZEN is split into two independent verdicts - it is not one
yes/no answer:**

```
DECISIONS A-D (source surface / diagnostic identity /
file identity / source range authority):           FROZEN
CANONICAL-CARRIER OWNER SELECTION:                  NOT FROZEN
```

**Decisions A-D: FROZEN.** Each is internally complete (current state /
decision / invariant / why / rejected alternatives / implementation +
test consequence, all above); the canonical-carrier *boundary* (internal
model vs. external schema, §6) and the verifier/runtime mapping law are
stated precisely enough to judge a future PR against, independent of
which crate ends up building them.

**Canonical-carrier owner selection: NOT FROZEN, by design.** Re-reading
`ton618_compatibility_perimeter_scope.md` found that document's own
closed governance track disqualifies `ton618-core` outright for any new
ownership role, including a diagnostic carrier. An exhaustive pass over
every architecturally-relevant existing crate - `sm-profile`, `sm-front`,
`sm-sema`, `sm-ir`, `sm-emit`, `sm-format`, `sm-runtime-core`, and
`ton618-core` - under three independent criteria (dependency-graph
correctness, ownership-policy consistency, responsibility cohesion; see
"Dependency-owner analysis") found none satisfies all three; a new
`sm-*` Construction-zone leaf crate is the leading direction, not a
selection. This is not a gap left by oversight - it is the correct,
honest output of re-evaluating the evidence, and it means a separate
governance/ownership checkpoint (not this document, not an
implementation PR) must select and authorize the final owner before any
carrier code is written.

**Public claim check**: this document does not claim source spans are
already fully qualified (Decision D states the opposite precisely), does
not claim one canonical carrier already exists (§6/dependency analysis
state the opposite - `Diagnostic<M>` is dead code and structurally
unreachable from half the pipeline), does not claim machine-readable
diagnostics already exist (unaffected by this decision, still `ABSENT`
per the SSF09-E0 reconnaissance), does not claim verifier/runtime already
map every error to source (the mapping law's own "if and only if" clause
states today's real answer is "never, in every current production case"),
does not claim an LSP exists, does not claim a carrier owner has been
selected, does not touch the formatter's already-accurate narrow-scope
claim in `docs/spec/source_style.md`, does not claim a bare
candidate-probe rejection with no prior positive evidence is
authoritative (corrected this round - Decision A's law now requires
positive evidence first), does not claim `ton618_core::Span`/`SourceMark`
is the future canonical range type or that `ton618-core` gains a new
range-carrying role (corrected this round - Decision D cites them only
as current/legacy evidence), does not claim a successful
`DecodedDebugSymbol` lookup by itself proves a canonical source range
(corrected this round - it lacks file identity and byte offsets), does
not claim a CWD-relative path is a stable external file identity, does
not claim a bare file name/basename is a collision-free external
identity either (corrected this round - a rootless standalone source's
identity is absent, with a file name available only as a separate,
non-canonical `display_name`), does not claim a bare project-root-relative
path is unique across dependency package boundaries (corrected this
round - external identity for a package-admitted module is scoped by
`package_name` in addition to `module_path`), does not claim a
classifier that stops at the first positive-evidence candidate without
checking for a conflicting second one satisfies the surface-
classification law (corrected this round - renamed to UNIQUE POSITIVE
SURFACE CLAIM to close that reading), does not claim `#1704`'s repair
grants `ton618-core` canonical registry ownership (Decision B states it
explicitly as qualification-only), does not claim a future LSP adapter
converts canonical file identity to a URI as if that identity is always
present and always filesystem-shaped (corrected this round - the
Presentation/LSP boundary now separates canonical logical identity,
which may be absent, from a client-supplied transport/document locator,
which is not compiler semantic authority), and does not claim every
authority's fail-closed outcome is "absent" (corrected this round - the
Fail-closed rules section now states Decision A's outcome is a reported
classification error and Decision B's is preservation, not absence;
"absent" is specifically Decisions C and D's outcome).
No historical document is rewritten by this decision; the TON618
perimeter's own closure record is read, not altered.

**Sequencing implication**: a future implementation checkpoint may
proceed directly from Decisions A-D and the repair DAG for the
currently-filed issues (`#1670`, `#1697`, `#1698`, `#1699`, `#1704`)
without a further architecture pass on those. It may **not** proceed to
carrier implementation until the separate governance/ownership
checkpoint above has run - that is a hard sequencing gate, not a
recommendation.

**Wait for owner review and a separate implementation GO.**
