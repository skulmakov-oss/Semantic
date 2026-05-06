# Third-Party Dependency Register

Status: dependency register placeholder
Scope: actual admitted third-party dependencies only

## Purpose

This file records actual third-party dependencies used by Semantic UI or related UI adapters.

Do not add a project here merely because it inspired the architecture.

A project belongs here only if it is actually used as:

- a Rust crate;
- an npm package;
- a vendored library;
- a linked runtime dependency;
- copied source code;
- modified fork.

## Current Semantic UI dependency status

As of this document:

```text
No Semantic UI third-party runtime dependency is admitted by this document.
```

## Future Entry Format

| Dependency | Version/source | Purpose | License | Notes |
| --- | --- | --- | --- | --- |
| TBD | TBD | TBD | TBD | TBD |

## Rule

Before adding a dependency:

- check license;
- check transitive dependencies;
- check whether it affects the Semantic language/runtime boundary;
- document whether it is runtime, build-time, dev-only, or adapter-only.

## Non-Dependency Note

The following projects may be listed in `third_party_influence.md` as architectural influences, but are not dependencies unless explicitly added here:

- Slint
- Lapce / Floem
- Makepad
- Zed / GPUI
- Tauri
- Monaco / CodeMirror
- React Flow / Cytoscape / ELK
