# Historical Workbench Beta Packaging

> [!NOTE]
> The former TypeScript/Tauri packaging path (`scripts/package_workbench_beta.ps1` and `apps/workbench_ts_tauri_legacy`) was retired under Issue **#1859** to eliminate unmaintained npm and Tauri dependency surfaces from `main`. See [`docs/history/workbench_ts_tauri_legacy.md`](../history/workbench_ts_tauri_legacy.md) for historical provenance.

The canonical native Semantic Workbench (`examples/workbench_semantic`) is validated via:

```powershell
pwsh -File scripts/workbench_native_launch_smoke.ps1
```
