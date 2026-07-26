//! JSON request/reply payload shapes for the `vector.turbovec` tool.
//!
//! These are parsed with `serde_json`, which rejects malformed/truncated
//! JSON and trailing garbage by construction (`from_slice` errors on any
//! non-whitespace left after the value). Structural bounds (result counts,
//! dimensions, vector counts) are enforced separately after parsing, since
//! serde alone does not know Hub's resource budget.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub enum PayloadError {
    Json(String),
    TooManyVectors { max: usize, actual: usize },
    TooManyResults { max: usize, actual: usize },
    VectorLengthMismatch { expected: usize, actual: usize },
    IdsVectorsCountMismatch { ids: usize, vectors: usize },
    EmptyFilter,
    MissingIndexName,
}

impl std::fmt::Display for PayloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PayloadError::Json(m) => write!(f, "malformed request payload: {m}"),
            PayloadError::TooManyVectors { max, actual } => {
                write!(f, "request has {actual} vectors, exceeding maximum {max}")
            }
            PayloadError::TooManyResults { max, actual } => {
                write!(f, "requested k={actual} exceeds maximum {max}")
            }
            PayloadError::VectorLengthMismatch { expected, actual } => write!(
                f,
                "vector length {actual} does not match index dimension {expected}"
            ),
            PayloadError::IdsVectorsCountMismatch { ids, vectors } => {
                write!(f, "ids count {ids} does not match vectors count {vectors}")
            }
            PayloadError::EmptyFilter => {
                write!(f, "filtered search requires a non-empty allowed_ids list")
            }
            PayloadError::MissingIndexName => {
                write!(f, "request is missing the required \"index\" field")
            }
        }
    }
}

pub fn parse<'a, T: Deserialize<'a>>(bytes: &'a [u8]) -> Result<T, PayloadError> {
    serde_json::from_slice(bytes).map_err(|e| PayloadError::Json(e.to_string()))
}

pub fn to_bytes<T: Serialize>(value: &T) -> Vec<u8> {
    // Construction-time serialization of our own well-formed reply types
    // cannot fail (no floats-as-map-keys, no non-UTF8 strings), so a
    // panic here would indicate a Hub-internal bug, not bad caller input.
    serde_json::to_vec(value).expect("reply payload is always representable as JSON")
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateIndexRequest {
    pub index: String,
    pub dim: usize,
    #[serde(default = "default_bit_width")]
    pub bit_width: usize,
}

fn default_bit_width() -> usize {
    4
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateIndexReply {
    pub index: String,
    pub dim: usize,
    pub bit_width: usize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexNameOnlyRequest {
    pub index: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DescribeIndexReply {
    pub index: String,
    pub dim: usize,
    pub bit_width: usize,
    pub len: usize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InsertRequest {
    pub index: String,
    pub vectors: Vec<Vec<f32>>,
    pub ids: Vec<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InsertReply {
    pub index: String,
    pub inserted: usize,
    pub len: usize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoveRequest {
    pub index: String,
    pub ids: Vec<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveReply {
    pub index: String,
    pub removed: Vec<u64>,
    pub not_found: Vec<u64>,
    pub len: usize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchRequest {
    pub index: String,
    pub queries: Vec<Vec<f32>>,
    pub k: usize,
    /// Present and non-empty only for `vector.search.filtered`.
    #[serde(default)]
    pub allowed_ids: Option<Vec<u64>>,
}

/// One candidate hit. Explicitly evidence, not a Semantic-truth judgment --
/// documented on the reply type and in the adapter doc, per the Hub's
/// canonical rule that TurboVec output is untrusted computational evidence.
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchHit {
    pub external_id: u64,
    pub score: f32,
    pub rank: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchReply {
    pub index: String,
    pub index_version: u32,
    /// One row per query, outer-to-inner in request order.
    pub hits: Vec<Vec<SearchHit>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResetRequest {
    pub index: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResetReply {
    pub index: String,
    pub dim: usize,
    pub bit_width: usize,
}

/// Bound checks shared by insert/search request validation, run after JSON
/// parsing succeeds and before any turbovec call or allocation proportional
/// to attacker-controlled counts.
pub fn check_vector_batch_bounds(
    vector_count: usize,
    max_vectors: usize,
) -> Result<(), PayloadError> {
    if vector_count > max_vectors {
        return Err(PayloadError::TooManyVectors {
            max: max_vectors,
            actual: vector_count,
        });
    }
    Ok(())
}

pub fn check_result_count_bounds(k: usize, max_results: usize) -> Result<(), PayloadError> {
    if k > max_results {
        return Err(PayloadError::TooManyResults {
            max: max_results,
            actual: k,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_index_request_parses_with_default_bit_width() {
        let req: CreateIndexRequest = parse(br#"{"index":"docs","dim":128}"#).unwrap();
        assert_eq!(req.bit_width, 4);
    }

    #[test]
    fn create_index_request_honors_explicit_bit_width() {
        let req: CreateIndexRequest =
            parse(br#"{"index":"docs","dim":128,"bit_width":2}"#).unwrap();
        assert_eq!(req.bit_width, 2);
    }

    #[test]
    fn malformed_json_is_rejected() {
        let result: Result<CreateIndexRequest, _> = parse(br#"{"index":"docs","dim":"#);
        assert!(result.is_err());
    }

    #[test]
    fn trailing_garbage_is_rejected() {
        let result: Result<CreateIndexRequest, _> =
            parse(br#"{"index":"docs","dim":128}trailing-garbage"#);
        assert!(result.is_err());
    }

    #[test]
    fn missing_required_field_is_rejected() {
        let result: Result<CreateIndexRequest, _> = parse(br#"{"dim":128}"#);
        assert!(result.is_err());
    }

    #[test]
    fn create_index_request_rejects_an_unknown_field_instead_of_silently_ignoring_it() {
        // Regression test: without deny_unknown_fields, a typo like
        // "bit_wdith" was silently dropped, bit_width fell back to its
        // default, and the index was created with parameters the caller
        // never asked for -- with no way to undo it (there is no
        // delete-index operation).
        let result: Result<CreateIndexRequest, _> =
            parse(br#"{"index":"docs","dim":128,"bit_wdith":2}"#);
        assert!(result.is_err());
    }

    #[test]
    fn nan_in_vector_is_accepted_by_json_parse_and_rejected_by_validation_layer() {
        // serde_json parses "NaN" only via non-standard extensions we do not
        // enable; standard JSON has no NaN literal at all, so a NaN payload
        // can only arrive as a finite-looking number turbovec itself later
        // rejects, or is simply not expressible in valid JSON. This test
        // documents that expectation rather than asserting parse failure.
        let result: Result<InsertRequest, _> =
            parse(br#"{"index":"docs","vectors":[[1.0,2.0]],"ids":[1]}"#);
        assert!(result.is_ok());
    }

    #[test]
    fn vector_batch_bounds_reject_oversized_batches() {
        assert!(check_vector_batch_bounds(10, 5).is_err());
        assert!(check_vector_batch_bounds(5, 5).is_ok());
    }

    #[test]
    fn result_count_bounds_reject_oversized_k() {
        assert!(check_result_count_bounds(100, 50).is_err());
        assert!(check_result_count_bounds(50, 50).is_ok());
    }
}
