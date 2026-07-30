//! Multidimensional resource budget and usage tracking.
//!
//! Mirrors `sm_runtime_core::RuntimeQuotas`'s non-panicking
//! `exceed(kind, used) -> Option<QuotaExceeded>` pattern, for a different
//! resource domain (external-tool wall time/memory/bytes rather than VM
//! step/register/frame counts). All arithmetic is checked; overflow rejects
//! safely instead of wrapping or panicking.

use std::fmt;
use std::time::Duration;

/// One budgeted resource dimension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HubResourceKind {
    WallTimeMillis,
    MemoryBytes,
    InputBytes,
    OutputBytes,
    IndexItemCount,
    VectorDimensions,
    ResultCount,
    QueueDepth,
    ConcurrentRequests,
    StorageReadBytes,
    StorageWriteBytes,
    AuditBytes,

    /// Session-scoped dimensions: a *cumulative*, whole-session ceiling,
    /// distinct from the per-request dimensions above (e.g.
    /// `InputBytes`). Only ever produced by session-level admission
    /// checks (see `crate::session`), never by the single-request
    /// `HubResourceBudget::first_violation` check.
    SessionRequestCount,
    SessionInputBytes,
    SessionOutputBytes,
    SessionWallTimeMillis,
}

impl HubResourceKind {
    pub const fn as_str(&self) -> &'static str {
        match self {
            HubResourceKind::WallTimeMillis => "WallTimeMillis",
            HubResourceKind::MemoryBytes => "MemoryBytes",
            HubResourceKind::InputBytes => "InputBytes",
            HubResourceKind::OutputBytes => "OutputBytes",
            HubResourceKind::IndexItemCount => "IndexItemCount",
            HubResourceKind::VectorDimensions => "VectorDimensions",
            HubResourceKind::ResultCount => "ResultCount",
            HubResourceKind::QueueDepth => "QueueDepth",
            HubResourceKind::ConcurrentRequests => "ConcurrentRequests",
            HubResourceKind::StorageReadBytes => "StorageReadBytes",
            HubResourceKind::StorageWriteBytes => "StorageWriteBytes",
            HubResourceKind::AuditBytes => "AuditBytes",
            HubResourceKind::SessionRequestCount => "SessionRequestCount",
            HubResourceKind::SessionInputBytes => "SessionInputBytes",
            HubResourceKind::SessionOutputBytes => "SessionOutputBytes",
            HubResourceKind::SessionWallTimeMillis => "SessionWallTimeMillis",
        }
    }

    /// Whether this dimension is hard-enforced in Hub v0 (rejected before or
    /// during dispatch) as opposed to only observed/recorded as advisory
    /// evidence. Kept explicit so docs/audit never overstate enforcement.
    pub const fn is_hard_enforced_v0(&self) -> bool {
        matches!(
            self,
            HubResourceKind::WallTimeMillis
                | HubResourceKind::InputBytes
                | HubResourceKind::OutputBytes
                | HubResourceKind::IndexItemCount
                | HubResourceKind::VectorDimensions
                | HubResourceKind::ResultCount
                | HubResourceKind::QueueDepth
                | HubResourceKind::ConcurrentRequests
                | HubResourceKind::SessionRequestCount
                | HubResourceKind::SessionInputBytes
                | HubResourceKind::SessionOutputBytes
                | HubResourceKind::SessionWallTimeMillis
        )
    }
}

impl fmt::Display for HubResourceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A single admitted invocation's immutable resource budget. Once built via
/// [`HubResourceBudgetBuilder`], no field can be widened -- the adapter
/// receives only `&HubResourceBudget` (shared reference), never a mutable
/// one, so it structurally cannot raise its own limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HubResourceBudget {
    pub wall_time_millis: u64,
    pub memory_bytes: u64,
    pub input_bytes: u64,
    pub output_bytes: u64,
    pub index_item_count: u64,
    pub vector_dimensions: u32,
    pub result_count: u32,
    pub queue_depth: u32,
    pub concurrent_requests: u32,
    pub storage_read_bytes: u64,
    pub storage_write_bytes: u64,
    pub audit_bytes: u64,
}

impl HubResourceBudget {
    /// A conservative default budget suitable for CLI-driven TurboVec
    /// invocations. Callers may request a smaller budget; Hub v0 does not
    /// allow a request to exceed these ceilings.
    pub const V0_CEILING: HubResourceBudget = HubResourceBudget {
        wall_time_millis: 30_000,
        memory_bytes: 512 * 1024 * 1024,
        input_bytes: 64 * 1024 * 1024,
        output_bytes: 16 * 1024 * 1024,
        index_item_count: 1_000_000,
        vector_dimensions: 4096,
        result_count: 10_000,
        queue_depth: 256,
        concurrent_requests: 32,
        storage_read_bytes: 256 * 1024 * 1024,
        storage_write_bytes: 256 * 1024 * 1024,
        audit_bytes: 1024 * 1024,
    };

    /// True if `self` (a requested budget) does not exceed `ceiling` on any
    /// dimension.
    pub fn within(&self, ceiling: &HubResourceBudget) -> bool {
        self.first_violation(ceiling).is_none()
    }

    /// Returns the first dimension (in a fixed, deterministic field order)
    /// on which `self` exceeds `ceiling`, with the real limit and requested
    /// value -- never a placeholder -- so admission rejections carry
    /// genuine evidence instead of a fabricated fault payload.
    pub fn first_violation(&self, ceiling: &HubResourceBudget) -> Option<HubBudgetExceeded> {
        macro_rules! check {
            ($field:ident, $kind:expr) => {
                if self.$field as u64 > ceiling.$field as u64 {
                    return Some(HubBudgetExceeded {
                        kind: $kind,
                        limit: ceiling.$field as u64,
                        attempted: self.$field as u64,
                    });
                }
            };
        }
        check!(wall_time_millis, HubResourceKind::WallTimeMillis);
        check!(memory_bytes, HubResourceKind::MemoryBytes);
        check!(input_bytes, HubResourceKind::InputBytes);
        check!(output_bytes, HubResourceKind::OutputBytes);
        check!(index_item_count, HubResourceKind::IndexItemCount);
        check!(vector_dimensions, HubResourceKind::VectorDimensions);
        check!(result_count, HubResourceKind::ResultCount);
        check!(queue_depth, HubResourceKind::QueueDepth);
        check!(concurrent_requests, HubResourceKind::ConcurrentRequests);
        check!(storage_read_bytes, HubResourceKind::StorageReadBytes);
        check!(storage_write_bytes, HubResourceKind::StorageWriteBytes);
        check!(audit_bytes, HubResourceKind::AuditBytes);
        None
    }

    pub const fn wall_time(&self) -> Duration {
        Duration::from_millis(self.wall_time_millis)
    }

    /// Deterministic canonical text of every field, in a fixed order --
    /// the single shared encoding used both by the audit trail's packed
    /// budget column and to derive a stable provenance digest, so the two
    /// can never silently drift apart.
    pub fn canonical_text(&self) -> String {
        [
            self.wall_time_millis.to_string(),
            self.memory_bytes.to_string(),
            self.input_bytes.to_string(),
            self.output_bytes.to_string(),
            self.index_item_count.to_string(),
            self.vector_dimensions.to_string(),
            self.result_count.to_string(),
            self.queue_depth.to_string(),
            self.concurrent_requests.to_string(),
            self.storage_read_bytes.to_string(),
            self.storage_write_bytes.to_string(),
            self.audit_bytes.to_string(),
        ]
        .join(",")
    }
}

/// Observed usage for one invocation, recorded alongside the budget in the
/// audit record. `None` on a field means "not measured", never a fabricated
/// zero -- Hub v0 does not claim exact memory accounting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HubResourceUsage {
    pub wall_time_millis: Option<u64>,
    pub peak_memory_bytes: Option<u64>,
    pub input_bytes: Option<u64>,
    pub output_bytes: Option<u64>,
    pub index_item_count: Option<u64>,
    pub result_count: Option<u32>,
    pub storage_read_bytes: Option<u64>,
    pub storage_write_bytes: Option<u64>,
    pub audit_bytes: Option<u64>,
}

/// One budget dimension was exceeded (or would overflow if permitted).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HubBudgetExceeded {
    pub kind: HubResourceKind,
    pub limit: u64,
    pub attempted: u64,
}

impl fmt::Display for HubBudgetExceeded {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "resource budget exceeded: {} limit={} attempted={}",
            self.kind, self.limit, self.attempted
        )
    }
}

/// Checked-arithmetic budget check: returns the exceeded-by amount without
/// panicking or wrapping on overflow. `used + delta` overflowing u64 is
/// itself treated as an excess (an impossible request), never wrapped.
pub fn check_budget(
    kind: HubResourceKind,
    used: u64,
    delta: u64,
    limit: u64,
) -> Result<u64, HubBudgetExceeded> {
    let total = used.checked_add(delta).ok_or(HubBudgetExceeded {
        kind,
        limit,
        attempted: u64::MAX,
    })?;
    if total > limit {
        Err(HubBudgetExceeded {
            kind,
            limit,
            attempted: total,
        })
    } else {
        Ok(total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requested_budget_within_ceiling_is_accepted() {
        let requested = HubResourceBudget {
            wall_time_millis: 1_000,
            ..HubResourceBudget::V0_CEILING
        };
        assert!(requested.within(&HubResourceBudget::V0_CEILING));
    }

    #[test]
    fn requested_budget_exceeding_any_single_dimension_is_rejected() {
        let requested = HubResourceBudget {
            vector_dimensions: HubResourceBudget::V0_CEILING.vector_dimensions + 1,
            ..HubResourceBudget::V0_CEILING
        };
        assert!(!requested.within(&HubResourceBudget::V0_CEILING));
    }

    #[test]
    fn check_budget_accepts_usage_under_limit() {
        assert_eq!(
            check_budget(HubResourceKind::OutputBytes, 10, 5, 100),
            Ok(15)
        );
    }

    #[test]
    fn check_budget_rejects_usage_over_limit() {
        let err = check_budget(HubResourceKind::OutputBytes, 90, 20, 100).unwrap_err();
        assert_eq!(err.kind, HubResourceKind::OutputBytes);
        assert_eq!(err.limit, 100);
        assert_eq!(err.attempted, 110);
    }

    #[test]
    fn check_budget_rejects_overflow_instead_of_wrapping() {
        let err =
            check_budget(HubResourceKind::MemoryBytes, u64::MAX - 1, 10, u64::MAX).unwrap_err();
        assert_eq!(err.attempted, u64::MAX);
    }

    #[test]
    fn hard_enforced_v0_dimensions_match_documented_policy() {
        assert!(HubResourceKind::WallTimeMillis.is_hard_enforced_v0());
        assert!(HubResourceKind::ResultCount.is_hard_enforced_v0());
        assert!(!HubResourceKind::MemoryBytes.is_hard_enforced_v0());
    }
}
