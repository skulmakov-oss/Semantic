//! Minimal UI DNA v2 Role Dictionary.
//!
//! This module is independent from Projection Source, Static UI IR, runtime,
//! renderer, backend, and admission layers.
#![allow(dead_code)]

use crate::contract_primitives::{ContractVersion, SchemaVersion};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct RoleId(&'static str);

impl RoleId {
    pub(crate) const fn new(raw: &'static str) -> Self {
        Self(raw)
    }

    pub(crate) const fn as_str(self) -> &'static str {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum BuiltInRole {
    Root,
    Surface,
    NumericReadout,
    DangerAction,
    EvidencePanel,
    RecoveryOutlet,
    Text,
    Fragment,
}

impl BuiltInRole {
    pub(crate) const fn id(self) -> RoleId {
        match self {
            Self::Root => RoleId::new("root"),
            Self::Surface => RoleId::new("surface"),
            Self::NumericReadout => RoleId::new("numeric_readout"),
            Self::DangerAction => RoleId::new("danger_action"),
            Self::EvidencePanel => RoleId::new("evidence_panel"),
            Self::RecoveryOutlet => RoleId::new("recovery_outlet"),
            Self::Text => RoleId::new("text"),
            Self::Fragment => RoleId::new("fragment"),
        }
    }
}

const BUILT_IN_ROLES: [BuiltInRole; 8] = [
    BuiltInRole::DangerAction,
    BuiltInRole::EvidencePanel,
    BuiltInRole::Fragment,
    BuiltInRole::NumericReadout,
    BuiltInRole::RecoveryOutlet,
    BuiltInRole::Root,
    BuiltInRole::Surface,
    BuiltInRole::Text,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct RoleDictionary {
    schema_version: SchemaVersion,
    contract_version: ContractVersion,
    roles: &'static [BuiltInRole],
}

impl RoleDictionary {
    pub(crate) const fn current() -> Self {
        Self {
            schema_version: SchemaVersion::CURRENT,
            contract_version: ContractVersion::CURRENT,
            roles: &BUILT_IN_ROLES,
        }
    }

    pub(crate) const fn schema_version(self) -> SchemaVersion {
        self.schema_version
    }

    pub(crate) const fn contract_version(self) -> ContractVersion {
        self.contract_version
    }

    pub(crate) const fn roles(self) -> &'static [BuiltInRole] {
        self.roles
    }

    pub(crate) fn resolve_name(self, raw: &str) -> Option<RoleId> {
        self.roles
            .iter()
            .map(|built_in| built_in.id())
            .find(|role| role.as_str() == raw)
    }

    pub(crate) fn validate(self, role: RoleId) -> Result<BuiltInRole, RoleDictionaryError> {
        for built_in in self.roles {
            if built_in.id() == role {
                return Ok(*built_in);
            }
        }
        Err(RoleDictionaryError::UnknownRole(role))
    }

    pub(crate) fn contains(self, role: RoleId) -> bool {
        self.validate(role).is_ok()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum RoleDictionaryError {
    UnknownRole(RoleId),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_dictionary_validates_built_in_roles() {
        let dictionary = RoleDictionary::current();

        assert_eq!(
            dictionary.validate(RoleId::new("numeric_readout")),
            Ok(BuiltInRole::NumericReadout)
        );
        assert!(dictionary.contains(RoleId::new("danger_action")));
    }

    #[test]
    fn role_dictionary_rejects_unknown_roles() {
        let dictionary = RoleDictionary::current();

        assert_eq!(
            dictionary.validate(RoleId::new("renderer_button")),
            Err(RoleDictionaryError::UnknownRole(RoleId::new(
                "renderer_button"
            )))
        );
    }

    #[test]
    fn role_dictionary_resolves_borrowed_names_without_allocation() {
        let dictionary = RoleDictionary::current();

        assert_eq!(
            dictionary.resolve_name("root"),
            Some(BuiltInRole::Root.id())
        );
        assert_eq!(
            dictionary.resolve_name("numeric_readout"),
            Some(BuiltInRole::NumericReadout.id())
        );
        assert_eq!(dictionary.resolve_name("renderer_button"), None);
        assert_eq!(dictionary.resolve_name("Root"), None);
    }

    #[test]
    fn role_dictionary_versions_are_stable() {
        let dictionary = RoleDictionary::current();

        assert_eq!(dictionary.schema_version().raw(), 1);
        assert_eq!(dictionary.contract_version().raw(), 1);
    }

    #[test]
    fn role_dictionary_roles_are_canonically_ordered() {
        let roles = RoleDictionary::current().roles();

        for pair in roles.windows(2) {
            assert!(pair[0].id() < pair[1].id());
        }
    }

    #[test]
    fn role_dictionary_does_not_import_renderer_taxonomy() {
        let dictionary = RoleDictionary::current();

        assert!(!dictionary.contains(RoleId::new("button")));
        assert!(!dictionary.contains(RoleId::new("wgpu_surface")));
    }
}
