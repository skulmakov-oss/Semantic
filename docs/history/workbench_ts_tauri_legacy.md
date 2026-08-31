# Historical Provenance: Legacy TypeScript / Tauri Workbench (`apps/workbench_ts_tauri_legacy`)

## Status

**Retired and Non-Canonical** — Historical Evidence Only.

This document records the historical provenance of the original TypeScript/React/Tauri Workbench application tree (`apps/workbench_ts_tauri_legacy`). This tree was permanently removed from the active dependency graph of `main` under GitHub Issue **#1859**.

This document is historical evidence only. It is **not** an architectural specification, nor is it a guide or recipe for re-introducing an active fallback or dual-maintenance implementation.

---

## Migration & Superseding Authority

1. **Canonical Workbench**: The native Semantic + Prom UI Workbench (`examples/workbench_semantic/`), built on the native Iced UI substrate, is the sole canonical Workbench implementation for Semantic.
2. **Canonical Migration Point**: The native Workbench was introduced and designated as canonical in **PR #1567** (commit `b7f2327b`).
3. **Release & Context Reference**: **#1568** (commit `cf355b17`, release notes for `semantic-workbench-v0.2.0-beta1`) documents the generation change replacing the legacy TS/Tauri implementation with the native Iced implementation.

---

## Retirement Rationale & Dependency Surface

- **Active Surface Removal**: The archived tree `apps/workbench_ts_tauri_legacy/` contained npm manifests (`package.json`, `package-lock.json`) and a Tauri Rust backend (`src-tauri/Cargo.toml`). Leaving this archived tree in the repository caused security vulnerability scanners (such as GitHub Dependabot) to continuously audit an unmaintained dependency graph.
- **Intentional Non-Maintenance**: The old npm/Tauri dependency surface is intentionally **not** maintained on `main`. No dependency upgrades (`npm audit fix`, Vite upgrades, React Router upgrades, etc.) or manifest renaming tricks were applied to hide alerts. The attack surface was permanently eliminated by removing the obsolete application tree.
- **Zero Fallback Invariant**: The repository contains zero fallback execution routes, zero compatibility shims, and zero dynamic dispatch to the retired TS/Tauri Workbench.

---

## Historical Recovery via Git History

The historical implementation remains fully preserved and recoverable through Git history.

- **Pre-removal Recovery Commit**: `3d0adf1bb791a84622fde4c3f1a172f6cbdda044`
- **Non-Mutating Git Inspection Command**:
  ```bash
  git ls-tree -r --name-only 3d0adf1bb791a84622fde4c3f1a172f6cbdda044 -- apps/workbench_ts_tauri_legacy
  ```
