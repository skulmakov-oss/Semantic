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
}
