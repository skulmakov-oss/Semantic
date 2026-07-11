//! Deterministic mapping of UI actions to transport intents.
#![allow(dead_code)]

use crate::action_ir::ActionRoute;
use crate::admission_contract::{ActionIntent, ActionIntentDiagnostic, ActionInvocationContext};
use crate::contract_primitives::{DiagnosticCode, DiagnosticCoordinate};

pub(crate) fn map_action_intent(
    route: &ActionRoute,
    context: &ActionInvocationContext,
) -> Result<ActionIntent, ActionIntentDiagnostic> {
    if route.context_reqs.requires_actor && context.actor.is_none() {
        return Err(ActionIntentDiagnostic {
            coordinate: DiagnosticCoordinate::new(None, DiagnosticCode::new("MISSING_ACTOR"), 0, 0),
        });
    }
    if route.context_reqs.requires_session && context.session.is_none() {
        return Err(ActionIntentDiagnostic {
            coordinate: DiagnosticCoordinate::new(
                None,
                DiagnosticCode::new("MISSING_SESSION"),
                0,
                0,
            ),
        });
    }
    if route.context_reqs.requires_client && context.client.is_none() {
        return Err(ActionIntentDiagnostic {
            coordinate: DiagnosticCoordinate::new(
                None,
                DiagnosticCode::new("MISSING_CLIENT"),
                0,
                0,
            ),
        });
    }
    if route.context_reqs.required_revision.is_some() && context.observed_revision.is_none() {
        return Err(ActionIntentDiagnostic {
            coordinate: DiagnosticCoordinate::new(
                None,
                DiagnosticCode::new("MISSING_REVISION"),
                0,
                0,
            ),
        });
    }
    if route.context_reqs.required_epoch.is_some() && context.observed_epoch.is_none() {
        return Err(ActionIntentDiagnostic {
            coordinate: DiagnosticCoordinate::new(None, DiagnosticCode::new("MISSING_EPOCH"), 0, 0),
        });
    }

    Ok(ActionIntent {
        route_id: route.id,
        semantic_action: route.semantic_action,
        source_node: context.source_node,
        target_node: context.target_node,
        actor: context.actor,
        session: context.session,
        client: context.client,
        observed_revision: context.observed_revision,
        observed_epoch: context.observed_epoch,
        route_required_revision: route.context_reqs.required_revision,
        route_required_epoch: route.context_reqs.required_epoch,
        capability: route.capability,
    })
}
