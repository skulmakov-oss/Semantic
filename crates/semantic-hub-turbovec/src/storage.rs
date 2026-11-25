//! Scoped, traversal-safe on-disk storage for one TurboVec adapter
//! instance.
//!
//! The Hub CLI is one short-lived process per invocation, so the adapter's
//! actual mutable state lives on disk (a `.tvim` file per index) inside a
//! directory fixed at adapter construction -- never a caller-supplied path.
//! An index name is validated to a small charset and mapped to exactly one
//! file inside that directory, so no request can ever name a path outside
//! the scoped directory.

use std::path::{Path, PathBuf};

/// Maximum index name length and the allowed charset keep the derived file
/// name short and free of anything that could be interpreted as a path
/// separator or traversal token on any supported OS.
pub const MAX_INDEX_NAME_LEN: usize = 64;

/// Maximum number of distinct persisted indexes the adapter will create.
/// Bounds on-disk state growth from a single Hub caller.
pub const MAX_INDEX_COUNT: usize = 256;

/// A scoped-storage safety violation: the root itself turned out to be a
/// symlink (or, on Windows, an NTFS junction/mount-point reparse point)
/// rather than a real directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopedStorageError {
    RootIsSymlink,
    IndexFileIsSymlink,
    PrivateFileIsSymlink,
    InvalidPrivateFileName,
    Io(String),
}

impl std::fmt::Display for ScopedStorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScopedStorageError::RootIsSymlink => write!(
                f,
                "scoped storage root is a symlink or reparse point, which could resolve outside the scoped directory"
            ),
            ScopedStorageError::IndexFileIsSymlink => write!(
                f,
                "index file is a symlink or reparse point, which could resolve outside the scoped directory"
            ),
            ScopedStorageError::PrivateFileIsSymlink => write!(
                f,
                "adapter-private file is a symlink or reparse point, which could resolve outside the scoped directory"
            ),
            ScopedStorageError::InvalidPrivateFileName => {
                write!(f, "adapter-private file name is not a safe leaf name")
            }
            ScopedStorageError::Io(msg) => write!(f, "{msg}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexNameError {
    Empty,
    TooLong { max: usize, actual: usize },
    InvalidCharacter(char),
}

impl std::fmt::Display for IndexNameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IndexNameError::Empty => write!(f, "index name must not be empty"),
            IndexNameError::TooLong { max, actual } => {
                write!(f, "index name length {actual} exceeds maximum {max}")
            }
            IndexNameError::InvalidCharacter(c) => {
                write!(f, "index name contains invalid character '{c}'")
            }
        }
    }
}

/// A validated index name: lowercase ASCII alphanumeric, `_`, and `-` only.
/// This is deliberately stricter than a general filename -- it rules out
/// `.`, `/`, `\`, and any other character that could carry path-traversal
/// meaning on any target OS, so validation alone (no canonicalization
/// dance) is sufficient to prove the derived path stays inside the scoped
/// directory.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IndexName(String);

impl IndexName {
    pub fn new(value: impl Into<String>) -> Result<Self, IndexNameError> {
        let value = value.into();
        if value.is_empty() {
            return Err(IndexNameError::Empty);
        }
        if value.len() > MAX_INDEX_NAME_LEN {
            return Err(IndexNameError::TooLong {
                max: MAX_INDEX_NAME_LEN,
                actual: value.len(),
            });
        }
        for c in value.chars() {
            let ok = c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-';
            if !ok {
                return Err(IndexNameError::InvalidCharacter(c));
            }
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Owns the one scoped directory this adapter instance is allowed to read
/// and write. Never derived from request input.
#[derive(Debug, Clone)]
pub struct ScopedStorage {
    root: PathBuf,
}

impl ScopedStorage {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn index_path(&self, name: &IndexName) -> PathBuf {
        self.root.join(format!("{}.tvim", name.as_str()))
    }

    pub fn exists(&self, name: &IndexName) -> bool {
        self.index_path(name).is_file()
    }

    pub fn ensure_root(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.root)
    }

    /// Reject a root whose path contains a symlink/junction in ANY
    /// component, not only the final one. `symlink_metadata` on the full
    /// joined path only inspects the *last* component's own link status --
    /// the OS resolves every component before it (following symlinks) just
    /// to locate that last component's parent directory. So checking only
    /// `self.root` as a whole would miss an ancestor such as `.semantic`
    /// or `.semantic/hub` being a symlink while the final component is a
    /// real directory reached through it. Walking every successively-
    /// longer prefix with its own `symlink_metadata` call (never
    /// `canonicalize`, which would silently resolve through the very link
    /// this is checking for) catches that. A root that does not exist yet
    /// is not a violation -- it will be a real directory once
    /// `create_dir_all` runs.
    fn check_root_is_not_a_symlink(&self) -> Result<(), ScopedStorageError> {
        // Start below a verified OS-owned alias (macOS /tmp, /var, /etc);
        // every remaining component is still checked. See os_alias_walk_start.
        let (mut probe, skip) = os_alias_walk_start(&self.root);
        for component in self.root.components().skip(skip) {
            probe.push(component);
            match std::fs::symlink_metadata(&probe) {
                Ok(meta) if meta.file_type().is_symlink() => {
                    return Err(ScopedStorageError::RootIsSymlink)
                }
                Ok(_) => continue,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
                Err(e) => return Err(ScopedStorageError::Io(e.to_string())),
            }
        }
        Ok(())
    }

    /// Fallible variant of [`Self::ensure_root`] that first confirms the
    /// root has not been swapped for a symlink -- every derived path is
    /// `root.join(<validated-name>)`, so if `root` itself resolved
    /// elsewhere (e.g. a malicious checked-out project shipping
    /// `.semantic/hub/vector.turbovec` as a pre-planted symlink), the OS
    /// would silently follow it on every read/write despite the index
    /// name itself being charset-validated. This must run before
    /// `create_dir_all`, not only before the final path join, so a
    /// pre-planted symlink is rejected before any directory-creation
    /// side effect too.
    pub fn ensure_root_checked(&self) -> Result<(), ScopedStorageError> {
        self.check_root_is_not_a_symlink()?;
        std::fs::create_dir_all(&self.root).map_err(|e| ScopedStorageError::Io(e.to_string()))
    }

    /// Fallible variant of [`Self::index_path`] that rejects a symlinked
    /// root OR a symlinked final `<name>.tvim` file before returning the
    /// path. Use this (not `index_path`) on every path that is actually
    /// read from or written to. Checking only the root's components (as
    /// `check_root_is_not_a_symlink` does) is not enough: if the leaf
    /// `<name>.tvim` file itself is a pre-planted symlink -- even inside
    /// a perfectly ordinary root directory -- `turbovec::IdMapIndex::load`
    /// would follow it via a plain `std::fs::File::open`, which
    /// dereferences symlinks by default. A not-yet-created index file is
    /// not a violation -- it will be a real file once `save_atomic`'s
    /// first write completes.
    pub fn checked_index_path(&self, name: &IndexName) -> Result<PathBuf, ScopedStorageError> {
        self.check_root_is_not_a_symlink()?;
        let path = self.index_path(name);
        match std::fs::symlink_metadata(&path) {
            Ok(meta) if meta.file_type().is_symlink() => {
                Err(ScopedStorageError::IndexFileIsSymlink)
            }
            Ok(_) => Ok(path),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(path),
            Err(e) => Err(ScopedStorageError::Io(e.to_string())),
        }
    }

    /// Resolve one adapter-private leaf (for example a transaction record
    /// or its candidate artifact) without following a pre-planted link.
    /// The caller supplies a file name only; traversal and absolute paths
    /// are rejected before joining it to the scoped root.
    pub(crate) fn checked_private_file_path(
        &self,
        file_name: &str,
    ) -> Result<PathBuf, ScopedStorageError> {
        // `\` is not a separator to Rust's `Path` parser on non-Windows
        // targets, so the explicit checks below are still needed there.
        // They alone are not enough on Windows: `PathBuf::join` treats an
        // argument that carries its own prefix (a drive-relative name like
        // `C:evil.tvim.txn`, containing neither `/` nor `\`) as replacing
        // `self.root` outright rather than appending to it. `Path::components`
        // only recognizes that prefix syntax when compiled for a Windows
        // target, so a name like `C:evil.tvim.txn` parses as a single
        // `Normal` component (and would be wrongly accepted) when this
        // crate's tests run on a non-Windows CI host -- checking for the
        // drive-prefix syntax explicitly, as a plain string pattern, keeps
        // the rejection identical on every host regardless of which
        // platform actually built the binary.
        let mut components = Path::new(file_name).components();
        let is_bare_leaf = matches!(components.next(), Some(std::path::Component::Normal(_)))
            && components.next().is_none();
        let has_drive_prefix = file_name
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphabetic)
            && file_name.as_bytes().get(1) == Some(&b':');
        if file_name.is_empty()
            || file_name == "."
            || file_name == ".."
            || file_name.contains('/')
            || file_name.contains('\\')
            || has_drive_prefix
            || !is_bare_leaf
        {
            return Err(ScopedStorageError::InvalidPrivateFileName);
        }
        self.check_root_is_not_a_symlink()?;
        let path = self.root.join(file_name);
        match std::fs::symlink_metadata(&path) {
            Ok(meta) if meta.file_type().is_symlink() => {
                Err(ScopedStorageError::PrivateFileIsSymlink)
            }
            Ok(_) => Ok(path),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(path),
            Err(e) => Err(ScopedStorageError::Io(e.to_string())),
        }
    }

    /// Count of currently persisted indexes, for the bounded-index-count
    /// check. Best-effort: a read error is treated as zero rather than
    /// failing the caller, since this is only used as an admission ceiling
    /// check, not a source of truth.
    pub fn index_count(&self) -> usize {
        std::fs::read_dir(&self.root)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().is_some_and(|ext| ext == "tvim"))
                    .count()
            })
            .unwrap_or(0)
    }
}

/// Where a component-by-component symlink walk starts: the verified physical
/// prefix of a macOS OS-owned alias and the number of leading components it
/// replaces, or `(PathBuf::new(), 0)`.
///
/// On macOS `/tmp`, `/var` and `/etc` are OS-owned symlinks to
/// `/private/*` and the per-user temp directory (`/var/folders/...`) is
/// always reached through one, so a root under it was always
/// `RootIsSymlink`. Only exactly those names, only as the first component of
/// an absolute path, and only after verifying that the alias is a uid-0
/// symlink with literal target `private/<name>` and that `/private` and
/// `/private/<name>` are real uid-0 directories. Nothing is canonicalized and
/// every later component is still checked. This is the twin of
/// `smc_cli::platform_paths` (the crates share no dependency); keep them
/// identical.
fn os_alias_walk_start(path: &Path) -> (PathBuf, usize) {
    #[cfg(target_os = "macos")]
    {
        if let Some(start) = verified_os_alias_start(Path::new("/"), path, 0) {
            return start;
        }
    }
    let _ = path;
    (PathBuf::new(), 0)
}

#[cfg(unix)]
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
const OS_OWNED_ALIASES: [&str; 3] = ["tmp", "var", "etc"];

#[cfg(unix)]
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn verified_os_alias_start(
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
    Some((physical, 2))
}

#[cfg(all(test, unix))]
mod os_alias_tests {
    use super::*;
    use std::os::unix::fs::{symlink, MetadataExt};

    fn fixture_root(label: &str) -> (PathBuf, u32) {
        let root = std::env::temp_dir().join(format!(
            "hub_turbovec_os_alias_{label}_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        for name in OS_OWNED_ALIASES {
            std::fs::create_dir_all(root.join("private").join(name)).unwrap();
            symlink(Path::new("private").join(name), root.join(name)).unwrap();
        }
        let uid = std::fs::metadata(&root).unwrap().uid();
        (root, uid)
    }

    #[test]
    fn exact_alias_accepted_and_lookalikes_rejected() {
        let (root, uid) = fixture_root("names");
        assert_eq!(
            verified_os_alias_start(&root, Path::new("/var/folders/x"), uid),
            Some((root.join("private/var"), 2))
        );
        for path in [
            "/varx/a",
            "/private/var/a",
            "var/a",
            "/Users/var/a",
            "/../var/a",
        ] {
            assert_eq!(
                verified_os_alias_start(&root, Path::new(path), uid),
                None,
                "{path}"
            );
        }
        assert_eq!(
            verified_os_alias_start(&root, Path::new("/var/a"), uid.wrapping_add(1)),
            None
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn retargeted_or_substituted_alias_is_rejected() {
        let (root, uid) = fixture_root("retarget");
        std::fs::create_dir_all(root.join("elsewhere")).unwrap();
        std::fs::remove_file(root.join("tmp")).unwrap();
        symlink("elsewhere", root.join("tmp")).unwrap();
        assert_eq!(
            verified_os_alias_start(&root, Path::new("/tmp/a"), uid),
            None
        );
        std::fs::remove_dir_all(root.join("private/var")).unwrap();
        symlink(root.join("elsewhere"), root.join("private/var")).unwrap();
        assert_eq!(
            verified_os_alias_start(&root, Path::new("/var/a"), uid),
            None
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn scoped_root_under_tmpdir_is_admitted_but_links_below_it_are_not() {
        let base = std::env::temp_dir().join(format!("hub_tv_alias_root_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join("real")).unwrap();
        assert!(
            ScopedStorage::new(base.join("real/.semantic/hub/vector.turbovec"))
                .ensure_root_checked()
                .is_ok()
        );
        symlink(base.join("real"), base.join("link")).unwrap();
        assert_eq!(
            ScopedStorage::new(base.join("link/.semantic/hub/vector.turbovec"))
                .ensure_root_checked(),
            Err(ScopedStorageError::RootIsSymlink)
        );
        let _ = std::fs::remove_dir_all(&base);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_names_are_accepted() {
        assert!(IndexName::new("my-index_01").is_ok());
    }

    #[test]
    fn empty_name_is_rejected() {
        assert_eq!(IndexName::new("").unwrap_err(), IndexNameError::Empty);
    }

    #[test]
    fn traversal_characters_are_rejected() {
        assert!(matches!(
            IndexName::new("../escape"),
            Err(IndexNameError::InvalidCharacter(_))
        ));
        assert!(matches!(
            IndexName::new("a/b"),
            Err(IndexNameError::InvalidCharacter(_))
        ));
        assert!(matches!(
            IndexName::new("a\\b"),
            Err(IndexNameError::InvalidCharacter(_))
        ));
        assert!(matches!(
            IndexName::new("a.b"),
            Err(IndexNameError::InvalidCharacter(_))
        ));
    }

    #[test]
    fn oversized_name_is_rejected() {
        let long = "a".repeat(MAX_INDEX_NAME_LEN + 1);
        assert!(matches!(
            IndexName::new(long),
            Err(IndexNameError::TooLong { .. })
        ));
    }

    #[test]
    fn index_path_stays_inside_scoped_root() {
        let storage = ScopedStorage::new("C:/scoped/root");
        let name = IndexName::new("my-index").unwrap();
        let path = storage.index_path(&name);
        assert!(path.starts_with("C:/scoped/root"));
        assert_eq!(path.file_name().unwrap(), "my-index.tvim");
    }

    fn temp_dir(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "semantic-hub-turbovec-storage-test-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn checked_index_path_accepts_a_real_directory_root() {
        let root = temp_dir("real-root");
        let storage = ScopedStorage::new(&root);
        storage.ensure_root_checked().unwrap();
        let name = IndexName::new("docs").unwrap();
        assert!(storage.checked_index_path(&name).is_ok());
    }

    #[test]
    fn checked_index_path_accepts_a_not_yet_created_root() {
        // A root that does not exist yet is not a violation -- it becomes
        // a real directory the first time ensure_root_checked runs.
        let root = temp_dir("not-yet-created");
        let storage = ScopedStorage::new(&root);
        let name = IndexName::new("docs").unwrap();
        assert!(storage.checked_index_path(&name).is_ok());
    }

    #[test]
    fn checked_private_file_path_rejects_non_leaf_names() {
        let storage = ScopedStorage::new(temp_dir("private-name"));
        for invalid in [
            "",
            ".",
            "..",
            "../outside",
            "sub/file",
            "sub\\file",
            // Regression: `PathBuf::join` treats an argument that carries
            // its own prefix (a Windows drive-relative name) as replacing
            // the base path outright rather than appending to it, even
            // though this name contains neither `/` nor `\` and so passed
            // the separator checks alone.
            "C:evil.tvim.txn",
            r"C:\evil.tvim.txn",
        ] {
            assert_eq!(
                storage.checked_private_file_path(invalid).unwrap_err(),
                ScopedStorageError::InvalidPrivateFileName,
                "expected {invalid:?} to be rejected"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn checked_index_path_rejects_a_symlinked_root() {
        // Regression test: a scoped root that is itself a symlink must be
        // rejected before any path derived from it is used for real I/O --
        // otherwise every read/write silently resolves through the OS to
        // wherever the symlink points, outside the directory this adapter
        // is scoped to.
        let target = temp_dir("symlink-target");
        std::fs::create_dir_all(&target).unwrap();
        let link = temp_dir("symlink-root");
        std::os::unix::fs::symlink(&target, &link).unwrap();

        let storage = ScopedStorage::new(&link);
        let name = IndexName::new("docs").unwrap();
        assert_eq!(
            storage.checked_index_path(&name).unwrap_err(),
            ScopedStorageError::RootIsSymlink
        );
        assert_eq!(
            storage.ensure_root_checked().unwrap_err(),
            ScopedStorageError::RootIsSymlink
        );
    }

    #[cfg(unix)]
    #[test]
    fn checked_index_path_rejects_a_symlink_in_an_ancestor_component_not_just_the_root_itself() {
        // Regression test: symlink_metadata on the full joined path only
        // inspects the FINAL component's own link status -- an ancestor
        // component (e.g. ".semantic" itself, above the actual scoped
        // root) being a symlink is silently followed by the OS while
        // locating that final component, so checking only whole-path
        // metadata would miss it entirely even though the final
        // component itself is a real directory.
        let real_target = temp_dir("ancestor-symlink-real-target");
        let nested_root = real_target.join("hub").join("vector.turbovec");
        std::fs::create_dir_all(&nested_root).unwrap();

        let fake_ancestor = temp_dir("ancestor-symlink-fake-ancestor");
        std::os::unix::fs::symlink(&real_target, &fake_ancestor).unwrap();

        let root_through_link = fake_ancestor.join("hub").join("vector.turbovec");
        let storage = ScopedStorage::new(&root_through_link);
        let name = IndexName::new("docs").unwrap();
        assert_eq!(
            storage.checked_index_path(&name).unwrap_err(),
            ScopedStorageError::RootIsSymlink
        );
    }

    #[cfg(unix)]
    #[test]
    fn checked_index_path_rejects_a_symlinked_index_file_inside_a_real_root() {
        // Regression test: check_root_is_not_a_symlink only ever inspects
        // self.root's own components -- it never sees the <name>.tvim
        // leaf appended afterward. A perfectly ordinary root directory
        // containing a pre-planted symlink AT the index file's own path
        // must still be rejected, since turbovec::IdMapIndex::load
        // dereferences symlinks via a plain std::fs::File::open.
        let root = temp_dir("leaf-symlink-root");
        std::fs::create_dir_all(&root).unwrap();
        let outside_target = temp_dir("leaf-symlink-outside-target");
        std::fs::write(&outside_target, b"not a real .tvim file").unwrap();

        let storage = ScopedStorage::new(&root);
        let name = IndexName::new("docs").unwrap();
        let leaf_path = storage.index_path(&name);
        std::os::unix::fs::symlink(&outside_target, &leaf_path).unwrap();

        assert_eq!(
            storage.checked_index_path(&name).unwrap_err(),
            ScopedStorageError::IndexFileIsSymlink
        );
    }

    #[cfg(unix)]
    #[test]
    fn checked_private_file_path_rejects_a_symlinked_leaf() {
        let root = temp_dir("private-leaf-symlink-root");
        std::fs::create_dir_all(&root).unwrap();
        let target = temp_dir("private-leaf-symlink-target");
        std::fs::write(&target, b"outside").unwrap();
        std::os::unix::fs::symlink(&target, root.join("docs.tvim.txn")).unwrap();

        let storage = ScopedStorage::new(root);
        assert_eq!(
            storage
                .checked_private_file_path("docs.tvim.txn")
                .unwrap_err(),
            ScopedStorageError::PrivateFileIsSymlink
        );
    }
}
