# Package Baseline v0

Status: Stable Foundation contract candidate
Contract: `semantic.foundation.package/0.1`

## Contour

The baseline extends the existing `Semantic.package` loader; it does not add a
second package authority. A manifest has one descriptive package name, a
contained package/module root, zero or more local path dependencies, and zero
or more capability requests. Local dependency paths are explicit and relative.
They may name a sibling package with `..`; the dependency's own manifest and
module root then become its declared containment boundary.

```text
format 1
package app
manifest_dir .
module_root src
dep math math ../math
capability fs.read
```

Package-qualified imports retain the existing `<alias>::<module>` form. All
resolved modules must remain under the dependency's admitted `module_root`.
Absolute dependency paths, escaping `manifest_dir`/`module_root` values, and
symlink/reparse paths are rejected. Enumeration and diagnostics use sorted,
normalized paths.

## Identity and provenance

`smc package inspect <project-root>` recursively reads every declared local
dependency and prints deterministic JSON with schema
`semantic.foundation.package.provenance/0.1`. The record contains:

- the root package label;
- packages sorted by name;
- sorted dependency edges and normalized declared local paths;
- a normalized manifest fingerprint;
- a source-content fingerprint over sorted module-root files;
- sorted capability requests;
- a graph fingerprint binding the complete record.

The baseline fingerprint is `fnv1a64:<16 lowercase hex digits>`. It is a
deterministic change detector, explicitly **not** a cryptographic trust claim.
Cryptographic artifact provenance and signing belong to SSF-10.

A dependency may pin both observed values:

```text
dep math math ../math fnv1a64:0123456789abcdef fnv1a64:fedcba9876543210
```

When pins are present, loading and graph inspection fail deterministically on
manifest or content mismatch. Names and future version labels never replace
these inputs and never authorize verifier admission.

## Authority boundary

`capability <id>` is inventory only. It does not construct or modify a runtime
`CapabilityManifest`, does not propagate transitively, and cannot grant host
authority. SemCode produced from package sources still passes through normal
verifier admission before execution.

The inspector is read-only: it prints a provenance-equivalent record and writes
no lockfile, source, cache, or package data. There is no network or registry
path in this contract.

## Deterministic failures

The baseline rejects missing dependency manifests, declared/actual package-name
mismatch, duplicate package names at different roots, dependency cycles,
invalid or stale fingerprints, root escape, and link/reparse traversal. Cycle
diagnostics use logical names in deterministic edge order and contain no
checkout-specific absolute path.

## Explicit deferrals

No registry, remote fetch, version solver, publish/install workflow, build
script, install hook, implicit capability grant, cryptographic signature, or
broad workspace model is included. Bounded workspaces remain deferred because
the canonical local composition examples do not require them.

SSF-07 may start from this closed local package contour. It may use package
composition as qualification evidence but must not widen package resolution.
