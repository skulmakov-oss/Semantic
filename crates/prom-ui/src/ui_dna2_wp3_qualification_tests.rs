//! WP3 Contract Qualification Tests
#![cfg(test)]
#![allow(dead_code, unused_imports)]

use crate::action_intent_map::*;
use crate::action_ir::*;
use crate::admission_contract::*;
use crate::binding_graph::*;
use crate::contract_primitives::{DiagnosticCode, DiagnosticCoordinate, SourceRef, StaticNodeId};
use crate::semantic_refs::*;
use crate::static_ir::*;
use alloc::vec::Vec;

fn dummy_static_doc() -> StaticUiDocument {
    let mut doc = StaticUiDocument::new(
        crate::contract_primitives::StaticDocumentId::new(1).unwrap(),
        crate::contract_primitives::Revision::new(1),
        crate::contract_primitives::Epoch::new(1),
    );
    doc.push_node(StaticUiNode::new(
        StaticNodeId::new(1).unwrap(),
        crate::role_dictionary::RoleId::new("test"),
        crate::contract_primitives::CollectionKey::new(1).unwrap(),
        None,
    ));
    doc.push_node(StaticUiNode::new(
        StaticNodeId::new(2).unwrap(),
        crate::role_dictionary::RoleId::new("test2"),
        crate::contract_primitives::CollectionKey::new(2).unwrap(),
        None,
    ));
    doc
}

// ============================================================================
// Semantic Refs
// ============================================================================
#[test]
fn test_semantic_refs_width() {
    assert_eq!(core::mem::size_of::<SemanticValueRef>(), 8);
    assert_eq!(core::mem::size_of::<SemanticActionRef>(), 8);
    assert_eq!(core::mem::size_of::<SemanticEvidenceRef>(), 8);
    assert_eq!(core::mem::size_of::<RevisionRef>(), 8);
    assert_eq!(core::mem::size_of::<EpochRef>(), 8);
    assert_eq!(core::mem::size_of::<ActorRef>(), 8);
    assert_eq!(core::mem::size_of::<SessionRef>(), 8);
    assert_eq!(core::mem::size_of::<ClientRef>(), 8);
    assert_eq!(core::mem::size_of::<CapabilityRef>(), 8);
}

#[test]
fn test_semantic_refs_deterministic_ordering() {
    let a = SemanticValueRef::new(1);
    let b = SemanticValueRef::new(2);
    assert!(a < b);
    assert_eq!(a, SemanticValueRef::new(1));
}

// ============================================================================
// Binding Graph Tests
// ============================================================================

fn make_bg_decl(id: u64, node: u64, deps: Vec<u64>) -> BindingDeclaration {
    BindingDeclaration {
        id: BindingId(id),
        target: BindingTarget {
            node: StaticNodeId::new(node).unwrap_or(StaticNodeId::new(1).unwrap()),
            slot: BindingSlot(1),
        },
        domain: BindingValueDomain::Scalar,
        dependencies: deps
            .into_iter()
            .map(|d| BindingDependency {
                target_binding: BindingId(d),
            })
            .collect(),
        collection_key: None,
        semantic_value: Some(SemanticValueRef::new(id)),
        source_ref: None,
    }
}

#[test]
fn test_bg_valid_graph_accepted() {
    let doc = dummy_static_doc();
    let b1 = make_bg_decl(1, 1, alloc::vec![]);
    let b2 = make_bg_decl(2, 2, alloc::vec![1]);
    let bg = BindingGraphDocument::build(alloc::vec![b1, b2], &doc);
    assert!(bg.is_ok());
}

#[test]
fn test_bg_duplicate_binding_id_rejected() {
    let doc = dummy_static_doc();
    let b1 = make_bg_decl(1, 1, alloc::vec![]);
    let b2 = make_bg_decl(1, 2, alloc::vec![]); // Dup ID
    let bg = BindingGraphDocument::build(alloc::vec![b1, b2], &doc);
    assert!(bg.is_err());
    let errs = bg.unwrap_err();
    assert!(errs
        .iter()
        .any(|d| d.coordinate.code().as_str() == "BG_DUP_BINDING_ID"));
}

#[test]
fn test_bg_missing_target_rejected() {
    let doc = dummy_static_doc();
    let b1 = make_bg_decl(1, 999, alloc::vec![]); // 999 not in doc
    let bg = BindingGraphDocument::build(alloc::vec![b1], &doc);
    assert!(bg.is_err());
    assert!(bg
        .unwrap_err()
        .iter()
        .any(|d| d.coordinate.code().as_str() == "BG_MISSING_TARGET"));
}

#[test]
fn test_bg_duplicate_target_slot_rejected() {
    let doc = dummy_static_doc();
    let b1 = make_bg_decl(1, 1, alloc::vec![]);
    let b2 = make_bg_decl(2, 1, alloc::vec![]); // Same target node/slot
    let bg = BindingGraphDocument::build(alloc::vec![b1, b2], &doc);
    assert!(bg.is_err());
    assert!(bg
        .unwrap_err()
        .iter()
        .any(|d| d.coordinate.code().as_str() == "BG_DUP_TARGET_SLOT"));
}

#[test]
fn test_bg_dangling_dependency_rejected() {
    let doc = dummy_static_doc();
    let b1 = make_bg_decl(1, 1, alloc::vec![99]);
    let bg = BindingGraphDocument::build(alloc::vec![b1], &doc);
    assert!(bg.is_err());
    assert!(bg
        .unwrap_err()
        .iter()
        .any(|d| d.coordinate.code().as_str() == "BG_DANGLING_DEP"));
}

#[test]
fn test_bg_duplicate_dependency_rejected() {
    let doc = dummy_static_doc();
    let b2 = make_bg_decl(2, 2, alloc::vec![]);
    let b1 = make_bg_decl(1, 1, alloc::vec![2, 2]);
    let bg = BindingGraphDocument::build(alloc::vec![b1, b2], &doc);
    assert!(bg.is_err());
    assert!(bg
        .unwrap_err()
        .iter()
        .any(|d| d.coordinate.code().as_str() == "BG_DUP_DEP"));
}

#[test]
fn test_bg_self_dependency_rejected() {
    let doc = dummy_static_doc();
    let b1 = make_bg_decl(1, 1, alloc::vec![1]);
    let bg = BindingGraphDocument::build(alloc::vec![b1], &doc);
    assert!(bg.is_err());
    assert!(bg
        .unwrap_err()
        .iter()
        .any(|d| d.coordinate.code().as_str() == "BG_SELF_DEP"));
}

#[test]
fn test_bg_multi_node_cycle_rejected() {
    let doc = dummy_static_doc();
    let b1 = make_bg_decl(1, 1, alloc::vec![2]);
    let mut b2 = make_bg_decl(2, 2, alloc::vec![1]);
    b2.target.slot = BindingSlot(2);
    let bg = BindingGraphDocument::build(alloc::vec![b1, b2], &doc);
    assert!(bg.is_err());
    assert!(bg
        .unwrap_err()
        .iter()
        .any(|d| d.coordinate.code().as_str() == "BG_CYCLE"));
}

#[test]
fn test_bg_missing_collection_key_rejected() {
    let doc = dummy_static_doc();
    let mut b1 = make_bg_decl(1, 1, alloc::vec![]);
    b1.domain = BindingValueDomain::Collection;
    let bg = BindingGraphDocument::build(alloc::vec![b1], &doc);
    assert!(bg.is_err());
    assert!(bg
        .unwrap_err()
        .iter()
        .any(|d| d.coordinate.code().as_str() == "BG_MISSING_COLL_KEY"));
}

#[test]
fn test_bg_invalid_collection_key_rejected() {
    let doc = dummy_static_doc();
    let mut b1 = make_bg_decl(1, 1, alloc::vec![]);
    b1.domain = BindingValueDomain::Scalar;
    b1.collection_key = Some(42);
    let bg = BindingGraphDocument::build(alloc::vec![b1], &doc);
    assert!(bg.is_err());
    assert!(bg
        .unwrap_err()
        .iter()
        .any(|d| d.coordinate.code().as_str() == "BG_INVALID_COLL_KEY"));
}

#[test]
fn test_bg_invalid_zero_semantic_ref_rejected() {
    let doc = dummy_static_doc();
    let mut b1 = make_bg_decl(1, 1, alloc::vec![]);
    b1.semantic_value = Some(SemanticValueRef::new(0));
    let bg = BindingGraphDocument::build(alloc::vec![b1], &doc);
    assert!(bg.is_err());
    assert!(bg
        .unwrap_err()
        .iter()
        .any(|d| d.coordinate.code().as_str() == "BG_ZERO_SEM_REF"));
}

#[test]
fn test_bg_storage_permutation_invariance() {
    let doc = dummy_static_doc();
    let b1 = make_bg_decl(1, 1, alloc::vec![]);
    let mut b2 = make_bg_decl(2, 2, alloc::vec![1]);
    b2.target.slot = BindingSlot(2);

    let bg_a = BindingGraphDocument::build(alloc::vec![b1.clone(), b2.clone()], &doc).unwrap();
    let bg_b = BindingGraphDocument::build(alloc::vec![b2.clone(), b1.clone()], &doc).unwrap();

    assert_eq!(bg_a.bindings, bg_b.bindings);
}

#[test]
fn test_bg_dependency_permutation_invariance() {
    let doc = dummy_static_doc();
    let b2 = make_bg_decl(2, 2, alloc::vec![]);
    let mut b3 = make_bg_decl(3, 1, alloc::vec![]);
    b3.target.slot = BindingSlot(2);

    let b1a = make_bg_decl(1, 1, alloc::vec![2, 3]);
    let b1b = make_bg_decl(1, 1, alloc::vec![3, 2]);

    let bg_a = BindingGraphDocument::build(alloc::vec![b2.clone(), b3.clone(), b1a], &doc).unwrap();
    let bg_b = BindingGraphDocument::build(alloc::vec![b2.clone(), b3.clone(), b1b], &doc).unwrap();

    assert_eq!(bg_a.bindings, bg_b.bindings);
}

#[test]
fn test_bg_normalization_idempotence() {
    let doc = dummy_static_doc();
    let b1 = make_bg_decl(1, 1, alloc::vec![]);
    let bg = BindingGraphDocument::build(alloc::vec![b1], &doc).unwrap();
    let bg_again = BindingGraphDocument::build(bg.bindings.clone(), &doc).unwrap();
    assert_eq!(bg.bindings, bg_again.bindings);
}

#[test]
fn test_bg_changed_semantics_distinction() {
    let doc = dummy_static_doc();
    let b1 = make_bg_decl(1, 1, alloc::vec![]);
    let mut b2 = make_bg_decl(1, 1, alloc::vec![]);
    b2.semantic_value = Some(SemanticValueRef::new(99)); // Diff semantics

    let bg_a = BindingGraphDocument::build(alloc::vec![b1], &doc).unwrap();
    let bg_b = BindingGraphDocument::build(alloc::vec![b2], &doc).unwrap();
    assert_ne!(bg_a.bindings, bg_b.bindings);
}

#[test]
fn test_bg_deep_graph_traversal_safety() {
    let doc = dummy_static_doc();
    let mut doc_large = doc.clone();
    for i in 1..=512 {
        doc_large.push_node(StaticUiNode::new(
            StaticNodeId::new(i).unwrap(),
            crate::role_dictionary::RoleId::new("test"),
            crate::contract_primitives::CollectionKey::new(i).unwrap(),
            None,
        ));
    }

    let mut bindings = alloc::vec::Vec::new();
    for i in 1..=512 {
        let deps = if i > 1 {
            alloc::vec![i - 1]
        } else {
            alloc::vec![]
        };
        bindings.push(make_bg_decl(i, i, deps));
    }

    let bg = BindingGraphDocument::build(bindings, &doc_large);
    assert!(bg.is_ok());
}

#[test]
fn test_bg_diagnostic_order_invariant() {
    let doc = dummy_static_doc();
    let b1 = make_bg_decl(1, 999, alloc::vec![]); // missing target
    let b2 = make_bg_decl(1, 1, alloc::vec![]); // dup id
    let b3 = make_bg_decl(2, 999, alloc::vec![99]); // missing target + dangling dep

    let errs1 = BindingGraphDocument::build(alloc::vec![b1.clone(), b2.clone(), b3.clone()], &doc)
        .unwrap_err();
    let errs2 = BindingGraphDocument::build(alloc::vec![b3.clone(), b1.clone(), b2.clone()], &doc)
        .unwrap_err();
    assert_eq!(errs1, errs2);
}

// ============================================================================
// Action IR Tests
// ============================================================================

fn make_air_route(id: u64, node: u64, order: u32) -> ActionRoute {
    ActionRoute {
        target: ActionTarget {
            node: StaticNodeId::new(node).unwrap_or(StaticNodeId::new(1).unwrap()),
            slot: ActionTargetSlot(1),
        },
        order: ActionRouteOrder(order),
        id: ActionRouteId(id),
        semantic_action: SemanticActionRef::new(id),
        source_ref: None,
        context_reqs: ActionContextRequirements {
            requires_actor: false,
            requires_session: false,
            requires_client: false,
            required_revision: None,
            required_epoch: None,
        },
        capability: None,
    }
}

#[test]
fn test_air_valid_route_accepted() {
    let doc = dummy_static_doc();
    let r1 = make_air_route(1, 1, 1);
    let r2 = make_air_route(2, 2, 1);
    let air = ActionIrDocument::build(alloc::vec![r1, r2], &doc);
    assert!(air.is_ok());
}

#[test]
fn test_air_duplicate_route_id_rejected() {
    let doc = dummy_static_doc();
    let r1 = make_air_route(1, 1, 1);
    let r2 = make_air_route(1, 2, 1); // Dup ID
    let air = ActionIrDocument::build(alloc::vec![r1, r2], &doc);
    assert!(air.is_err());
    assert!(air
        .unwrap_err()
        .iter()
        .any(|d| d.coordinate.code().as_str() == "AIR_DUP_ROUTE_ID"));
}

#[test]
fn test_air_missing_target_rejected() {
    let doc = dummy_static_doc();
    let r1 = make_air_route(1, 999, 1);
    let air = ActionIrDocument::build(alloc::vec![r1], &doc);
    assert!(air.is_err());
    assert!(air
        .unwrap_err()
        .iter()
        .any(|d| d.coordinate.code().as_str() == "AIR_MISSING_TARGET"));
}

#[test]
fn test_air_duplicate_route_order_same_target_slot_rejected() {
    let doc = dummy_static_doc();
    let r1 = make_air_route(1, 1, 1);
    let r2 = make_air_route(2, 1, 1); // Same node, slot, and order!
    let air = ActionIrDocument::build(alloc::vec![r1, r2], &doc);
    assert!(air.is_err());
    assert!(air
        .unwrap_err()
        .iter()
        .any(|d| d.coordinate.code().as_str() == "AIR_DUP_ROUTE_ORDER"));
}

#[test]
fn test_air_invalid_zero_semantic_action_ref_rejected() {
    let doc = dummy_static_doc();
    let mut r1 = make_air_route(1, 1, 1);
    r1.semantic_action = SemanticActionRef::new(0);
    let air = ActionIrDocument::build(alloc::vec![r1], &doc);
    assert!(air.is_err());
    assert!(air
        .unwrap_err()
        .iter()
        .any(|d| d.coordinate.code().as_str() == "AIR_ZERO_ACTION_REF"));
}

#[test]
fn test_air_storage_permutation_invariance() {
    let doc = dummy_static_doc();
    let r1 = make_air_route(1, 1, 1);
    let r2 = make_air_route(2, 2, 2);

    let a = ActionIrDocument::build(alloc::vec![r1.clone(), r2.clone()], &doc).unwrap();
    let b = ActionIrDocument::build(alloc::vec![r2.clone(), r1.clone()], &doc).unwrap();
    assert_eq!(a.routes, b.routes);
}

#[test]
fn test_air_route_order_semantic_distinction() {
    let doc = dummy_static_doc();
    let r1 = make_air_route(1, 1, 1);
    let r2 = make_air_route(1, 1, 2);

    let a = ActionIrDocument::build(alloc::vec![r1], &doc).unwrap();
    let b = ActionIrDocument::build(alloc::vec![r2], &doc).unwrap();
    assert_ne!(a.routes, b.routes);
}

#[test]
fn test_air_normalization_idempotence() {
    let doc = dummy_static_doc();
    let r1 = make_air_route(1, 1, 1);
    let a = ActionIrDocument::build(alloc::vec![r1], &doc).unwrap();
    let b = ActionIrDocument::build(a.routes.clone(), &doc).unwrap();
    assert_eq!(a.routes, b.routes);
}

#[test]
fn test_air_diagnostic_order_invariant() {
    let doc = dummy_static_doc();
    let r1 = make_air_route(1, 999, 1); // missing target
    let r2 = make_air_route(1, 1, 1); // dup id
    let r3 = make_air_route(2, 999, 1); // missing target

    let errs1 =
        ActionIrDocument::build(alloc::vec![r1.clone(), r2.clone(), r3.clone()], &doc).unwrap_err();
    let errs2 =
        ActionIrDocument::build(alloc::vec![r3.clone(), r1.clone(), r2.clone()], &doc).unwrap_err();
    assert_eq!(errs1, errs2);
}

// ============================================================================
// ActionIntent Mapper Tests
// ============================================================================

fn make_test_context() -> ActionInvocationContext {
    ActionInvocationContext {
        source_node: StaticNodeId::new(1).unwrap(),
        target_node: StaticNodeId::new(2).unwrap(),
        actor: Some(ActorRef::new(1)),
        session: Some(SessionRef::new(1)),
        client: Some(ClientRef::new(1)),
        observed_revision: Some(RevisionRef::new(1)),
        observed_epoch: Some(EpochRef::new(1)),
    }
}

#[test]
fn test_mapper_valid_complete_context() {
    let mut r = make_air_route(1, 1, 1);
    r.context_reqs.requires_actor = true;
    let ctx = make_test_context();
    let intent = map_action_intent(&r, &ctx);
    assert!(intent.is_ok());
}

#[test]
fn test_mapper_missing_actor() {
    let mut r = make_air_route(1, 1, 1);
    r.context_reqs.requires_actor = true;
    let mut ctx = make_test_context();
    ctx.actor = None;
    let intent = map_action_intent(&r, &ctx);
    assert!(intent.is_err());
    assert_eq!(
        intent.unwrap_err().coordinate.code().as_str(),
        "MISSING_ACTOR"
    );
}

#[test]
fn test_mapper_missing_session() {
    let mut r = make_air_route(1, 1, 1);
    r.context_reqs.requires_session = true;
    let mut ctx = make_test_context();
    ctx.session = None;
    let intent = map_action_intent(&r, &ctx);
    assert!(intent.is_err());
    assert_eq!(
        intent.unwrap_err().coordinate.code().as_str(),
        "MISSING_SESSION"
    );
}

#[test]
fn test_mapper_missing_client() {
    let mut r = make_air_route(1, 1, 1);
    r.context_reqs.requires_client = true;
    let mut ctx = make_test_context();
    ctx.client = None;
    let intent = map_action_intent(&r, &ctx);
    assert!(intent.is_err());
    assert_eq!(
        intent.unwrap_err().coordinate.code().as_str(),
        "MISSING_CLIENT"
    );
}

#[test]
fn test_mapper_missing_revision() {
    let mut r = make_air_route(1, 1, 1);
    r.context_reqs.required_revision = Some(RevisionRef::new(1));
    let mut ctx = make_test_context();
    ctx.observed_revision = None;
    let intent = map_action_intent(&r, &ctx);
    assert!(intent.is_err());
    assert_eq!(
        intent.unwrap_err().coordinate.code().as_str(),
        "MISSING_REVISION"
    );
}

#[test]
fn test_mapper_missing_epoch() {
    let mut r = make_air_route(1, 1, 1);
    r.context_reqs.required_epoch = Some(EpochRef::new(1));
    let mut ctx = make_test_context();
    ctx.observed_epoch = None;
    let intent = map_action_intent(&r, &ctx);
    assert!(intent.is_err());
    assert_eq!(
        intent.unwrap_err().coordinate.code().as_str(),
        "MISSING_EPOCH"
    );
}

#[test]
fn test_mapper_deterministic_mapping() {
    let mut r = make_air_route(1, 1, 1);
    r.context_reqs.requires_actor = true;
    let ctx = make_test_context();
    let intent1 = map_action_intent(&r, &ctx).unwrap();
    let intent2 = map_action_intent(&r, &ctx).unwrap();
    assert_eq!(intent1.route_id, intent2.route_id);
    assert_eq!(intent1.semantic_action, intent2.semantic_action);
    assert_eq!(intent1.actor, intent2.actor);
}

#[test]
fn test_mapper_deterministic_diagnostic() {
    let mut r = make_air_route(1, 1, 1);
    r.context_reqs.requires_actor = true;
    let mut ctx = make_test_context();
    ctx.actor = None;
    let err1 = map_action_intent(&r, &ctx).unwrap_err();
    let err2 = map_action_intent(&r, &ctx).unwrap_err();
    assert_eq!(err1.coordinate, err2.coordinate);
}
