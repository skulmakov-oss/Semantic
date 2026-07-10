# UI-IMPL-1 Renderer Contract Test Tightening Branch Plan

## Status

Result: READY-WITH-WARNINGS

This is a branch/implementation plan only.

No code was changed.
No tests were changed.
No examples/7hell files were changed.
No runtime/verifier/VM/SemCode files were changed.
No PCC/CTF files were changed.
No untracked residue was touched.

## Purpose

Define the exact first implementation branch boundary for renderer contract test tightening.

## Source repo state

- branch: `main`
- HEAD: `fca5e55108fd47246679bc19e32a0579e2dce835`
- origin/main: `fca5e55108fd47246679bc19e32a0579e2dce835`
- HEAD == origin/main: yes
- AGENTS.md state: clean
- tracked dirty state: none
- untracked residue: local PCC/audit residue remains present and untouched

## Source/test inspection

| File | Role | Finding |
|---|---|---|
| `crates/prom-ui/src/renderer.rs` | renderer contract source | Defines the inert renderer model, node, marker, diagnostics, trace, marker, and inspection presentation APIs. It already reads as a presentation-contract boundary, not a runtime or backend owner. |
| `crates/prom-ui/src/projection.rs` | projection source | Supplies the projected artifact and projected node types that feed the renderer contract without moving authority into the renderer. |
| `crates/prom-ui/src/tree_bridge.rs` | tree bridge source | Preserves source-to-render mapping as a bridge layer, not a semantic authority layer. |
| `crates/prom-ui/tests/renderer_public_api_lock.rs` | target test | Locks the public renderer entrypoint, public types, and accessor surface; also contains inertness smoke checks. This is the correct seam for a test-only tightening slice. |

## Proposed future branch

- branch: `codex/ui-renderer-contract-test-tightening`
- commit message: `test(ui): tighten renderer public contract lock`
- owning crate: `prom-ui`
- allowed files: `crates/prom-ui/tests/renderer_public_api_lock.rs`
- forbidden files: `crates/prom-ui/src/*`, `crates/prom-ui-runtime/*`, `crates/prom-ui-backend-native/*`, tests outside `crates/prom-ui/tests/renderer_public_api_lock.rs`, `examples/*`, `tools/7hell/*`, `docs/roadmap/pcc/*`, `runtime/verifier/VM/SemCode` files

## Implementation boundary

| Area | Allowed | Forbidden |
|---|---|---|
| renderer public contract | add explicit assertions for already-existing public surface locks | change renderer API or add new renderer behavior |
| production code | none | any production code change in `crates/prom-ui/src/renderer.rs` or elsewhere |
| projection/tree bridge | none | change projection semantics or tree bridge semantics |
| native/WGPU/backend | none | add or change native/WGPU behavior, host bridge, or backend ownership |
| runtime/verifier/VM/SemCode | none | any authority, admission, or execution change |
| PCC/CTF | none | any PCC/CTF doc or trail change |

## Acceptance criteria

Future implementation is acceptable only if:

1. diff is limited to `crates/prom-ui/tests/renderer_public_api_lock.rs`;
2. no production code changes are required;
3. no runtime/verifier/VM/SemCode changes are required;
4. no native/WGPU backend changes are required;
5. no PCC/CTF residue is touched;
6. targeted validation passes:

```powershell
cargo test -p prom-ui --test renderer_public_api_lock
```

## Rollback criteria

Revert or stop if:

- production code must change;
- renderer API must change;
- the test reveals an unclear or unstable public contract;
- the targeted test requires broad refactor or extra crates;
- the diff escapes the approved file list;
- tracked dirty files reappear.

## Required gates before implementation

1. Create a dedicated branch from clean `main == origin/main`.
2. Change only the approved test file.
3. Do not touch production renderer code unless owner approval is explicit.
4. Do not touch runtime/verifier/VM/SemCode.
5. Do not touch PCC/CTF residue.
6. Run the targeted validation above before any broader validation.
7. If the targeted test fails because the contract is broader than expected, stop and replan before touching production code.

## Non-goals

- no renderer rewrite
- no backend switch
- no native/WGPU implementation change
- no runtime/verifier/VM changes
- no PCC/CTF changes
- no examples/7hell changes
- no cleanup of local residue

## Recommended next step

Choose exactly one:

- start implementation branch
- request owner decision
- block implementation

## Final verdict

The implementation branch may start only as a one-file, test-only `prom-ui` contract-tightening branch on `crates/prom-ui/tests/renderer_public_api_lock.rs`.

The repository is aligned enough for that narrow branch, but untracked local residue must remain isolated and out of scope.
