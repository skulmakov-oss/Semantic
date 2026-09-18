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
   A source file containing **only** `import`/`pulse`/`profile`
   directives therefore parses as
   `Ok(LogosProgram { system: None, entities: vec![], laws: vec![] })`
   - a **successful** Logos parse that the `system.is_some() ||
   !entities.is_empty() || !laws.is_empty()` discriminator cannot see at
   all, so it silently falls through to RustLike without ever
   representing that this input was also recognized Logos syntax. This
   is a distinct failure mode from the swallowed-`Err` case above: the
   parse here does not fail at all, it succeeds and is invisible to the
   discriminator. **Correction (E1A, round 3)**: the discriminator's
   blindness to `import`/`pulse`/`profile` content is real and is the
   defect for both - but its correct repaired outcome differs by which
   keyword is present, and round 1/2's own fixes to this same document
   had already established that difference elsewhere without this
   paragraph being swept to match. For a bare `pulse`-only or
   `profile`-only file: the correct repaired outcome **is** "recognize
   it as Logos" - these keywords are Logos-exclusive (see the corrected
   UNIQUE POSITIVE SURFACE CLAIM law below), so the fix is exactly
   surfacing what the discriminator currently discards. For a bare
   `import`-only file: `import` is *shared vocabulary*, so the correct
   repaired outcome depends on the concrete content, not on "recognize
   it as Logos" - the confirmed dual-admissible instance is specifically
   `Import "a.sm"` (a quoted string, satisfying both grammars' actual
   requirements to completion), which must be reported as an explicit
   conflict rather than resolved silently toward either grammar; an
   import-only file whose content RustLike's own `parse_import_decl`
   would reject (e.g. an unquoted path) has no RustLike claim to
   conflict with and is UNIQUE POSITIVE LOGOS CLAIM instead. This
   paragraph does not generalize "bare import-only" to mean "always
   ambiguous" - see the corrected law below for the precise rule.
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
exactly what lets the confirmed dual-admissible import case above
(`Import "a.sm"`) stay misclassified, and separately what lets a bare
`pulse`/`profile`-only file stay misclassified as "empty," because
neither is an error nor genuinely empty of recognized Logos content.
Any future classifier (wherever `Auto` or an implicit probe is
implemented) MUST distinguish three outcomes, semantically, not just as
an implementation detail:

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
  without any conflict check does not).

  **CORRECTION NOTE (owner review round - E1A, discovered during E2
  implementation reconnaissance, before any production code was
  written)**: this section previously claimed that *any* successful
  parse of a Logos-recognized declaration form - explicitly including
  import-only Logos source ("a successful parse that only consumed
  `import`/`pulse`/`profile` directives is a positive Logos claim") -
  by itself constitutes *unique* positive evidence. That is false for
  `import`, and the error is specifically an over-generalization from
  "this is valid Logos syntax" to "this is uniquely Logos," which does
  not follow when the same syntax is *also* valid under the other
  candidate grammar. Verified directly against both parsers at this
  exact baseline: `crates/sm-front/src/parser.rs:89-135`
  (`parse_program`) lists `TokenKind::KwImport` as one of RustLike's own
  seven recognized top-level forms, and `parse_import_decl`
  (`parser.rs:137-160`) accepts exactly `Import <optional pub>
  <string-literal> <optional as/*/{}>` - a shape a real Logos fixture
  already in this codebase satisfies verbatim
  (`crates/sm-sema/src/std_adapters.rs:995`: `Import "math::core.sm"`).
  A bare top-level `Import "a.sm"` and nothing else therefore parses to
  completion, successfully, under **both**
  `parse_logos_program_with_profile` (Logos's `KwImport`/`KwPulse`/
  `KwProfile` handling requires only `CompatibilityMode::LegacySupport` -
  which is exactly what `cli_profile()` sets via `ParserProfile::
  foundation_default()`, `crates/smc-cli/src/app.rs:81-83` - and then
  unconditionally skips to the next newline with no content validation
  at all) **and** `parse_program_with_profile` (RustLike's own structured
  import declaration). `KwImport` is the *only* keyword shared between
  the two grammars - confirmed by checking that `KwSystem`/`KwEntity`/
  `KwLaw`/`KwPulse`/`KwProfile` never appear in RustLike's parser and
  RustLike's own top-level keywords (`enum`/`fn`/`record`/`schema`/
  `trait`/`impl`) never appear in Logos's.

  **CORRECTION NOTE (owner review round - E1A, round 1)**: the first
  draft of this correction over-generalized the collision from `import`
  (the one genuinely shared keyword) to `import`/`pulse`/`profile` as a
  single category, in several places implying `pulse` and `profile`
  alone are also merely "shared" evidence requiring combination with
  `system`/`entity`/`law` to become unique. That is wrong and
  self-contradicts this same section's own dependency-graph-style
  evidence a few lines above: `KwPulse`/`KwProfile` never appear in
  RustLike's parser at all (only `KwSystem`/`KwEntity`/`KwLaw`/
  `KwImport`/`KwPulse`/`KwProfile` are Logos's six top-level forms;
  RustLike's own seven are `Import`/`enum`/`fn`/`record`/`schema`/
  `trait`/`impl`/role-marked-schema-decl). `KwImport` is confirmed the
  **only** keyword the two grammars share. A bare, valid `Pulse ...` or
  `Profile ...` top-level directive, with nothing else, is therefore
  already Logos-exclusive positive evidence on its own - RustLike has no
  grammar rule that could ever accept it, so there is no second
  candidate to conflict with. Only `import` needs the ambiguity
  treatment.

  **Corrected law**: a declaration/token being *valid* Logos syntax does
  NOT by itself imply it is *unique* Logos evidence - but this only
  bites for evidence that is *actually* shared with another candidate
  grammar, not for every Logos-recognized form. The general rule, stated
  once for any current or future shared surface vocabulary:
  - Logos-exclusive positive evidence - `system`/`entity`/`law`, **or
    `pulse`/`profile` on their own, with no combination requirement** -
    with no RustLike-positive evidence -> UNIQUE POSITIVE LOGOS CLAIM.
  - RustLike-exclusive positive evidence (`enum`/`fn`/`record`/`schema`/
    `trait`/`impl`, or a role-marked schema declaration) with no
    Logos-positive evidence -> UNIQUE POSITIVE RUSTLIKE CLAIM.
  - **`import` requires a distinct, sharper rule than the other two,
    because `KwImport` being *shared vocabulary* does not by itself mean
    every `import`-led input is ambiguous.** `KwImport`'s mere presence
    only means BOTH grammars have a rule that *could* apply - whether
    either rule actually accepts the concrete input still depends on
    what follows. **CORRECTION NOTE (owner review round - E1A, round 2)**:
    round 1's wording collapsed this distinction, stating that `import`
    "when it is the only positive evidence" is unconditionally
    AMBIGUOUS/CONFLICTING - that overshoots what the grammars actually
    prove. Verified precisely: RustLike's `parse_import_decl`
    (`crates/sm-front/src/parser.rs:137-160`) requires a specific
    structured shape after `Import`/optional `pub` - a **string-literal
    token specifically** (`expect_string_literal_text`,
    `parser.rs:3530-3540`, which errors outright if the next token is
    not `TokenKind::String`), then optional `as`/`*`/`{...}`. Logos's
    `KwImport` handling has no equivalent content requirement at all -
    it accepts anything up to the next newline once legacy-compatibility
    passes. So an `Import`-led line whose content does **not** satisfy
    RustLike's structured shape (e.g. an unquoted/bare path with no
    string literal) is accepted by Logos but genuinely **rejected** by
    RustLike's own parser - that is not a conflict, it is a real
    RustLike parse failure with nothing to be ambiguous against, and
    correctly falls to UNIQUE POSITIVE LOGOS CLAIM (Logos is the only
    grammar that produced any claim for that specific content) or, if
    the caller is instead attempting `RustLike` explicitly, an ordinary
    RustLike parse error - never an invented ambiguity. The correct
    rule is therefore about the **concrete source**, not the keyword
    category: an `import`-led input is AMBIGUOUS/CONFLICTING only when
    that specific input independently satisfies both grammars' full
    admission requirements to completion - confirmed concretely for
    `Import "a.sm"` (a quoted string literal, satisfying both Logos's
    permissive skip and RustLike's structured requirement) and NOT
    asserted, by this decision, for `import`-led content in general. If
    `import` appears *alongside* genuinely Logos-exclusive evidence in
    the same input (e.g. a file with both a `Law` declaration and an
    `Import` line), the Logos-exclusive evidence already establishes
    UNIQUE POSITIVE LOGOS CLAIM for the whole input regardless of what
    the `import` line's own dual-parseability would otherwise be.

  This corrected law still fully covers what actually motivated the
  original wording: the current discriminator
  (`system.is_some() || !entities.is_empty() || !laws.is_empty()`) has
  two, distinct forms of blindness, not one: (1) it cannot observe
  `import` evidence at all, so it cannot represent the confirmed
  `Import "a.sm"` concrete-ambiguity case (see below); (2) it cannot
  observe `pulse`/`profile` evidence at all, so it cannot recognize a
  bare `pulse`/`profile`-only file as the UNIQUE POSITIVE LOGOS CLAIM it
  actually is. Both are `#1670`'s own filed observation and remain a
  real, valid defect (see the repair-direction correction below) - the
  fix is just that the *previously expected* correct classification for
  a bare import-only file was itself wrong in its own way each round:
  first it was claimed unconditionally unique to Logos (pre-E1A), then
  unconditionally ambiguous merely for sharing a keyword (E1A round 1);
  the correct rule is neither - it depends on whether the concrete input
  is actually admitted by both grammars, which for the confirmed
  `Import "a.sm"` case, it is.

  **This correction concerns the implicit `Auto` classification path
  only, and states a normative requirement, not a description of
  already-conforming production code.** By the frozen INVARIANT below,
  an explicit `CompileProfile::Logos` or `CompileProfile::RustLike`
  request MUST bypass Auto surface classification entirely (explicit
  `RustLike` must not probe Logos; explicit `Logos` must not probe
  RustLike) - conceptually, a caller who explicitly names the surface
  for a bare `import`-only file gets that surface by caller authority,
  with no ambiguity question to resolve. **Current production code does
  not yet satisfy this for path 3**: `compile_program_to_ir_with_options_and_profile`
  still unconditionally invokes the Logos parser even under explicit
  `CompileProfile::RustLike` (see this Decision's own path-3 evidence
  above) - that is a separate, already-tracked production defect
  ([#1920](https://github.com/skulmakov-oss/Semantic/issues/1920)), not
  something this correction is claiming is already fixed. Ambiguity
  under `Auto` is a concern specifically because no caller-supplied
  authority exists to break the tie there, which today is exactly the
  implicit-`Auto` `check_source_with_profile` path `#1670` is filed
  against.

  Reaching UNIQUE POSITIVE SURFACE CLAIM (for either grammar, under the
  corrected law above) ends classification: no other surface may be
  attempted, and precisely two sub-outcomes follow - **success**
  (parsing/policy subsequently accepts the input under the owning
  surface), or **AUTHORITATIVE FAILURE** (parsing/policy subsequently
  rejects it under the owning surface; that failure is preserved as the
  outcome for this input, and it alone - never a bare probe rejection
  with no prior positive evidence - is what the swallowed-`Err` half of
  `#1670`, and path 3's `unwrap_or(false)`, currently destroy).
- **AMBIGUOUS / CONFLICTING CLAIMS** - more than one candidate surface
  independently produces sufficient positive evidence **for the same
  concrete input** (evaluated on what that specific input actually
  parses as, not merely on which keyword introduces it), or the
  classifier cannot deterministically resolve which surface owns it.
  **A concrete, confirmed instance of this outcome (not merely
  hypothetical) is the bare top-level source `Import "a.sm"` and nothing
  else**, which parses to completion under both `parse_logos_program_with_profile`
  and `parse_program_with_profile` - see the corrected UNIQUE POSITIVE
  SURFACE CLAIM law above for the evidence, including why this does
  **not** generalize to "every `import`-led input is ambiguous": `KwImport`
  being vocabulary shared between the grammars only means both *could*
  claim an input, not that both *do* for any given one - a Logos-accepted
  `import` line whose content RustLike's own `parse_import_decl` would
  reject (e.g. an unquoted path, since RustLike requires a string
  literal specifically) produces no RustLike claim to conflict with, and
  is UNIQUE POSITIVE LOGOS CLAIM instead. A bare `pulse`-only or
  `profile`-only file, separately, has no RustLike-side counterpart at
  all (those keywords don't exist in RustLike's grammar) and is UNIQUE
  POSITIVE LOGOS CLAIM, never this outcome. This outcome must remain
  genuinely reachable by the classifier, not
  defined away by an implementation that stops evaluating after the
  first candidate satisfies UNIQUE POSITIVE SURFACE CLAIM's evidence
  threshold without checking uniqueness. This is itself a deterministic
  classification error, reported as such - **never** resolved by
  picking a "winner" through evaluation order, fallback, or any other
  implicit tie-break. Folding ambiguity into "no match, try the next
  candidate" (an earlier draft's error) is precisely the
  guessed-source-surface outcome the "Fail-closed rules" section below
  already forbids.

**CORRECTION NOTE (owner review round - E1A, round 2)**: an earlier
draft said this decision freezes only "the semantic distinction," not
"which exact declaration forms count toward a positive claim beyond
`system`/`entities`/`laws`/imports" - that's now stale, since this same
correction round explicitly freezes `pulse`/`profile`'s evidence status.
Restated precisely:

**FROZEN by this decision**:
- the three-outcome semantic classification law itself (NO SURFACE
  CLAIM / UNIQUE POSITIVE SURFACE CLAIM / AMBIGUOUS-CONFLICTING CLAIMS);
- the evidence status of every currently-evidenced top-level form:
  `system`/`entity`/`law`/`pulse`/`profile` are Logos-exclusive;
  `import` is shared vocabulary whose ambiguity depends on the concrete
  input, not the keyword alone (confirmed concretely for
  `Import "a.sm"`); `enum`/`fn`/`record`/`schema`/`trait`/`impl`/a
  role-marked schema declaration are RustLike-exclusive.

**NOT FROZEN by this decision**:
- the concrete classifier's Rust type or helper placement;
- its scanning/parsing implementation strategy or evaluation mechanics
  (e.g. whether `import`'s dual-admissibility is checked by attempting
  both parsers, by a lighter-weight grammar check, or some other means);
- treatment of any future declaration form this decision has no
  evidence about - such a form's evidence status would need its own
  evidence pass, not an assumption from this list.

This decision does not freeze a first-token classifier (dispatch on
which keyword appears first) as the implementation strategy - the
`import` case specifically requires evaluating the concrete input, not
just its leading keyword, and no implementation choice for *how* that
evaluation happens is made here.

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
  shared discriminator**, which has two distinct, separate blind spots -
  **CORRECTION NOTE (owner review round - E1A, round 2)**: an earlier
  draft of this note collapsed both into one `import`/`pulse`/`profile`
  category and re-introduced the "combined with genuinely Logos-exclusive
  evidence" requirement the corrected law above already rejects for
  `pulse`/`profile` alone. Restated as two independent facts:
  - it cannot observe `import` evidence at all, so it cannot represent
    the confirmed concrete ambiguity of `Import "a.sm"` (per the
    corrected UNIQUE POSITIVE SURFACE CLAIM law above, this is
    AMBIGUOUS/CONFLICTING specifically because that exact content is
    independently valid RustLike too - not because `import` is a shared
    keyword in general);
  - it cannot observe `pulse`/`profile` evidence at all, so it cannot
    recognize that a bare `pulse`-only or `profile`-only file is already
    UNIQUE POSITIVE LOGOS CLAIM on its own, with no combination
    requirement.

  The discriminator's actual defect in both cases is being blind to
  that content's existence at all - not misclassifying which corrected
  outcome applies once the content is actually observed. See path 2's
  evidence above and `#1670`'s own filed text.

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
attempt's outcome using the corrected law above (NO SURFACE CLAIM /
UNIQUE POSITIVE SURFACE CLAIM-with-success-or-AUTHORITATIVE-FAILURE /
AMBIGUOUS-CONFLICTING CLAIMS, where `KwImport` is *shared vocabulary* -
not itself shared *evidence* - and a concrete `import`-led input's
ownership/ambiguity must be derived from that specific input's actual
admissibility under each grammar, never inferred from the keyword's
presence alone; `Import "a.sm"` is the confirmed ambiguous fixture, not
a stand-in for "every import-led input." Bare `pulse`/`profile` alone
count as Logos-exclusive *unique* evidence unconditionally, needing no
combination with `system`/`entity`/`law`) - not the two-state split an
earlier draft of this decision used, not the broader "or is being
classified" framing a later draft used, not the "import-only is
automatically unique Logos evidence" claim a still-later draft used,
not the "import/pulse/profile are all equally shared" over-generalization
a first E1A draft introduced, and not the "import, whenever it is the
sole evidence, is unconditionally ambiguous" over-correction a second
E1A draft introduced - all before this correction - before deciding
whether to attempt RustLike; a classifier satisfying this law does NOT
dispatch on first token alone. That classification must NOT treat a
bare probe
rejection with no prior positive evidence as authoritative (an ordinary
`RustLike` program failing a Logos probe at the first token must still
allow `RustLike` to be attempted), and must NOT treat a bare
`import`-only file as owned by either grammar when it is simultaneously
valid RustLike syntax - that specific input must surface as a
classification error instead, while a bare `pulse`-only or
`profile`-only file (which RustLike's grammar never recognizes at all)
is owned by Logos outright with no such error;
`compile_program_to_ir_with_options_and_profile` must stop invoking
`parse_logos_program_with_profile` at all when `profile` is explicitly
`RustLike` (not merely stop acting on its result) - this is `#1920`'s
own scope, not yet performed - and its `Auto` branch needs the same
corrected classification instead of `unwrap_or(false)`; and
`cmd_check`'s `.or_else` must stop discarding a real multi-module load
failure in favor of a silently-narrower single-file check (`#1919`'s
scope). Whether these are one filed issue or several is an
implementation-sequencing question, not a decision-authority question -
out of scope here; see "Durable tracking for newly discovered defects"
below.

**TEST CONSEQUENCE** (not performed here): a regression per path proving
that a genuinely invalid (not merely empty) grammar-specific input is
rejected with a diagnostic attributed to the surface that actually failed,
and never silently succeeds under a different surface or a narrower
mechanism - covering `check_source_with_profile`,
`compile_program_to_ir_with_options_and_profile`'s `Auto` branch, and
`cmd_check`'s multi-module-to-single-file fallback independently. Plus
these additional, distinct regressions: (1) for explicit `RustLike`,
`compile_program_to_ir_with_options_and_profile(input, CompileProfile::RustLike,
..)` must never invoke the Logos parser at all for any input, verified
directly (not merely that its result is ignored); (2) **corrected (E1A,
round 1)** - a bare source file containing **only** an `import`
directive (e.g. `Import "a.sm"`, the confirmed concrete collision, with
no `system`/`entity`/`law`/`pulse`/`profile`), which is simultaneously a
complete valid RustLike program, must be rejected with a deterministic
classification-ambiguity outcome - never silently resolved to Logos,
never silently resolved to RustLike; (2a) **separately, and distinctly**
- a bare source file containing **only** a `pulse` or `profile`
directive (with no `import`/`system`/`entity`/`law` either) has no
RustLike-side counterpart at all and must reach UNIQUE POSITIVE LOGOS
CLAIM on its own, with no combination requirement - this is not the same
regression as (2) and must not be conflated with it; (2b) a source
containing genuinely Logos-exclusive evidence (`system`/`entity`/`law`)
alongside an `import` line must still reach UNIQUE POSITIVE LOGOS CLAIM
for the whole input, proving `import`'s ambiguity is specific to being
the *sole* evidence present, not a property of the keyword in
combination; (3) an ordinary, valid `RustLike` program (e.g. `fn main()
{ return; }`) must still compile successfully under `Auto` even though a
Logos probe against it fails immediately with no prior positive evidence
- proving the classifier does not treat every candidate-probe rejection
as authoritative; (4) more generally, any input that independently
satisfies both grammars' positive-claim conditions (the confirmed
bare-`import` case in (2) is the concrete instance found so far, but the
requirement is general) must be rejected with an explicit
classification-ambiguity diagnostic, never silently resolved to
whichever grammar was tried first.

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
- **CORRECTION NOTE (owner review round - E1A, round 3)**: this
  requirement previously said `#1670`'s fix "does not change any
  admitted program's compilation result - only failure-classification
  behavior." That is no longer compatible with this decision as
  corrected: the corrected law explicitly requires the confirmed
  `Import "a.sm"` collision to become a deterministic AMBIGUOUS/
  CONFLICTING classification error under implicit `Auto`, replacing
  its current RustLike-side terminal failure (see the E1B correction
  note below for the precise baseline evidence) - a real, intentional
  change to that specific input's result, not a regression. The
  requirement is restated more precisely:
  - Inputs with exactly one authoritative surface (a UNIQUE POSITIVE
    SURFACE CLAIM under the corrected law) must preserve their existing
    admitted semantic result - an ordinary valid `RustLike` program, or
    a Logos program uniquely owned by `system`/`entity`/`law`/`pulse`/
    `profile` evidence, must not regress.
  - Inputs whose current result exists **only** because today's
    fail-open dispatch silently resolves a real authority conflict
    toward one grammar are explicitly **not** covered by that
    preservation guarantee - their result is expected, and required,
    to change, regardless of whether that current result is itself a
    success or a failure attributed to the wrong authority.
  - **CORRECTION NOTE (owner review round - E1B)**: this bullet
    previously described the confirmed `Import "a.sm"` collision as
    "currently admitted" via "silent-success-via-RustLike-fallback."
    Baseline evidence, re-verified directly against
    `check_source_with_profile` on this decision's own merged-main SHA
    (`262adc00369e23fb1ba92e5f093656041e983aa9`), shows that
    description is factually wrong: Logos parses `Import "a.sm"`
    successfully, but today's discriminator cannot see import-only
    evidence (`system`/`entities`/`laws` are all empty) and silently
    discards it, falling through to RustLike; RustLike's own parse of
    the same input also succeeds (one import declaration), but the
    subsequent `type_check_program` call then rejects the resulting
    program with `E0201` ("program must define fn main()"), since a
    bare `Import "a.sm"` file has no `main`. The confirmed
    `Import "a.sm"` collision is the concrete instance: its current
    RustLike-side terminal failure (`E0201`, missing `fn main`, reached
    only after Logos's own successful parse is silently discarded) is
    expected to become a deterministic AMBIGUOUS/CONFLICTING
    classification error, and that change must not be treated as a
    regression to guard against.
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

**CORRECTION NOTE (owner review round - E1A, round 1)**: since this
document merged, both issues below were actually created post-merge and
are now tracked separately: **[#1919](https://github.com/skulmakov-oss/Semantic/issues/1919)
(issue A) and [#1920](https://github.com/skulmakov-oss/Semantic/issues/1920)
(issue B) are both OPEN.** The text below is kept as the original,
frozen proposal record (what was approved and why); it is bookkeeping
history now, not a pending action.

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
`sm-sema`-scoped). **Created post-merge as `#1919` (issue A) and
`#1920` (issue B), both currently OPEN**, matching the approved,
separate scoping above; their issue numbers were recorded in a
follow-up comment on `#1580`, per the original plan, rather than
reopening `#1918`'s exact HEAD solely to add them. This PR does not
create, edit, or otherwise mutate either issue.

## Decision E - Surface Admission Contract

**Status: FROZEN - owner-approved architecture contract**, after four
owner review rounds (see "Audit history" under "Frozen decision" below
for what each round found and corrected), **plus one round-5 precision
amendment** (this revision) discovered during Stage 1 implementation of
this same contract (PR `#1925`) - see the "Round 5" audit-history entry
and the two paragraphs marked "FROZEN by owner ruling, round 5" below.
This section is a new architectural decision added after Decisions A-D
above were already frozen and merged (via `#1918`/`#1921`/`#1922`); it
does not reopen or reinterpret any of them. **Freezing this contract
does not authorize production implementation**: `sm-front` and
`sm-sema` remain untouched by this checkpoint, and implementing
`admit_program_with_profile`/`admit_logos_program_with_profile` (or
rewriting `sm-sema`'s `check_source_with_profile` on top of them) each
requires its own, separate, future implementation-only checkpoint GO -
exactly as "Migration plan" below already specifies and has specified
since this section's first draft. **This amendment is itself
docs-only**: it corrects the frozen contract's own text before PR
`#1925` resumes implementing it; `sm-front` is not touched by this
revision, and PR `#1925`'s existing code is deliberately left
unmodified pending it.

### Motivating evidence

`#1670`'s implementation (PR `#1923`) went through four correction
rounds, each fixing one class of false classification in `sm-sema`'s
own from-scratch reconstruction of "which grammar owns this top-level
declaration" by scanning the raw token stream, and each subsequently
found - by independent adversarial review, not by the existing test
suite or hosted CI, both of which stayed green throughout - to have
introduced or left a *different* class of false classification:

1. Round 1: scanning the whole token stream let a keyword nested inside
   the *other* grammar's own body (e.g. `Entity` inside a RustLike `fn`
   body) count as evidence.
2. Round 3: restricting evidence to tokens outside any bracket/indent
   nesting still let a keyword in a *non-head* position within a
   declaration's own header at depth 0 count as evidence (e.g. `Entity`
   as the function *name* in `fn Entity() {}`).
3. Round 3 self-review: an excess closing delimiter in malformed input
   could drive the nesting-depth counter negative, resurrecting false
   "top level" status for everything after it.
4. Round 4 (reverted): restricting the declaration-head boundary to
   only fire after a real block closes (`RBrace`/`Dedent` returning to
   depth 0) fixed round 3's residual gap, but is *itself* a regression:
   `System`, `Pulse`, `Profile`, and a bare `Import` never produce a
   `RBrace` or `Dedent` at all - so this rule permanently disables
   further evidence detection for the rest of the file after any one of
   them. Confirmed concretely:
   - `Pulse "tick"\nfn main() {\n    return;\n}\n` - genuine dual
     evidence (`Pulse` Logos-exclusive, `fn` RustLike-exclusive) -
     silently misclassified as Logos-only, reporting `"expected Logos
     declaration"` instead of the required `AMBIGUOUS/CONFLICTING`.
   - `Import "a.sm"\nEntity A:\n    state x: quad\n` under
     `allow_logos_surface = false` - genuine Logos-exclusive evidence
     (`Entity`) masked by the preceding `Import`, silently
     misclassified as no evidence at all, reporting a RustLike parse
     error instead of the required Logos policy rejection (the same
     invariant class as T2).

   Reverted in `60c1e44c`; PR `#1923` is parked at that commit
   (byte-identical to round 3) pending this decision.

Separately, the same review round found `Import "a.sm" fn main() {
return; }` (two declarations sharing one physical line with zero
separator) still produces `AMBIGUOUS/CONFLICTING` rather than
recognizing `fn` as the sole real head. This was investigated and is
**not** attributed to a defect in the classification law itself:
`parse_import_decl` requires no trailing terminator, so there is no
lexical signal at all between the two declarations, and the resulting
ambiguity is arguably consistent with the frozen law's own definition
(Logos does admit this concrete input, via its already-documented
content-blind legacy `Import`/`Pulse`/`Profile` skip - see Decision A's
current-state discussion of `parse_logos_program`). It is listed here
only as a data point for the "Cross-grammar authority matrix" section
below, not as evidence requiring a fix.

**Root cause (common to all four rounds' failures)**: different
declaration kinds terminate via genuinely different, non-uniform
lexical signals - `Newline` (Logos's legacy `Import`/`Pulse`/`Profile`
skip), `Semi` (a RustLike expression-bodied function,
`crates/sm-front/src/parser.rs:213-220`), `RBrace` (a braced body),
`Dedent` (an indented body), or **no signal at all** (a bare `Import`
with no trailing `as`/`*`/`{}`, which can be immediately followed by
another declaration on the same physical line). No single rule over
token kinds and bracket/indent depth can determine "has the current
declaration finished" without first knowing *which kind* of declaration
is currently open - and that is exactly the grammar-level knowledge
`#1670`'s own brief prohibits `sm-sema` from duplicating (STOP clause:
*"If existing public sm-front APIs do not let sm-sema establish the
necessary top-level evidence without duplicating a material part of
the grammar, introducing a brittle home-grown parser, or modifying
sm-front, STOP and report before widening scope"*). Four rounds of
concrete, reproducible counterexamples are the evidence that this STOP
condition has been reached.

### Current state (verified against baseline SHA
`c0e2e50600e418f72af156a3c57616971d391494`)

- `parse_program`'s top-level loop
  (`crates/sm-front/src/parser.rs:89-135`): single-shot
  `Result<Program, FrontendError>`. The first time any iteration hits
  the `_ => Err(...)` branch (a genuinely unrecognized top-level token)
  *or* a declaration's own sub-parser propagates an `Err` via `?`, the
  whole function returns immediately, discarding every declaration
  already successfully collected. Nothing distinguishes, from the
  returned `Result` alone, "the very first token was never a
  recognized declaration head" from "a head was recognized and its own
  content later failed."
- `parse_logos_program`'s top-level loop
  (`crates/sm-front/src/parser.rs:2954-3020`): accumulates *multiple*
  errors across iterations via `recover_logos_anchor()`-based recovery;
  discards all partial success (`out`) if `errors` is non-empty at the
  end, returning one merged `Err`; otherwise returns `Ok(LogosProgram)`.
  Same gap: nothing distinguishes "never recognized a single
  System/Entity/Law/legacy-directive" from "recognized at least one,
  but something else in the file failed."
- `FrontendError` (`crates/sm-front/src/types.rs:867-870`) is exactly
  `{ pos: usize, message: String }` - no field distinguishes these two
  cases either; the distinction currently exists only transiently, as
  an artifact of *which code path* raised the error, and is discarded
  the moment either function returns its flat `Result`.
- `sm-sema`'s `check_source_with_profile` (`#1670`, PR `#1923`)
  currently tries to reconstruct this exact missing information from
  *outside* sm-front, by scanning the already-lexed token stream for
  "declaration-head" positions using its own independently-invented
  boundary heuristic (`top_level_declaration_head_kinds`). This section
  proposes eliminating that reconstruction entirely by having sm-front
  expose the information its own parsers already have transiently.

### Frozen decision (owner-approved architecture, round 4)

Add two new, purely **additive** public functions to `sm-front`
returning a new type in place of today's flat `Result`, alongside - not
replacing - every existing public parsing function:

```rust
pub enum GrammarAdmission<T> {
    /// This grammar established no sufficient positive evidence for
    /// ownership of this input. NOT the same as "this grammar's
    /// dispatch never recognized a candidate keyword" - a shared-
    /// vocabulary candidate (e.g. `Import`) can be dispatched and still
    /// fail to cross this grammar's own sufficient-evidence threshold
    /// (see `Import foo.bar` under RustLike, below), in which case the
    /// correct result is still `NoClaim`.
    NoClaim,
    /// This grammar established sufficient evidence *only* through
    /// vocabulary shared with the other grammar (`Import`, currently
    /// the only member). Carries the grammar's own parse outcome for
    /// the whole input.
    Shared(Result<T, FrontendError>),
    /// This grammar established sufficient evidence through at least
    /// one declaration form exclusive to it (`System`/`Entity`/`Law`/
    /// `Pulse`/`Profile` for Logos; `enum`/`fn`/`record`/`schema`/
    /// `trait`/`impl`/role-marked-schema for RustLike) anywhere in the
    /// input. Carries the grammar's own parse outcome for the whole
    /// input.
    Exclusive(Result<T, FrontendError>),
}

pub fn admit_program_with_profile(
    source: &str,
    tokens: &[Token],
    profile: &ParserProfile,
) -> GrammarAdmission<Program>;

pub fn admit_logos_program_with_profile(
    source: &str,
    tokens: &[Token],
    profile: &ParserProfile,
) -> GrammarAdmission<LogosProgram>;
```

**Source-context parameter (FROZEN by owner ruling, round 5)**: both
signatures take `source: &str` in addition to `tokens: &[Token]` - this
supersedes the original round-1..4 signature (`tokens` + `profile`
only), discovered incomplete during PR `#1925`'s Stage 1 implementation
(see "Round 5" audit history below for the concrete failing evidence).
`source` is **diagnostic context only**, governed by one precondition
and three prohibitions:

- **Precondition**: `source` MUST be the exact source text `tokens` was
  produced from (i.e. `tokens == lex(source)`, or the equivalent
  already-lexed value). Callers that lex once and reuse `tokens` across
  both `admit_*` calls (the whole reason `&[Token]` was chosen over
  `&str` in the first place - see below) already have `source` on hand
  from that same lex call; this adds no new lexing burden.
- `source` MUST NOT be re-lexed by either `admit_*` function.
- `source` MUST NOT participate in evidence classification - it has no
  bearing on `NoClaim`/`Shared`/`Exclusive`, on where either grammar's
  shared threshold sits, or on any `Ok`/`Err` determination. `tokens`
  alone remains the sole syntactic/admission scan input, exactly as
  rounds 1-4 already froze.
- `source` MUST NOT become a second source-surface authority of any
  kind - it exists solely so a `Parser` constructed inside `admit_*` can
  populate the same source-line/caret-bearing `FrontendError.message`
  that the existing `parse_*_with_profile` functions already produce
  (via `Parser::error_at_token`/`format_parser_error_at_input`), instead
  of an empty one.

**On the precondition being unenforced (round 5 adversarial-review
finding)**: the precondition above is a caller contract, not something
`admit_*` can verify at runtime - checking it would require re-lexing
`source`, which the second prohibition already forbids. This is
deliberate, not an oversight: a caller violating it (passing `source`
that doesn't match `tokens`) is fail-safe by construction, never a
correctness or memory-safety hazard - `Parser::error_at_token`'s only
use of `source` is `SourceMap::line(...).unwrap_or_default()` feeding a
cosmetic caret-diagnostic line, which degrades to an empty or wrong
line on mismatch and nothing else; `evidence_basis`, `Ok`/`Err`, and
every other observable admission outcome are computed from `tokens`
alone and cannot be affected by what `source` contains.

The exact Rust representation above is illustrative, not frozen; the
**semantics** are what this decision fixes: a grammar-local admission
reports two independent facts - **evidence basis** (did this grammar
establish ownership through vocabulary it shares with the other
grammar, or through vocabulary exclusive to it - `NoClaim` covers
"neither") and **parse outcome** (`Ok`/`Err`, nested inside `Shared`/
`Exclusive`). Critically, a grammar-local `Err` is **not, by itself, an
authoritative failure** - see "Cross-grammar evidence resolver" below;
authority is decided only after the two grammars' evidence bases are
compared.

**Naming - conceptual firewall (FROZEN by owner ruling, round 1)**: the
type is named `GrammarAdmission<T>`, not `SurfaceAdmission<T>` as an
earlier draft of this section proposed. This is a deliberate lexical
firewall, not a style preference. `GrammarAdmission<T>` names what
**one grammar's own parser** concludes about an input in isolation;
Decision A's own frozen vocabulary - `NO SURFACE CLAIM`/`UNIQUE
POSITIVE SURFACE CLAIM`/`AMBIGUOUS-CONFLICTING CLAIMS`, and any future
`SurfaceAuthority`/`SurfaceVerdict` type naming the **cross-grammar
coordinator's** decision - must stay visually and lexically distinct
from it. Illustrative shape (naming only - `resolve_surface_authority`
and any `SurfaceVerdict` type are not designed by this decision; their
concrete shape is deferred to the implementation checkpoint in
"Migration plan" below):

```rust
let logos: GrammarAdmission<LogosProgram> = admit_logos_program_with_profile(...);
let rustlike: GrammarAdmission<Program> = admit_program_with_profile(...);
let verdict: SurfaceVerdict = resolve_surface_authority(logos, rustlike);
```

**Audit history: how this shape was reached (owner review rounds 1-5)**
- kept per the standing rule of this document not to delete correction
history, condensed rather than reproduced verbatim:

- **Round 1** proposed a flat `NoClaim`/`Accepted(T)`/
  `ClaimedButFailed(FrontendError)` enum, with an implementation sketch
  tracking `any_declaration_accepted`, set `true` only once a
  declaration's own sub-parser *succeeded*. Wrong: ownership happens
  before success or failure is known, not "success creates ownership."
- **Round 2** corrected the flag to `any_declaration_claimed`, set
  `true` universally *at dispatch* - the instant any top-level keyword
  is recognized, including the shared `Import` keyword. Adversarial
  review found this directly contradicted Decision A's own frozen
  unquoted-`Import` ruling ("produces no RustLike claim to conflict
  with... UNIQUE POSITIVE LOGOS CLAIM"): the round-2 model made an
  unquoted import `ClaimedButFailed` on the RustLike side, which the
  frozen matrix then resolved to AMBIGUOUS/CONFLICTING.
- **Round 3** deferred the shared keyword's commit point to each
  grammar's own frozen sufficient-evidence threshold (dispatch for
  Logos; `expect_string_literal_text` success for RustLike), which
  fixed the round-2 counterexample exactly. But a second adversarial
  pass found this still broke ordinary multi-declaration files: `Import
  "a.sm"` followed by genuine exclusive evidence in *either* grammar
  (`Entity`, or `fn main() {}`) produced `Accepted`+`ClaimedButFailed`
  in the frozen matrix - AMBIGUOUS/CONFLICTING - directly contradicting
  Decision A's explicit rule that "if `import` appears alongside
  genuinely Logos-exclusive evidence in the same input... the
  Logos-exclusive evidence already establishes UNIQUE POSITIVE LOGOS
  CLAIM for the whole input regardless of what the `import` line's own
  dual-parseability would otherwise be" (and, by the document's own
  stated symmetry, the mirror case for RustLike-exclusive evidence).
  Root cause: a flat `ClaimedButFailed` treats "claimed only shared
  vocabulary, then the rest of the file belongs to the other grammar"
  identically to "claimed real exclusive evidence, then genuinely
  failed" - two situations Decision A already treats as having
  different strength, which the three-state enum had no way to
  represent.
- **Round 4** is the fix for the round-1..3 failure class: evidence
  basis (`NoClaim`/`Shared`/`Exclusive`) and parse outcome (`Ok`/`Err`)
  become two independent axes instead of one flat three-state enum.
  This is not a new invention - it is Decision A's own already-frozen
  shared-vs-exclusive distinction, promoted from prose into the type so
  `sm-sema` can consume it directly instead of re-deriving it.
- **Round 5 (this revision)** is a precision correction discovered
  during PR `#1925`'s Stage 1 implementation of round 4's frozen
  contract - not a redesign of evidence strength, `Shared`/`Exclusive`
  semantics, the RustLike/Logos scan-mechanics asymmetry, or the
  cross-grammar resolver, all of which are unchanged by this revision.
  Two independent gaps were found empirically, both by a regression
  comparing `admit_logos_program_with_profile`'s nested `FrontendError`
  against `parse_logos_with_profile`'s error for the identical source:
  (1) `admit_*`, taking only `tokens` and `profile`, had no access to
  the original source text, so its `Parser` was constructed with
  `source: String::new()` - `Parser::error_at_token` reads `self.source`
  to embed a caret-diagnostic source line into `FrontendError.message`,
  so `admit_*`'s errors carried an empty line where `parse_*`'s carry
  the real one; (2) independently, an implementation-side fix for a
  *different*, related problem (a genuine policy-violation error being
  merged through `merge_logos_errors`'s "multiple parser errors (N):"
  wrapping, which strips the `"policy violation:"` message-prefix
  `FrontendError::kind()` checks for, silently downgrading it to
  `Syntax`) had used an ad hoc `errors.len() == 1` shortcut that itself
  produced a *third* message shape, matching neither the old parser's
  always-wrapped single-error format nor a policy-preserving one. This
  revision resolves both by widening the signature (source-context
  parameter, above) and by freezing an explicit policy-vs-syntax
  finalization law (below) in place of the ad hoc shortcut - see both
  paragraphs marked "FROZEN by owner ruling, round 5."

Taking `&[Token]` (never re-lexed) as the sole syntactic input, with
`source: &str` added in round 5 purely for diagnostic context: lexing
stays a prior, separate step exactly as it is today - a lex failure is
not a surface-admission outcome for *either* grammar, it is a
deterministic pre-parse failure `sm-sema` already handles on its own
(SSF09-E2 round 1's lex-error-preservation fix). A caller lexes once,
keeps both the resulting `tokens` and the `source` string it lexed them
from, and passes both to each `admit_*` call - `source` adds no new
lexing burden and does not reintroduce the redundant-re-lexing problem
this paragraph originally existed to rule out, because it is never fed
back into a lexer or a classification decision, only into
`Parser::error_at_token`'s existing caret-diagnostic formatting.

**Implementation sketch** (for a future, separate implementation
checkpoint - not authorized by this decision-only checkpoint):

Each `admit_*` function tracks one local `evidence_basis` value
(`None`/`Shared`/`Exclusive`, starting at `None`) alongside computing
the grammar's own parse outcome exactly as today's `parse_program`/
`parse_logos_program` already do (unchanged: RustLike aborts on the
first sub-parser `Err` via `?`, discarding partial success, exactly as
`Result<Program, FrontendError>` does today; Logos accumulates errors
per iteration via `recover_logos_anchor()` and merges them at the end,
exactly as today). `evidence_basis` promotion is **absorbing and
monotonic**: `Exclusive` dominates `Shared` dominates `None`, and once
reached it is never downgraded for the rest of the scan, regardless of
what any individual declaration's own sub-parser does afterward. The
final `GrammarAdmission` is `NoClaim` if `evidence_basis` stayed `None`;
otherwise `Shared(outcome)` or `Exclusive(outcome)`, where `outcome` is
the grammar's already-computed `Result<T, FrontendError>` for the whole
input.

- `parse_program`'s loop (RustLike): dispatch on any of the six
  exclusive heads or role-marked-schema promotes `evidence_basis` to
  `Exclusive` immediately, regardless of that declaration's own
  eventual success or failure - illustrated by `fn main(\n` (a
  recognized `fn` head whose own parameter list then fails to parse)
  yielding `Exclusive(Err(...))`. For `Import`: dispatch on `KwImport`
  alone does **not** promote anything. `parse_import_decl`
  (`crates/sm-front/src/parser.rs:137-173`) reaches RustLike's shared
  threshold only once `expect_string_literal_text` (`parser.rs:140`)
  *succeeds* - exactly Decision A's own already-frozen "RustLike
  requires a string literal specifically" rule, restated as a
  threshold, not a new rule. An unquoted/bare path fails that check
  *before* the threshold is reached, so `evidence_basis` is left
  unchanged (an unquoted, standalone `Import` therefore leaves
  `evidence_basis` at `None` -> `NoClaim` overall, since nothing else
  was ever recognized). Once the string literal succeeds,
  `evidence_basis` promotes to `Shared` (unless already `Exclusive`
  from an earlier or later declaration in the same file, which always
  wins per the absorbing rule); the optional `pub`/`as`/`*`/`{...}`
  clauses that follow are threshold-*internal* to `Import`'s own shared
  production, so a failure there (e.g. `Import "a.sm" as 123`) still
  only yields `Shared(Err(...))`, never promotes to `Exclusive`.
- `parse_logos_program`'s loop (Logos): dispatch on `System`/`Entity`/
  `Law`/`Pulse`/`Profile` promotes `evidence_basis` to `Exclusive`
  immediately - illustrated by `Entity\n    <malformed entity
  body/header>` yielding `Exclusive(Err(...))`. For `Import`: Decision
  A's frozen law imposes no additional structural requirement on
  Logos's own `Import` handling beyond the keyword and the
  `require_legacy_compatibility` policy check - Logos's content
  handling "accepts anything up to the next newline once
  legacy-compatibility passes," with zero further validation - so
  Logos's shared threshold for `Import` is the keyword recognition
  itself, promoting `evidence_basis` to `Shared` (unless already
  `Exclusive`) at that point, independent of whether the subsequent
  policy check passes.
- **Concrete examples, frozen by this ruling**:

  | Input | Logos | RustLike | `Auto` verdict |
  |---|---|---|---|
  | `Import "a.sm"` alone | `Shared(Ok)` | `Shared(Ok)` | AMBIGUOUS/CONFLICTING |
  | `Import foo.bar` alone (unquoted) | `Shared(Ok)` | `NoClaim` | Logos accepted (unique) |
  | `Import "a.sm"` + `Entity Player: ...` | `Exclusive(Ok)` | `Shared(Err)` | Logos accepted (unique) |
  | `Import "a.sm"` + `fn main() {}` | `Shared(Err)` | `Exclusive(Ok)` | RustLike accepted (unique) |
  | `fn main(` alone (malformed, no Logos evidence) | `NoClaim` | `Exclusive(Err)` | RustLike authoritative failure |
  | `Entity\n    <malformed>` alone (no RustLike evidence) | `Exclusive(Err)` | `NoClaim` | Logos authoritative failure |

  RustLike never establishes shared evidence merely by dispatching on
  `KwImport`; its shared threshold is not crossed before the required
  string-literal shape is satisfied - this is what makes the unquoted
  case `NoClaim` rather than a competing claim, with **zero
  `Import`-specific logic anywhere outside `admit_program_with_profile`
  itself**. `Pulse`/`Profile` remain unconditional Logos-`Exclusive`
  claims per T5's already-frozen invariant - unchanged - because
  RustLike's grammar has no production for them at all, so they are
  never `Shared` under any input.

- **Policy-gate re-examination (`require_logos_surface`,
  `require_legacy_compatibility`)**: neither may be classified by where
  its `?` currently sits in the control flow - that was round 3's
  mistake, and it is now resolved cleanly by the basis/outcome split:
  **evidence basis is determined by scanning the input's syntax alone,
  as if policy were not a factor**; policy pass/fail then applies
  purely as an **outcome** modifier on whatever basis was already
  found, never as a basis modifier. Concretely: if a file contains
  genuine exclusive evidence (e.g. `Entity`) and the Logos surface is
  policy-disabled, the result is `Exclusive(Err(policy violation))` -
  the policy failure must not erase already-established exclusive
  evidence by reporting `NoClaim` instead (today's code's blind
  pre-loop `require_logos_surface(...)?` would do exactly that, since
  it aborts before the loop ever sees `Entity` - the future
  implementation must scan for evidence basis *before* applying this
  gate, not call it as a pre-loop check). For `Import` specifically:
  since Logos's own shared threshold is the bare keyword (no further
  content requirement, per the paragraph above), and
  `require_legacy_compatibility` fires *after* that keyword is
  recognized, its failure is an outcome on an already-`Shared` basis -
  `Shared(Err(policy violation))` - never a basis question. If a file
  has no evidence of either kind, the result stays `NoClaim` regardless
  of policy (there is nothing for a disabled surface to have blocked).
  This also resolves a self-contradiction round 3 introduced between
  two of its own paragraphs about whether `Import`'s Logos-side
  "commit" happened before or after the policy check passed - the
  basis/outcome split removes the ambiguity entirely, since "when is
  evidence established" (syntax) and "did policy allow it" (outcome)
  are no longer the same question.

**Policy-vs-syntax finalization law (FROZEN by owner ruling, round 5)**:
round 4's text above correctly established that policy is an *outcome*
modifier, never a *basis* modifier, but left the exact aggregation
underspecified - PR `#1925`'s implementation filled the gap with an ad
hoc `errors.len() == 1` shortcut (accumulate every failure, including
policy violations, into the same `errors: Vec<FrontendError>` used for
ordinary syntax errors, then special-case the single-entry case to
avoid `merge_logos_errors` stripping a policy violation's message
prefix). That shortcut is **rejected**: continuing the evidence-basis
scan past a legacy-compatibility policy failure (so later evidence can
still strengthen `Shared` to `Exclusive`) does not mean the policy
diagnostic itself becomes an ordinary accumulated syntax error to be
merged alongside unrelated syntax failures - conflating the two loses
the old parser's own terminal-policy behavior (today's
`require_legacy_compatibility(...)?` aborts the whole function
immediately, discarding everything else) without gaining anything the
scan-continuation requirement actually needs. `admit_logos_program`
tracks a policy violation **separately** from the ordinary syntax
`errors: Vec`, and finalizes as follows:

```
if evidence_basis == None:
    NoClaim
else if the global Logos-surface policy check (`require_logos_surface`) failed:
    Shared/Exclusive(Err(<that global policy FrontendError, unmodified>))
else if a legacy-compatibility policy failure was encountered during the scan:
    Shared/Exclusive(Err(<the FIRST such FrontendError encountered, unmodified>))
else:
    Shared/Exclusive(<the ordinary Logos syntax outcome, using
        parse_logos_program's existing error-aggregation exactly -
        Ok if the `errors: Vec` is empty, else Err(merge_logos_errors(errors)),
        with NO `errors.len() == 1` special case>)
```

(`Shared`/`Exclusive` above is whichever the completed evidence-basis
scan discovered - the finalization law governs the wrapped outcome
only, never which variant wraps it.) In prose:

- The global `require_logos_surface` gate outranks a
  `require_legacy_compatibility` failure. **Precision note (round 5
  adversarial-review finding)**: today's `parse_logos_program` doesn't
  actually contain an explicit precedence *rule* between the two gates
  to match - it calls `require_logos_surface` once, unconditionally,
  before its loop runs at all (`parser.rs:2954-2956`), so a disabled
  surface aborts the whole function immediately for *every* input, and
  `require_legacy_compatibility` is simply never reached, regardless of
  content. This ordering is a formalization of that existing early-exit
  behavior's *outcome* - for any input with Logos evidence, today's code
  and this law agree on which `FrontendError` results when the surface
  is disabled - not a claim that today's code deliberately weighs one
  gate against the other.
- A legacy-compatibility failure is preserved **exactly as
  `require_legacy_compatibility` constructed it** - never routed
  through `merge_logos_errors`, never merged with unrelated syntax
  errors found elsewhere in the same scan, so `FrontendError::kind()`
  reliably reports `PolicyViolation` for it. This is the "preserve
  policy failure separately" requirement: a mixed input (e.g. a
  policy-disabled `Import` followed by a genuinely malformed `Entity`)
  returns the policy error as the grammar's outcome, not a merge of
  both - the malformed `Entity`'s own syntax error is real evidence
  toward `Exclusive` (basis), but is not what the caller sees as the
  `FrontendError`.
- Only when **no** policy failure of either kind occurred does the
  ordinary syntax path run, and it must reproduce
  `parse_logos_program`'s exact existing aggregation - including
  wrapping a *single* accumulated syntax error through
  `merge_logos_errors` exactly as today, not returning it unwrapped.
  `FrontendErrorKind::Syntax` for this path is correct and expected;
  only a genuine policy violation needs `PolicyViolation` preserved.

This is a finalization-time law only: it changes nothing about when
`evidence_basis` promotes to `Shared`/`Exclusive` (round 4's rules,
unchanged), and nothing about the cross-grammar evidence resolver
(unchanged - it still only ever sees a completed `GrammarAdmission`
value, regardless of which of the three finalization branches produced
its wrapped outcome).

- Every existing public sm-front parsing function
  (`parse_program(_with_profile)`, `parse_logos_program(_with_profile)`,
  `parse_rustlike(_with_profile)`, `parse_logos(_with_profile)`) keeps
  its exact current signature and behavior unchanged - reimplementable,
  if desired, as a thin wrapper over the new `admit_*` functions
  (`Shared(Ok(t)) | Exclusive(Ok(t)) => Ok(t)`, `NoClaim => Err(<today's
  exact "expected top-level..." / "expected Logos declaration"
  message>)`, `Shared(Err(e)) | Exclusive(Err(e)) => Err(e)`), so there
  is exactly one source of truth with zero forced migration for any
  other caller in the workspace.
- `sm-sema`'s future implementation (a later, separate
  implementation-only checkpoint) would delete
  `top_level_declaration_head_kinds`, `has_logos_exclusive_evidence`,
  `has_rustlike_exclusive_evidence`, and all depth/boundary tracking
  entirely, replacing `check_source_with_profile`'s classification with
  a direct call to the generic evidence-resolution law below over
  `(admit_logos_program_with_profile(...), admit_program_with_profile(...))`.
  `sm-sema` owns **none** of the following - it lives entirely inside
  `sm-front`'s two `admit_*` functions: which forms are exclusive vs.
  shared per grammar, where each grammar's shared threshold sits,
  parser-branch identity, or declaration-boundary lexical signals
  (`Newline`/`Semi`/`RBrace`/`Dedent`). `sm-sema` receives two
  `GrammarAdmission` values and performs only the generic,
  grammar-agnostic resolution law in the next section.

### Cross-grammar evidence resolver (FROZEN by owner ruling, round 4 -
supersedes the round-1 matrix, preserved below for audit history)

Resolution happens in two stages: **evidence strength first, parse
outcome second** - never the reverse, since resolving by outcome first
is exactly what would let a `Shared`-only claim (e.g. bare `Import`)
silently outrank the other grammar's genuine `Exclusive` evidence.

**Stage 1 - evidence strength** (`Exclusive` > `Shared` > `NoClaim`):

| Logos | RustLike | Resolution |
|---|---|---|
| `Exclusive` | `NoClaim` | Logos owns |
| `NoClaim` | `Exclusive` | RustLike owns |
| `Exclusive` | `Shared` | Logos owns |
| `Shared` | `Exclusive` | RustLike owns |
| `Exclusive` | `Exclusive` | AMBIGUOUS/CONFLICTING (both are the strongest class - tie, regardless of either side's own `Ok`/`Err`) |
| `Shared` | `NoClaim` | Logos owns |
| `NoClaim` | `Shared` | RustLike owns |
| `NoClaim` | `NoClaim` | NO SURFACE CLAIM |
| `Shared` | `Shared` | resolved by **Stage 2** below |

When one side "owns," that side's own already-computed outcome becomes
the `Auto` verdict directly: `Ok` -> accepted; `Err` -> **authoritative
failure**. This is the fail-closed invariant this whole investigation
exists to establish, stated precisely for the first time: **a
grammar-local `Err` only becomes an authoritative failure after that
grammar has won ownership resolution** - it is never itself sufficient
grounds to discard an `Exclusive` claim in favor of the other side's
`Shared`/`NoClaim` result (that would be exactly the fail-open pattern
`#1670` exists to remove), but a `Shared(Err)` losing to the other
side's `Exclusive` result (of either outcome) is not fail-open - the
`Shared` side never held the strongest evidence to begin with.

**Stage 2 - `Shared` vs `Shared`, resolved by parse outcome**:

| Logos | RustLike | Resolution |
|---|---|---|
| `Ok` | `Ok` | AMBIGUOUS/CONFLICTING |
| `Ok` | `Err` | Logos owns (accepted) |
| `Err` | `Ok` | RustLike owns (accepted) |
| `Err` | `Err` | AMBIGUOUS/CONFLICTING |

**Worked examples** (the six from "Implementation sketch" above, traced
through the resolver):

- `Import "a.sm"` alone: `Shared(Ok)` vs `Shared(Ok)` -> Stage 2,
  `Ok`+`Ok` -> AMBIGUOUS/CONFLICTING. Exactly Decision A's
  already-confirmed concrete collision.
- `Import foo.bar` alone: `Shared(Ok)` vs `NoClaim` -> Stage 1, `Shared`
  vs `NoClaim` -> Logos owns, `Ok` -> Logos accepted. Exactly Decision
  A's frozen ruling for this shape.
- `Import "a.sm"` + `Entity ...`: `Exclusive(Ok)` vs `Shared(Err)` ->
  Stage 1, `Exclusive` vs `Shared` -> Logos owns, `Ok` -> Logos
  accepted. Preserves Decision A's "Logos-exclusive evidence alongside
  `import` still reaches UNIQUE POSITIVE LOGOS CLAIM" rule exactly.
- `Import "a.sm"` + `fn main() {}`: `Shared(Err)` vs `Exclusive(Ok)` ->
  Stage 1, `Shared` vs `Exclusive` -> RustLike owns, `Ok` -> RustLike
  accepted - the symmetric rule required to avoid ordinary RustLike
  files beginning with `Import` being falsely classified as ambiguous
  (the round-3 defect this round fixes).
- `fn main(` alone (malformed): `NoClaim` vs `Exclusive(Err)` -> Stage
  1, `NoClaim` vs `Exclusive` -> RustLike owns, `Err` -> **RustLike
  authoritative failure**.
- Validating trace not in the required list, included because it
  stress-tests the model against a case none of rounds 1-3 got right:
  `Import "a.sm"` + a **malformed** `fn main(`: Logos = `Shared(Err)`
  (fails on the unrecognized `fn` token after committing to `Import`);
  RustLike = `Exclusive(Err)` (commits on `fn`'s dispatch, then
  genuinely fails inside it). Stage 1: `Shared` vs `Exclusive` ->
  RustLike owns; RustLike's own outcome is `Err` -> **RustLike
  authoritative failure** - correctly distinct from the well-formed
  case above (`Ok` -> accepted), and correctly not silently reinterpreted
  as Logos's problem merely because Logos also failed.
- As a side effect, this model also resolves - with no special case -
  a shape a round-2 adversarial pass flagged as unclear:
  `Import "a.sm" as 123` (a valid string literal followed by a
  malformed alias): RustLike commits at the string literal, then fails
  in the *shared* production's own trailing clause -> `Shared(Err)`,
  never promoted to `Exclusive` since nothing exclusive was ever
  touched. Logos has no further content requirement -> `Shared(Ok)`.
  Stage 2: `Ok`+`Err` -> Logos owns, accepted. Decision A never froze
  this exact shape, so this is a consistent extension, not a
  contradiction.

**Evidence-gathering is bounded by each grammar's own existing scan
mechanics, not a fresh whole-file re-scan (round 4 adversarial-review
finding, corrected during verification)**: `evidence_basis` is tracked
*during* the same loop that already computes parse outcome, not by a
separate pass over the whole token stream. This matters because the
two grammars' loops are not symmetric - a fact this document's own
"Current state" section already establishes: `parse_program`'s loop
(`crates/sm-front/src/parser.rs:97-123`) `return`s immediately via `?`
on the *first* unrecognized top-level token, discarding everything
after it, while `parse_logos_program`'s loop accumulates errors via
`recover_logos_anchor()` and keeps scanning to the end of the input.
Consequence for a source that mixes both grammars' exclusive vocabulary
in an order RustLike cannot get past - e.g. `Entity Player:\n    state
hp: int\nfn main() {}` (`Entity` first, `fn` second): Logos sees
`Entity` (promotes to `Exclusive`), then fails on the unrecognized `fn`
-> `Exclusive(Err)`. RustLike's loop returns on its very *first*
iteration - `Entity` matches none of its top-level forms - **before
`evidence_basis` is ever promoted and before `fn` is ever inspected**,
regardless of what recognizable RustLike content exists later in the
file -> `NoClaim`, not `Exclusive(Err)`. Stage 1 then gives `Exclusive`
vs `NoClaim` -> **Logos authoritative failure**, not AMBIGUOUS/
CONFLICTING (an initial trace of this exact input during adversarial
review incorrectly assumed RustLike's scan reaches `fn` regardless of
what precedes it, which the code above disproves). This is not a defect
in the model: `NoClaim` here is not a fabricated absence of evidence -
RustLike's own parser genuinely never reaches `fn` for this concrete
input, so it genuinely never establishes a claim, the same way a
human's ability to spot `fn` "in the file" is not the same claim as
"RustLike's parser establishes evidence from it." Flagged explicitly to
prevent a future implementer (or reviewer) from assuming both `admit_*`
functions scan symmetrically; neither this document nor Decision A
requires that they do.

**Evidence preservation**: both `Exclusive`-vs-`Exclusive` and
`Shared`-`Err`-vs-`Shared`-`Err` are AMBIGUOUS/CONFLICTING outcomes
where **neither underlying `FrontendError` may be silently dropped** -
both are genuine evidence, and neither is more authoritative than the
other absent an explicit profile choosing one grammar. This does not
require designing a full, versioned diagnostic carrier now (deferred to
the future carrier/schema checkpoint, per Decision B/§6) - only that
whatever concrete representation is eventually built preserves both,
e.g. in spirit:

```
Primary:
AMBIGUOUS / CONFLICTING SOURCE SURFACE

Evidence:
Logos     -> <original Logos FrontendError, or "accepted" if Ok>
RustLike  -> <original RustLike FrontendError, or "accepted" if Ok>
```

**Why the round-1 matrix (below) is superseded, not merely extended**:
the round-1 table was a pure function of a *flat* three-state
`GrammarAdmission`, which implicitly assumed every positive claim has
uniform strength. That assumption is exactly what Decision A's own
shared-vs-exclusive distinction already contradicted - the round-1
table cannot be patched with additional rows or a special case for
`Import`, because the missing information (evidence strength) does not
exist anywhere in its input. It is preserved immediately below, in
full, as audit history - not deleted, and not reachable from the
`Proposed decision` above, which now targets this section's two-stage
resolver instead.

<details>
<summary>Superseded round-1 matrix (kept for audit history only - do
not implement)</summary>

For `CompileProfile::Auto`, an earlier draft proposed the coordinator's
verdict as a pure function of the pair `(GrammarAdmission` from Logos,
`GrammarAdmission` from RustLike`)` under the flat `NoClaim`/
`Accepted(T)`/`ClaimedButFailed(FrontendError)` enum:

| Logos              | RustLike           | Auto verdict                    |
|--------------------|---------------------|----------------------------------|
| `NoClaim`          | `NoClaim`           | NO SURFACE CLAIM                |
| `Accepted`         | `NoClaim`           | Logos accepted                  |
| `ClaimedButFailed` | `NoClaim`           | Logos authoritative failure     |
| `NoClaim`          | `Accepted`          | RustLike accepted               |
| `NoClaim`          | `ClaimedButFailed`  | RustLike authoritative failure  |
| `Accepted`         | `Accepted`          | AMBIGUOUS / CONFLICTING         |
| `Accepted`         | `ClaimedButFailed`  | AMBIGUOUS / CONFLICTING         |
| `ClaimedButFailed` | `Accepted`          | AMBIGUOUS / CONFLICTING         |
| `ClaimedButFailed` | `ClaimedButFailed`  | AMBIGUOUS / CONFLICTING         |

Rounds 2 and 3 both attempted to fix how `GrammarAdmission` itself was
*computed* while keeping this table fixed, and both were shown by
adversarial review to break real inputs the table cannot distinguish
(an unquoted `Import` in round 2; `Import` alongside genuine exclusive
evidence from either grammar in round 3). The table itself was never
wrong in isolation - it is simply insufficiently expressive for
Decision A's own frozen law, which is why round 4 replaces it with the
two-stage resolver above rather than patching it again.

</details>

### Dependency / call-site impact

Workspace census (against baseline SHA `c0e2e50600e418f72af156a3c57616971d391494`)
confirms the additive approach is not merely convenient but the *only*
non-breaking option:

- **~270+ call sites** across the workspace consume the 9 existing
  public sm-front parse/lex functions, every one written against
  `Result<T, FrontendError>` syntax - `?`, `.map_err(...)?`, `.expect`,
  `.expect_err`, `.unwrap_or(...)`, `if let Ok(...)`, `match { Ok/Err }`.
  None of that syntax compiles unmodified against a bespoke
  `GrammarAdmission<T>` enum (no `Try`/`?` support, no `.map_err`,
  `if let Ok(...)` will not match a differently-named variant). Changing
  any existing function's
  *return type* in place would be a hard breaking change to all of
  them; adding new, separately-named `admit_*` functions alongside
  touches **zero** existing call sites.
- **Direct dependents of sm-front**: `semantic_language` (root),
  `smc-cli`, `sm-sema`, `sm-ir`. **Transitive dependents** (via
  `sm-ir`/`sm-sema`): `sm-emit`, `sm-vm`, `sm-verify`, `prom-runtime`,
  plus the `workbench_semantic` and `quad_logic_calculator` examples.
  No other workspace crate references sm-front or anything depending on
  it (confirmed by a workspace-wide `Cargo.toml` grep). None of these
  require any change under the additive proposal.
- **Notable existing pattern, corroborating the root-cause diagnosis**:
  several *other* call sites already do their own ad hoc version of
  exactly the probe sm-sema's `check_source_with_profile` does -
  `crates/smc-cli/src/executable_bundle.rs:25-28` uses
  `match { Ok(p) => p, Err(_) => return Ok(source) }` as a binary
  "is this RustLike at all" fallback; `crates/smc-cli/src/app.rs:1069-1076`
  (and its `:1875` twin) use the identical
  `if let Ok(logos) = parse_logos_program_with_profile(...) {..} else {
  parse_program_with_profile(...)? }` shape #1670 is repairing, for
  `--profile auto` CLI detection. These are exactly `#1919`/`#1920`'s
  own filed scope (independent of `#1670`, untouched by this decision)
  - noted here only because they confirm the "collapse any failure into
  a single undifferentiated bucket" pattern is systemic, not local to
  sm-sema, which is further evidence for fixing it once at the source
  (sm-front) rather than re-deriving a workaround at each call site.
- **Reconciliation needed with an existing type**: `FrontendError`
  already has a `kind() -> FrontendErrorKind` method
  (`crates/sm-front/src/types.rs`) distinguishing `Syntax` from
  `PolicyViolation` via a `"policy violation:"` string-prefix check on
  `message`. This is an orthogonal axis to evidence basis (`NoClaim`/
  `Shared`/`Exclusive`) - the `Err` nested inside a `Shared`/`Exclusive`
  result can itself be either `Syntax` or `PolicyViolation` - and
  requires no change: `GrammarAdmission`'s `Shared`/`Exclusive` variants
  simply wrap the existing `Result<T, FrontendError>` unchanged, so
  `.kind()` remains available on the nested error exactly as today.
- No call site anywhere in the workspace currently branches on
  `FrontendErrorKind` or inspects `pos`/`message` to distinguish *where*
  a failure occurred beyond the one `render_diag(...)`-embedding use in
  `std_adapters.rs` - confirming today's flat `Result` genuinely
  discards information no existing caller depends on, so recovering it
  via new, additive functions is safe by construction, not merely by
  convention.

### Migration plan

1. **This checkpoint (decision-only, FROZEN)**: the API shape (including
   round 5's `source: &str` parameter and policy-vs-syntax finalization
   law), the `GrammarAdmission<T>` naming, the evidence-basis/parse-
   outcome split, and the cross-grammar evidence resolver above are
   frozen (round 4 owner ruling, round 5 precision amendment). No code
   changes; `sm-front` is not touched by this PR. Freezing this
   contract does not by itself authorize step 2 or 3 below - each still
   requires its own separate GO.
2. **A future, separate implementation-only checkpoint** (its own GO,
   already in progress as PR `#1925` at the time of round 5 - to be
   rebased onto this amendment once merged): implement
   `admit_program_with_profile`/`admit_logos_program_with_profile` in
   `sm-front` exactly as frozen here, with `sm-front`'s own regressions
   proving `NoClaim`/`Shared`/`Exclusive` are each reachable for both
   grammars, that `Exclusive` is absorbing over `Shared`, that the six
   frozen concrete examples (plus the policy-gate examples) classify
   exactly as this document specifies, **and** that
   `admit_logos_program_with_profile`'s nested `FrontendError` is
   exactly equal - not merely same-variant/same-`kind()`/same-`pos` - to
   `parse_logos_with_profile`'s error for the same source, for: an
   ordinary single Logos syntax failure; ordinary multiple Logos syntax
   failures; a legacy-policy failure followed by later exclusive
   evidence (outcome stays the preserved policy error even though basis
   promotes to `Exclusive`); and a global Logos-surface denial with
   actual Logos evidence present.
3. **A further, separate implementation-only checkpoint** (its own GO)
   to rewrite `sm-sema`'s `check_source_with_profile` on top of the new
   contract, deleting the lexical-heuristic classifier entirely and
   restoring `#1670`/PR `#1923` to a design that cannot repeat this
   failure class - re-running every regression from `#1923`'s four
   rounds that remains meaningful once the heuristic itself is gone,
   plus new regressions for the concrete counterexamples this
   investigation found.
4. `#1670`/PR `#1923` stays OPEN/parked throughout; `#1580` stays OPEN;
   no issue-lifecycle changes anywhere in this checkpoint.

### Non-goals (explicit)

- No change to `ParserProfile` - profiles remain surface-independent,
  per Decision A's own invariant, reaffirmed rather than reopened here.
- No change to `CompileProfile`.
- No change to `FrontendError`'s own structure - the new admission
  types wrap the existing error type unchanged, preserving Decision B's
  diagnostic-identity-preservation invariant.
- No implementation in this checkpoint - `sm-front` is not touched;
  this is a docs-only decision artifact.
- Does not reopen or reinterpret Decisions A-D themselves - this is a
  mechanism proposal for how `sm-sema` can safely *compute* the inputs
  Decision A's own law already requires, not a change to that law.
- **Round 5 addendum**: this revision is an implementation-discovered
  precision correction to round 4's own already-frozen contract, not a
  redesign of it. It does not change evidence-basis semantics, the
  `Shared`/`Exclusive` absorbing/monotonic rule, the RustLike-abort-
  first/Logos-recover-and-continue scan-mechanics asymmetry, or the
  cross-grammar evidence resolver - all unchanged. It changes only (a)
  the `admit_*` signatures (adding `source: &str` as diagnostic-only
  context) and (b) how a completed scan's policy-vs-syntax findings are
  finalized into one `FrontendError`. `sm-front` is not touched by this
  revision; PR `#1925` (which discovered the gap) is left unmodified
  pending this amendment landing on `main`. No change to `sm-sema`; no
  change to `#1923`.

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
"absent" is specifically Decisions C and D's outcome), and does not
claim a bare `import`-only file is uniquely Logos evidence (corrected
- `import` is confirmed shared with RustLike's own top-level grammar;
found during SSF09-E2 implementation reconnaissance for `#1670`, before
any production code was written), **and does not claim, conversely,
that every `import`-led input is therefore ambiguous merely because the
keyword is shared** (round 1 of this same correction over-generalized in
exactly that direction and is fixed in round 2): `KwImport` being shared
*vocabulary* only means both grammars have a rule that could apply -
whether a specific input satisfies both depends on its actual content
(RustLike's `parse_import_decl` requires a string-literal token
specifically; Logos's handling has no equivalent requirement), so this
document asserts AMBIGUOUS/CONFLICTING only for the one input concretely
verified to satisfy both grammars to completion - `Import "a.sm"` - not
for `import`-led content in general. Separately, and **does not weaken
this into a claim that `pulse`/`profile` are also merely shared**
(that same round-1 draft conflated all three): `pulse`/`profile` never
appear in RustLike's grammar at all, so a bare `pulse`-only or
`profile`-only file remains genuinely, unconditionally, uniquely Logos
evidence with no combination requirement. This document also does not
claim that explicit `CompileProfile::RustLike` already bypasses Auto
classification in current production code - path 3
(`compile_program_to_ir_with_options_and_profile`) is documented above,
unchanged, as still invoking the Logos parser unconditionally even
under explicit `RustLike`; the INVARIANT states what MUST hold, and
`#1920` is the separate, already-tracked repair for where it does not
yet hold. This document also does not claim that repairing `#1670`
leaves every currently-admitted program's result unchanged - the
"Qualification requirements" section above is corrected to state
plainly that the confirmed `Import "a.sm"` collision is expected, and
required, to change from its current result to a deterministic
classification error, since that current result exists only because
today's fail-open dispatch silently discards Logos's own successful
parse of this input and falls through to RustLike. **Corrected
(E1B)**: that current result is a RustLike-side terminal failure
(`E0201`, missing `fn main`) - not a silent success, as an earlier
round's wording claimed; see the "Qualification requirements"
section's E1B correction note above for the re-verified baseline
evidence.
No historical document is rewritten by this decision; the TON618
perimeter's own closure record is read, not altered.

**Sequencing implication**: a future implementation checkpoint may
proceed directly from Decisions A-D and the repair DAG for the
currently-filed issues (`#1670`, `#1697`, `#1698`, `#1699`, `#1704`)
without a further architecture pass on those. It may **not** proceed to
carrier implementation until the separate governance/ownership
checkpoint above has run - that is a hard sequencing gate, not a
recommendation.

## Decision F - Canonical Cross-Grammar Authority Resolver

**FROZEN (2026-09-14).**

**Evidence**: three independent call sites each needed the exact same
cross-grammar authority resolution over a `GrammarAdmission<L>`/
`GrammarAdmission<R>` pair, and each implemented it privately -
`sm-sema`'s `resolve_surface_authority`/`SurfaceVerdict` (`#1670`),
`smc-cli`'s `resolve_project_route`/`ProjectRoute` (`#1919`, converged
only after two wrong local interpretations), and `sm-ir`'s attempted
`logos_owns_outright` (`#1920`, PR #1929 - a boolean-only projection
that silently lost `Ambiguous` and authoritative `Err`, caught before
merge and stopped as an architectural blocker rather than corrected a
fourth time in place). Canonical law (Decision E) existed; a canonical
*executable* resolver did not - leaving this to reviewer convention
demonstrably drifts.

**Owner**: `sm-front`. `GrammarAdmission`, `FrontendError`, and
`CompileProfile` already live there; `sm-sema`, `sm-ir`, and `smc-cli`
already depend on `sm-front`, so this adds no new dependency edge. A
`sm-sema`-owned resolver (the runner-up candidate, since the first
private copy happened to live there) was rejected specifically because
it would require a new `sm-ir -> sm-sema` edge that does not otherwise
exist, coupling source-surface selection to the semantic-analysis layer
it logically precedes. A new dedicated crate was rejected as
unnecessary fragmentation for one pure function already reachable
through an existing, shared dependency.

**Frozen contract** (`crates/sm-front/src/types.rs`):

```rust
pub enum SurfaceAuthority<L, R> {
    LogosOwns(Result<L, FrontendError>),
    RustLikeOwns(Result<R, FrontendError>),
    Ambiguous {
        logos: Result<L, FrontendError>,
        rustlike: Result<R, FrontendError>,
    },
    NoSurfaceClaim,
}

pub fn resolve_surface_authority<L, R>(
    logos: GrammarAdmission<L>,
    rustlike: GrammarAdmission<R>,
) -> SurfaceAuthority<L, R>;
```

Reproduces the frozen twelve-cell table (Decision E) exactly - moved
verbatim from `sm-sema`'s former private implementation, not
re-derived. Resolves classification only: no lexing, parsing,
type-checking, IR-lowering, or diagnostic rendering - those stay
consumer-owned, built from the `Result<_, FrontendError>` payloads
preserved here.

**`NoSurfaceClaim` is terminal**, not an invitation to retry: this
decision's own "only `NO SURFACE CLAIM` permits evaluating another
surface" (Decision A) refers to the classification process having a
candidate left to try. `resolve_surface_authority` is called only after
*both* of this system's two grammars have already been admitted into
the pairing - there is no third candidate remaining, so
`NoSurfaceClaim` ends classification exactly as `LogosOwns`/
`RustLikeOwns`/`Ambiguous` do, never as a signal to attempt either
grammar's parser again.

**No shared diagnostic-message carrier** is introduced. `sm-front` owns
authority structure only; `sm-sema` owns semantic diagnostic rendering,
`sm-ir` owns its own lowering-facing error representation, `smc-cli`
owns CLI presentation. `SurfaceAuthority::Ambiguous` already preserves
both underlying outcomes in full, which is sufficient for each consumer
to render according to its own contract without this decision
prescribing a shared message format.

**Anti-duplication enforcement**: production code outside `sm-front`
must not destructure/compare multiple `GrammarAdmission` values to
independently re-derive cross-grammar ownership - mechanically
enforced by `tests/surface_authority_guard.rs`, not merely documented
here. Pre-existing local resolvers are tracked as *measured* debt (an
exact expected occurrence count per file, not a whole-file allowlist),
so a second, unrelated local resolver added to an already-exempted file
still fails the guard.

**Consumer migration status**: all three known pre-Decision-F consumers
have migrated to consume `sm_front::resolve_surface_authority` directly,
each its own separate checkpoint with its own owner GO. `sm-sema`
migrated in the same PR that froze this decision (private
`SurfaceVerdict`/`resolve_surface_authority` deleted,
`check_source_with_profile` consumes the canonical type directly).
`sm-ir`'s rebase of PR #1929 (`#1920`) migrated next - its private
`logos_owns_outright` adapter deleted, `compile_program_to_ir_with_
options_and_profile` consumes the canonical type directly (`#1920`
CLOSED). `smc-cli`'s `resolve_project_route`/`ProjectRoute` (`#1919`)
migrated last (`#1931`) - `check_root_with_project_authority` consumes
the canonical type directly, and the same checkpoint repaired a sibling
defect in `cmd_dump_ir`/`cmd_hash_ir` that independently equated an
authoritative Logos parse failure with RustLike ownership.

**Implementation status**: Decision F foundation is implemented by
PR #1930. Consumer-local `GrammarAdmission`-pair resolver debt is now
zero repository-wide - `tests/surface_authority_guard.rs`'s
`EXPECTED_LEGACY_VIOLATION_COUNTS` is empty, and any future local
resolver anywhere outside `sm-front` fails the guard immediately.

## #1933 Addendum - Authority-Gated Executable Bundling

**FROZEN (2026-09-17). Narrow implementation consequence of Decision F,
not a new decision.** Does not change Decision F's frozen semantic table
(`SurfaceAuthority`/`resolve_surface_authority` above are unchanged).

**Problem**: `smc-cli`'s pre-existing executable-bundling step
(`crates/smc-cli/src/executable_bundle.rs`'s former
`read_source_with_package_admission`) decided whether to bundle a root's
`Import` targets by asking "does the root parse as RustLike and does it
have imports?" - a *re-derivation* of ownership from the root's own parse
result, run independently of, and prior to, `resolve_surface_authority`.
For a genuinely `Ambiguous` root (Decision E/F's confirmed concrete
instance, `Import "a.sm"` and nothing else) whose import target happened
to be valid RustLike, this let helper content silently decide the root
was RustLike-owned and bundle it - the ambiguity was never surfaced. This
is a distinct defect from anything Decision F's consumer-migration
status (above) already covers: it is not a second local
`GrammarAdmission`-pair resolver (the guard's own enforcement scope), it
is bundling itself running before authority is asked at all.

**Decision**: raw-root authority, from `resolve_surface_authority`
alone, precedes executable bundling unconditionally, and is sticky
afterward:

- **Raw-root authority precedes bundling.** `prepare_source`
  (`executable_bundle.rs`) is the sole seam every `smc-cli` source
  consumer goes through: it classifies the raw root text exactly once,
  before any bundling can run, and returns a `PreparedSource` carrying
  that frozen classification. Only `PreparedSource::RustLikeOwned(Ok(_))`
  may ever authorize `compose_executable_bundle`/
  `rustlike_effective_program`; `LogosOwned`, `RustLikeOwned(Err(_))`,
  `Ambiguous`, and `NoSurfaceClaim` never do, regardless of what an
  import target's own content looks like.
- **Authority is sticky after composition.** A composed/bundled
  effective source produced from a `RustLikeOwned(Ok(_))` root is an
  internal RustLike composition artifact, not a fresh classification
  candidate - it is never fed back into `resolve_surface_authority`, and
  no consumer re-opens Auto classification on it. `check_rustlike_program`
  (new, see below) type-checks the effective program directly, bypassing
  `check_source_with_profile`'s own Auto dispatch specifically to avoid
  this.
- **Helper contents cannot influence root ownership.** Bundling only
  ever runs *after* the root's own authority is already frozen; a
  helper's grammar, validity, or content has no path back into the
  ownership decision. The worst confirmed counterexample (an `Ambiguous`
  root whose import target happens to be valid RustLike) stays
  `Ambiguous`.
- **A synthetic bundle is not a new Auto source candidate.** The
  composed text exists only to be RustLike-reparsed
  (`rustlike_effective_program`) or type-checked
  (`check_rustlike_program`) under the authority the raw root already
  established - never reclassified.
- **Explicit RustLike/Logos rules unchanged, now enforced at the
  seam too.** Explicit RustLike may bundle unconditionally once its own
  root parse succeeds, with no Logos probe; explicit Logos never bundles
  and never probes RustLike. `read_raw_source` (the explicit-profile
  entry point) performs no classification at all, matching this.
- **Surface errors remain structured; bundler errors are a separate
  domain.** `PreparedSource` preserves `FrontendError` structurally in
  every variant (`RustLikeOwned(Result<Program, FrontendError>)`,
  `Ambiguous { logos: Result<..>, rustlike: Result<..> }`) - the
  preparation seam never collapses a surface classification failure into
  a `String`, matching Decision F's "no shared diagnostic-message
  carrier." Once a root is authorized to bundle, composition-domain
  failures (missing helper, malformed helper, cycle, unsupported import
  form) remain their own, separately-owned bundler error representation
  - the two domains are never merged into one carrier.
- **Terminal authority precedes result-cache reuse.** `LogosOwned(Err)`,
  `RustLikeOwned(Err)`, `Ambiguous`, and `NoSurfaceClaim` are gated before
  any AST/IR/SMC pack cache lookup or `check`/`lint` result-cache reuse
  (`is_cache_eligible`), so a pre-#1933 cached artifact can never mask a
  newly-correct terminal outcome. The three routing-sensitive pack-cache
  version tags were bumped accordingly: `ast_pack_key`
  (`frontend-v2-auto` -> `frontend-v3-auto`), `ir_pack_key`
  (`lowering=v2` -> `lowering=v3`), `smc_pack_key` (`emit=v1` ->
  `emit=v2`).

**New narrow `sm-sema` public API**: `check_rustlike_program(program:
&Program, source: &str) -> Result<SemanticReport, SemanticError>`
(`crates/sm-sema/src/std_adapters.rs`), extracted verbatim from
`check_source_with_profile`'s existing `RustLikeOwns(Ok(parsed))` arm -
zero lexing/parsing/admission/authority-resolution of its own.
`check_source_with_profile` now delegates to it internally, so this is a
refactor-with-new-entry-point, not new semantic behavior. It exists
because `smc-cli` needed a type-checking entry point for an
already-classified, already-composed RustLike `Program` without
re-opening `check_source_with_profile`'s own Auto dispatch on the
composed text (which would itself be the sticky-authority violation this
addendum forbids). This is not a new authority API and does not touch
`SurfaceAuthority`/`resolve_surface_authority`.

**Implementation status**: implemented on
`fix/1933-authority-gated-executable-bundling` from base
`2a83b35106118c7c4b592c379db2a8b38fd17d5c`. All 14 known `smc-cli`
source consumers were migrated off the removed
`read_source_with_package_admission`: the 11 Auto-capable callers
(`work prove`, `compile`, `check`, `watch`, `lint`, `dump-ast`,
`dump-ir`, `dump-bytecode`, `hash-ast`, `hash-ir`, `hash-smc`) route
through `prepare_source` and consume its frozen `PreparedSource` result
directly, while the 3 hard explicit-RustLike callers (`run`,
`run-controlled-observation`, `verify`) route through the separate
`effective_rustlike_source`/`read_raw_source` path instead and so never
call `prepare_source` or `resolve_surface_authority` at all - calling
either would itself be the "explicit RustLike must not probe Logos"
violation Decision F's INVARIANT already forbids. See the PR closing
[#1933](https://github.com/skulmakov-oss/Semantic/issues/1933) for the
full regression matrix, mutation-testing results, and review
disposition.
