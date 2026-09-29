# Semantic Project Health Ledger — Interactive Dashboard

**Repository:** `skulmakov-oss/Semantic`  
**Audit Baseline Commit:** `e41ff311` (`origin/main`)  
**Ledger Version:** `v1.0-frozen`  
**Primary Entrypoint:** [`index.html`](index.html)  
**Single-File Distribution:** [`../Semantic_Project_Health_Dashboard.html`](../Semantic_Project_Health_Dashboard.html)

---

## 1. Overview & Purpose

This interactive HTML dashboard serves as the **read-only presentation and operational exploration layer** over the frozen **Semantic Project Health Ledger**. It provides maintainers, architects, and contributors with instant, traceable access to historical PR review debt, root causes, remediation packages, PR coverage, review threads, SSF mapping, and security/privacy artifacts.

### Architectural Hierarchy

```text
Immutable Raw Audit Evidence Layer
reports/security/historical_pr_review_audit.csv
                      │
                      ▼
Derived Source-of-Truth Management Layer
reports/security/Semantic_Project_Health_Ledger.xlsx
                      │
                      ▼ (deterministic fail-closed generator)
reports/security/dashboard/ledger_data.json / ledger_data.js
                      │
                      ▼
Interactive HTML Dashboard (index.html / Semantic_Project_Health_Dashboard.html)
```

The HTML dashboard is strictly a presentation layer and does not introduce or modify any underlying technical state or classification.

---

## 2. Dataset Metrics & Invariants

The generator validates all canonical metrics before emitting any data artifacts:

| Entity | Canonical Metric | Notes |
| :--- | :--- | :--- |
| **Closed PRs** | **1,477** | 1,372 merged + 105 closed-unmerged |
| **Review Threads** | **819** | 375 unresolved + 444 resolved |
| **Candidate Findings** | **813** | 100% 1:1 mapped to raw CSV evidence |
| **STILL_PRESENT** | **220** | Confirmed present on `origin/main` |
| **PARTIALLY_FIXED** | **4** | Partially resolved, remaining debt active |
| **FIXED_LATER** | **440** | Resolved in subsequent commits/PRs |
| **OBSOLETE** | **44** | Superseded by architectural rewrites |
| **FALSE_POSITIVE** | **97** | Invalid or non-reproducible historical review claims |
| **UNVERIFIED** | **8** | Obsolete or non-reproducible external environment assumptions |
| **Total Active Debt** | **224** | 220 `STILL_PRESENT` + 4 `PARTIALLY_FIXED` |
| **Root Causes** | **29** | 27 active debt causes + 2 resolved |
| **Remediation Packages** | **27** | `REM-001` .. `REM-027` (average 8.3 findings/PR) |
| **Credential Exposures** | **0** | Zero leaked tokens, secrets, or private keys across 1,477 PRs |
| **Privacy Path Leaks** | **119** | Local dev directory paths across 58 PRs (usernames masked) |
| **CodeQL Alerts** | **1** | `SEC-001` (PR #1872 CWE-532 cleartext session_id logging) |
| **Security Records** | **120** | 1 CodeQL Confidentiality Alert + 119 Privacy Path Artifacts |

---

## 3. How to Run & Refresh

### Offline Double-Click Execution
The dashboard is designed for **100% offline usage** without external dependencies:
1. Double-click `reports/security/dashboard/index.html` in any modern browser (Chrome, Edge, Firefox, Safari).
2. Alternatively, double-click the self-contained single-file bundle `reports/security/Semantic_Project_Health_Dashboard.html`.

No web server (`http://`), Node.js, CDN, or network connection is required. `ledger_data.js` injects the dataset directly into `window.SEMANTIC_LEDGER_DATA`, bypassing browser `file://` CORS restrictions.

### Regenerating from Ledger
If `Semantic_Project_Health_Ledger.xlsx` is updated, run the deterministic generator:

```bash
python tools/generate_health_dashboard.py
```

The generator will:
1. Assert all 41 canonical totals and invariants.
2. Validate 100% referential integrity (foreign keys between findings, root causes, remediation packages, PRs, and review threads).
3. Stop generation with a non-zero exit code if any drift or contradiction is detected.
4. Update `ledger_data.json`, `ledger_data.js`, and `Semantic_Project_Health_Dashboard.html`.

---

## 4. Dashboard Features & Capabilities

- **Executive Overview**: High-impact KPI cards and 6 custom vanilla SVG charts (Findings by status, Active debt by severity, Subsystem distribution, PR era accumulation timeline, Remediation packages by area, SSF alignment).
- **Findings Explorer**: Full-text instant search across 10 fields, multi-filtering (Status, Severity, Area, Finding Type, Security Class, SSF, Root Cause, Remediation), column sorting, pagination (25/50/100/250/All), and RFC 4180 CSV export.
- **Slide-Over Detail Drawer**: Comprehensive inspection panel showing finding metadata, original review comments, thread status, lineage file/line, validation evidence, and cross-navigation links.
- **Root Causes**: 29 architectural cards with active debt counts, affected PRs, remediation strategy, and one-click filtering to all related findings.
- **Remediation Queue**: 27 cohesive PR packages with an interactive SVG Dependency DAG modeling unblocked vs. dependent packages (`REM-004 -> REM-005 -> REM-007`, `REM-001 + REM-008 -> REM-011`, etc.).
- **PR Coverage**: Explorer for all 1,477 PRs with review thread counts, candidate findings, active debt, and PR drill-down drawer.
- **Review Threads**: 819 historical review threads with clear visual distinction between *Review State* (Unresolved/Resolved) and *Technical State* (Still Present/Fixed Later/etc.).
- **SSF Mapping**: Analysis of how the 813 findings map to the Semantic Specification Framework, verifying zero logical contradictions.
- **Security & Privacy**: Separated view of architectural security impact findings (19) vs privacy path leaks (119 across 58 PRs) with sanitized evidence and active GitHub links.
- **Deep Linking**: Hash-based URL navigation (`#view=findings`, `#finding=FND-069`, `#rc=RC-012`, `#rem=REM-012`, `#pr=385`).
- **Dark / Light Theme**: High-contrast, accessibility-conscious engineering themes persisted in `localStorage`.
