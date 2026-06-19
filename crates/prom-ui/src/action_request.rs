//! Semantic UI Action Request metadata.
//!
//! This module defines the formal request that the UI layer produces
//! when an interaction implies a state mutation or host capability invocation.
//! It does not execute the action.

use crate::interaction::{ElementId, InteractionIntentId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ActionRequestId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ActionKind {
    // These variants will be expanded as needed, but serve as placeholders
    // for actual semantic actions (e.g. "Toggle", "SubmitForm", "CloseWindow").
    GenericSemanticAction,
    Unknown,
}

/// A purely descriptive record that the UI intends to perform an action.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ActionRequestDescriptor {
    pub id: ActionRequestId,
    pub kind: ActionKind,
    pub source_element: Option<ElementId>,
    pub cause_intent: InteractionIntentId,
}

impl ActionRequestDescriptor {
    pub const fn new(
        id: ActionRequestId,
        kind: ActionKind,
        source_element: Option<ElementId>,
        cause_intent: InteractionIntentId,
    ) -> Self {
        Self {
            id,
            kind,
            source_element,
            cause_intent,
        }
    }
}
