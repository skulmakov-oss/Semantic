//! Action admission transport structures.
#![allow(dead_code)]

use core::cmp::Ordering;

use crate::action_ir::ActionRouteId;
use crate::contract_primitives::DiagnosticCoordinate;
use crate::contract_primitives::StaticNodeId;
use crate::semantic_refs::{
    ActorRef, CapabilityRef, ClientRef, EpochRef, RevisionRef, SemanticActionRef, SessionRef,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ActionInvocationContext {
    pub(crate) source_node: StaticNodeId,
    pub(crate) target_node: StaticNodeId,
    pub(crate) actor: Option<ActorRef>,
    pub(crate) session: Option<SessionRef>,
    pub(crate) client: Option<ClientRef>,
    pub(crate) observed_revision: Option<RevisionRef>,
    pub(crate) observed_epoch: Option<EpochRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ActionIntent {
    pub(crate) route_id: ActionRouteId,
    pub(crate) semantic_action: SemanticActionRef,
    pub(crate) source_node: StaticNodeId,
    pub(crate) target_node: StaticNodeId,
    pub(crate) actor: Option<ActorRef>,
    pub(crate) session: Option<SessionRef>,
    pub(crate) client: Option<ClientRef>,
    pub(crate) observed_revision: Option<RevisionRef>,
    pub(crate) observed_epoch: Option<EpochRef>,
    pub(crate) route_required_revision: Option<RevisionRef>,
    pub(crate) route_required_epoch: Option<EpochRef>,
    pub(crate) capability: Option<CapabilityRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ActionIntentDiagnostic {
    pub(crate) coordinate: DiagnosticCoordinate,
}

impl Ord for ActionIntentDiagnostic {
    fn cmp(&self, other: &Self) -> Ordering {
        self.coordinate.cmp(&other.coordinate)
    }
}

impl PartialOrd for ActionIntentDiagnostic {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
