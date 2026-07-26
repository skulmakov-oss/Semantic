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
    Io(String),
}

impl std::fmt::Display for ScopedStorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScopedStorageError::RootIsSymlink => write!(
                f,
                "scoped storage root is a symlink or reparse point, which could resolve outside the scoped directory"
            ),
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
        let mut probe = PathBuf::new();
        for component in self.root.components() {
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
    /// root before deriving the path. Use this (not `index_path`) on
    /// every path that is actually read from or written to.
    pub fn checked_index_path(&self, name: &IndexName) -> Result<PathBuf, ScopedStorageError> {
        self.check_root_is_not_a_symlink()?;
        Ok(self.index_path(name))
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
}
