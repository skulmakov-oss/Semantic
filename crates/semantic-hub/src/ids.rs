//! Identity and version types for the Semantic Hub public contract.
//!
//! Every identifier here has a validated canonical syntax, a deterministic
//! `Display`/`FromStr` round trip, and a bounded maximum length. None of
//! these types grant authority by existing -- they are stable names used by
//! the registry, admission path, and audit/provenance records.

use core::fmt;
use core::str::FromStr;

/// Maximum byte length accepted for a `HubToolId` or `HubOperationId` segment
/// string. Chosen to keep audit records and canonical serialization bounded.
pub const MAX_DOTTED_ID_LEN: usize = 128;

/// Maximum byte length accepted for a `HubRequestId` / `HubSessionId` /
/// `HubCallerIdentity` string.
pub const MAX_HANDLE_LEN: usize = 128;

/// Reasons a candidate identifier string fails validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdSyntaxError {
    Empty,
    TooLong { max: usize, actual: usize },
    EmptySegment,
    InvalidCharacter(char),
    LeadingOrTrailingDot,
}

impl fmt::Display for IdSyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IdSyntaxError::Empty => write!(f, "identifier must not be empty"),
            IdSyntaxError::TooLong { max, actual } => {
                write!(f, "identifier length {actual} exceeds maximum {max}")
            }
            IdSyntaxError::EmptySegment => write!(f, "identifier has an empty dotted segment"),
            IdSyntaxError::InvalidCharacter(c) => {
                write!(f, "identifier contains invalid character '{c}'")
            }
            IdSyntaxError::LeadingOrTrailingDot => {
                write!(f, "identifier must not start or end with '.'")
            }
        }
    }
}

fn validate_dotted_id(s: &str, max_len: usize) -> Result<(), IdSyntaxError> {
    if s.is_empty() {
        return Err(IdSyntaxError::Empty);
    }
    if s.len() > max_len {
        return Err(IdSyntaxError::TooLong {
            max: max_len,
            actual: s.len(),
        });
    }
    if s.starts_with('.') || s.ends_with('.') {
        return Err(IdSyntaxError::LeadingOrTrailingDot);
    }
    for segment in s.split('.') {
        if segment.is_empty() {
            return Err(IdSyntaxError::EmptySegment);
        }
        for c in segment.chars() {
            let ok = c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-';
            if !ok {
                return Err(IdSyntaxError::InvalidCharacter(c));
            }
        }
    }
    Ok(())
}

fn validate_handle(s: &str, max_len: usize) -> Result<(), IdSyntaxError> {
    if s.is_empty() {
        return Err(IdSyntaxError::Empty);
    }
    if s.len() > max_len {
        return Err(IdSyntaxError::TooLong {
            max: max_len,
            actual: s.len(),
        });
    }
    for c in s.chars() {
        let ok = c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == ':' || c == '.';
        if !ok {
            return Err(IdSyntaxError::InvalidCharacter(c));
        }
    }
    Ok(())
}

macro_rules! define_dotted_id {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            /// Validate and construct from an owned string.
            pub fn new(value: impl Into<String>) -> Result<Self, IdSyntaxError> {
                let value = value.into();
                validate_dotted_id(&value, MAX_DOTTED_ID_LEN)?;
                Ok(Self(value))
            }

            /// Borrow the canonical string form.
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl FromStr for $name {
            type Err = IdSyntaxError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Self::new(s)
            }
        }
    };
}

macro_rules! define_handle_id {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            /// Validate and construct from an owned string.
            pub fn new(value: impl Into<String>) -> Result<Self, IdSyntaxError> {
                let value = value.into();
                validate_handle(&value, MAX_HANDLE_LEN)?;
                Ok(Self(value))
            }

            /// Borrow the canonical string form.
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl FromStr for $name {
            type Err = IdSyntaxError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Self::new(s)
            }
        }
    };
}

define_dotted_id!(
    HubToolId,
    "Stable dotted identity for one tool implementation, e.g. `vector.turbovec`."
);
define_dotted_id!(
    HubOperationId,
    "Stable dotted identity for one bounded operation, e.g. `vector.search`."
);
define_handle_id!(
    HubRequestId,
    "Caller-supplied or Hub-issued correlation handle for one invocation."
);
define_handle_id!(HubSessionId, "Correlation handle for one caller session.");
define_handle_id!(
    HubCallerIdentity,
    "Stable identity of the component or user that issued a `HubRequest`."
);

/// Semantic version of one tool implementation: `major.minor.patch`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HubToolVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl HubToolVersion {
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

impl fmt::Display for HubToolVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl FromStr for HubToolVersion {
    type Err = IdSyntaxError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('.');
        let (Some(major), Some(minor), Some(patch), None) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            return Err(IdSyntaxError::InvalidCharacter('.'));
        };
        let parse = |p: &str| -> Result<u32, IdSyntaxError> {
            p.parse::<u32>()
                .map_err(|_| IdSyntaxError::InvalidCharacter('.'))
        };
        Ok(Self::new(parse(major)?, parse(minor)?, parse(patch)?))
    }
}

/// Version of the generic Semantic Hub API contract itself, independent of
/// any one tool's version. Bumped only when the envelope/registry/admission
/// contract changes in a way callers must be aware of.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HubApiVersion {
    pub major: u32,
    pub minor: u32,
}

impl HubApiVersion {
    pub const fn new(major: u32, minor: u32) -> Self {
        Self { major, minor }
    }

    /// The Hub API version implemented by this crate.
    pub const CURRENT: HubApiVersion = HubApiVersion::new(0, 1);

    /// Whether a request built against `other` may be admitted by an
    /// implementation of `self`. v0 policy: exact major match, caller minor
    /// may be less than or equal to the implementation's minor.
    pub const fn is_compatible_with(&self, other: HubApiVersion) -> bool {
        self.major == other.major && other.minor <= self.minor
    }
}

impl fmt::Display for HubApiVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_id_accepts_canonical_dotted_form() {
        assert!(HubToolId::new("vector.turbovec").is_ok());
        assert_eq!(
            HubToolId::new("vector.turbovec").unwrap().as_str(),
            "vector.turbovec"
        );
    }

    #[test]
    fn tool_id_rejects_empty() {
        assert_eq!(HubToolId::new("").unwrap_err(), IdSyntaxError::Empty);
    }

    #[test]
    fn tool_id_rejects_uppercase_and_symbols() {
        assert!(matches!(
            HubToolId::new("Vector.TurboVec"),
            Err(IdSyntaxError::InvalidCharacter(_))
        ));
        assert!(matches!(
            HubToolId::new("vector/turbovec"),
            Err(IdSyntaxError::InvalidCharacter(_))
        ));
    }

    #[test]
    fn tool_id_rejects_leading_trailing_or_empty_segment() {
        assert_eq!(
            HubToolId::new(".vector").unwrap_err(),
            IdSyntaxError::LeadingOrTrailingDot
        );
        assert_eq!(
            HubToolId::new("vector.").unwrap_err(),
            IdSyntaxError::LeadingOrTrailingDot
        );
        assert_eq!(
            HubToolId::new("vector..turbovec").unwrap_err(),
            IdSyntaxError::EmptySegment
        );
    }

    #[test]
    fn tool_id_rejects_oversized_input() {
        let long = "a".repeat(MAX_DOTTED_ID_LEN + 1);
        assert!(matches!(
            HubToolId::new(long),
            Err(IdSyntaxError::TooLong { .. })
        ));
    }

    #[test]
    fn tool_id_display_and_fromstr_round_trip() {
        let id = HubToolId::new("vector.turbovec").unwrap();
        let rendered = format!("{id}");
        let parsed: HubToolId = rendered.parse().unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn operation_id_accepts_canonical_dotted_form() {
        assert!(HubOperationId::new("vector.search.filtered").is_ok());
    }

    #[test]
    fn request_id_accepts_handle_characters() {
        assert!(HubRequestId::new("req-0001").is_ok());
        assert!(HubRequestId::new("req:local:0001").is_ok());
        assert!(HubRequestId::new("req 0001").is_err());
    }

    #[test]
    fn tool_version_display_and_parse_round_trip() {
        let v = HubToolVersion::new(1, 2, 3);
        let s = format!("{v}");
        assert_eq!(s, "1.2.3");
        assert_eq!(s.parse::<HubToolVersion>().unwrap(), v);
    }

    #[test]
    fn tool_version_rejects_malformed_input() {
        assert!("1.2".parse::<HubToolVersion>().is_err());
        assert!("1.2.3.4".parse::<HubToolVersion>().is_err());
        assert!("a.b.c".parse::<HubToolVersion>().is_err());
    }

    #[test]
    fn api_version_compatibility_requires_matching_major_and_caller_minor_at_most() {
        let impl_version = HubApiVersion::new(0, 3);
        assert!(impl_version.is_compatible_with(HubApiVersion::new(0, 0)));
        assert!(impl_version.is_compatible_with(HubApiVersion::new(0, 3)));
        assert!(!impl_version.is_compatible_with(HubApiVersion::new(0, 4)));
        assert!(!impl_version.is_compatible_with(HubApiVersion::new(1, 0)));
    }

    #[test]
    fn api_version_display_is_stable() {
        assert_eq!(format!("{}", HubApiVersion::new(0, 1)), "0.1");
    }
}
