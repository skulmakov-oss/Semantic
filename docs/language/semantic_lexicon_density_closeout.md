# Semantic Lexicon Density Closeout

Status: closeout draft for `#479`

See also:

- [`semantic_command_lexicon.md`](semantic_command_lexicon.md)
- [`semantic_style.md`](semantic_style.md)
- [`semantic_hello_world_shape.md`](semantic_hello_world_shape.md)

## 1. Purpose

This document closes `#479` Lexicon/Density.

## 2. Closure Basis

### `#617` LEXICON-A - command lexicon skeleton

- file: `docs/language/semantic_command_lexicon.md`
- purpose: create the initial command / primitive lexicon skeleton
- result: lexicon entry schema, category skeleton, and first-pass table were
  added

### `#618` LEXICON-B - entry / lifecycle and completion vocabulary

- file: `docs/language/semantic_command_lexicon.md`
- purpose: refine entry/lifecycle and transition/completion vocabulary
- result: `entry`, `complete`, `transition`, and related bridge boundaries were
  defined as non-executable directions

### `#619` LEXICON-C - requirement / verification / admission vocabulary

- file: `docs/language/semantic_command_lexicon.md`
- purpose: refine requirement, verification, and admission vocabulary
- result: `require`, `verify`, `admit`, `assert`, and `check` were separated
  into directional / bridge / tooling roles

### `#620` LEXICON-D - observation / effect vocabulary

- file: `docs/language/semantic_command_lexicon.md`
- purpose: refine observation and controlled-effect vocabulary
- result: `observe`, `observation sink`, `controlled effect`, `print`,
  `stdout`, and `emit` were classified with bridge and implementation-detail
  boundaries

### `#621` LEXICON-E - compact quad-style density rules

- file: `docs/language/semantic_style.md`
- purpose: define compact quad-style density and formatting principles
- result: style guidance, density table, and legacy-shaped anti-pattern versus
  Semantic-native directional shape were recorded

### `#622` LEXICON-F - Hello World canonical shape decision

- file: `docs/language/semantic_hello_world_shape.md`
- purpose: prepare the canonical Hello World shape decision
- result: legacy `fn main` / `print` / `return` was rejected as canonical and
  the verbose Semantic directional shape was recommended as the canonical
  direction for later implementation planning

## 3. `#479` Acceptance Criteria Status

| `#479` acceptance criterion | Evidence | Status |
|---|---|---|
| command / primitive lexicon document added | `#617` and `docs/language/semantic_command_lexicon.md` | covered |
| syntax density / style document or section added | `#621` and `docs/language/semantic_style.md` | covered |
| legacy terms classified | `#617` through `#620` | covered |
| primitive / command categories defined | `#617` and `#618` | covered |
| quad / semantic density principles defined | `#621` | covered |
| legacy / airy examples identified | `#621` and `#622` | covered |
| follow-up PR list produced | `#617` through `#622` | covered |
| Hello World does not canonize airy `print` / `return` / `main` style | `#622` and `docs/language/semantic_hello_world_shape.md` | covered |

## 4. Established Lexicon / Density Decisions

- `entry` is the preferred direction, not executable yet.
- `fn main` / `main` remain bridge-only / rejected as canonical.
- `complete` is the preferred direction, not executable yet.
- `return` remains bridge-only.
- `require` is the preferred direction, not executable yet.
- `assert` remains bridge-only.
- `verify` and `admit` are verifier / admission vocabulary, not general source
  syntax by default.
- `check` remains CLI / tooling.
- `observe` is the preferred direction, not executable yet.
- `print` is rejected-as-canonical.
- `stdout` is implementation-detail / host-channel wording.
- generic `I/O` is not canonical source vocabulary.
- `controlled effect` is preferred model wording.
- compact quad-style density is style guidance, not grammar.
- recommended Hello World direction is verbose Semantic directional shape,
  non-executable and not grammar-final.

## 5. Deferred / Blocked Work

The following remain blocked until later accepted issues:

- `#477` M-Hello implementation
- grammar work for `entry`, `state`, `require`, `observe`, `complete`
- observe / effect runtime implementation
- capability / effect admission changes
- SemCode lowering plan for new surface
- VM / runtime behavior
- tests / fixtures / golden SemCode
- CTF impact check for implementation
- README / examples alignment
- formatter / linter implementation
- Linguist readiness `#356..#362`
- UI / Workbench / I70

## 6. Next Track Recommendation

After `#479` closeout, the next possible track is not automatic.

Recommended next candidates:

- `M-HELLO-0 — docs(hello): implementation readiness checklist for #477`
- `GRAMMAR-ENTRY-0 — docs(grammar): prepare entry/require/observe/complete
  grammar plan`
- `README-EXAMPLES-0 — docs(examples): prepare post-lexicon example alignment
  plan`

This PR starts none of them.

Given the current dependency order, recommend:

- `M-HELLO-0` only if the maintainer explicitly accepts moving toward `#477`
  planning.

## 7. Phase Result

`#479` Lexicon/Density: `closed after maintainer acceptance`.

`#477` M-Hello: `still blocked pending explicit implementation planning`.

## 8. Acceptance Checklist

- `#479` closure basis recorded
- `#479` acceptance criteria mapped to evidence
- established lexicon / density decisions summarized
- blocked work listed
- next track candidates listed
- no code/test/fixture changes
- no grammar changes
- no Hello World implementation
- no `print` / `observe` implementation
- no README/examples rewrite
- no formatter implementation
- no Linguist readiness
- no UI / Workbench / I70
