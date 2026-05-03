# Roadmap Wave 2 — 7hell Update

Project: **Semantic Language**
Track: **Practical Readiness + 7-Hell Qualification**
Status: Draft v0.2

This document defines Wave 2 as a focused readiness and qualification wave.

```text
Wave 2 = language + execution + diagnostics + 7hell qualification
Workbench = Dessert Track after Wave 2 stabilizes
```

## Executive Summary

Semantic already has a strong verified execution foundation:

```text
source -> frontend -> semantic analysis -> IR -> optimization -> SemCode -> verifier -> VM -> result
```

Wave 2 must not become a broad feature expansion. Its purpose is to close the practical gaps required to use Semantic as a reliable language and to introduce a strict qualification gate:

```bash
smc 7hell program.sm
```

`7hell` means seven ordered qualification stages:

1. Syntax Hell — lexer/parser stability
2. Type Hell — type system and practical value model
3. Lowering Hell — AST to IR correctness
4. Verifier Hell — SemCode admission integrity
5. VM Hell — execution, traps, quotas, determinism
6. Practical Hell — real programs without workarounds
7. User Pain Hell — diagnostics and developer usability

## Priority Model

| Priority | Meaning | Rule |
|---|---|---|
| P0 | Blocker | Blocks 7hell or verified execution correctness |
| P1 | High | Needed for practical language readiness |
| P2 | Medium | Hardening, quality, polish |
| P3 | Later | Must not enter Wave 2 unless explicitly scoped |

## Wave 2 Non-Goals

Do not include these in Wave 2:

```text
- UI application runtime
- Workbench full implementation
- Workbench monitor / foreign-language monitor
- Workbench packaging UX
- package registry
- async/concurrency
- macro system
- broad generics/traits
- TinyLM/LLM research track
- GPU/Metal/AVX-512 backends as required paths
- broad PROMETHEUS runtime widening
```

## Workbench Dessert Track Policy

Workbench is not cancelled. It is intentionally deferred.

Correct placement:

```text
Wave 2        = language + execution + diagnostics + 7hell qualification
Dessert Track = Workbench / UI / monitor / packaging UX after Wave 2 stabilizes
```

Workbench may begin only when:

```text
- smc 7hell exists
- weather_station.sm passes all 7 stages
- control_flow_gauntlet.sm passes all 7 stages
- type_matrix.sm passes all 7 stages
- Project Model v0 is minimally defined
- diagnostics have stable machine-readable JSON
- CLI pipeline can be called without UI-specific hacks
```

Later Dessert Track scope:

```text
D1 — Workbench Blueprint freeze
D2 — Toolchain orchestration API
D3 — Project Explorer + Source Editor shell
D4 — Diagnostics / 7hell panel
D5 — IR / SemCode / Verifier inspector
D6 — Package Builder UX
D7 — Optional Semantic Monitor over foreign code
```

## P0 Roadmap

### W2-P0.0 — Readiness Truth Freeze

Goal: freeze what is stable, experimental, partial, and out of scope.

Acceptance:

- docs do not overclaim practical readiness;
- main features and stable features are separated;
- 7hell is described as qualification mode, not a release promise.

### W2-P0.1 — Syntax Hell: Parser Inferno

Goal: make parser behavior predictable on ordinary code and malformed code.

Must cover:

```text
fn, let, let mut, return, if/else, match, while, loop, break, continue,
records, ADT constructors, imports, comments, empty lines, nested blocks,
syntax errors
```

Acceptance:

- valid syntax fixtures pass;
- invalid syntax fixtures fail with stable diagnostic codes;
- no parser panic;
- no ambiguous fallback errors where a specific diagnostic is possible.

### W2-P0.2 — Type Hell: Type Abyss

Goal: close practical type behavior for the core language.

Must cover:

```text
quad vs bool separation, i32/u32/fx/f64 compatibility, return path typing,
record field typing, ADT constructor typing, scope shadowing, assignment,
reassignment, let mut rules, match arm type compatibility
```

Acceptance:

- type fixtures exist for every primitive type;
- wrong quad/bool usage has stable diagnostics;
- numeric operators compile to SemCode and execute in VM;
- type errors include source spans.

### W2-P0.3 — Lowering Hell: IR Purgatory

Goal: prove AST to IR remains correct for practical constructs.

Must cover:

```text
if/else -> labels + conditional jumps
while -> loop labels + conditional exit
loop -> unconditional back edge
break -> jump to loop exit
continue -> jump to loop head
match -> compare/jump chain or branch table
function calls -> CALL/RET discipline
records/ADT carriers -> canonical IR values
```

Critical ownership rule:

```text
sm-ir   owns IR
sm-emit owns SemCode binary format and emission
```

### W2-P0.4 — Verifier Hell: Admission Gate

Goal: keep VM from executing unverified or malformed SemCode.

Must cover:

```text
header magic/version, function envelope, string table, instruction boundary,
jump target validity, call target validity, register budget, capability bits,
resource limits, debug section bounds
```

Required policy:

```text
standard execution path = verify first, then run
```

### W2-P0.5 — VM Hell: Execution Furnace

Goal: ensure execution is deterministic, bounded, and trap-safe.

Must cover:

```text
frames, registers, calls, returns, trap taxonomy, quota consumption,
deterministic repeated run, symbol table behavior, runtime values,
host-call denial without capability
```

### W2-P0.6 — smc 7hell Skeleton

Goal: create the qualification command before all stages are complete.

Commands:

```bash
smc 7hell program.sm
smc seven-hell program.sm
```

Required modes:

```bash
smc 7hell program.sm
smc 7hell program.sm --json
```

Acceptance:

- command exists;
- it runs existing stages even if later stages are initially shallow;
- failure includes stage ID, code, message, and optional source span;
- it can be used by CI.

## P1 Roadmap

### W2-P1.1 — Practical Hell: Real Code Gauntlet

Canonical examples:

| Example | Purpose |
|---|---|
| weather_station.sm | deterministic decision program |
| control_flow_gauntlet.sm | while/loop/break/continue/match |
| type_matrix.sm | quad/bool/i32/u32/fx/f64 behavior |
| records_and_adt.sm | records + constructors |
| module_imports.sm | imports/re-exports practical contour |

### W2-P1.2 — Collections and Text v0

Required order:

```text
text -> Sequence -> Map
```

### W2-P1.3 — Minimal Standard Library v0

| Module | Contents | Priority |
|---|---|---|
| core | assert, trap helpers, compare helpers | P1 |
| quad | is_true, is_false, is_known, is_conflict | P1 |
| math | min, max, abs, clamp | P1 |
| text | concat, len, equality, to_text | P1 |
| seq | len, get, set, push, pop | P1 |
| map | get, set, contains, remove, len | P2 |
| debug | print/debug through capability boundary | P1 |
| rand | deterministic seed PRNG | P2 |

### W2-P1.4 — Project Model v0

Proposed layout:

```text
my_project/
  semantic.toml
  src/
    main.sm
    lib.sm
  examples/
  tests/
```

Required commands:

```bash
smc new my_project
smc check
smc build
smc run
smc 7hell
```

### W2-P1.5 — Diagnostics / User Pain Hell

Goal: bad code must produce useful, stable errors.

Required areas:

```text
syntax errors, type mismatch, unknown symbol, import conflict,
invalid break/continue, verifier reject, runtime trap
```

## P2 Roadmap

### W2-P2.1 — Runtime Ownership v0

Scope:

```text
AccessPath = root SymbolId + [TupleIndex(u16)]
Lifetime = frame-local
Overlap = exact / parent-child / child-parent
Siblings allowed
```

### W2-P2.2 — Cache / Incremental Qualification

Scope:

```text
smc 7hell --no-cache
smc 7hell --trace-cache
cache hit/miss reporting per stage
cache invalidation snapshot tests
```

### W2-P2.3 — Formatter v0.2

Scope:

```text
stable indentation, preserve comments, normalize trailing spaces,
check/write modes, no semantic rewriting
```

### W2-P2.4 — Golden Trace Suite

Goldens:

```text
AST snapshots, IR snapshots, SemCode hashes, verifier reports,
VM traces, runtime trap reports, 7hell JSON reports
```

## P3 — Deferred

| Track | Status | Reason |
|---|---|---|
| UI application boundary | Deferred | Separate POST-UI track after Wave 2 |
| Workbench full UI | Dessert Track | Needs stable CLI/toolchain/7hell first |
| Foreign code semantic monitor | Dessert+ | After Workbench shell |
| TinyLM / LLM inference | Research | Needs collections/text/math maturity |
| GPU/Metal backend | Research | Not required for correctness |
| Package registry | Later | Project model first |
| async/concurrency | Later | Too much surface area now |
| macro system | Later | Could destabilize syntax/type system |
| broad generics/traits | Later | Not required for Wave 2 readiness |

## Suggested PR Waves

### Wave 2A — Qualification Skeleton

| PR | Title | Priority |
|---|---|---|
| W2A-01 | docs: add Roadmap Wave 2 7hell update | P0 |
| W2A-02 | cli: add smc 7hell command skeleton | P0 |
| W2A-03 | cli: add 7hell JSON report format | P0 |
| W2A-04 | tests: add first 7hell smoke fixture | P0 |

### Wave 2B — Syntax + Control Flow

| PR | Title | Priority |
|---|---|---|
| W2B-01 | parser: normalize comments in Rust-like surface | P0 |
| W2B-02 | parser: add while/loop surface | P0 |
| W2B-03 | parser: add break/continue surface | P0 |
| W2B-04 | tests: syntax hell valid/invalid fixture suite | P0 |

### Wave 2C — Type Core

| PR | Title | Priority |
|---|---|---|
| W2C-01 | sema: enforce bool conditions for while/if | P0 |
| W2C-02 | sema: complete i32/u32 numeric checks | P0 |
| W2C-03 | sema: close fx minimal execution typing | P0 |
| W2C-04 | tests: type hell matrix | P0 |

### Wave 2D — Lowering + Emit Ownership

| PR | Title | Priority |
|---|---|---|
| W2D-01 | ir: lower while/loop/break/continue | P0 |
| W2D-02 | ir: add IR goldens for control flow | P0 |
| W2D-03 | emit: begin SemCode ownership separation from sm-ir | P0 |
| W2D-04 | tests: lowering hell fixture suite | P0 |

### Wave 2E — Verifier + VM Hardening

| PR | Title | Priority |
|---|---|---|
| W2E-01 | verify: add malformed SemCode rejection fixtures | P0 |
| W2E-02 | vm: enforce verified execution default path | P0 |
| W2E-03 | runtime-core: freeze trap taxonomy | P0 |
| W2E-04 | vm: repeated-run determinism tests | P0 |

### Wave 2F — Practical Core

| PR | Title | Priority |
|---|---|---|
| W2F-01 | stdlib: add core assert and quad helpers | P1 |
| W2F-02 | stdlib: add text v0 | P1 |
| W2F-03 | stdlib: add seq v0 | P1 |
| W2F-04 | examples: add control flow and type matrix demos | P1 |

### Wave 2G — Diagnostics + Release Gate

| PR | Title | Priority |
|---|---|---|
| W2G-01 | diagnostics: add 7hell stage-specific failure codes | P1 |
| W2G-02 | explain: extend smc explain for Wave 2 codes | P1 |
| W2G-03 | ci: add 7hell smoke job | P0 |
| W2G-04 | docs: add Wave 2 qualification checklist | P0 |

## Merge Order

```text
1. W2A — docs + 7hell skeleton
2. W2B — syntax/control flow
3. W2C — type core
4. W2D — lowering/emit boundary
5. W2E — verifier/VM hardening
6. W2F — stdlib/practical examples
7. W2G — diagnostics/release gate
```

Do not merge practical examples that require unsupported language constructs before the corresponding parser/type/lowering/VM work is merged.

Workbench-related PRs must not be merged into Wave 2 unless they are docs-only and explicitly marked as Dessert Track planning. Any code-level Workbench work starts after Wave 2 DoD is satisfied.

## Definition of Done

Wave 2 is done when:

```text
✅ smc 7hell exists
✅ at least one canonical example passes all 7 stages
✅ weather_station.sm passes all 7 stages
✅ control_flow_gauntlet.sm passes all 7 stages
✅ type_matrix.sm passes all 7 stages
✅ invalid syntax/type/verifier fixtures fail with stable diagnostics
✅ VM execution is verifier-first by default
✅ no production panic path in Wave 2 code
✅ docs do not overclaim readiness
✅ CI has a 7hell smoke gate
✅ Workbench remains deferred to Dessert Track with no UI/runtime creep in Wave 2
```

## Final Engineering Position

Wave 2 must be treated as a qualification and hardening wave, not a feature explosion.

Correct mental model:

```text
Semantic already works.
Wave 2 proves where it breaks.
Then Wave 2 fixes those breaks in dependency order.
```

The highest-value next step is not “add more language”, but:

```text
add smc 7hell, then let it expose the real gaps stage by stage.
```
