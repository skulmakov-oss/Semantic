//! Trusted operating-system path aliases for symlink-rejecting path walks.
//!
//! Package admission (`package_manifest::reject_reparse_path`) and Hub storage
//! (`hub::check_dir_is_not_a_symlink`) reject every symlinked component of a
//! path, so that a checked-out project cannot redirect reads or writes through
//! a planted link. On macOS, however, the operating system itself ships
//! `/tmp`, `/var` and `/etc` as root-owned symlinks to `/private/tmp`,
//! `/private/var` and `/private/etc`; the per-user temporary directory
//! (`$TMPDIR`, under `/var/folders`) is therefore always reached through one.
//! Without a platform boundary every such path is rejected.
//!
//! This module recognizes exactly those aliases and nothing else:
//!
//! - only as the first component of an absolute path (no prefix matching:
//!   `/tmpx` is not `/tmp`; `..` before it is not accepted);
//! - only after verifying on the live host that the alias is a symlink owned
//!   by the platform owner (uid 0) whose literal target is `private/<name>`,
//!   and that `/private` and `/private/<name>` are real directories (not links)
//!   owned by the same owner.
//!
//! The walk then starts at the verified physical prefix and continues to check
//! every remaining component exactly as before, so a user-created link
//! anywhere below the alias is still rejected. Paths are never canonicalized
//! here. Anything that does not verify returns the default (start at the
//! beginning), which keeps the caller's existing fail-closed behaviour.

use std::path::{Path, PathBuf};

/// Where a component-by-component symlink walk over `path` starts: the
/// physical prefix to push first and the number of leading components of
/// `path` that prefix replaces. `(PathBuf::new(), 0)` means "walk everything".
pub(crate) fn symlink_walk_start(path: &Path) -> (PathBuf, usize) {
    #[cfg(target_os = "macos")]
    {
        if let Some(start) = verified_alias_start(Path::new("/"), path, 0) {
            return start;
        }
    }
    let _ = path;
    (PathBuf::new(), 0)
}

/// OS-owned alias names (macOS). The physical directory is `private/<name>`.
#[cfg(unix)]
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
const OS_OWNED_ALIASES: [&str; 3] = ["tmp", "var", "etc"];

/// Core verification, parameterized by the filesystem root and the required
/// owner so that it can be exercised against a fixture tree in tests. In
/// production it is called with `/` and uid 0, on macOS only.
#[cfg(unix)]
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub(crate) fn verified_alias_start(
    fs_root: &Path,
    path: &Path,
    owner_uid: u32,
) -> Option<(PathBuf, usize)> {
    use std::os::unix::fs::MetadataExt;
    use std::path::Component;

    let mut components = path.components();
    if components.next() != Some(Component::RootDir) {
        return None;
    }
    let name = match components.next() {
        Some(Component::Normal(name)) => name.to_str()?,
        _ => return None,
    };
    if !OS_OWNED_ALIASES.contains(&name) {
        return None;
    }

    let alias = fs_root.join(name);
    let alias_meta = std::fs::symlink_metadata(&alias).ok()?;
    if !alias_meta.file_type().is_symlink() || alias_meta.uid() != owner_uid {
        return None;
    }
    if std::fs::read_link(&alias).ok()? != Path::new("private").join(name) {
        return None;
    }
    let private = fs_root.join("private");
    let private_meta = std::fs::symlink_metadata(&private).ok()?;
    if !private_meta.file_type().is_dir() || private_meta.uid() != owner_uid {
        return None;
    }
    let physical = private.join(name);
    let physical_meta = std::fs::symlink_metadata(&physical).ok()?;
    if !physical_meta.file_type().is_dir() || physical_meta.uid() != owner_uid {
        return None;
    }
    // The returned prefix is spelled from the real root so callers see an
    // ordinary absolute path; with a fixture root it is the fixture path.
    Some((physical, 2))
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::{symlink, MetadataExt};
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    /// A fixture filesystem root with `private/{tmp,var,etc}` and the three
    /// OS-style aliases, all owned by the test user.
    fn fixture_root(label: &str) -> (PathBuf, u32) {
        let root = std::env::temp_dir().join(format!(
            "smc_platform_paths_{label}_{}_{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        for name in OS_OWNED_ALIASES {
            std::fs::create_dir_all(root.join("private").join(name)).unwrap();
            symlink(Path::new("private").join(name), root.join(name)).unwrap();
        }
        let uid = std::fs::metadata(&root).unwrap().uid();
        (root, uid)
    }

    #[test]
    fn exact_os_alias_with_expected_target_is_accepted() {
        let (root, uid) = fixture_root("accept");
        for name in OS_OWNED_ALIASES {
            let path = Path::new("/").join(name).join("project/src/main.sm");
            let (prefix, consumed) = verified_alias_start(&root, &path, uid).expect("alias");
            assert_eq!(prefix, root.join("private").join(name));
            assert_eq!(consumed, 2);
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn similarly_named_and_non_leading_components_are_not_aliases() {
        let (root, uid) = fixture_root("names");
        for path in [
            "/tmpx/a.sm",
            "/tm/a.sm",
            "/TMP/a.sm",
            "/private/tmp/a.sm",
            "/Users/tmp/a.sm",
            "tmp/a.sm",
            "/../tmp/a.sm",
            "/",
        ] {
            assert_eq!(
                verified_alias_start(&root, Path::new(path), uid),
                None,
                "{path}"
            );
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn alias_with_wrong_owner_is_rejected() {
        let (root, uid) = fixture_root("owner");
        let other = uid.wrapping_add(1);
        assert_eq!(
            verified_alias_start(&root, Path::new("/tmp/a.sm"), other),
            None
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn alias_redirected_elsewhere_is_rejected() {
        let (root, uid) = fixture_root("retarget");
        std::fs::create_dir_all(root.join("elsewhere")).unwrap();
        std::fs::remove_file(root.join("tmp")).unwrap();
        symlink("elsewhere", root.join("tmp")).unwrap();
        assert_eq!(
            verified_alias_start(&root, Path::new("/tmp/a.sm"), uid),
            None
        );
        // An absolute target spelling is not the OS layout either.
        std::fs::remove_file(root.join("tmp")).unwrap();
        symlink(root.join("private/tmp"), root.join("tmp")).unwrap();
        assert_eq!(
            verified_alias_start(&root, Path::new("/tmp/a.sm"), uid),
            None
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn alias_that_is_a_real_directory_or_missing_is_not_an_alias() {
        let (root, uid) = fixture_root("realdir");
        std::fs::remove_file(root.join("var")).unwrap();
        std::fs::create_dir_all(root.join("var")).unwrap();
        assert_eq!(
            verified_alias_start(&root, Path::new("/var/a.sm"), uid),
            None
        );
        std::fs::remove_file(root.join("etc")).unwrap();
        assert_eq!(
            verified_alias_start(&root, Path::new("/etc/a.sm"), uid),
            None
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn substituted_physical_root_is_rejected() {
        // `private/tmp` itself replaced by a link (root substitution).
        let (root, uid) = fixture_root("physical");
        std::fs::create_dir_all(root.join("attacker")).unwrap();
        std::fs::remove_dir_all(root.join("private/tmp")).unwrap();
        symlink(root.join("attacker"), root.join("private/tmp")).unwrap();
        assert_eq!(
            verified_alias_start(&root, Path::new("/tmp/a.sm"), uid),
            None
        );
        // `private` itself a link.
        let (root2, uid2) = fixture_root("private_link");
        std::fs::rename(root2.join("private"), root2.join("real_private")).unwrap();
        symlink(root2.join("real_private"), root2.join("private")).unwrap();
        assert_eq!(
            verified_alias_start(&root2, Path::new("/tmp/a.sm"), uid2),
            None
        );
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_dir_all(&root2);
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn production_boundary_is_inert_off_macos() {
        assert_eq!(
            symlink_walk_start(Path::new("/tmp/a.sm")),
            (PathBuf::new(), 0)
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn production_boundary_recognizes_the_macos_layout() {
        for name in OS_OWNED_ALIASES {
            let (prefix, consumed) = symlink_walk_start(&Path::new("/").join(name).join("x"));
            assert_eq!(prefix, Path::new("/private").join(name));
            assert_eq!(consumed, 2);
        }
        assert_eq!(
            symlink_walk_start(Path::new("/Users/x")),
            (PathBuf::new(), 0)
        );
    }
}
