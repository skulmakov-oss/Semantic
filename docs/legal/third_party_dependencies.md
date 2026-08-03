# Third-Party Dependency Register

Status: Draft
Track: R12 / POST-UI / Workbench dependency posture
Scope: documentation only

## 1. Purpose

This register records actual third-party dependencies admitted by repository manifests.

It is distinct from third-party influence or architectural inspiration.

It does not authorize new dependencies.

It does not authorize Workbench, Semantic Studio, renderer, browser/WebView, or widget framework ownership.

It does not claim final legal clearance.

License fields marked `pending verification` require later source verification.

## 2. Classification Rules

- Inspiration: architectural reference only; no code dependency.
- Dependency: crate or npm package used by repository manifests.
- Derivative / fork: copied or modified third-party code; requires explicit license and copyright handling.

Current audit found no derivative / fork evidence.

Dependency entries below are based on local manifests only.

## 3. Relationship To Influence Register

See:

- [`docs/legal/third_party_influence.md`](./third_party_influence.md)
- [`docs/dna/SEMANTIC_UI_DNA.md`](../dna/SEMANTIC_UI_DNA.md)

The following names remain influence-only unless they also appear in manifests:

- Slint
- Lapce / Floem
- Makepad
- Zed / GPUI
- Monaco / CodeMirror
- React Flow / Cytoscape / ELK

Tauri appears both as an architectural influence and as an actual dependency surface in the Workbench Tauri backend.

## 4. Dependency Groups

### 4.1 Workspace / shared Rust dependencies

| Dependency | Source manifest | Use class | Scope | Optional | License status | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| `serde` | `Cargo.toml` | serialization | shared infrastructure | yes | pending verification | Workspace-level shared Rust dependency. |
| `serde_json` | `Cargo.toml` | serialization | shared infrastructure | yes | pending verification | Workspace-level shared Rust dependency. |

### 4.2 Workbench Tauri backend dependencies

| Dependency | Source manifest | Use class | Scope | Optional | License status | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| `tauri-build` | `apps/workbench_ts_tauri_legacy/src-tauri/Cargo.toml` | build-time | Workbench backend | no | pending verification | Build dependency for the Tauri backend crate. |
| `serde` | `apps/workbench_ts_tauri_legacy/src-tauri/Cargo.toml` | serialization | Workbench backend | no | pending verification | Backend serialization dependency. |
| `serde_json` | `apps/workbench_ts_tauri_legacy/src-tauri/Cargo.toml` | serialization | Workbench backend | no | pending verification | Backend JSON serialization dependency. |
| `log` | `apps/workbench_ts_tauri_legacy/src-tauri/Cargo.toml` | logging | Workbench backend | no | pending verification | Logging dependency for backend diagnostics. |
| `tauri` | `apps/workbench_ts_tauri_legacy/src-tauri/Cargo.toml` | runtime | Workbench backend | no | pending verification | Actual Tauri runtime dependency for the Workbench shell. |
| `tauri-plugin-log` | `apps/workbench_ts_tauri_legacy/src-tauri/Cargo.toml` | runtime / logging | Workbench backend | no | pending verification | Tauri log plugin used by the backend shell. |

### 4.3 Workbench frontend runtime dependencies

| Dependency | Source manifest | Use class | Scope | Optional | License status | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| `@tauri-apps/api` | `apps/workbench_ts_tauri_legacy/package.json` | runtime | Workbench frontend | no | pending verification | Frontend Tauri API bridge. |
| `react` | `apps/workbench_ts_tauri_legacy/package.json` | runtime | Workbench frontend | no | pending verification | UI runtime framework for the Workbench shell. |
| `react-dom` | `apps/workbench_ts_tauri_legacy/package.json` | runtime | Workbench frontend | no | pending verification | React DOM renderer for the Workbench shell. |
| `react-router-dom` | `apps/workbench_ts_tauri_legacy/package.json` | runtime | Workbench frontend | no | pending verification | Routing/runtime support for Workbench UI navigation. |

### 4.4 Workbench frontend tooling / dev dependencies

| Dependency | Source manifest | Use class | Scope | Optional | License status | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| `@tauri-apps/cli` | `apps/workbench_ts_tauri_legacy/package.json` | dev-only / tooling | Workbench frontend | no | pending verification | CLI tooling for Tauri app development. |
| `@vitejs/plugin-react` | `apps/workbench_ts_tauri_legacy/package.json` | dev-only / tooling | Workbench frontend | no | pending verification | Vite React plugin. |
| `@eslint/js` | `apps/workbench_ts_tauri_legacy/package.json` | linting | Workbench frontend | no | pending verification | ESLint JS config package. |
| `@types/node` | `apps/workbench_ts_tauri_legacy/package.json` | type definitions | Workbench frontend | no | pending verification | TypeScript type support. |
| `@types/react` | `apps/workbench_ts_tauri_legacy/package.json` | type definitions | Workbench frontend | no | pending verification | React type definitions. |
| `@types/react-dom` | `apps/workbench_ts_tauri_legacy/package.json` | type definitions | Workbench frontend | no | pending verification | React DOM type definitions. |
| `eslint` | `apps/workbench_ts_tauri_legacy/package.json` | linting | Workbench frontend | no | pending verification | Linting tool. |
| `eslint-plugin-react-hooks` | `apps/workbench_ts_tauri_legacy/package.json` | linting | Workbench frontend | no | pending verification | React hooks lint rules. |
| `eslint-plugin-react-refresh` | `apps/workbench_ts_tauri_legacy/package.json` | linting | Workbench frontend | no | pending verification | React refresh lint rules. |
| `globals` | `apps/workbench_ts_tauri_legacy/package.json` | dev-only / tooling | Workbench frontend | no | pending verification | Shared global identifier definitions. |
| `typescript` | `apps/workbench_ts_tauri_legacy/package.json` | dev-only / tooling | Workbench frontend | no | pending verification | TypeScript compiler and language tooling. |
| `typescript-eslint` | `apps/workbench_ts_tauri_legacy/package.json` | linting | Workbench frontend | no | pending verification | TypeScript-aware ESLint tooling. |
| `vite` | `apps/workbench_ts_tauri_legacy/package.json` | bundling | Workbench frontend | no | pending verification | Frontend bundler and dev server. |

### 4.5 Optional native backend dependencies

| Dependency | Source manifest | Use class | Scope | Optional | License status | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| `winit` | `crates/prom-ui-backend-native/Cargo.toml` | runtime / adapter | native backend adapter | yes | pending verification | Optional feature-gated native backend dependency. This does not make renderer ownership part of Semantic UI. |

### 4.7 Iced native UI substrate (owner directive 2026-08-02, Workbench Iced substrate migration)

| Dependency | Source manifest | Use class | Scope | Optional | License status | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| `iced` (`=0.14.0`, `features = ["advanced", "smol"]`) | `crates/prom-ui-iced-adapter/Cargo.toml` | runtime | private Iced-backed Prom UI adapter | no | verified: MIT | Exact pinned crates.io release. Full upstream record (repository, version, license, features, local modifications: none, update procedure) in [`third_party/iced/UPSTREAM.md`](../../third_party/iced/UPSTREAM.md); verbatim upstream license text (fetched directly from the `iced-rs/iced` repository, not summarized) in [`third_party/iced/LICENSE-MIT`](../../third_party/iced/LICENSE-MIT). Transitively pulls `iced_winit ^0.14.0` and `wgpu 27.0.1` -- these coexist with, and do not replace, the pre-existing `winit`/optional-`wgpu` surface in `crates/prom-ui-backend-native` (row 4.5 above and `crates/quad_logic_calculator`'s own consumption of it are unaffected). The `smol` feature (MIT/Apache-2.0, `smol-rs/smol`) is Iced's own lightweight async executor choice, required for `iced::time::every` (the real periodic subscription backing `PromApplication::on_tick`) -- not a new top-level dependency, a feature flag on the already-authorized `iced` crate itself, chosen over the heavier `tokio` alternative Iced also offers since nothing else in this workspace needs a full async runtime. Only `crates/prom-ui-iced-adapter` (and, transitively, its own lower platform crates) may depend on `iced` -- enforced by real, passing dependency-boundary tests: `crates/prom-ui-iced-adapter/tests/dependency_boundary.rs` and `examples/workbench_semantic/tests/no_iced_dependency.rs`. |

### 4.6 Internal workspace crates not third-party dependencies

The following are workspace-owned crates and are not third-party dependency entries:

- `prom-ui`
- `prom-ui-runtime`
- `prom-ui-backend-native`
- `prom-ui-demo`
- `prom-abi`

## 5. Non-Adoption / Non-Ownership Notes

- React and Tauri are Workbench implementation dependencies, not Semantic UI model owners.
- `winit` is an optional backend-adapter surface, not a Semantic UI model owner.
- No dependency listed here owns UI Tree, UI AST, UI IR, Semantic state/update/event model, capability/effect discipline, diagnostics/fault model, or renderer adapter contract.
- No dependency listed here authorizes browser/WebView ownership of Semantic state.
- No dependency listed here authorizes widget framework scope.
- No dependency listed here widens release scope.

## 6. License Verification Status

License values are pending verification unless directly present in local manifest evidence.

A later legal pass must verify licenses against package or crate source metadata before release claims.

This register is an inventory and control document, not final legal approval.

## 7. Current Risk Summary

- Unregistered dependency gap before this patch: yes
- Derivative / fork evidence: none found
- Final legal clearance: not yet
- Release widening: no
- Implementation authorization: no

## 8. Follow-Up Items

- Verify licenses for all direct third-party dependencies.
- Decide whether to add version columns from lockfiles in a later pass.
- Keep influence, dependency, and derivative categories separate.
- Update this register when manifests change.
- Keep the `#675` pause active for Workbench and Semantic Studio implementation.
