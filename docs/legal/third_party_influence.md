# Third-Party Influence Register

Status: documentation-only influence register
Scope: architectural inspiration, not dependency inventory

## Purpose

This file records third-party projects that influenced Semantic UI architecture thinking.

An entry in this file does not mean:

- the project is a dependency;
- code was copied;
- Semantic UI is a derivative work;
- the project owns any part of Semantic runtime.

Actual dependencies must be recorded separately in:

```text
docs/legal/third_party_dependencies.md
```

## Influence Entries

| Project | Influence type | Notes |
| --- | --- | --- |
| Slint | declarative UI, component/property model, renderer separation | architecture reference only |
| Lapce / Floem | Rust-native editor/workbench patterns | architecture reference only |
| Makepad | live design and rendering-loop ideas | architecture reference only |
| Zed / GPUI | high-performance editor UX and command/workspace model | architecture reference only |
| Tauri | shell/bridge/IPC pattern | architecture reference only |
| Monaco / CodeMirror | mature code editor interaction model | possible temporary adapter reference |
| React Flow / Cytoscape / ELK | graph/layout/subgraph interaction patterns | architecture reference only |

## Rule

```text
Influence is not dependency.
Dependency is not derivative.
Derivative code requires explicit legal handling.
```
