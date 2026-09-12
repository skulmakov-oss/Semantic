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

**CURRENT STATE**: `crates/sm-sema/src/std_adapters.rs:92-101`
(`check_source_with_profile`) is the sole dispatch point deciding whether
input is treated as Logos or RustLike source:

```rust
pub fn check_source_with_profile(
    input: &str,
    profile: &ParserProfile,
) -> Result<SemanticReport, SemanticError> {
    if let Ok(logos) = parse_logos_program_with_profile(input, profile) {
        if logos.system.is_some() || !logos.entities.is_empty() || !logos.laws.is_empty() {
            return analyze_logos_program(&logos, input);
        }
    }
    let parsed = parse_program_with_profile(input, profile).map_err(|e| SemanticError {
        diag: render_diag(DiagLevel::Error, "E0000", e.message, SourceMark::default(), input),
    })?;
    ...
}
```

The `if let Ok(logos) = ...` pattern has no `else` arm: a genuine Logos
parse/policy **failure** (`Err`) is silently discarded, not just an
empty-but-successful Logos parse, and execution falls through to attempt
the entirely different RustLike grammar on the same input regardless
(`#1670`, FA-03-001). This is the exact same function where the
RustLike-path position loss (`#1698`) is hardcoded
(`SourceMark::default()`, lines 106/115) - the surface-selection defect
and one position-loss defect share one root site, which is why the
repair DAG (§9) sequences `#1670` immediately before `#1698`/`#1699`.
`ParserProfile` itself (`sm-profile` crate, a dependency of `sm-front`/
`sm-sema`/`sm-ir`/`sm-emit`) already carries an explicit, caller-supplied
profile selection (Rust-like / Logos / auto) - the ambiguity is not "no
profile exists," it is "the profile's own auto-detection silently
retries after a real failure instead of surfacing it."

**DECISION**: Source-surface selection is owned by `sm-front`/`sm-sema`'s
existing profile mechanism (`ParserProfile`), invoked exactly once per
compilation unit. A given input's surface becomes authoritative the
moment the first grammar attempt either (a) succeeds with a non-empty
program, or (b) fails with a **policy-classified** failure (a real parse/
policy error, not merely "produced an empty program"). Distinguish these
two "did not produce a Logos program" outcomes explicitly instead of
collapsing them into one `if let Ok(...)` branch: "syntactically valid
but empty" (legitimately eligible for RustLike fallback, matching today's
one correct half of the current behavior) vs. "syntactically or
policy-invalid" (never eligible for fallback - the failure itself is
authoritative and must be surfaced, not discarded).

**INVARIANT**: Once the canonical frontend has classified a real parse/
policy failure for a given surface, no later layer may reinterpret the
same source under a different surface. A `RustLike` attempt following a
classified Logos failure is not "trying harder," it is manufacturing a
second, unrelated diagnostic authority for the same input and discarding
the first.

**WHY**: Fixing `#1670` without freezing this invariant first risks a
narrow patch (e.g. "also propagate the specific Logos error") that still
leaves the *general* rule undefined for the next profile or the next
caller. Freezing "first classified failure is authoritative, no silent
grammar-hopping" as a standing rule - not just a fix to one function -
is what lets `#1704`'s catalog and any future third profile be judged
against the same law.

**REJECTED ALTERNATIVES**:
- *Try every profile and pick the "best" result heuristically* - rejected:
  reintroduces exactly the guessed-success-over-real-failure pattern the
  governing invariant forbids, and "best" has no principled definition
  here (both grammars can independently "succeed" on adversarial input).
- *Make RustLike the sole grammar, deprecate Logos auto-detection
  entirely* - rejected: out of scope for a diagnostic-authority decision;
  this is a language-surface decision belonging to SSF-01/02 (already
  closed, Model B), not SSF-09.

**IMPLEMENTATION CONSEQUENCE** (not performed here): `check_source_with_profile`
must classify the Logos attempt's outcome into (empty-but-valid /
policy-invalid) before deciding whether to attempt RustLike; a
policy-invalid outcome returns immediately with its own real diagnostic,
never falling through.

**TEST CONSEQUENCE** (not performed here): a regression proving that a
source string which is genuinely invalid Logos input (not merely empty)
is rejected with a Logos-attributed diagnostic and never silently
succeeds as a RustLike program.

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
identity today.

**DECISION**: File identity has two distinct representations, not one:

- **INTERNAL identity**: an opaque, cheap-to-compare identifier (a
  `FileId`-shaped value, or the existing type widened as needed) suitable
  for compiler-internal data structures - comparable, hashable, does not
  need to be human-legible.
- **EXTERNAL stable identity**: a **project-root-relative logical path**
  string, derived from the same canonical project/module authority
  `resolve_package_import_path`/`resolve_project_root_check_entry` already
  use - not an absolute filesystem path (not portable across machines or
  reproducible in tests/CI), not a URI (nothing in current canonical
  tooling produces or consumes URIs; inventing one now would be choosing
  a representation for LSP's future convenience, which this decision's
  own governing rule forbids), and not a synthetic in-memory label
  invented at the diagnostic layer itself. A single-file CLI invocation
  (`smc check <file.sm>` with no project root) resolves its external
  identity as the file's own path relative to the invocation's working
  directory, consistent with how `smc`'s existing single-file fallback
  already treats such invocations project-root-equivalent
  (`docs/roadmap/.../semantic_stable_foundation_matrix.md`'s "Single-file
  fallback" row). An imported module's external identity is the
  project-root-relative path already computed to *load* it - not
  re-derived independently at the diagnostic layer. Stdin/synthetic/
  virtual sources (none currently exist as a compiler input path) have no
  frozen representation here - out of scope until such an input path is
  added.

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

**IMPLEMENTATION CONSEQUENCE** (not performed here): thread the already-
computed project-root-relative path (available at the point
`check_file_with_provider`/`resolve_project_root_check_entry` already
resolve it) into the diagnostic construction path, replacing every
`file_id: 0`/string-prefix/`"<input>"` site; introduce an explicit
"identity unprovable" representation for single-file invocations with no
resolvable root.

**TEST CONSEQUENCE** (not performed here): positive tests for ordinary
project file and imported module (real, resolvable identity); a
deterministic-absence test for a source whose identity cannot be proven
(no invented `0`/`"<input>"` fallback).

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

**DECISION**: The canonical **internal** source-range law is: **UTF-8
byte offsets, zero-based, half-open `[start, end)`**, using a real range
(two offsets) rather than a single point, extending `ton618_core::Span`'s
existing shape (already a range type, already zero-dependency-safe per
§7) rather than inventing a new one. Line/column is a **derived
presentation view** computed from the canonical byte range plus the
source text, never itself the canonical identity - this directly reverses
today's lexer behavior (byte offset computed first, then immediately
collapsed to line/col and discarded), not merely documents it. Concrete
rules: an empty range is a valid zero-width `[n, n)`, not a special case
requiring its own type; an EOF-anchored diagnostic uses
`[len, len)` on the source's own byte length; a diagnostic with no
provable location (a synthetic/whole-program-level finding) carries **no
range at all**, not a `[0, 0)` placeholder presented as if it meant
something; invalid UTF-8 is not a range-law concern (the compiler's own
source-loading boundary already rejects non-UTF-8 input via
`fs::read_to_string`'s `Result::Err`, confirmed panic-free, before any
range is ever computed) and CRLF is not a range-law concern either (byte
offsets are representation-agnostic to line-ending style; only the
line/col *presentation* view needs a documented CRLF convention, which
this decision does not need to fix since it is a derived view, not the
canonical identity).

Downstream artifact/execution coordinates are explicitly **not** source
ranges and must never be presented as one without a validated mapping:
`DecodedDebugSymbol{pc,line,col}` is source-mapping *evidence* embedded in
a compiled artifact, not itself the canonical source range;
`VerificationDiagnostic.offset` (`crates/sm-verify/src/lib.rs:88-225`) is
a bytecode-byte-offset **artifact coordinate**; a VM's runtime program
counter is an **execution coordinate**. A verifier- or runtime-originated
diagnostic receives a canonical source range **only** when a validated
mapping (walking the artifact's own `debug_symbols` table back to a
source byte range) actually proves the relationship for that specific
diagnostic. Today, that mapping exists as data (`DebugSymbol` is
validated for structural well-formedness at VM load,
`crates/sm-vm/src/semcode_vm.rs:1668-1684`) but is exhaustively never
consulted at any of the eight production `RuntimeError::Trap(...)`
construction sites - so under this decision's own rule, every current
runtime trap correctly has **no** source range today, not a fabricated
`offset 0`/`line 0` standing in for one. Closing that gap (actually
performing the lookup) is future implementation, not this decision.

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
`FrontendError` and every downstream diagnostic gain a real
`Span`-shaped optional range field instead of/alongside the current point
`pos`; `sm-verify`/`sm-vm` gain the actual debug-symbol lookup at trap
sites, populating a range only when the lookup succeeds.

**TEST CONSEQUENCE** (not performed here): positive range-preservation
tests per error class (mirroring Decision C's per-class tests); an
explicit test that a runtime trap with no available debug symbols
produces a diagnostic with an absent range, not a fabricated one.

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
| `sm-front` | `ton618-core`, `sm-profile` | `sm-sema`, `sm-ir`, `smc-cli` |
| `sm-sema` | `ton618-core`, `sm-front`, `sm-profile` | `smc-cli` |
| `sm-ir` | `sm-front`, `sm-profile`, `sm-format` | `sm-emit`, `smc-cli` |
| `sm-emit` | `sm-format`, `sm-ir` | `smc-cli` |
| `sm-verify` | `sm-runtime-core`, `sm-format` | `sm-vm`, `smc-cli` |
| `sm-vm` | `sm-runtime-core`, `sm-verify`, `sm-format`, `prom-*` | `smc-cli` |
| `smc-cli` | (all of the above) | *(top of graph)* |

**Load-bearing finding**: `ton618-core`, `sm-format`, and
`sm-runtime-core` are three *separate*, non-overlapping zero-dependency
leaf crates. `sm-front`/`sm-sema` reach only `ton618-core`; `sm-verify`/
`sm-vm` reach only `sm-format`/`sm-runtime-core`. **Neither branch
currently depends on the other's leaf crate at all** - `ton618-core`'s
existing `Diagnostic<M>`/`SourceMark`/`Span` types are structurally
unreachable from `sm-verify`/`sm-vm` today, which is an independent,
architectural reason (not merely "it's dead code") why the current
`ton618_core::diagnostics::Diagnostic<M>` cannot yet be a real cross-stage
canonical carrier even in principle, regardless of whether anyone ever
constructs it.

**Candidate owners for the future internal canonical model**, classified:

| Candidate | Dependency-safe from `sm-verify`/`sm-vm`? | Architecturally correct? | Verdict |
|---|---|---|---|
| `ton618-core` | **YES** - zero dependencies itself, so any crate (including `sm-verify`/`sm-vm`) can add it as a dependency without creating a cycle | YES - already the semantic home for `SourceMark`/`Span`/diagnostic rendering/the code catalog; extending its reach is completing an existing role, not inventing a new one | **SELECTED** |
| `sm-runtime-core` | trivially yes for `sm-verify`/`sm-vm` (already a dependency) but **NO** for `sm-front`/`sm-sema` (would require the frontend to depend on a runtime-execution crate) | NO - semantically a runtime/quota/trap-taxonomy crate (`#1759`-`#1763`'s own home); frontend diagnostics depending on it would be a layering violation, not merely inconvenient | REJECTED |
| `sm-format` | trivially yes for `sm-verify`/`sm-vm`/`sm-ir`/`sm-emit` (already a dependency) but **NO** for `sm-front`/`sm-sema` | NO - a SemCode binary-format crate; diagnostic identity is not a bytecode-format concern | REJECTED |
| A brand-new leaf crate | YES (zero dependencies by construction) | Not architecturally *incorrect*, but unnecessary - `ton618-core` already exists, already has the right shape of responsibility, and already has real (if underused) content | REJECTED (no new crate needed) |

**Selected future carrier owner: `ton618-core`.** This is a graph-position
and existing-role finding, not an endorsement of `Diagnostic<M>`'s current
*shape* - per this checkpoint's own governing instruction, the fact that
`Diagnostic<M>` and `SourceMark` already live there is not by itself
sufficient reasoning (the reconnaissance found `Diagnostic<M>` is dead
code, and `SourceMark`/`Span` are both structurally inadequate per
Decisions C/D as they stand - `SourceMark` has no file-identity-absence
representation and `Span` is unused by the compiler itself). The decision
is that **wherever the future internal canonical model lands, it belongs
in `ton618-core`, in a shape this decision has now specified but not yet
written** - not that the existing `Diagnostic<M>` struct is adopted
as-is.

**Dependency-safe**: giving `sm-verify` and `sm-vm` a new direct
dependency on `ton618-core` is dependency-safe (no cycle: `ton618-core`
depends on nothing) and requires **no `.harness/current.task.yaml` scope
change** - `crates/sm-verify/**` and `crates/sm-vm/**` are already inside
the currently-authorized path list.

## Verifier/runtime source-mapping law

Stated fully under Decision D; restated here as the standalone rule
future qualification (§11) must check: a verifier- or runtime-originated
diagnostic's source range is present **if and only if** a validated
debug-symbol mapping proves it for that specific diagnostic instance: no
mapping attempted, no range; mapping attempted and failed (out-of-bounds,
missing table), no range; mapping attempted and succeeded, the real
mapped range. Never a placeholder in any of the first two cases.

## Presentation/LSP boundary

No editor/LSP node may own semantic truth (restated from the governing
invariant, made concrete for the authority graph, §7). A future LSP
adapter: converts the canonical UTF-8 byte range to LSP's UTF-16
code-unit convention (a pure, lossless-in-the-safe-direction numeric
conversion, not a reinterpretation); converts the canonical
project-root-relative file identity to a `file://` URI; relays the
canonical diagnostic identity (code/severity/family) unchanged; may cache
canonical results keyed by file identity + a content/version stamp, but
never recomputes a diagnostic itself. This mirrors, and is bound by, the
same rule `.agents/skills/semantic-ui-boundary-guard/SKILL.md` already
enforces for the unrelated UI-DNA2 presentation layer ("must NOT rewrite
or alter verifier diagnostics") - a precedent this decision generalizes
rather than invents.

## Fail-closed rules

Consolidated from all four decisions, restated once for the whole
document: unknown/ambiguous/unsupported/unprovable never becomes a
guessed diagnostic identity (Decision B), a guessed source surface
(Decision A), a guessed file identity (Decision C: `file_id = 0`/
`"<input>"` forbidden as a stand-in for absence), or a guessed source
range (Decision D: `[0,0)`/`pc 0`/`offset 0` forbidden as a stand-in for
absence). In every one of these cases the correct representation is a
deterministic, explicit "absent" - never a default value presented as if
it were meaningful, and never a silent retry under a different authority
(Decision A's grammar-hopping case) presented as if it were the original
authority succeeding.

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

Because `sm-verify`/`sm-vm` do not currently depend on `ton618-core`,
the *first* piece of actual carrier work (not authorized by this
decision, listed here only to keep §9's DAG accurate) will need to add
that dependency edge before `sm-verify`/`sm-vm` can construct the
canonical shape directly at the point of origin - this is a small,
one-line `Cargo.toml` change with a proven-safe (acyclic) graph position,
not a redesign.

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
- No `.harness/current.task.yaml` scope widening - the selected future
  carrier owner (`ton618-core`) and its new consumers (`sm-verify`,
  `sm-vm`) are all already inside the currently-authorized path list.

## Qualification requirements (for the future implementation checkpoint)

- Every one of Decision A-D's "TEST CONSEQUENCE" items.
- The registry-completeness mechanical check (Decision B).
- Confirmation that `sm-verify`/`sm-vm` gaining a `ton618-core` dependency
  does not introduce a build cycle (`cargo tree` / a full workspace build
  is sufficient evidence, no new tooling required).
- Re-confirmation that `#1670`'s fix does not change any *admitted*
  program's compilation result - only failure-classification behavior for
  currently-misclassified inputs.

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
canonical internal diagnostic carrier       (§6, ton618-core - Decisions B/C/D combined)
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
carrier must land before the external schema because the schema versions
the carrier's shape, not the other way around, per §6's own boundary.

## Exit gate

**CONTRACT FROZEN = YES** if: Decisions A-D are each internally complete
(current state / decision / invariant / why / rejected alternatives /
implementation + test consequence - all four are, above); the
canonical-carrier boundary and verifier/runtime mapping law are stated
precisely enough to judge a future PR against; the dependency-owner
analysis is evidence-derived, not assumed; and no public claim is
overstated (§10, checked below).

**Public claim check**: this document does not claim source spans are
already fully qualified (Decision D states the opposite precisely), does
not claim one canonical carrier already exists (§6/dependency analysis
state the opposite - `Diagnostic<M>` is dead code and structurally
unreachable from half the pipeline), does not claim machine-readable
diagnostics already exist (unaffected by this decision, still `ABSENT`
per the SSF09-E0 reconnaissance), does not claim verifier/runtime already
map every error to source (the mapping law's own "if and only if" clause
states today's real answer is "never, in every current production case"),
does not claim an LSP exists, and does not touch the formatter's already-
accurate narrow-scope claim in `docs/spec/source_style.md`. No historical
document is rewritten by this decision.

**Nothing above is left open.** A future implementation checkpoint may
proceed directly from this document, in the exact sequencing above,
without a further architecture pass.

**Wait for owner review and a separate implementation GO.**
