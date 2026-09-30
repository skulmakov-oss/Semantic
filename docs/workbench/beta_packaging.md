# Historical Workbench Beta Packaging

> [!IMPORTANT]
> **Retired contour — historical record.** Native Semantic UI, Workbench, and
> Semantic Studio are retired from the active roadmap (#1968 and #1862 closed
> as not planned). This document is preserved as written for its era; any
> readiness, planning, or "proposed" status below is not an active commitment,
> and its open items are not being remediated. See
> [UI, Workbench, and Studio retirement](../roadmap/ui_workbench_studio_retirement.md).

> [!NOTE]
> The former TypeScript/Tauri packaging path (`scripts/package_workbench_beta.ps1` and `apps/workbench_ts_tauri_legacy`) was retired under Issue **#1859** to eliminate unmaintained npm and Tauri dependency surfaces from `main`. See [`docs/history/workbench_ts_tauri_legacy.md`](../history/workbench_ts_tauri_legacy.md) for historical provenance.

The canonical native Semantic Workbench (`examples/workbench_semantic`) is validated via:

```powershell
pwsh -File scripts/workbench_native_launch_smoke.ps1
```
