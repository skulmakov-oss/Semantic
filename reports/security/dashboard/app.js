/**
 * Semantic Project Health Ledger — Main Dashboard Application
 * Pure Vanilla JavaScript (Zero external runtime dependencies, 100% offline file:// compatible)
 */

(function () {
  "use strict";

  // Application State
  const state = {
    data: null,
    currentView: "overview",
    filters: {
      findings: { search: "", status: "", severity: "", area: "", type: "", sec: "", ssf: "", rc: "", rem: "", quickActive: false, quickHigh: false, quickSec: false },
      rootCauses: { search: "", status: "", area: "", severity: "" },
      remediation: { search: "", area: "", priority: "", tier: "" },
      prs: { search: "", merged: "", threads: "", active: "" },
      threads: { search: "", resolved: "", status: "", author: "" },
      ssf: { search: "", relation: "", status: "" },
      sec: { search: "", category: "", type: "" }
    },
    sorting: {
      findings: { col: "Finding_ID", dir: "asc" },
      prs: { col: "PR_Number", dir: "desc" },
      threads: { col: "PR_Number", dir: "desc" },
      ssf: { col: "Finding_ID", dir: "asc" },
      sec: { col: "Security_ID", dir: "asc" }
    },
    pagination: {
      findings: { page: 1, pageSize: 25 },
      prs: { page: 1, pageSize: 25 },
      threads: { page: 1, pageSize: 25 },
      ssf: { page: 1, pageSize: 25 },
      sec: { page: 1, pageSize: 25 }
    },
    activeEntity: null, // { type: 'finding'|'rc'|'rem'|'pr'|'thread'|'sec', id: '...' }
  };

  const SEV_WEIGHT = { CRITICAL: 5, HIGH: 4, MEDIUM: 3, LOW: 2, UNKNOWN: 1 };
  const STATUS_WEIGHT = { STILL_PRESENT: 6, PARTIALLY_FIXED: 5, FIXED_LATER: 4, UNVERIFIED: 3, OBSOLETE: 2, FALSE_POSITIVE: 1 };

  // =========================================================================
  // Initialization & Bootstrapping
  // =========================================================================
  function init() {
    initTheme();

    if (window.SEMANTIC_LEDGER_DATA) {
      state.data = window.SEMANTIC_LEDGER_DATA;
      onDataLoaded();
    } else {
      fetch("ledger_data.json")
        .then(res => {
          if (!res.ok) throw new Error("Failed to fetch ledger_data.json: " + res.status);
          return res.json();
        })
        .then(data => {
          state.data = data;
          onDataLoaded();
        })
        .catch(err => {
          console.error("Ledger data load failure:", err);
          document.body.innerHTML = `<div style="padding:40px; color:#ef4444; font-family:sans-serif;">
            <h2>Fatal: Could not load Ledger Dataset</h2>
            <p>Ensure <code>ledger_data.js</code> or <code>ledger_data.json</code> exists in the dashboard directory.</p>
            <pre>${escapeHtml(err.message)}</pre>
          </div>`;
        });
    }
  }

  function onDataLoaded() {
    console.log("Semantic Ledger loaded successfully:", state.data.summary);
    populateDropdowns();
    setupNavigation();
    setupFiltersAndSearch();
    setupSorting();
    setupDrawer();
    setupExportButtons();

    // Handle initial URL hash
    handleHashChange();
    window.addEventListener("hashchange", handleHashChange);

    // Initial Render of overview
    renderOverview();
  }

  // =========================================================================
  // Theme Management (Dark / Light)
  // =========================================================================
  function initTheme() {
    const savedTheme = localStorage.getItem("semantic_ledger_theme") || "dark";
    setTheme(savedTheme);

    const toggleBtn = document.getElementById("btn-theme-toggle");
    if (toggleBtn) {
      toggleBtn.addEventListener("click", () => {
        const cur = document.documentElement.getAttribute("data-theme") || "dark";
        const next = cur === "dark" ? "light" : "dark";
        setTheme(next);
      });
    }
  }

  function setTheme(theme) {
    document.documentElement.setAttribute("data-theme", theme);
    localStorage.setItem("semantic_ledger_theme", theme);
    const icon = document.getElementById("theme-icon");
    const label = document.getElementById("theme-label");
    if (icon && label) {
      icon.textContent = theme === "dark" ? "☀️" : "🌙";
      label.textContent = theme === "dark" ? "Light Mode" : "Dark Mode";
    }
  }

  // =========================================================================
  // Navigation & Hash Routing
  // =========================================================================
  function setupNavigation() {
    const navItems = document.querySelectorAll(".sidebar-nav .nav-item");
    navItems.forEach(item => {
      item.addEventListener("click", e => {
        e.preventDefault();
        const view = item.getAttribute("data-view");
        navigateTo(view);
      });
    });
  }

  function navigateTo(view, queryParams = {}) {
    state.currentView = view;

    // Update active nav link
    document.querySelectorAll(".sidebar-nav .nav-item").forEach(item => {
      item.classList.toggle("active", item.getAttribute("data-view") === view);
    });

    // Update view pane
    document.querySelectorAll(".app-main .pane").forEach(pane => {
      pane.classList.toggle("active", pane.id === `pane-${view}`);
    });

    // Update Header Title
    const titleMap = {
      overview: "Executive Overview",
      findings: "Findings Explorer (813 Audited)",
      "root-causes": "Architectural Root Causes (29 Causes)",
      remediation: "Remediation Queue (27 PR Packages)",
      "pr-coverage": "Repository PR Coverage (1,477 PRs)",
      "review-threads": "Historical Review Threads (819 Ingested)",
      ssf: "SSF Architectural Alignment (813 Mappings)",
      security: "Security & Privacy Governance (120 Records)",
      metadata: "Metadata & Provenance Governance"
    };
    const titleEl = document.getElementById("page-title");
    if (titleEl) titleEl.textContent = titleMap[view] || "Semantic Ledger";

    // Apply any passed query params to view filters
    if (view === "findings" && queryParams) {
      if (queryParams.status) {
        state.filters.findings.status = queryParams.status;
        const el = document.getElementById("filter-findings-status");
        if (el) el.value = queryParams.status;
      }
      if (queryParams.rc) {
        state.filters.findings.rc = queryParams.rc;
        const el = document.getElementById("filter-findings-rc");
        if (el) el.value = queryParams.rc;
      }
      if (queryParams.rem) {
        state.filters.findings.rem = queryParams.rem;
        const el = document.getElementById("filter-findings-rem");
        if (el) el.value = queryParams.rem;
      }
      if (queryParams.area) {
        state.filters.findings.area = queryParams.area;
        const el = document.getElementById("filter-findings-area");
        if (el) el.value = queryParams.area;
      }
      if (queryParams.severity) {
        state.filters.findings.severity = queryParams.severity;
        const el = document.getElementById("filter-findings-severity");
        if (el) el.value = queryParams.severity;
      }
    }

    // Render corresponding view
    renderCurrentView();

    // Update URL hash
    let hash = `#view=${view}`;
    if (queryParams && Object.keys(queryParams).length > 0) {
      const q = Object.entries(queryParams).map(([k, v]) => `${k}=${encodeURIComponent(v)}`).join("&");
      hash += `&${q}`;
    }
    history.replaceState(null, "", hash);
  }

  function handleHashChange() {
    const rawHash = window.location.hash.substring(1);
    if (!rawHash) {
      navigateTo("overview");
      return;
    }

    const params = new URLSearchParams(rawHash);
    const view = params.get("view") || "overview";

    if (params.get("finding")) {
      navigateTo("findings");
      openDrawer("finding", params.get("finding"));
      return;
    }
    if (params.get("rc")) {
      navigateTo("root-causes");
      openDrawer("rc", params.get("rc"));
      return;
    }
    if (params.get("rem")) {
      navigateTo("remediation");
      openDrawer("rem", params.get("rem"));
      return;
    }
    if (params.get("pr")) {
      navigateTo("pr-coverage");
      openDrawer("pr", params.get("pr"));
      return;
    }

    const queryParams = {};
    for (const [k, v] of params.entries()) {
      if (k !== "view") queryParams[k] = v;
    }
    navigateTo(view, queryParams);
  }

  function renderCurrentView() {
    switch (state.currentView) {
      case "overview":
        renderOverview();
        break;
      case "findings":
        renderFindings();
        break;
      case "root-causes":
        renderRootCauses();
        break;
      case "remediation":
        renderRemediation();
        break;
      case "pr-coverage":
        renderPRCoverage();
        break;
      case "review-threads":
        renderReviewThreads();
        break;
      case "ssf":
        renderSSF();
        break;
      case "security":
        renderSecurity();
        break;
      case "metadata":
        renderMetadata();
        break;
    }
  }

  // =========================================================================
  // VIEW 1: OVERVIEW & SVG CHARTS
  // =========================================================================
  function renderOverview() {
    const s = state.data.summary;

    // Fill KPI numbers
    document.getElementById("kpi-active-debt").textContent = s.active_debt;
    document.getElementById("kpi-root-causes").textContent = s.active_root_causes;
    document.getElementById("kpi-rem-packages").textContent = s.remediation_packages;
    document.getElementById("kpi-fixed-later").textContent = s.fixed_later;
    document.getElementById("kpi-total-findings").textContent = s.candidate_findings;

    // 1. Chart: Findings by Technical Status (Donut)
    renderDonutChart("chart-status-container", [
      { label: "STILL_PRESENT", count: s.still_present, color: "#ef4444" },
      { label: "PARTIALLY_FIXED", count: s.partially_fixed, color: "#f59e0b" },
      { label: "FIXED_LATER", count: s.fixed_later, color: "#10b981" },
      { label: "OBSOLETE", count: s.obsolete, color: "#94a3b8" },
      { label: "FALSE_POSITIVE", count: s.false_positive, color: "#6366f1" },
      { label: "UNVERIFIED", count: s.unverified, color: "#a855f7" }
    ], {
      onClick: item => navigateTo("findings", { status: item.label })
    });

    // 2. Chart: Active Debt by Severity (Bar)
    const sevData = [
      { label: "HIGH", count: s.active_severity_distribution.HIGH || 0, color: "#f97316" },
      { label: "MEDIUM", count: s.active_severity_distribution.MEDIUM || 0, color: "#eab308" },
      { label: "LOW", count: s.active_severity_distribution.LOW || 0, color: "#38bdf8" },
      { label: "CRITICAL", count: s.active_severity_distribution.CRITICAL || 0, color: "#ef4444" }
    ];
    renderBarChart("chart-active-sev-container", sevData, {
      onClick: item => navigateTo("findings", { severity: item.label, status: "STILL_PRESENT,PARTIALLY_FIXED" })
    });

    // 3. Chart: Active Debt by Subsystem / Area (Horizontal Bar)
    const topAreas = Object.entries(s.active_area_distribution).slice(0, 8).map(([area, count]) => ({
      label: area,
      count: count,
      color: "#38bdf8"
    }));
    renderHorizontalBarChart("chart-active-area-container", topAreas, {
      onClick: item => navigateTo("findings", { area: item.label, status: "STILL_PRESENT,PARTIALLY_FIXED" })
    });

    // 4. Chart: Review Debt Across PR Eras (Grouped Timeline)
    renderTimelineChart("chart-pr-era-container", s.pr_era_buckets);

    // 5. Chart: Remediation Packages by Area
    const remAreas = Object.entries(s.rem_area_distribution).map(([area, count]) => ({
      label: area,
      count: count,
      color: "#818cf8"
    }));
    renderHorizontalBarChart("chart-rem-area-container", remAreas, {
      onClick: item => navigateTo("remediation", { area: item.label })
    });

    // 6. Chart: SSF Alignment (Donut)
    const ssfItems = Object.entries(s.ssf_relation_distribution).map(([rel, count]) => {
      let color = "#64748b";
      if (rel === "FIXED_BY_SSF") color = "#10b981";
      else if (rel === "RELATED_TO_SSF") color = "#f59e0b";
      else if (rel === "PRE_SSF_FIX") color = "#38bdf8";
      else if (rel === "POST_SSF_FIX") color = "#818cf8";
      return { label: rel, count, color };
    });
    renderDonutChart("chart-ssf-container", ssfItems, {
      onClick: item => navigateTo("ssf", { relation: item.label })
    });
  }

  // =========================================================================
  // VANILLA SVG CHARTS ENGINE (Zero Dependencies)
  // =========================================================================
  function renderDonutChart(containerId, items, options = {}) {
    const el = document.getElementById(containerId);
    if (!el) return;

    const total = items.reduce((sum, d) => sum + d.count, 0);
    const size = 230;
    const strokeWidth = 32;
    const radius = (size - strokeWidth) / 2;
    const circumference = 2 * Math.PI * radius;

    let accumulatedAngle = 0;
    const paths = items.map(item => {
      const pct = total > 0 ? item.count / total : 0;
      const strokeDasharray = `${pct * circumference} ${circumference}`;
      const strokeDashoffset = -accumulatedAngle * circumference;
      accumulatedAngle += pct;

      return `<circle cx="${size / 2}" cy="${size / 2}" r="${radius}"
        fill="transparent"
        stroke="${item.color}"
        stroke-width="${strokeWidth}"
        stroke-dasharray="${strokeDasharray}"
        stroke-dashoffset="${strokeDashoffset}"
        style="cursor:pointer; transition: stroke-width 0.15s, opacity 0.15s;"
        onmouseover="this.style.strokeWidth='${strokeWidth + 4}'; this.style.opacity='0.9';"
        onmouseout="this.style.strokeWidth='${strokeWidth}'; this.style.opacity='1';"
        onclick="window.__chartClick('${containerId}', '${escapeHtml(item.label)}')"
      >
        <title>${item.label}: ${item.count} (${(pct * 100).toFixed(1)}%)</title>
      </circle>`;
    }).join("");

    window.__chartClick = (cid, label) => {
      if (cid === containerId && options.onClick) {
        const item = items.find(i => i.label === label);
        if (item) options.onClick(item);
      }
    };

    const legend = `<div style="display:flex; flex-direction:column; gap:6px; margin-left:24px; font-size:11.5px; max-height:220px; overflow-y:auto;">
      ${items.map(i => {
        const pct = total > 0 ? ((i.count / total) * 100).toFixed(1) : 0;
        return `<div style="display:flex; align-items:center; gap:8px; cursor:pointer;" onclick="window.__chartClick('${containerId}', '${escapeHtml(i.label)}')">
          <span style="width:10px; height:10px; border-radius:2px; background:${i.color}; flex-shrink:0;"></span>
          <span style="color:var(--text-secondary);">${escapeHtml(i.label)}</span>
          <strong style="margin-left:auto; color:var(--text-primary); font-family:var(--font-mono);">${i.count}</strong>
          <span style="color:var(--text-muted); font-size:10.5px;">(${pct}%)</span>
        </div>`;
      }).join("")}
    </div>`;

    el.innerHTML = `<div style="display:flex; align-items:center; justify-content:center; width:100%; padding:10px;">
      <svg width="${size}" height="${size}" viewBox="0 0 ${size} ${size}" style="transform: rotate(-90deg); flex-shrink:0;">
        ${paths}
        <text x="${size / 2}" y="${size / 2}" text-anchor="middle" dominant-baseline="middle"
          fill="var(--text-primary)" font-size="22" font-weight="700" font-family="var(--font-mono)"
          style="transform: rotate(90deg); transform-origin: center;">
          ${total}
        </text>
      </svg>
      ${legend}
    </div>`;
  }

  function renderBarChart(containerId, items, options = {}) {
    const el = document.getElementById(containerId);
    if (!el) return;

    const maxVal = Math.max(...items.map(i => i.count), 1);
    const height = 180;

    const bars = items.map(item => {
      const barHeight = (item.count / maxVal) * (height - 40);
      return `<div style="flex:1; display:flex; flex-direction:column; align-items:center; height:100%; justify-content:flex-end; cursor:pointer;"
        onclick="window.__chartBarClick('${containerId}', '${escapeHtml(item.label)}')">
        <span style="font-family:var(--font-mono); font-size:12px; font-weight:700; color:var(--text-primary); margin-bottom:4px;">
          ${item.count}
        </span>
        <div style="width:70%; max-width:44px; height:${Math.max(barHeight, 4)}px; background:${item.color}; border-radius:4px 4px 0 0; transition: transform 0.15s;"
          onmouseover="this.style.transform='scaleY(1.05)'" onmouseout="this.style.transform='scaleY(1)'"></div>
        <span style="font-size:11px; color:var(--text-secondary); margin-top:8px; font-weight:600;">
          ${escapeHtml(item.label)}
        </span>
      </div>`;
    }).join("");

    window.__chartBarClick = (cid, label) => {
      if (cid === containerId && options.onClick) {
        const item = items.find(i => i.label === label);
        if (item) options.onClick(item);
      }
    };

    el.innerHTML = `<div style="width:100%; height:${height}px; display:flex; align-items:flex-end; gap:16px; padding:10px 20px;">
      ${bars}
    </div>`;
  }

  function renderHorizontalBarChart(containerId, items, options = {}) {
    const el = document.getElementById(containerId);
    if (!el) return;

    const maxVal = Math.max(...items.map(i => i.count), 1);
    const rows = items.map(item => {
      const pct = (item.count / maxVal) * 100;
      return `<div style="display:flex; align-items:center; gap:12px; margin-bottom:8px; cursor:pointer;"
        onclick="window.__chartHClick('${containerId}', '${escapeHtml(item.label)}')">
        <span style="width:130px; font-size:11.5px; color:var(--text-secondary); text-align:right; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; font-family:var(--font-mono);">
          ${escapeHtml(item.label)}
        </span>
        <div style="flex:1; background:var(--bg-subtle); height:16px; border-radius:3px; overflow:hidden; position:relative;">
          <div style="width:${pct}%; height:100%; background:${item.color || '#38bdf8'}; border-radius:3px;"></div>
        </div>
        <span style="width:40px; font-size:11.5px; font-family:var(--font-mono); font-weight:600; color:var(--text-primary);">
          ${item.count}
        </span>
      </div>`;
    }).join("");

    window.__chartHClick = (cid, label) => {
      if (cid === containerId && options.onClick) {
        const item = items.find(i => i.label === label);
        if (item) options.onClick(item);
      }
    };

    el.innerHTML = `<div style="width:100%; padding:10px 16px;">${rows}</div>`;
  }

  function renderTimelineChart(containerId, buckets) {
    const el = document.getElementById(containerId);
    if (!el) return;

    const maxVal = Math.max(...buckets.map(b => b.total), 1);
    const height = 180;

    const bars = buckets.map(b => {
      const totalH = (b.total / maxVal) * (height - 45);
      const activeH = (b.active / maxVal) * (height - 45);
      return `<div style="flex:1; display:flex; flex-direction:column; align-items:center; height:100%; justify-content:flex-end;">
        <div style="display:flex; gap:3px; align-items:flex-end; width:100%; justify-content:center;">
          <div style="width:38%; max-width:24px; height:${Math.max(totalH, 4)}px; background:#64748b; border-radius:3px 3px 0 0;" title="Total Findings: ${b.total}"></div>
          <div style="width:38%; max-width:24px; height:${Math.max(activeH, 4)}px; background:#ef4444; border-radius:3px 3px 0 0;" title="Active Still Present: ${b.active}"></div>
        </div>
        <span style="font-size:10px; color:var(--text-muted); margin-top:8px; white-space:nowrap; font-family:var(--font-mono);">
          ${escapeHtml(b.era)}
        </span>
      </div>`;
    }).join("");

    const legend = `<div style="display:flex; justify-content:center; gap:20px; font-size:11.5px; margin-top:10px;">
      <div style="display:flex; align-items:center; gap:6px;">
        <span style="width:10px; height:10px; background:#64748b; border-radius:2px;"></span>
        <span style="color:var(--text-secondary);">Total Evaluated</span>
      </div>
      <div style="display:flex; align-items:center; gap:6px;">
        <span style="width:10px; height:10px; background:#ef4444; border-radius:2px;"></span>
        <span style="color:var(--text-secondary);">Still Active Debt</span>
      </div>
    </div>`;

    el.innerHTML = `<div style="width:100%; padding:10px;">
      <div style="width:100%; height:${height}px; display:flex; align-items:flex-end; gap:8px;">${bars}</div>
      ${legend}
    </div>`;
  }

  // =========================================================================
  // VIEW 2: FINDINGS EXPLORER
  // =========================================================================
  function renderFindings() {
    const f = state.filters.findings;
    const query = f.search.trim().toLowerCase();

    // 1. Filter items
    let items = state.data.findings.filter(item => {
      if (f.quickActive) {
        if (!["STILL_PRESENT", "PARTIALLY_FIXED"].includes(item.Current_Status)) return false;
      }
      if (f.quickHigh) {
        if (!["CRITICAL", "HIGH"].includes(item.Current_Severity)) return false;
      }
      if (f.quickSec) {
        if (!item.Security_Class || item.Security_Class === "NONE") return false;
      }

      if (f.status && f.status !== "") {
        const statuses = f.status.split(",");
        if (!statuses.includes(item.Current_Status)) return false;
      }
      if (f.severity && item.Current_Severity !== f.severity) return false;
      if (f.area && item.Area !== f.area) return false;
      if (f.type && item.Finding_Type !== f.type) return false;
      if (f.sec && item.Security_Class !== f.sec) return false;
      if (f.ssf && item.SSF_Relation !== f.ssf) return false;
      if (f.rc && item.Root_Cause_ID !== f.rc) return false;
      if (f.rem && item.Remediation_Group !== f.rem) return false;

      if (query) {
        const matchText = [
          item.Finding_ID, item.PR_Number, item.PR_Title, item.Finding_Summary,
          item.Original_Comment_Summary, item.Current_Main_File, item.Original_File,
          item.Root_Cause_ID, item.Remediation_Group, item.Review_Author, item.Finding_Type
        ].filter(Boolean).join(" ").toLowerCase();
        if (!matchText.includes(query)) return false;
      }

      return true;
    });

    // 2. Sort items
    const s = state.sorting.findings;
    items.sort((a, b) => {
      let va = a[s.col] || "";
      let vb = b[s.col] || "";

      if (s.col === "Current_Severity") {
        va = SEV_WEIGHT[va] || 0;
        vb = SEV_WEIGHT[vb] || 0;
      } else if (s.col === "Current_Status") {
        va = STATUS_WEIGHT[va] || 0;
        vb = STATUS_WEIGHT[vb] || 0;
      } else if (s.col === "PR_Number") {
        va = Number(va) || 0;
        vb = Number(vb) || 0;
      }

      if (va < vb) return s.dir === "asc" ? -1 : 1;
      if (va > vb) return s.dir === "asc" ? 1 : -1;
      return 0;
    });

    // 3. Render Active Filter Chips
    renderFilterChips("active-chips-findings", f, [
      { key: "status", label: "Status" },
      { key: "severity", label: "Severity" },
      { key: "area", label: "Area" },
      { key: "type", label: "Type" },
      { key: "sec", label: "Security" },
      { key: "ssf", label: "SSF" },
      { key: "rc", label: "Root Cause" },
      { key: "rem", label: "Remediation" },
    ], () => {
      f.status = f.severity = f.area = f.type = f.sec = f.ssf = f.rc = f.rem = "";
      f.quickActive = f.quickHigh = f.quickSec = false;
      updateFilterControls("findings");
      renderFindings();
    });

    // 4. Paginate
    const p = state.pagination.findings;
    const totalItems = items.length;
    const pageSize = p.pageSize === "All" ? totalItems : Number(p.pageSize);
    const totalPages = Math.ceil(totalItems / pageSize) || 1;
    if (p.page > totalPages) p.page = 1;
    const startIdx = (p.page - 1) * pageSize;
    const pageItems = items.slice(startIdx, startIdx + pageSize);

    // 5. Populate Table
    const tbody = document.getElementById("tbody-findings");
    if (!tbody) return;

    if (pageItems.length === 0) {
      tbody.innerHTML = `<tr><td colspan="10" style="text-align:center; padding:32px; color:var(--text-muted);">No findings match current filter criteria.</td></tr>`;
    } else {
      tbody.innerHTML = pageItems.map(item => `
        <tr onclick="app.openDrawer('finding', '${escapeHtml(item.Finding_ID)}')">
          <td class="mono-cell" style="font-weight:600; color:var(--accent-primary);">${escapeHtml(item.Finding_ID)}</td>
          <td>${getSeverityBadge(item.Current_Severity)}</td>
          <td>${getStatusBadge(item.Current_Status)}</td>
          <td><span class="tag-area">${escapeHtml(item.Area || "")}</span></td>
          <td style="font-size:11.5px; color:var(--text-secondary);">${escapeHtml(item.Finding_Type || "")}</td>
          <td class="mono-cell" style="font-size:11.5px;">${escapeHtml(item.Root_Cause_ID || "")}</td>
          <td class="mono-cell" style="font-size:11.5px;">${escapeHtml(item.Remediation_Group || "-")}</td>
          <td><a class="provenance-link" href="https://github.com/skulmakov-oss/Semantic/pull/${item.PR_Number}" target="_blank" onclick="event.stopPropagation();">#${item.PR_Number}</a></td>
          <td>${getSecurityClassBadge(item.Security_Class)}</td>
          <td style="max-width:320px; overflow:hidden; text-overflow:ellipsis; white-space:nowrap;" title="${escapeHtml(item.Finding_Summary || "")}">${escapeHtml(item.Finding_Summary || "")}</td>
        </tr>
      `).join("");
    }

    // 6. Pagination Bar
    renderPagination("pagination-findings", p, totalItems, totalPages, newPage => {
      p.page = newPage;
      renderFindings();
    }, newSize => {
      p.pageSize = newSize;
      p.page = 1;
      renderFindings();
    });
  }

  // =========================================================================
  // VIEW 3: ROOT CAUSES
  // =========================================================================
  function renderRootCauses() {
    const f = state.filters.rootCauses;
    const query = f.search.trim().toLowerCase();

    let items = state.data.root_causes.filter(item => {
      if (f.status && item.Status !== f.status) return false;
      if (f.area && item.Area !== f.area) return false;
      if (f.severity && item.Severity !== f.severity) return false;

      if (query) {
        const text = [item.Root_Cause_ID, item.Root_Cause_Title, item.Area, item.Description, item.Remediation_Strategy].filter(Boolean).join(" ").toLowerCase();
        if (!text.includes(query)) return false;
      }
      return true;
    });

    const grid = document.getElementById("grid-root-causes");
    if (!grid) return;

    if (items.length === 0) {
      grid.innerHTML = `<div style="grid-column:1/-1; text-align:center; padding:40px; color:var(--text-muted);">No root causes match criteria.</div>`;
      return;
    }

    grid.innerHTML = items.map(rc => {
      const activeCount = Number(rc.Active_Finding_Count) || 0;
      const totalCount = Number(rc.Finding_Count) || 0;
      const isResolved = rc.Status === "RESOLVED";

      return `
        <div class="entity-card" onclick="app.openDrawer('rc', '${escapeHtml(rc.Root_Cause_ID)}')">
          <div>
            <div class="entity-card-header">
              <span class="mono-cell" style="font-weight:700; color:var(--accent-primary); font-size:14px;">${escapeHtml(rc.Root_Cause_ID)}</span>
              <div>
                ${getSeverityBadge(rc.Severity)}
                <span class="badge ${isResolved ? 'badge-fixed-later' : 'badge-still-present'}" style="margin-left:4px;">${escapeHtml(rc.Status)}</span>
              </div>
            </div>
            <div class="entity-card-title">${escapeHtml(rc.Root_Cause_Title)}</div>
            <div style="margin-bottom:8px;">
              <span class="tag-area">${escapeHtml(rc.Area)}</span>
              <span style="font-size:11px; color:var(--text-muted); margin-left:8px;">${escapeHtml(rc.SSF_Relation || "")}</span>
            </div>
            <div class="entity-card-body">
              ${escapeHtml(rc.Description || "")}
            </div>
          </div>
          <div>
            <div style="background:var(--bg-subtle); padding:8px 10px; border-radius:4px; font-size:11.5px; margin-bottom:10px;">
              <strong>Remediation:</strong> ${escapeHtml(rc.Remediation_Strategy || "Direct patch")}
            </div>
            <div class="entity-card-footer">
              <span style="color:var(--text-secondary);">
                Active Debt: <strong style="color:${activeCount > 0 ? 'var(--status-still-present-text)' : 'var(--status-fixed-later-text)'};">${activeCount}</strong> / ${totalCount}
              </span>
              <button class="btn-drawer-action" style="font-size:11px;" onclick="event.stopPropagation(); app.navigateTo('findings', { rc: '${escapeHtml(rc.Root_Cause_ID)}' });">
                View Findings (${totalCount}) →
              </button>
            </div>
          </div>
        </div>
      `;
    }).join("");
  }

  // =========================================================================
  // VIEW 4: REMEDIATION QUEUE & DEPENDENCY DAG
  // =========================================================================
  function renderRemediation() {
    renderRemediationDAG();

    const f = state.filters.remediation;
    const query = f.search.trim().toLowerCase();

    let items = state.data.remediation_queue.filter(item => {
      if (f.area && item.Area !== f.area) return false;
      if (f.priority && item.Priority !== f.priority) return false;
      if (f.tier && item.Risk_Tier !== f.tier) return false;

      if (query) {
        const text = [item.Queue_ID, item.Suggested_PR_Title, item.Suggested_PR_Scope, item.Area, item.Root_Cause_ID].filter(Boolean).join(" ").toLowerCase();
        if (!text.includes(query)) return false;
      }
      return true;
    });

    const grid = document.getElementById("grid-remediation");
    if (!grid) return;

    if (items.length === 0) {
      grid.innerHTML = `<div style="grid-column:1/-1; text-align:center; padding:40px; color:var(--text-muted);">No remediation packages match criteria.</div>`;
      return;
    }

    grid.innerHTML = items.map(rem => {
      const deps = rem.Dependencies && rem.Dependencies !== "None" ? rem.Dependencies : null;
      return `
        <div class="entity-card" onclick="app.openDrawer('rem', '${escapeHtml(rem.Queue_ID)}')">
          <div>
            <div class="entity-card-header">
              <span class="mono-cell" style="font-weight:700; color:var(--accent-primary); font-size:14px;">${escapeHtml(rem.Queue_ID)}</span>
              <div>
                <span class="badge badge-sev-medium">${escapeHtml(rem.Priority || "P2")}</span>
                <span class="badge badge-sec-boundary" style="margin-left:4px;">${escapeHtml(rem.Risk_Tier || "R2")}</span>
              </div>
            </div>
            <div class="entity-card-title">${escapeHtml(rem.Suggested_PR_Title)}</div>
            <div style="margin-bottom:8px;">
              <span class="tag-area">${escapeHtml(rem.Area)}</span>
              <span class="mono-cell" style="font-size:11px; color:var(--text-muted); margin-left:8px;">Cause: ${escapeHtml(rem.Root_Cause_ID)}</span>
            </div>
            <div class="entity-card-body" style="font-size:12px;">
              ${escapeHtml(rem.Suggested_PR_Scope || "")}
            </div>
          </div>
          <div>
            ${deps ? `<div style="background:rgba(245, 158, 11, 0.1); border:1px solid rgba(245, 158, 11, 0.3); padding:6px 8px; border-radius:4px; font-size:11px; color:var(--status-partially-fixed-text); margin-bottom:8px;">
              ⚠️ <strong>Blocked by:</strong> ${escapeHtml(deps)}
            </div>` : `<div style="background:rgba(16, 185, 129, 0.1); border:1px solid rgba(16, 185, 129, 0.3); padding:6px 8px; border-radius:4px; font-size:11px; color:var(--status-fixed-later-text); margin-bottom:8px;">
              ✓ <strong>Unblocked (Ready to Start)</strong>
            </div>`}
            <div class="entity-card-footer">
              <span style="color:var(--text-secondary);">
                Findings: <strong style="color:var(--accent-primary);">${rem.Finding_Count}</strong>
              </span>
              <button class="btn-drawer-action" style="font-size:11px;" onclick="event.stopPropagation(); app.navigateTo('findings', { rem: '${escapeHtml(rem.Queue_ID)}' });">
                Filter Findings (${rem.Finding_Count}) →
              </button>
            </div>
          </div>
        </div>
      `;
    }).join("");
  }

  function renderRemediationDAG() {
    const el = document.getElementById("remediation-dag-container");
    if (!el) return;

    // Build DAG nodes and layers
    const pkgs = state.data.remediation_queue;
    // Map dependencies: key -> array of prerequisite keys
    const depsMap = {};
    pkgs.forEach(p => {
      if (p.Dependencies && p.Dependencies !== "None") {
        depsMap[p.Queue_ID] = p.Dependencies.split(",").map(d => d.trim());
      } else {
        depsMap[p.Queue_ID] = [];
      }
    });

    // Topological layering
    const layers = [[], [], []]; // Layer 0: no deps, Layer 1: deps in layer 0, Layer 2: deps in layer 1
    pkgs.forEach(p => {
      const d = depsMap[p.Queue_ID];
      if (d.length === 0) {
        layers[0].push(p.Queue_ID);
      } else if (d.some(dep => ["REM-005", "REM-011", "REM-016"].includes(dep))) {
        layers[2].push(p.Queue_ID);
      } else {
        layers[1].push(p.Queue_ID);
      }
    });

    const svgWidth = 980;
    const svgHeight = 360;
    const colX = [120, 480, 840];
    const nodeWidth = 96;
    const nodeHeight = 30;

    const nodeCoords = {};
    layers.forEach((layerPkgs, layerIdx) => {
      const x = colX[layerIdx];
      const spacing = svgHeight / (layerPkgs.length + 1);
      layerPkgs.forEach((qid, rowIdx) => {
        const y = spacing * (rowIdx + 1);
        nodeCoords[qid] = { x, y };
      });
    });

    // Edges
    let edgesSvg = "";
    Object.entries(depsMap).forEach(([targetId, prerequisites]) => {
      const target = nodeCoords[targetId];
      if (!target) return;
      prerequisites.forEach(srcId => {
        const src = nodeCoords[srcId];
        if (!src) return;
        const x1 = src.x + nodeWidth / 2;
        const y1 = src.y;
        const x2 = target.x - nodeWidth / 2;
        const y2 = target.y;
        const dx = (x2 - x1) / 2;
        edgesSvg += `<path d="M ${x1} ${y1} C ${x1 + dx} ${y1}, ${x2 - dx} ${y2}, ${x2} ${y2}"
          fill="none" stroke="#64748b" stroke-width="1.8" marker-end="url(#arrow)" opacity="0.65" />`;
      });
    });

    // Nodes SVG
    let nodesSvg = "";
    pkgs.forEach(p => {
      const coord = nodeCoords[p.Queue_ID];
      if (!coord) return;
      const isUnblocked = (!p.Dependencies || p.Dependencies === "None");
      const fillColor = isUnblocked ? "#1e293b" : "#2a241e";
      const strokeColor = isUnblocked ? "#38bdf8" : "#f59e0b";

      nodesSvg += `
        <g class="dag-node" transform="translate(${coord.x - nodeWidth / 2}, ${coord.y - nodeHeight / 2})"
          onclick="app.openDrawer('rem', '${p.Queue_ID}')">
          <rect width="${nodeWidth}" height="${nodeHeight}" rx="5"
            fill="${fillColor}" stroke="${strokeColor}" stroke-width="1.5" />
          <text x="${nodeWidth / 2}" y="${nodeHeight / 2 - 2}" text-anchor="middle" dominant-baseline="middle"
            fill="#f8fafc" font-size="11" font-weight="700" font-family="var(--font-mono)">
            ${p.Queue_ID}
          </text>
          <text x="${nodeWidth / 2}" y="${nodeHeight / 2 + 9}" text-anchor="middle" dominant-baseline="middle"
            fill="var(--text-muted)" font-size="9" font-family="var(--font-mono)">
            ${p.Finding_Count} items
          </text>
        </g>
      `;
    });

    el.innerHTML = `
      <div style="display:flex; justify-content:space-between; margin-bottom:8px; font-size:11px; font-weight:600; color:var(--text-secondary); padding:0 60px;">
        <span>STAGE 1: UNBLOCKED / INDEPENDENT (${layers[0].length})</span>
        <span>STAGE 2: DEPENDENT PRs (${layers[1].length})</span>
        <span>STAGE 3: DOWNSTREAM CLOSURE (${layers[2].length})</span>
      </div>
      <svg class="dag-svg" viewBox="0 0 ${svgWidth} ${svgHeight}">
        <defs>
          <marker id="arrow" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
            <path d="M 0 1 L 10 5 L 0 9 z" fill="#64748b"/>
          </marker>
        </defs>
        ${edgesSvg}
        ${nodesSvg}
      </svg>
    `;
  }

  // =========================================================================
  // VIEW 5: PR COVERAGE EXPLORER
  // =========================================================================
  function renderPRCoverage() {
    const f = state.filters.prs;
    const query = f.search.trim().toLowerCase();

    let items = state.data.pr_coverage.filter(pr => {
      if (f.merged && pr.Merged !== f.merged) return false;
      if (f.threads) {
        if (f.threads === "YES" && (!pr.Review_Thread_Count || pr.Review_Thread_Count === 0)) return false;
        if (f.threads === "NO" && pr.Review_Thread_Count > 0) return false;
      }
      if (f.active) {
        if (f.active === "YES" && (!pr.Active_Finding_Count || pr.Active_Finding_Count === 0)) return false;
      }

      if (query) {
        const text = [String(pr.PR_Number), pr.PR_Title, pr.Coverage_Status].filter(Boolean).join(" ").toLowerCase();
        if (!text.includes(query)) return false;
      }
      return true;
    });

    // Sorting
    const s = state.sorting.prs;
    items.sort((a, b) => {
      let va = a[s.col] || 0;
      let vb = b[s.col] || 0;
      if (["PR_Number", "Review_Thread_Count", "Candidate_Finding_Count", "Active_Finding_Count"].includes(s.col)) {
        va = Number(va) || 0;
        vb = Number(vb) || 0;
      }
      if (va < vb) return s.dir === "asc" ? -1 : 1;
      if (va > vb) return s.dir === "asc" ? 1 : -1;
      return 0;
    });

    // Pagination
    const p = state.pagination.prs;
    const totalItems = items.length;
    const pageSize = p.pageSize === "All" ? totalItems : Number(p.pageSize);
    const totalPages = Math.ceil(totalItems / pageSize) || 1;
    if (p.page > totalPages) p.page = 1;
    const startIdx = (p.page - 1) * pageSize;
    const pageItems = items.slice(startIdx, startIdx + pageSize);

    const tbody = document.getElementById("tbody-prs");
    if (!tbody) return;

    if (pageItems.length === 0) {
      tbody.innerHTML = `<tr><td colspan="8" style="text-align:center; padding:32px; color:var(--text-muted);">No pull requests match criteria.</td></tr>`;
    } else {
      tbody.innerHTML = pageItems.map(pr => `
        <tr onclick="app.openDrawer('pr', '${pr.PR_Number}')">
          <td><a class="provenance-link" href="https://github.com/skulmakov-oss/Semantic/pull/${pr.PR_Number}" target="_blank" onclick="event.stopPropagation();">#${pr.PR_Number}</a></td>
          <td style="max-width:380px; overflow:hidden; text-overflow:ellipsis; white-space:nowrap;" title="${escapeHtml(pr.PR_Title || "")}">${escapeHtml(pr.PR_Title || "")}</td>
          <td><span class="badge ${pr.Merged === 'YES' ? 'badge-fixed-later' : 'badge-obsolete'}">${pr.Merged === 'YES' ? 'MERGED' : 'CLOSED'}</span></td>
          <td class="mono-cell" style="font-size:11.5px; color:var(--text-muted);">${escapeHtml(pr.Merge_Date || "-")}</td>
          <td class="mono-cell" style="text-align:center;">${pr.Review_Thread_Count || 0}</td>
          <td class="mono-cell" style="text-align:center;">${pr.Candidate_Finding_Count || 0}</td>
          <td class="mono-cell" style="text-align:center; color:${pr.Active_Finding_Count > 0 ? 'var(--status-still-present-text)' : 'var(--text-muted)'}; font-weight:600;">
            ${pr.Active_Finding_Count || 0}
          </td>
          <td><span class="badge badge-sec-none">${escapeHtml(pr.Coverage_Status || "VERIFIED")}</span></td>
        </tr>
      `).join("");
    }

    renderPagination("pagination-prs", p, totalItems, totalPages, newPage => {
      p.page = newPage;
      renderPRCoverage();
    }, newSize => {
      p.pageSize = newSize;
      p.page = 1;
      renderPRCoverage();
    });
  }

  // =========================================================================
  // VIEW 6: HISTORICAL REVIEW THREADS
  // =========================================================================
  function renderReviewThreads() {
    const f = state.filters.threads;
    const query = f.search.trim().toLowerCase();

    let items = state.data.review_threads.filter(t => {
      if (f.resolved && t.Thread_Resolved !== f.resolved) return false;
      if (f.status && t.Current_Status !== f.status) return false;
      if (f.author && t.Review_Author !== f.author) return false;

      if (query) {
        const text = [String(t.PR_Number), t.Thread_ID, t.Review_Author, t.Comment_Summary, t.Comment_Body, t.File, t.Mapped_Finding_ID].filter(Boolean).join(" ").toLowerCase();
        if (!text.includes(query)) return false;
      }
      return true;
    });

    const s = state.sorting.threads;
    items.sort((a, b) => {
      let va = a[s.col] || "";
      let vb = b[s.col] || "";
      if (s.col === "PR_Number") {
        va = Number(va) || 0;
        vb = Number(vb) || 0;
      }
      if (va < vb) return s.dir === "asc" ? -1 : 1;
      if (va > vb) return s.dir === "asc" ? 1 : -1;
      return 0;
    });

    const p = state.pagination.threads;
    const totalItems = items.length;
    const pageSize = p.pageSize === "All" ? totalItems : Number(p.pageSize);
    const totalPages = Math.ceil(totalItems / pageSize) || 1;
    if (p.page > totalPages) p.page = 1;
    const startIdx = (p.page - 1) * pageSize;
    const pageItems = items.slice(startIdx, startIdx + pageSize);

    const tbody = document.getElementById("tbody-threads");
    if (!tbody) return;

    if (pageItems.length === 0) {
      tbody.innerHTML = `<tr><td colspan="8" style="text-align:center; padding:32px; color:var(--text-muted);">No review threads match criteria.</td></tr>`;
    } else {
      tbody.innerHTML = pageItems.map(t => `
        <tr onclick="app.openDrawer('thread', '${escapeHtml(t.Thread_ID)}')">
          <td><a class="provenance-link" href="https://github.com/skulmakov-oss/Semantic/pull/${t.PR_Number}" target="_blank" onclick="event.stopPropagation();">#${t.PR_Number}</a></td>
          <td class="mono-cell" style="font-size:11px; max-width:120px; overflow:hidden; text-overflow:ellipsis; white-space:nowrap;" title="${escapeHtml(t.Thread_ID)}">${escapeHtml(t.Thread_ID)}</td>
          <td><span class="badge ${t.Thread_Resolved === 'NO' ? 'badge-still-present' : 'badge-fixed-later'}">${t.Thread_Resolved === 'NO' ? 'UNRESOLVED' : 'RESOLVED'}</span></td>
          <td>${getStatusBadge(t.Current_Status)}</td>
          <td style="font-size:11.5px; color:var(--text-secondary);">${escapeHtml(t.Review_Author || "")}</td>
          <td class="mono-cell" style="font-size:11px; max-width:180px; overflow:hidden; text-overflow:ellipsis; white-space:nowrap;" title="${escapeHtml(t.File || '')}:${t.Original_Line || 0}">
            ${escapeHtml(t.File || "-")}:${t.Original_Line || 0}
          </td>
          <td class="mono-cell" style="font-size:11.5px; color:var(--accent-primary);">${escapeHtml(t.Mapped_Finding_ID || "-")}</td>
          <td style="max-width:280px; overflow:hidden; text-overflow:ellipsis; white-space:nowrap;" title="${escapeHtml(t.Comment_Summary || '')}">
            ${escapeHtml(t.Comment_Summary || "")}
          </td>
        </tr>
      `).join("");
    }

    renderPagination("pagination-threads", p, totalItems, totalPages, newPage => {
      p.page = newPage;
      renderReviewThreads();
    }, newSize => {
      p.pageSize = newSize;
      p.page = 1;
      renderReviewThreads();
    });
  }

  // =========================================================================
  // VIEW 7: SSF MAPPING
  // =========================================================================
  function renderSSF() {
    const f = state.filters.ssf;
    const query = f.search.trim().toLowerCase();

    let items = state.data.ssf_mapping.filter(m => {
      if (f.relation && m.Relation_Type !== f.relation) return false;
      if (f.status && m.Status !== f.status) return false;

      if (query) {
        const text = [m.Finding_ID, String(m.Origin_PR), m.SSF_Relation, m.SSF_Issue, m.SSF_Milestone, m.Evidence].filter(Boolean).join(" ").toLowerCase();
        if (!text.includes(query)) return false;
      }
      return true;
    });

    const s = state.sorting.ssf;
    items.sort((a, b) => {
      let va = a[s.col] || "";
      let vb = b[s.col] || "";
      if (s.col === "Origin_PR") {
        va = Number(va) || 0;
        vb = Number(vb) || 0;
      }
      if (va < vb) return s.dir === "asc" ? -1 : 1;
      if (va > vb) return s.dir === "asc" ? 1 : -1;
      return 0;
    });

    const p = state.pagination.ssf;
    const totalItems = items.length;
    const pageSize = p.pageSize === "All" ? totalItems : Number(p.pageSize);
    const totalPages = Math.ceil(totalItems / pageSize) || 1;
    if (p.page > totalPages) p.page = 1;
    const startIdx = (p.page - 1) * pageSize;
    const pageItems = items.slice(startIdx, startIdx + pageSize);

    const tbody = document.getElementById("tbody-ssf");
    if (!tbody) return;

    if (pageItems.length === 0) {
      tbody.innerHTML = `<tr><td colspan="7" style="text-align:center; padding:32px; color:var(--text-muted);">No SSF mappings match criteria.</td></tr>`;
    } else {
      tbody.innerHTML = pageItems.map(m => `
        <tr onclick="app.openDrawer('finding', '${escapeHtml(m.Finding_ID)}')">
          <td class="mono-cell" style="font-weight:600; color:var(--accent-primary);">${escapeHtml(m.Finding_ID)}</td>
          <td><a class="provenance-link" href="https://github.com/skulmakov-oss/Semantic/pull/${m.Origin_PR}" target="_blank" onclick="event.stopPropagation();">#${m.Origin_PR}</a></td>
          <td><span class="tag-area">${escapeHtml(m.SSF_Relation || "-")}</span></td>
          <td><span class="badge ${m.Relation_Type === 'FIXED_BY_SSF' ? 'badge-fixed-later' : (m.Relation_Type === 'RELATED_TO_SSF' ? 'badge-partially-fixed' : 'badge-sec-none')}">${escapeHtml(m.Relation_Type)}</span></td>
          <td>${getStatusBadge(m.Status)}</td>
          <td class="mono-cell" style="font-size:11.5px;">${escapeHtml(m.SSF_Issue || "-")}</td>
          <td style="max-width:320px; overflow:hidden; text-overflow:ellipsis; white-space:nowrap;" title="${escapeHtml(m.Evidence || '')}">${escapeHtml(m.Evidence || "")}</td>
        </tr>
      `).join("");
    }

    renderPagination("pagination-ssf", p, totalItems, totalPages, newPage => {
      p.page = newPage;
      renderSSF();
    }, newSize => {
      p.pageSize = newSize;
      p.page = 1;
      renderSSF();
    });
  }

  // =========================================================================
  // VIEW 8: SECURITY & PRIVACY
  // =========================================================================
  function renderSecurity() {
    const f = state.filters.sec;
    const query = f.search.trim().toLowerCase();

    let items = state.data.security_privacy.filter(s => {
      if (f.category && s.Category !== f.category) return false;
      if (f.type && s.Pattern_Type !== f.type) return false;

      if (query) {
        const text = [s.Security_ID, String(s.PR_Number), s.Category, s.Pattern_Type, s.Location, s.Redacted_Evidence].filter(Boolean).join(" ").toLowerCase();
        if (!text.includes(query)) return false;
      }
      return true;
    });

    const s = state.sorting.sec;
    items.sort((a, b) => {
      let va = a[s.col] || "";
      let vb = b[s.col] || "";
      if (s.col === "PR_Number") {
        va = Number(va) || 0;
        vb = Number(vb) || 0;
      }
      if (va < vb) return s.dir === "asc" ? -1 : 1;
      if (va > vb) return s.dir === "asc" ? 1 : -1;
      return 0;
    });

    const p = state.pagination.sec;
    const totalItems = items.length;
    const pageSize = p.pageSize === "All" ? totalItems : Number(p.pageSize);
    const totalPages = Math.ceil(totalItems / pageSize) || 1;
    if (p.page > totalPages) p.page = 1;
    const startIdx = (p.page - 1) * pageSize;
    const pageItems = items.slice(startIdx, startIdx + pageSize);

    const tbody = document.getElementById("tbody-sec");
    if (!tbody) return;

    if (pageItems.length === 0) {
      tbody.innerHTML = `<tr><td colspan="9" style="text-align:center; padding:32px; color:var(--text-muted);">No security or privacy records match criteria.</td></tr>`;
    } else {
      tbody.innerHTML = pageItems.map(sec => `
        <tr onclick="app.openDrawer('sec', '${escapeHtml(sec.Security_ID)}')">
          <td class="mono-cell" style="font-weight:600; color:var(--accent-primary);">${escapeHtml(sec.Security_ID)}</td>
          <td><span class="badge ${sec.Category === 'CODEQL_CONFIDENTIALITY_ALERT' ? 'badge-sec-boundary' : 'badge-sec-confidentiality'}">${escapeHtml(sec.Category)}</span></td>
          <td>${getSeverityBadge(sec.Severity)}</td>
          <td><a class="provenance-link" href="https://github.com/skulmakov-oss/Semantic/pull/${sec.PR_Number}" target="_blank" onclick="event.stopPropagation();">#${sec.PR_Number}</a></td>
          <td style="font-size:11.5px; color:var(--text-secondary);">${escapeHtml(sec.Pattern_Type || "")}</td>
          <td class="mono-cell" style="font-size:11px;">${escapeHtml(sec.Location || "")}</td>
          <td class="mono-cell" style="font-size:11.5px; color:var(--status-partially-fixed-text);">${escapeHtml(sec.Redacted_Evidence || "")}</td>
          <td><span class="badge badge-sec-none">${escapeHtml(sec.Current_Status || "")}</span></td>
          <td style="max-width:260px; overflow:hidden; text-overflow:ellipsis; white-space:nowrap;" title="${escapeHtml(sec.Recommended_Action || '')}">${escapeHtml(sec.Recommended_Action || "")}</td>
        </tr>
      `).join("");
    }

    renderPagination("pagination-sec", p, totalItems, totalPages, newPage => {
      p.page = newPage;
      renderSecurity();
    }, newSize => {
      p.pageSize = newSize;
      p.page = 1;
      renderSecurity();
    });
  }

  // =========================================================================
  // VIEW 9: METADATA & GOVERNANCE
  // =========================================================================
  function renderMetadata() {
    const metaList = document.getElementById("metadata-key-value-list");
    if (metaList && state.data.metadata) {
      metaList.innerHTML = Object.entries(state.data.metadata).map(([k, v]) => `
        <div style="display:flex; justify-content:space-between; padding:6px 0; border-bottom:1px solid var(--border-subtle);">
          <span style="color:var(--text-secondary);">${escapeHtml(k)}:</span>
          <span style="color:var(--text-primary); text-align:right; max-width:60%; word-break:break-all;">${escapeHtml(String(v))}</span>
        </div>
      `).join("");
    }

    const dictList = document.getElementById("dictionaries-list");
    if (dictList && state.data.dictionaries) {
      dictList.innerHTML = Object.entries(state.data.dictionaries).map(([name, vals]) => `
        <div style="background:var(--bg-input); padding:12px; border-radius:6px; border:1px solid var(--border-default);">
          <div style="font-weight:700; color:var(--accent-primary); font-size:12px; margin-bottom:6px; font-family:var(--font-mono);">
            ${escapeHtml(name)} (${vals.length})
          </div>
          <div style="display:flex; flex-wrap:wrap; gap:4px;">
            ${vals.map(v => `<span class="badge badge-sec-none" style="font-size:10px;">${escapeHtml(v)}</span>`).join("")}
          </div>
        </div>
      `).join("");
    }
  }

  // =========================================================================
  // SLIDE-OVER DETAIL DRAWER
  // =========================================================================
  function setupDrawer() {
    const backdrop = document.getElementById("drawer-backdrop");
    const closeBtn = document.getElementById("btn-close-drawer");
    const copyBtn = document.getElementById("btn-drawer-copy-link");

    if (backdrop) backdrop.addEventListener("click", closeDrawer);
    if (closeBtn) closeBtn.addEventListener("click", closeDrawer);

    if (copyBtn) {
      copyBtn.addEventListener("click", () => {
        if (!state.activeEntity) return;
        const url = `${window.location.origin}${window.location.pathname}#${state.activeEntity.type}=${encodeURIComponent(state.activeEntity.id)}`;
        navigator.clipboard.writeText(url).then(() => {
          copyBtn.textContent = "Copied!";
          setTimeout(() => { copyBtn.textContent = "Copy Link"; }, 1800);
        });
      });
    }

    document.addEventListener("keydown", e => {
      if (e.key === "Escape") closeDrawer();
    });
  }

  function openDrawer(type, id) {
    state.activeEntity = { type, id };
    const drawer = document.getElementById("detail-drawer");
    const backdrop = document.getElementById("drawer-backdrop");
    const idEl = document.getElementById("drawer-id");
    const statusEl = document.getElementById("drawer-badge-status");
    const sevEl = document.getElementById("drawer-badge-sev");
    const ghBtn = document.getElementById("btn-drawer-gh-link");
    const bodyEl = document.getElementById("drawer-body");

    if (!drawer || !backdrop) return;

    idEl.textContent = id;

    // Render entity specific drawer body
    if (type === "finding") {
      const item = state.data.findings.find(f => f.Finding_ID === id);
      if (!item) return;

      statusEl.className = `badge ${getStatusBadgeClass(item.Current_Status)}`;
      statusEl.textContent = item.Current_Status;
      sevEl.className = `badge ${getSeverityBadgeClass(item.Current_Severity)}`;
      sevEl.textContent = item.Current_Severity;

      ghBtn.href = `https://github.com/skulmakov-oss/Semantic/pull/${item.PR_Number}`;
      ghBtn.style.display = "inline-flex";

      bodyEl.innerHTML = `
        <div class="drawer-section">
          <div class="drawer-section-title">Core Classification</div>
          <div class="drawer-grid">
            <div>
              <div class="drawer-field-label">Architectural Area</div>
              <div class="drawer-field-value"><span class="tag-area">${escapeHtml(item.Area)}</span></div>
            </div>
            <div>
              <div class="drawer-field-label">Finding Type</div>
              <div class="drawer-field-value">${escapeHtml(item.Finding_Type || "-")}</div>
            </div>
            <div>
              <div class="drawer-field-label">Security Class</div>
              <div class="drawer-field-value">${getSecurityClassBadge(item.Security_Class)}</div>
            </div>
            <div>
              <div class="drawer-field-label">Review Author & Date</div>
              <div class="drawer-field-value">${escapeHtml(item.Review_Author || "-")} (${escapeHtml(item.Review_Date || "-")})</div>
            </div>
          </div>
        </div>

        <div class="drawer-section">
          <div class="drawer-section-title">Finding Summary</div>
          <div style="font-size:13.5px; font-weight:600; color:var(--text-primary); margin-bottom:8px;">
            ${escapeHtml(item.Finding_Summary || "")}
          </div>
        </div>

        <div class="drawer-section">
          <div class="drawer-section-title">Original Historical Review Comment</div>
          <div class="code-block">${escapeHtml(item.Original_Comment_Summary || item.Validation_Evidence || "No comment body preserved.")}</div>
          <div style="margin-top:6px; font-size:11px; color:var(--text-muted);">
            Thread: <code>${escapeHtml(item.Thread_ID || "-")}</code> | Original Sev: <code>${escapeHtml(item.Original_Severity || "-")}</code> | Resolved: <code>${item.Thread_Resolved || "UNKNOWN"}</code>
          </div>
        </div>

        <div class="drawer-section">
          <div class="drawer-section-title">Lineage & File Locations</div>
          <div class="drawer-grid">
            <div>
              <div class="drawer-field-label">Original Location</div>
              <div class="drawer-field-value mono-cell" style="font-size:11.5px;">${escapeHtml(item.Original_File || "-")}:${item.Original_Line || 0}</div>
            </div>
            <div>
              <div class="drawer-field-label">Current Main Location (@ ${escapeHtml(item.Current_Main_SHA || "e41ff311")})</div>
              <div class="drawer-field-value mono-cell" style="font-size:11.5px; color:var(--accent-primary);">${escapeHtml(item.Current_Main_File || "-")}:${item.Current_Main_Line || 0}</div>
            </div>
          </div>
        </div>

        <div class="drawer-section">
          <div class="drawer-section-title">Validation Evidence</div>
          <div class="code-block">${escapeHtml(item.Validation_Evidence || "Validated via git diff and compiler source review.")}</div>
        </div>

        <div class="drawer-section">
          <div class="drawer-section-title">Remediation & Architecture</div>
          <div class="drawer-grid">
            <div>
              <div class="drawer-field-label">Root Cause</div>
              <div class="drawer-field-value">
                <button class="btn-drawer-action" onclick="app.openDrawer('rc', '${item.Root_Cause_ID}')">
                  ${escapeHtml(item.Root_Cause_ID)} →
                </button>
              </div>
            </div>
            <div>
              <div class="drawer-field-label">Remediation Package</div>
              <div class="drawer-field-value">
                ${item.Remediation_Group ? `<button class="btn-drawer-action" onclick="app.openDrawer('rem', '${item.Remediation_Group}')">${escapeHtml(item.Remediation_Group)} →</button>` : `<span style="color:var(--text-muted);">-</span>`}
              </div>
            </div>
            <div>
              <div class="drawer-field-label">SSF Relation</div>
              <div class="drawer-field-value"><span class="badge badge-sec-none">${escapeHtml(item.SSF_Relation || "-")}</span></div>
            </div>
            <div>
              <div class="drawer-field-label">SSF Issue / PR</div>
              <div class="drawer-field-value">${escapeHtml(item.SSF_PR || "-")}</div>
            </div>
          </div>
        </div>

        <div class="drawer-section">
          <div class="drawer-section-title">Recommended Action</div>
          <div style="font-size:12.5px; color:var(--text-primary); background:var(--bg-input); padding:10px; border-radius:6px; border:1px solid var(--border-default);">
            ${escapeHtml(item.Recommended_Action || "Inspect current codebase and apply cohesive patch per remediation package.")}
          </div>
        </div>
      `;
    } else if (type === "rc") {
      const rc = state.data.root_causes.find(r => r.Root_Cause_ID === id);
      if (!rc) return;

      statusEl.className = `badge ${rc.Status === 'RESOLVED' ? 'badge-fixed-later' : 'badge-still-present'}`;
      statusEl.textContent = rc.Status;
      sevEl.className = `badge ${getSeverityBadgeClass(rc.Severity)}`;
      sevEl.textContent = rc.Severity;
      ghBtn.style.display = "none";

      bodyEl.innerHTML = `
        <div class="drawer-section">
          <div class="drawer-section-title">Root Cause Title</div>
          <div style="font-size:14px; font-weight:700; color:var(--text-primary); margin-bottom:6px;">
            ${escapeHtml(rc.Root_Cause_Title)}
          </div>
          <span class="tag-area">${escapeHtml(rc.Area)}</span>
        </div>

        <div class="drawer-section">
          <div class="drawer-section-title">Description</div>
          <div style="font-size:13px; color:var(--text-secondary); line-height:1.6;">
            ${escapeHtml(rc.Description || "")}
          </div>
        </div>

        <div class="drawer-section">
          <div class="drawer-section-title">Remediation Strategy</div>
          <div class="code-block">${escapeHtml(rc.Remediation_Strategy || "")}</div>
        </div>

        <div class="drawer-section">
          <div class="drawer-section-title">Statistics</div>
          <div class="drawer-grid">
            <div>
              <div class="drawer-field-label">Total Findings</div>
              <div class="drawer-field-value mono-cell" style="font-size:16px; font-weight:700;">${rc.Finding_Count}</div>
            </div>
            <div>
              <div class="drawer-field-label">Active Debt</div>
              <div class="drawer-field-value mono-cell" style="font-size:16px; font-weight:700; color:var(--status-still-present-text);">${rc.Active_Finding_Count}</div>
            </div>
            <div>
              <div class="drawer-field-label">Affected PRs</div>
              <div class="drawer-field-value mono-cell">${rc.Affected_PRs || "-"}</div>
            </div>
            <div>
              <div class="drawer-field-label">SSF Relation</div>
              <div class="drawer-field-value">${escapeHtml(rc.SSF_Relation || "-")}</div>
            </div>
          </div>
        </div>

        <div class="drawer-section">
          <button class="btn-drawer-action" style="width:100%; padding:10px; font-weight:600;" onclick="app.closeDrawer(); app.navigateTo('findings', { rc: '${rc.Root_Cause_ID}' });">
            Explore All ${rc.Finding_Count} Findings for ${rc.Root_Cause_ID} →
          </button>
        </div>
      `;
    } else if (type === "rem") {
      const rem = state.data.remediation_queue.find(r => r.Queue_ID === id);
      if (!rem) return;

      statusEl.className = "badge badge-still-present";
      statusEl.textContent = rem.Status || "READY";
      sevEl.className = `badge ${getSeverityBadgeClass(rem.Severity)}`;
      sevEl.textContent = rem.Severity;
      ghBtn.style.display = "none";

      bodyEl.innerHTML = `
        <div class="drawer-section">
          <div class="drawer-section-title">Suggested PR Title</div>
          <div style="font-size:14px; font-weight:700; color:var(--text-primary); margin-bottom:6px;">
            ${escapeHtml(rem.Suggested_PR_Title)}
          </div>
          <div>
            <span class="tag-area">${escapeHtml(rem.Area)}</span>
            <span class="badge badge-sev-medium" style="margin-left:6px;">${escapeHtml(rem.Priority || "P2")}</span>
            <span class="badge badge-sec-boundary" style="margin-left:4px;">${escapeHtml(rem.Risk_Tier || "R2")}</span>
          </div>
        </div>

        <div class="drawer-section">
          <div class="drawer-section-title">PR Scope & Objectives</div>
          <div style="font-size:13px; color:var(--text-secondary); line-height:1.6;">
            ${escapeHtml(rem.Suggested_PR_Scope || "")}
          </div>
        </div>

        <div class="drawer-section">
          <div class="drawer-section-title">Dependencies & Execution Constraints</div>
          <div class="drawer-grid">
            <div>
              <div class="drawer-field-label">Prerequisites</div>
              <div class="drawer-field-value mono-cell">${escapeHtml(rem.Dependencies || "None (Unblocked)")}</div>
            </div>
            <div>
              <div class="drawer-field-label">SSF Milestone Dependency</div>
              <div class="drawer-field-value">${escapeHtml(rem.SSF_Dependency || "-")}</div>
            </div>
            <div>
              <div class="drawer-field-label">Regression Tests Required</div>
              <div class="drawer-field-value"><span class="badge ${rem.Regression_Test_Required === 'YES' ? 'badge-still-present' : 'badge-sec-none'}">${rem.Regression_Test_Required || 'YES'}</span></div>
            </div>
            <div>
              <div class="drawer-field-label">Documentation Update</div>
              <div class="drawer-field-value"><span class="badge badge-sec-none">${rem.Documentation_Update_Required || 'NO'}</span></div>
            </div>
          </div>
        </div>

        <div class="drawer-section">
          <div class="drawer-section-title">Associated Root Cause</div>
          <button class="btn-drawer-action" onclick="app.openDrawer('rc', '${rem.Root_Cause_ID}')">
            ${escapeHtml(rem.Root_Cause_ID)} →
          </button>
        </div>

        <div class="drawer-section">
          <button class="btn-drawer-action" style="width:100%; padding:10px; font-weight:600;" onclick="app.closeDrawer(); app.navigateTo('findings', { rem: '${rem.Queue_ID}' });">
            Explore All ${rem.Finding_Count} Findings in ${rem.Queue_ID} →
          </button>
        </div>
      `;
    } else if (type === "pr") {
      const pr = state.data.pr_coverage.find(p => String(p.PR_Number) === String(id));
      if (!pr) return;

      statusEl.className = `badge ${pr.Merged === 'YES' ? 'badge-fixed-later' : 'badge-obsolete'}`;
      statusEl.textContent = pr.Merged === "YES" ? "MERGED" : "CLOSED";
      sevEl.className = "badge badge-sec-none";
      sevEl.textContent = `PR #${pr.PR_Number}`;

      ghBtn.href = `https://github.com/skulmakov-oss/Semantic/pull/${pr.PR_Number}`;
      ghBtn.style.display = "inline-flex";

      // Findings from this PR
      const prFindings = state.data.findings.filter(f => String(f.PR_Number) === String(pr.PR_Number));
      const prThreads = state.data.review_threads.filter(t => String(t.PR_Number) === String(pr.PR_Number));
      const prSec = state.data.security_privacy.filter(s => String(s.PR_Number) === String(pr.PR_Number));

      bodyEl.innerHTML = `
        <div class="drawer-section">
          <div class="drawer-section-title">Pull Request Metadata</div>
          <div style="font-size:14px; font-weight:700; color:var(--text-primary); margin-bottom:6px;">
            ${escapeHtml(pr.PR_Title)}
          </div>
          <div class="drawer-grid">
            <div>
              <div class="drawer-field-label">Merge Date</div>
              <div class="drawer-field-value mono-cell">${escapeHtml(pr.Merge_Date || "-")}</div>
            </div>
            <div>
              <div class="drawer-field-label">Coverage Status</div>
              <div class="drawer-field-value">${escapeHtml(pr.Coverage_Status || "VERIFIED")}</div>
            </div>
          </div>
        </div>

        <div class="drawer-section">
          <div class="drawer-section-title">Associated Review Threads (${prThreads.length})</div>
          ${prThreads.length === 0 ? `<div style="color:var(--text-muted); font-size:12px;">No historical review threads recorded on this PR.</div>` : `
            <div style="display:flex; flex-direction:column; gap:6px;">
              ${prThreads.map(t => `
                <div style="background:var(--bg-input); padding:8px 10px; border-radius:4px; border:1px solid var(--border-default); cursor:pointer;"
                  onclick="app.openDrawer('thread', '${t.Thread_ID}')">
                  <div style="display:flex; justify-content:space-between; margin-bottom:4px;">
                    <span class="mono-cell" style="font-size:11px; color:var(--accent-primary);">${escapeHtml(t.Thread_ID)}</span>
                    <span class="badge ${t.Thread_Resolved === 'NO' ? 'badge-still-present' : 'badge-fixed-later'}">${t.Thread_Resolved === 'NO' ? 'UNRESOLVED' : 'RESOLVED'}</span>
                  </div>
                  <div style="font-size:11.5px; color:var(--text-secondary);">${escapeHtml(t.Comment_Summary || '')}</div>
                </div>
              `).join("")}
            </div>
          `}
        </div>

        <div class="drawer-section">
          <div class="drawer-section-title">Originating Findings (${prFindings.length})</div>
          ${prFindings.length === 0 ? `<div style="color:var(--text-muted); font-size:12px;">No candidate debt findings originated from this PR.</div>` : `
            <div style="display:flex; flex-direction:column; gap:6px;">
              ${prFindings.map(f => `
                <div style="background:var(--bg-input); padding:8px 10px; border-radius:4px; border:1px solid var(--border-default); cursor:pointer;"
                  onclick="app.openDrawer('finding', '${f.Finding_ID}')">
                  <div style="display:flex; justify-content:space-between; margin-bottom:4px;">
                    <span class="mono-cell" style="font-weight:600; color:var(--accent-primary);">${escapeHtml(f.Finding_ID)}</span>
                    <div>
                      ${getSeverityBadge(f.Current_Severity)}
                      ${getStatusBadge(f.Current_Status)}
                    </div>
                  </div>
                  <div style="font-size:11.5px; color:var(--text-secondary);">${escapeHtml(f.Finding_Summary || '')}</div>
                </div>
              `).join("")}
            </div>
          `}
        </div>

        ${prSec.length > 0 ? `
          <div class="drawer-section">
            <div class="drawer-section-title">Privacy / Environment Artifacts (${prSec.length})</div>
            ${prSec.map(s => `
              <div style="background:var(--bg-input); padding:8px 10px; border-radius:4px; border:1px solid var(--border-default); font-size:11.5px;">
                <div style="color:var(--status-partially-fixed-text); font-family:var(--font-mono);">${escapeHtml(s.Redacted_Evidence)}</div>
                <div style="color:var(--text-muted); font-size:10.5px; margin-top:2px;">${escapeHtml(s.Location)}</div>
              </div>
            `).join("")}
          </div>
        ` : ''}
      `;
    } else if (type === "thread") {
      const t = state.data.review_threads.find(th => th.Thread_ID === id);
      if (!t) return;

      statusEl.className = `badge ${t.Thread_Resolved === 'NO' ? 'badge-still-present' : 'badge-fixed-later'}`;
      statusEl.textContent = t.Thread_Resolved === "NO" ? "UNRESOLVED" : "RESOLVED";
      sevEl.className = `badge ${getStatusBadgeClass(t.Current_Status)}`;
      sevEl.textContent = t.Current_Status;

      ghBtn.href = `https://github.com/skulmakov-oss/Semantic/pull/${t.PR_Number}`;
      ghBtn.style.display = "inline-flex";

      bodyEl.innerHTML = `
        <div class="drawer-section">
          <div class="drawer-section-title">Thread Metadata</div>
          <div class="drawer-grid">
            <div>
              <div class="drawer-field-label">PR Origin</div>
              <div class="drawer-field-value"><a class="provenance-link" href="https://github.com/skulmakov-oss/Semantic/pull/${t.PR_Number}" target="_blank">#${t.PR_Number}</a></div>
            </div>
            <div>
              <div class="drawer-field-label">Review Author</div>
              <div class="drawer-field-value">${escapeHtml(t.Review_Author || "-")}</div>
            </div>
            <div>
              <div class="drawer-field-label">File & Line</div>
              <div class="drawer-field-value mono-cell" style="font-size:11.5px;">${escapeHtml(t.File || "-")}:${t.Original_Line || 0}</div>
            </div>
            <div>
              <div class="drawer-field-label">Mapped Finding</div>
              <div class="drawer-field-value">
                ${t.Mapped_Finding_ID ? `<button class="btn-drawer-action" onclick="app.openDrawer('finding', '${t.Mapped_Finding_ID}')">${escapeHtml(t.Mapped_Finding_ID)} →</button>` : `<span style="color:var(--text-muted);">-</span>`}
              </div>
            </div>
          </div>
        </div>

        <div class="drawer-section">
          <div class="drawer-section-title">Comment Body</div>
          <div class="code-block">${escapeHtml(t.Comment_Body || t.Comment_Summary || "No body preserved.")}</div>
        </div>
      `;
    } else if (type === "sec") {
      const s = state.data.security_privacy.find(sec => sec.Security_ID === id);
      if (!s) return;

      statusEl.className = `badge ${s.Category === 'CODEQL_CONFIDENTIALITY_ALERT' ? 'badge-sec-boundary' : 'badge-sec-confidentiality'}`;
      statusEl.textContent = s.Category;
      sevEl.className = `badge ${getSeverityBadgeClass(s.Severity)}`;
      sevEl.textContent = s.Severity;

      ghBtn.href = `https://github.com/skulmakov-oss/Semantic/pull/${s.PR_Number}`;
      ghBtn.style.display = "inline-flex";

      bodyEl.innerHTML = `
        <div class="drawer-section">
          <div class="drawer-section-title">Artifact Classification</div>
          <div class="drawer-grid">
            <div>
              <div class="drawer-field-label">PR Number</div>
              <div class="drawer-field-value"><a class="provenance-link" href="https://github.com/skulmakov-oss/Semantic/pull/${s.PR_Number}" target="_blank">#${s.PR_Number}</a></div>
            </div>
            <div>
              <div class="drawer-field-label">Pattern Type</div>
              <div class="drawer-field-value">${escapeHtml(s.Pattern_Type || "")}</div>
            </div>
            <div>
              <div class="drawer-field-label">Location</div>
              <div class="drawer-field-value mono-cell" style="font-size:11.5px;">${escapeHtml(s.Location || "")}</div>
            </div>
            <div>
              <div class="drawer-field-label">Status</div>
              <div class="drawer-field-value"><span class="badge badge-sec-none">${escapeHtml(s.Current_Status || "")}</span></div>
            </div>
          </div>
        </div>

        <div class="drawer-section">
          <div class="drawer-section-title">Redacted Evidence (Masked Path)</div>
          <div class="code-block" style="color:var(--status-partially-fixed-text);">${escapeHtml(s.Redacted_Evidence || "")}</div>
        </div>

        <div class="drawer-section">
          <div class="drawer-section-title">Recommended Action</div>
          <div style="font-size:12.5px; color:var(--text-primary); background:var(--bg-input); padding:10px; border-radius:6px; border:1px solid var(--border-default);">
            ${escapeHtml(s.Recommended_Action || "")}
          </div>
        </div>
      `;
    }

    drawer.classList.add("open");
    backdrop.classList.add("open");
  }

  function closeDrawer() {
    state.activeEntity = null;
    const drawer = document.getElementById("detail-drawer");
    const backdrop = document.getElementById("drawer-backdrop");
    if (drawer) drawer.classList.remove("open");
    if (backdrop) backdrop.classList.remove("open");
  }

  // =========================================================================
  // Filtering & Search Setup
  // =========================================================================
  function setupFiltersAndSearch() {
    // Findings search and filters
    const searchFindings = document.getElementById("search-findings");
    if (searchFindings) {
      searchFindings.addEventListener("input", e => {
        state.filters.findings.search = e.target.value;
        state.pagination.findings.page = 1;
        renderFindings();
      });
    }

    ["status", "severity", "area", "type", "sec", "ssf", "rc", "rem"].forEach(key => {
      const el = document.getElementById(`filter-findings-${key}`);
      if (el) {
        el.addEventListener("change", e => {
          state.filters.findings[key] = e.target.value;
          state.pagination.findings.page = 1;
          renderFindings();
        });
      }
    });

    // Quick toggle buttons
    const btnQuickActive = document.getElementById("btn-quick-active");
    if (btnQuickActive) {
      btnQuickActive.addEventListener("click", () => {
        state.filters.findings.quickActive = !state.filters.findings.quickActive;
        btnQuickActive.classList.toggle("active", state.filters.findings.quickActive);
        renderFindings();
      });
    }

    const btnQuickHigh = document.getElementById("btn-quick-high");
    if (btnQuickHigh) {
      btnQuickHigh.addEventListener("click", () => {
        state.filters.findings.quickHigh = !state.filters.findings.quickHigh;
        btnQuickHigh.classList.toggle("active", state.filters.findings.quickHigh);
        renderFindings();
      });
    }

    const btnQuickSec = document.getElementById("btn-quick-security");
    if (btnQuickSec) {
      btnQuickSec.addEventListener("click", () => {
        state.filters.findings.quickSec = !state.filters.findings.quickSec;
        btnQuickSec.classList.toggle("active", state.filters.findings.quickSec);
        renderFindings();
      });
    }

    // Root Causes
    const searchRC = document.getElementById("search-root-causes");
    if (searchRC) {
      searchRC.addEventListener("input", e => {
        state.filters.rootCauses.search = e.target.value;
        renderRootCauses();
      });
    }
    ["status", "area", "severity"].forEach(key => {
      const el = document.getElementById(`filter-rc-${key}`);
      if (el) {
        el.addEventListener("change", e => {
          state.filters.rootCauses[key] = e.target.value;
          renderRootCauses();
        });
      }
    });

    // Remediation
    const searchRem = document.getElementById("search-remediation");
    if (searchRem) {
      searchRem.addEventListener("input", e => {
        state.filters.remediation.search = e.target.value;
        renderRemediation();
      });
    }
    ["area", "priority", "tier"].forEach(key => {
      const el = document.getElementById(`filter-rem-${key}`);
      if (el) {
        el.addEventListener("change", e => {
          state.filters.remediation[key] = e.target.value;
          renderRemediation();
        });
      }
    });

    // PR Coverage
    const searchPRs = document.getElementById("search-prs");
    if (searchPRs) {
      searchPRs.addEventListener("input", e => {
        state.filters.prs.search = e.target.value;
        state.pagination.prs.page = 1;
        renderPRCoverage();
      });
    }
    ["merged", "threads", "active"].forEach(key => {
      const el = document.getElementById(`filter-prs-${key}`);
      if (el) {
        el.addEventListener("change", e => {
          state.filters.prs[key] = e.target.value;
          state.pagination.prs.page = 1;
          renderPRCoverage();
        });
      }
    });

    // Review Threads
    const searchThreads = document.getElementById("search-threads");
    if (searchThreads) {
      searchThreads.addEventListener("input", e => {
        state.filters.threads.search = e.target.value;
        state.pagination.threads.page = 1;
        renderReviewThreads();
      });
    }
    ["resolved", "status", "author"].forEach(key => {
      const el = document.getElementById(`filter-threads-${key}`);
      if (el) {
        el.addEventListener("change", e => {
          state.filters.threads[key] = e.target.value;
          state.pagination.threads.page = 1;
          renderReviewThreads();
        });
      }
    });

    // SSF
    const searchSSF = document.getElementById("search-ssf");
    if (searchSSF) {
      searchSSF.addEventListener("input", e => {
        state.filters.ssf.search = e.target.value;
        state.pagination.ssf.page = 1;
        renderSSF();
      });
    }
    ["relation", "status"].forEach(key => {
      const el = document.getElementById(`filter-ssf-${key}`);
      if (el) {
        el.addEventListener("change", e => {
          state.filters.ssf[key] = e.target.value;
          state.pagination.ssf.page = 1;
          renderSSF();
        });
      }
    });

    // Security
    const searchSec = document.getElementById("search-sec");
    if (searchSec) {
      searchSec.addEventListener("input", e => {
        state.filters.sec.search = e.target.value;
        state.pagination.sec.page = 1;
        renderSecurity();
      });
    }
    ["category", "type"].forEach(key => {
      const el = document.getElementById(`filter-sec-${key}`);
      if (el) {
        el.addEventListener("change", e => {
          state.filters.sec[key] = e.target.value;
          state.pagination.sec.page = 1;
          renderSecurity();
        });
      }
    });
  }

  function populateDropdowns() {
    const dict = state.data.dictionaries;

    // Findings Status
    fillSelect("filter-findings-status", dict.Current_Status);
    fillSelect("filter-findings-severity", dict.Severity);
    fillSelect("filter-findings-area", dict.Area);
    fillSelect("filter-findings-type", dict.Finding_Type);
    fillSelect("filter-findings-sec", dict.Security_Class);
    fillSelect("filter-findings-ssf", dict.SSF_Relation_Type);

    // Root Cause dropdowns in Findings
    const rcs = state.data.root_causes.map(r => r.Root_Cause_ID).sort();
    fillSelect("filter-findings-rc", rcs);

    // REM dropdowns in Findings
    const rems = state.data.remediation_queue.map(r => r.Queue_ID).sort();
    fillSelect("filter-findings-rem", rems);

    // Root Causes view
    fillSelect("filter-rc-area", dict.Area);
    fillSelect("filter-rc-severity", dict.Severity);

    // Remediation view
    fillSelect("filter-rem-area", dict.Area);
    fillSelect("filter-rem-priority", dict.Priority);

    // Threads View
    fillSelect("filter-threads-status", dict.Current_Status);
    const authors = Array.from(new Set(state.data.review_threads.map(t => t.Review_Author).filter(Boolean))).sort();
    fillSelect("filter-threads-author", authors);

    // SSF View
    fillSelect("filter-ssf-relation", dict.SSF_Relation_Type);
    fillSelect("filter-ssf-status", dict.Current_Status);

    // Security View
    const secTypes = Array.from(new Set(state.data.security_privacy.map(s => s.Pattern_Type).filter(Boolean))).sort();
    fillSelect("filter-sec-type", secTypes);
  }

  function fillSelect(selectId, items) {
    const el = document.getElementById(selectId);
    if (!el || !items) return;
    items.forEach(val => {
      const opt = document.createElement("option");
      opt.value = val;
      opt.textContent = val;
      el.appendChild(opt);
    });
  }

  function updateFilterControls(view) {
    if (view === "findings") {
      const f = state.filters.findings;
      ["status", "severity", "area", "type", "sec", "ssf", "rc", "rem"].forEach(k => {
        const el = document.getElementById(`filter-findings-${k}`);
        if (el) el.value = f[k];
      });
      document.getElementById("btn-quick-active").classList.toggle("active", f.quickActive);
      document.getElementById("btn-quick-high").classList.toggle("active", f.quickHigh);
      document.getElementById("btn-quick-security").classList.toggle("active", f.quickSec);
    }
  }

  function renderFilterChips(containerId, filtersObj, fieldMap, onClearAll) {
    const el = document.getElementById(containerId);
    if (!el) return;

    const activeList = [];
    fieldMap.forEach(f => {
      const val = filtersObj[f.key];
      if (val) {
        activeList.push({ key: f.key, label: f.label, val });
      }
    });

    if (activeList.length === 0) {
      el.innerHTML = "";
      return;
    }

    el.innerHTML = activeList.map(item => `
      <div class="filter-chip">
        <span>${escapeHtml(item.label)}: <strong>${escapeHtml(item.val)}</strong></span>
        <span class="chip-remove" onclick="app.removeFilter('${containerId}', '${item.key}')">&times;</span>
      </div>
    `).join("") + `<button class="btn-clear-filters" onclick="app.clearAllFilters('${containerId}')">Clear all filters</button>`;

    app.removeFilter = (cid, key) => {
      if (cid === containerId) {
        filtersObj[key] = "";
        updateFilterControls("findings");
        renderFindings();
      }
    };

    app.clearAllFilters = cid => {
      if (cid === containerId && onClearAll) onClearAll();
    };
  }

  // =========================================================================
  // Sorting & Pagination Setup
  // =========================================================================
  function setupSorting() {
    ["findings", "prs", "threads", "ssf", "sec"].forEach(view => {
      const table = document.getElementById(`table-${view}`);
      if (!table) return;

      const ths = table.querySelectorAll("thead th[data-sort]");
      ths.forEach(th => {
        th.addEventListener("click", () => {
          const col = th.getAttribute("data-sort");
          const s = state.sorting[view];
          if (s.col === col) {
            s.dir = s.dir === "asc" ? "desc" : "asc";
          } else {
            s.col = col;
            s.dir = "asc";
          }

          // Update header classes
          ths.forEach(h => {
            h.classList.toggle("sorted", h === th);
            const icon = h.querySelector(".sort-icon");
            if (icon) {
              icon.textContent = h === th ? (s.dir === "asc" ? "▲" : "▼") : "↕";
            }
          });

          renderCurrentView();
        });
      });
    });
  }

  function renderPagination(containerId, pState, totalItems, totalPages, onPageChange, onSizeChange) {
    const el = document.getElementById(containerId);
    if (!el) return;

    const startItem = totalItems === 0 ? 0 : (pState.page - 1) * (pState.pageSize === "All" ? totalItems : Number(pState.pageSize)) + 1;
    const endItem = pState.pageSize === "All" ? totalItems : Math.min(pState.page * Number(pState.pageSize), totalItems);

    el.innerHTML = `
      <div>
        Showing <strong>${startItem} - ${endItem}</strong> of <strong>${totalItems}</strong> records
      </div>
      <div class="pagination-controls">
        <label style="font-size:11.5px; color:var(--text-muted); margin-right:4px;">Per Page:</label>
        <select class="page-size-select" id="${containerId}-pagesize">
          <option value="25" ${pState.pageSize == 25 ? 'selected' : ''}>25</option>
          <option value="50" ${pState.pageSize == 50 ? 'selected' : ''}>50</option>
          <option value="100" ${pState.pageSize == 100 ? 'selected' : ''}>100</option>
          <option value="250" ${pState.pageSize == 250 ? 'selected' : ''}>250</option>
          <option value="All" ${pState.pageSize === 'All' ? 'selected' : ''}>All</option>
        </select>
        <button class="btn-page" id="${containerId}-first" ${pState.page <= 1 ? 'disabled' : ''}>«</button>
        <button class="btn-page" id="${containerId}-prev" ${pState.page <= 1 ? 'disabled' : ''}>‹ Prev</button>
        <span style="font-weight:600; padding:0 6px;">Page ${pState.page} of ${totalPages}</span>
        <button class="btn-page" id="${containerId}-next" ${pState.page >= totalPages ? 'disabled' : ''}>Next ›</button>
        <button class="btn-page" id="${containerId}-last" ${pState.page >= totalPages ? 'disabled' : ''}>»</button>
      </div>
    `;

    document.getElementById(`${containerId}-first`).onclick = () => onPageChange(1);
    document.getElementById(`${containerId}-prev`).onclick = () => onPageChange(pState.page - 1);
    document.getElementById(`${containerId}-next`).onclick = () => onPageChange(pState.page + 1);
    document.getElementById(`${containerId}-last`).onclick = () => onPageChange(totalPages);
    document.getElementById(`${containerId}-pagesize`).onchange = e => onSizeChange(e.target.value);
  }

  // =========================================================================
  // CSV Export Engine (Client-Side, Zero Dependencies)
  // =========================================================================
  function setupExportButtons() {
    const btnExpFnd = document.getElementById("btn-export-findings");
    if (btnExpFnd) {
      btnExpFnd.addEventListener("click", () => {
        exportTableToCSV(
          state.data.findings,
          ["Finding_ID", "Current_Severity", "Current_Status", "Area", "Finding_Type", "Root_Cause_ID", "Remediation_Group", "PR_Number", "Security_Class", "Finding_Summary", "Current_Main_File", "Current_Main_Line", "Recommended_Action"],
          "semantic_findings_filtered.csv"
        );
      });
    }

    const btnExpPrs = document.getElementById("btn-export-prs");
    if (btnExpPrs) {
      btnExpPrs.addEventListener("click", () => {
        exportTableToCSV(
          state.data.pr_coverage,
          ["PR_Number", "PR_Title", "PR_State", "Merged", "Merge_Date", "Review_Thread_Count", "Candidate_Finding_Count", "Active_Finding_Count", "Coverage_Status"],
          "semantic_pr_coverage.csv"
        );
      });
    }

    const btnExpTh = document.getElementById("btn-export-threads");
    if (btnExpTh) {
      btnExpTh.addEventListener("click", () => {
        exportTableToCSV(
          state.data.review_threads,
          ["PR_Number", "Thread_ID", "Thread_Resolved", "Current_Status", "Review_Author", "File", "Original_Line", "Mapped_Finding_ID", "Comment_Summary"],
          "semantic_review_threads.csv"
        );
      });
    }

    const btnExpSSF = document.getElementById("btn-export-ssf");
    if (btnExpSSF) {
      btnExpSSF.addEventListener("click", () => {
        exportTableToCSV(
          state.data.ssf_mapping,
          ["Finding_ID", "Origin_PR", "SSF_Relation", "Relation_Type", "Status", "SSF_Issue", "Evidence"],
          "semantic_ssf_mapping.csv"
        );
      });
    }

    const btnExpSec = document.getElementById("btn-export-sec");
    if (btnExpSec) {
      btnExpSec.addEventListener("click", () => {
        exportTableToCSV(
          state.data.security_privacy,
          ["Security_ID", "Category", "Severity", "PR_Number", "Pattern_Type", "Location", "Redacted_Evidence", "Current_Status", "Recommended_Action"],
          "semantic_security_privacy.csv"
        );
      });
    }
  }

  function exportTableToCSV(records, headers, filename) {
    if (!records || records.length === 0) {
      alert("No data available to export.");
      return;
    }

    const csvRows = [];
    csvRows.push(headers.join(","));

    records.forEach(r => {
      const row = headers.map(h => {
        let val = r[h];
        if (val === null || val === undefined) val = "";
        val = String(val).replace(/"/g, '""');
        return `"${val}"`;
      });
      csvRows.push(row.join(","));
    });

    const csvString = csvRows.join("\r\n");
    const blob = new Blob([csvString], { type: "text/csv;charset=utf-8;" });
    const link = document.createElement("a");
    const url = URL.createObjectURL(blob);
    link.setAttribute("href", url);
    link.setAttribute("download", filename);
    link.style.visibility = "hidden";
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  }

  // =========================================================================
  // Badge Helpers
  // =========================================================================
  function getStatusBadge(status) {
    const cls = getStatusBadgeClass(status);
    return `<span class="badge ${cls}">${escapeHtml(status || "UNKNOWN")}</span>`;
  }

  function getStatusBadgeClass(status) {
    switch (status) {
      case "STILL_PRESENT": return "badge-still-present";
      case "PARTIALLY_FIXED": return "badge-partially-fixed";
      case "FIXED_LATER": return "badge-fixed-later";
      case "OBSOLETE": return "badge-obsolete";
      case "FALSE_POSITIVE": return "badge-false-positive";
      case "UNVERIFIED": return "badge-unverified";
      default: return "badge-sec-none";
    }
  }

  function getSeverityBadge(sev) {
    const cls = getSeverityBadgeClass(sev);
    return `<span class="badge ${cls}">${escapeHtml(sev || "UNKNOWN")}</span>`;
  }

  function getSeverityBadgeClass(sev) {
    switch (sev) {
      case "CRITICAL": return "badge-sev-critical";
      case "HIGH": return "badge-sev-high";
      case "MEDIUM": return "badge-sev-medium";
      case "LOW": return "badge-sev-low";
      default: return "badge-sev-unknown";
    }
  }

  function getSecurityClassBadge(sc) {
    if (!sc || sc === "NONE") {
      return `<span class="badge badge-sec-none">NONE</span>`;
    }
    let cls = "badge-sec-boundary";
    if (sc === "CONFIDENTIALITY") cls = "badge-sec-confidentiality";
    else if (sc === "CAPABILITY_GATE") cls = "badge-sec-capability";
    else if (sc === "ARITHMETIC_SAFETY") cls = "badge-sec-arithmetic";
    else if (sc === "VERIFIER_GATE") cls = "badge-sec-verifier";
    return `<span class="badge ${cls}">${escapeHtml(sc)}</span>`;
  }

  function escapeHtml(str) {
    if (!str) return "";
    return String(str)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#039;");
  }

  // Public Interface for Inline Callbacks
  window.app = {
    navigateTo,
    openDrawer,
    closeDrawer,
    removeFilter: () => {},
    clearAllFilters: () => {}
  };

  // Launch on DOM ready
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();
