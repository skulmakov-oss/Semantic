//! Tool and operation descriptors: the static, registry-validated shape of
//! what a tool claims to provide.

use std::collections::BTreeSet;

use crate::capability::HubCapability;
use crate::execution::{HubDeterminismClass, HubExecutionMode, HubTrustClass};
use crate::ids::{HubApiVersion, HubOperationId, HubToolId, HubToolVersion};
use crate::resource::HubResourceBudget;

/// Reasons a descriptor fails validation at registration time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DescriptorError {
    NoOperations,
    DuplicateOperation(HubOperationId),
    ApiVersionIncompatible {
        tool: HubApiVersion,
        hub: HubApiVersion,
    },
    OperationRequiresSensitiveCapability {
        operation: HubOperationId,
        capability: HubCapability,
    },
}

impl std::fmt::Display for DescriptorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DescriptorError::NoOperations => write!(f, "tool descriptor declares no operations"),
            DescriptorError::DuplicateOperation(id) => {
                write!(f, "duplicate operation id in descriptor: {id}")
            }
            DescriptorError::ApiVersionIncompatible { tool, hub } => write!(
                f,
                "tool descriptor requires hub api {tool}, running hub api {hub}"
            ),
            DescriptorError::OperationRequiresSensitiveCapability {
                operation,
                capability,
            } => write!(
                f,
                "operation {operation} declares sensitive capability {capability}, which is denied by default in Hub v0"
            ),
        }
    }
}

/// One bounded operation a tool provides.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HubOperationDescriptor {
    pub operation_id: HubOperationId,
    pub required_capabilities: BTreeSet<HubCapability>,
    pub determinism: HubDeterminismClass,
    pub mutates_tool_state: bool,
}

impl HubOperationDescriptor {
    pub fn new(
        operation_id: HubOperationId,
        required_capabilities: impl IntoIterator<Item = HubCapability>,
        determinism: HubDeterminismClass,
        mutates_tool_state: bool,
    ) -> Self {
        Self {
            operation_id,
            required_capabilities: required_capabilities.into_iter().collect(),
            determinism,
            mutates_tool_state,
        }
    }
}

/// The static, validated shape of one registered tool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HubToolDescriptor {
    pub tool_id: HubToolId,
    pub name: String,
    pub tool_version: HubToolVersion,
    pub hub_api_version: HubApiVersion,
    pub execution_mode: HubExecutionMode,
    pub trust_class: HubTrustClass,
    pub operations: Vec<HubOperationDescriptor>,
    pub resource_ceiling: HubResourceBudget,
    pub adapter_provenance: String,
}

impl HubToolDescriptor {
    /// Validate internal consistency: at least one operation, no duplicate
    /// operation ids, compatible Hub API version, and no operation may
    /// require a capability that is sensitive (denied by default in v0) --
    /// such a descriptor could never be admitted, so registration rejects it
    /// up front rather than accepting a tool nothing can ever invoke.
    pub fn validate(&self, running_hub_api: HubApiVersion) -> Result<(), DescriptorError> {
        if self.operations.is_empty() {
            return Err(DescriptorError::NoOperations);
        }
        let mut seen = BTreeSet::new();
        for op in &self.operations {
            if !seen.insert(op.operation_id.clone()) {
                return Err(DescriptorError::DuplicateOperation(op.operation_id.clone()));
            }
            for cap in &op.required_capabilities {
                if cap.is_sensitive() {
                    return Err(DescriptorError::OperationRequiresSensitiveCapability {
                        operation: op.operation_id.clone(),
                        capability: *cap,
                    });
                }
            }
        }
        if !running_hub_api.is_compatible_with(self.hub_api_version) {
            return Err(DescriptorError::ApiVersionIncompatible {
                tool: self.hub_api_version,
                hub: running_hub_api,
            });
        }
        Ok(())
    }

    pub fn operation(&self, id: &HubOperationId) -> Option<&HubOperationDescriptor> {
        self.operations.iter().find(|op| &op.operation_id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_descriptor() -> HubToolDescriptor {
        HubToolDescriptor {
            tool_id: HubToolId::new("vector.turbovec").unwrap(),
            name: "TurboVec".into(),
            tool_version: HubToolVersion::new(0, 1, 0),
            hub_api_version: HubApiVersion::CURRENT,
            execution_mode: HubExecutionMode::InProcess,
            trust_class: HubTrustClass::InProcessUnisolated,
            operations: vec![HubOperationDescriptor::new(
                HubOperationId::new("vector.search").unwrap(),
                [HubCapability::VectorSearch],
                HubDeterminismClass::Unknown,
                false,
            )],
            resource_ceiling: HubResourceBudget::V0_CEILING,
            adapter_provenance: "semantic-hub-turbovec 0.1.0".into(),
        }
    }

    #[test]
    fn valid_descriptor_passes() {
        assert!(sample_descriptor().validate(HubApiVersion::CURRENT).is_ok());
    }

    #[test]
    fn descriptor_with_no_operations_is_rejected() {
        let mut d = sample_descriptor();
        d.operations.clear();
        assert_eq!(
            d.validate(HubApiVersion::CURRENT).unwrap_err(),
            DescriptorError::NoOperations
        );
    }

    #[test]
    fn descriptor_with_duplicate_operation_ids_is_rejected() {
        let mut d = sample_descriptor();
        let dup = d.operations[0].clone();
        d.operations.push(dup);
        assert!(matches!(
            d.validate(HubApiVersion::CURRENT).unwrap_err(),
            DescriptorError::DuplicateOperation(_)
        ));
    }

    #[test]
    fn descriptor_requiring_newer_hub_api_minor_is_rejected() {
        let mut d = sample_descriptor();
        d.hub_api_version = HubApiVersion::new(0, 99);
        assert!(matches!(
            d.validate(HubApiVersion::CURRENT).unwrap_err(),
            DescriptorError::ApiVersionIncompatible { .. }
        ));
    }

    #[test]
    fn descriptor_requiring_sensitive_capability_is_rejected() {
        let mut d = sample_descriptor();
        d.operations[0]
            .required_capabilities
            .insert(HubCapability::NetworkAccess);
        assert!(matches!(
            d.validate(HubApiVersion::CURRENT).unwrap_err(),
            DescriptorError::OperationRequiresSensitiveCapability { .. }
        ));
    }

    #[test]
    fn operation_lookup_finds_registered_operation_and_none_for_unknown() {
        let d = sample_descriptor();
        assert!(d
            .operation(&HubOperationId::new("vector.search").unwrap())
            .is_some());
        assert!(d
            .operation(&HubOperationId::new("vector.remove").unwrap())
            .is_none());
    }
}
