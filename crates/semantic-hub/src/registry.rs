//! Deterministic tool registry. Registration is static (no dynamic loading,
//! no filesystem discovery); this only tracks what the host process linked
//! in and started up explicitly.

use std::collections::BTreeMap;

use crate::descriptor::HubToolDescriptor;
use crate::ids::{HubApiVersion, HubToolId};
use crate::worker::{HubSupervisionPolicy, HubWorkerHealth, HubWorkerState};

/// Reasons registration is rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryError {
    DescriptorInvalid(crate::descriptor::DescriptorError),
    DuplicateToolId(HubToolId),
    ConflictingDescriptor { tool_id: HubToolId, reason: String },
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryError::DescriptorInvalid(e) => write!(f, "invalid descriptor: {e}"),
            RegistryError::DuplicateToolId(id) => write!(f, "tool already registered: {id}"),
            RegistryError::ConflictingDescriptor { tool_id, reason } => {
                write!(f, "conflicting registration for {tool_id}: {reason}")
            }
        }
    }
}

struct RegisteredTool {
    descriptor: HubToolDescriptor,
    health: HubWorkerHealth,
}

/// Static registry of admitted tools, keyed by `HubToolId`. Iteration order
/// is always the `HubToolId`'s `Ord` order (via `BTreeMap`), so `smc hub
/// tools` output is deterministic across runs and platforms.
pub struct ToolRegistry {
    hub_api_version: HubApiVersion,
    tools: BTreeMap<HubToolId, RegisteredTool>,
}

impl ToolRegistry {
    pub fn new(hub_api_version: HubApiVersion) -> Self {
        Self {
            hub_api_version,
            tools: BTreeMap::new(),
        }
    }

    /// Register one tool. Rejects: an invalid descriptor, an exact
    /// duplicate `HubToolId`, and -- distinctly -- a *different* descriptor
    /// re-using the same `HubToolId` with a changed `tool_version` (v0 has
    /// no supported tool-replacement lifecycle, so this is a conflict, not
    /// an upgrade).
    pub fn register(&mut self, descriptor: HubToolDescriptor) -> Result<(), RegistryError> {
        descriptor
            .validate(self.hub_api_version)
            .map_err(RegistryError::DescriptorInvalid)?;

        if let Some(existing) = self.tools.get(&descriptor.tool_id) {
            if existing.descriptor == descriptor {
                return Err(RegistryError::DuplicateToolId(descriptor.tool_id));
            }
            return Err(RegistryError::ConflictingDescriptor {
                tool_id: descriptor.tool_id,
                reason: "a different descriptor is already registered under this tool id \
                         (no supported replace-in-place lifecycle exists in Hub v0)"
                    .into(),
            });
        }

        let health = HubWorkerHealth::new(HubSupervisionPolicy::conservative_default());
        self.tools.insert(
            descriptor.tool_id.clone(),
            RegisteredTool { descriptor, health },
        );
        Ok(())
    }

    pub fn descriptor(&self, tool_id: &HubToolId) -> Option<&HubToolDescriptor> {
        self.tools.get(tool_id).map(|t| &t.descriptor)
    }

    pub fn worker_state(&self, tool_id: &HubToolId) -> Option<HubWorkerState> {
        self.tools.get(tool_id).map(|t| t.health.state())
    }

    pub fn health_mut(&mut self, tool_id: &HubToolId) -> Option<&mut HubWorkerHealth> {
        self.tools.get_mut(tool_id).map(|t| &mut t.health)
    }

    /// Deterministic listing in ascending `HubToolId` order.
    pub fn list(&self) -> impl Iterator<Item = &HubToolDescriptor> {
        self.tools.values().map(|t| &t.descriptor)
    }

    pub fn contains(&self, tool_id: &HubToolId) -> bool {
        self.tools.contains_key(tool_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::HubCapability;
    use crate::descriptor::HubOperationDescriptor;
    use crate::execution::{HubDeterminismClass, HubExecutionMode, HubTrustClass};
    use crate::ids::{HubOperationId, HubToolVersion};
    use crate::resource::HubResourceBudget;

    fn tool(id: &str, version: (u32, u32, u32)) -> HubToolDescriptor {
        HubToolDescriptor {
            tool_id: HubToolId::new(id).unwrap(),
            name: id.into(),
            tool_version: HubToolVersion::new(version.0, version.1, version.2),
            hub_api_version: HubApiVersion::CURRENT,
            execution_mode: HubExecutionMode::InProcess,
            trust_class: HubTrustClass::InProcessUnisolated,
            operations: vec![HubOperationDescriptor::new(
                HubOperationId::new("op.one").unwrap(),
                [HubCapability::CpuCompute],
                HubDeterminismClass::Unknown,
                false,
            )],
            resource_ceiling: HubResourceBudget::V0_CEILING,
            adapter_provenance: "test".into(),
        }
    }

    #[test]
    fn register_then_lookup_round_trips() {
        let mut reg = ToolRegistry::new(HubApiVersion::CURRENT);
        reg.register(tool("vector.turbovec", (0, 1, 0))).unwrap();
        assert!(reg.contains(&HubToolId::new("vector.turbovec").unwrap()));
        assert!(reg
            .descriptor(&HubToolId::new("vector.turbovec").unwrap())
            .is_some());
    }

    #[test]
    fn exact_duplicate_registration_is_rejected() {
        let mut reg = ToolRegistry::new(HubApiVersion::CURRENT);
        reg.register(tool("vector.turbovec", (0, 1, 0))).unwrap();
        let err = reg
            .register(tool("vector.turbovec", (0, 1, 0)))
            .unwrap_err();
        assert!(matches!(err, RegistryError::DuplicateToolId(_)));
    }

    #[test]
    fn conflicting_registration_under_same_id_is_rejected_distinctly() {
        let mut reg = ToolRegistry::new(HubApiVersion::CURRENT);
        reg.register(tool("vector.turbovec", (0, 1, 0))).unwrap();
        let err = reg
            .register(tool("vector.turbovec", (0, 2, 0)))
            .unwrap_err();
        assert!(matches!(err, RegistryError::ConflictingDescriptor { .. }));
    }

    #[test]
    fn unknown_tool_is_distinct_from_registered_tool_unknown_operation() {
        let mut reg = ToolRegistry::new(HubApiVersion::CURRENT);
        reg.register(tool("vector.turbovec", (0, 1, 0))).unwrap();

        assert!(reg
            .descriptor(&HubToolId::new("solver.z3").unwrap())
            .is_none());

        let known = reg
            .descriptor(&HubToolId::new("vector.turbovec").unwrap())
            .unwrap();
        assert!(known
            .operation(&HubOperationId::new("op.unknown").unwrap())
            .is_none());
    }

    #[test]
    fn listing_order_is_deterministic_ascending_by_tool_id() {
        let mut reg = ToolRegistry::new(HubApiVersion::CURRENT);
        reg.register(tool("vector.turbovec", (0, 1, 0))).unwrap();
        reg.register(tool("solver.z3", (0, 1, 0))).unwrap();
        reg.register(tool("analyzer.clang", (0, 1, 0))).unwrap();

        let ids: Vec<_> = reg.list().map(|d| d.tool_id.as_str().to_string()).collect();
        assert_eq!(ids, vec!["analyzer.clang", "solver.z3", "vector.turbovec"]);
    }

    #[test]
    fn freshly_registered_tool_starts_in_registered_state() {
        let mut reg = ToolRegistry::new(HubApiVersion::CURRENT);
        reg.register(tool("vector.turbovec", (0, 1, 0))).unwrap();
        assert_eq!(
            reg.worker_state(&HubToolId::new("vector.turbovec").unwrap()),
            Some(HubWorkerState::Registered)
        );
    }
}
