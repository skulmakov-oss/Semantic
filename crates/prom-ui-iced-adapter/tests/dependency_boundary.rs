//! Proves this crate's own public API surface is Iced-free -- a consumer
//! (`workbench_semantic` or the proof-gate fixture) only ever needs to
//! import `prom_ui_iced_adapter::{PromNode, PromApplication, run, ...}`,
//! never `iced::` itself. Complements
//! `examples/workbench_semantic/tests/no_iced_dependency.rs` (which
//! proves the boundary from the *consumer* side, via the real resolved
//! dependency graph); this test proves it from the *adapter's own source*
//! side, so drift is caught even before a consumer exists to test against.

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/prom-ui-iced-adapter is two levels under the repo root")
        .to_path_buf()
}

#[test]
fn lib_rs_public_surface_names_no_iced_type() {
    let lib_path = repo_root().join("crates/prom-ui-iced-adapter/src/lib.rs");
    let text = std::fs::read_to_string(&lib_path)
        .unwrap_or_else(|e| panic!("must be able to read {}: {e}", lib_path.display()));

    // Real check: every `pub use` line (the crate's actual public
    // re-export surface) must not mention `iced` anywhere in its path.
    // Deliberately scoped to `pub use` lines rather than the whole file,
    // since the file's *private* `mod` declarations legitimately wire up
    // modules that use `iced` internally (`app`, `convert`, `message`) --
    // that's exactly the point of the boundary.
    let mut checked_any = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("pub use") {
            checked_any = true;
            assert!(
                !trimmed.contains("iced"),
                "a `pub use` line in prom-ui-iced-adapter's lib.rs names `iced`, \
                 leaking it across the public boundary: {trimmed:?}"
            );
        }
    }
    assert!(
        checked_any,
        "expected at least one real `pub use` line in lib.rs -- if this fails, \
         the crate's public surface changed shape and this check needs updating, \
         not that the boundary is somehow fine with zero exports"
    );
}

#[test]
fn only_authorized_modules_import_iced() {
    // The real, reviewed list of source files inside this crate that are
    // allowed to write `use iced` / `iced::` -- everything else in this
    // crate (and, by the sibling test in workbench_semantic, everything
    // outside this crate) must not.
    let authorized = [
        "src/app.rs",
        "src/convert.rs",
        "src/message.rs",
        "src/theme.rs",
        "examples/fixture.rs", // consumer example: intentionally Iced-free -- verified separately below
        // lib.rs's own `#[cfg(test)]` block references `iced::Element` as
        // a type annotation to verify the converter -- an internal test,
        // not the crate's real public surface. That surface (`pub use`)
        // is separately, specifically checked by
        // `lib_rs_public_surface_names_no_iced_type` above.
        "src/lib.rs",
    ];

    let crate_root = repo_root().join("crates/prom-ui-iced-adapter");
    let mut offenders = Vec::new();
    let mut checked_node_rs = false;
    for entry in walk_rs_files(&crate_root.join("src")) {
        let rel = entry
            .strip_prefix(&crate_root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let text = std::fs::read_to_string(&entry).unwrap();
        // Skip full comment lines (`//`, `///`, `//!`) -- this check is
        // about real code referencing `iced`, not doc prose that
        // legitimately explains the boundary by naming it (e.g. this
        // very file, and `node.rs`'s own module doc, both say the word
        // "iced" while deliberately containing zero real `iced` code).
        let names_iced = text.lines().any(|l| {
            let trimmed = l.trim_start();
            !trimmed.starts_with("//") && (l.contains("iced::") || trimmed.starts_with("use iced"))
        });
        let is_authorized = authorized.iter().any(|a| *a == rel);
        if names_iced && !is_authorized {
            offenders.push(rel.clone());
        }
        if rel == "src/node.rs" {
            checked_node_rs = true;
            // Extra-strong, specifically-named assertion for the public
            // contract module itself, not just "happens not to be in the
            // authorized list" -- this is the one file where an `iced`
            // reference would be the clearest possible boundary
            // violation.
            assert!(
                !names_iced,
                "src/node.rs is the public PromNode contract -- it must never \
                 reference `iced` at all"
            );
        }
    }
    assert!(
        checked_node_rs,
        "expected to find and check src/node.rs -- if this fails, the crate's \
         file layout changed and this test needs updating"
    );
    assert!(
        offenders.is_empty(),
        "these prom-ui-iced-adapter source files reference `iced` but are not in \
         the reviewed `authorized` list -- either this is real, reviewed drift \
         (update `authorized`) or a real boundary violation: {offenders:?}"
    );
}

#[test]
fn fixture_example_names_no_iced_type() {
    let fixture_path = repo_root().join("crates/prom-ui-iced-adapter/examples/fixture.rs");
    let text = std::fs::read_to_string(&fixture_path)
        .unwrap_or_else(|e| panic!("must be able to read {}: {e}", fixture_path.display()));
    assert!(
        !text.contains("iced::") && !text.lines().any(|l| l.trim_start().starts_with("use iced")),
        "the proof-gate fixture must consume only the public, Iced-free \
         prom_ui_iced_adapter surface -- if it ever needs to name `iced` \
         directly, that's a real sign the public contract is missing \
         something, not something to work around in the fixture"
    );
}

fn walk_rs_files(dir: &std::path::Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(walk_rs_files(&path));
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
    out
}
