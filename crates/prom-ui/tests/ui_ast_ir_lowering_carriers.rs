use prom_ui::lowering::{lower_ast_to_ir, UiLoweringConfig, UiLoweringDiagnosticKind};
use prom_ui::model::{UiAst, UiAstNode, UiAstNodeId, UiAstNodeKind, UiIrNodeId, UiIrNodeKind};
use prom_ui::projection::{project_ir_to_projection, UiProjectedNodeKind};
use prom_ui::renderer::{render_projection_to_model, UiRenderMarker};

#[test]
fn ast_attribute_lowers_to_ir_property() {
    let mut ast = UiAst::new();
    let root_id = UiAstNodeId::new(1);
    let attr_id = UiAstNodeId::new(2);
    let mut root_node = UiAstNode::new(root_id, UiAstNodeKind::Root);
    root_node.push_child(attr_id);
    ast.push_node(root_node);
    ast.push_node(UiAstNode::with_parent(
        attr_id,
        UiAstNodeKind::Attribute,
        root_id,
    ));

    let ir = lower_ast_to_ir(&ast, &UiLoweringConfig).expect("supported subset");

    let attr_ir = ir.nodes().iter().find(|n| n.id().raw() == 2).unwrap();
    assert_eq!(attr_ir.kind(), UiIrNodeKind::Property);
}

#[test]
fn ast_action_lowers_to_ir_action() {
    let mut ast = UiAst::new();
    let root_id = UiAstNodeId::new(1);
    let action_id = UiAstNodeId::new(2);
    let mut root_node = UiAstNode::new(root_id, UiAstNodeKind::Root);
    root_node.push_child(action_id);
    ast.push_node(root_node);
    ast.push_node(UiAstNode::with_parent(
        action_id,
        UiAstNodeKind::Action,
        root_id,
    ));

    let ir = lower_ast_to_ir(&ast, &UiLoweringConfig).expect("supported subset");

    let action_ir = ir.nodes().iter().find(|n| n.id().raw() == 2).unwrap();
    assert_eq!(action_ir.kind(), UiIrNodeKind::Action);
}

#[test]
fn lowered_attribute_projects_to_property_carrier() {
    let mut ast = UiAst::new();
    let root_id = UiAstNodeId::new(1);
    let attr_id = UiAstNodeId::new(2);
    let mut root_node = UiAstNode::new(root_id, UiAstNodeKind::Root);
    root_node.push_child(attr_id);
    ast.push_node(root_node);
    ast.push_node(UiAstNode::with_parent(
        attr_id,
        UiAstNodeKind::Attribute,
        root_id,
    ));

    let ir = lower_ast_to_ir(&ast, &UiLoweringConfig).expect("supported subset");
    let projection = project_ir_to_projection(&ir).expect("valid projection");

    let attr_proj = projection
        .nodes()
        .iter()
        .find(|n| n.id().raw() == 2)
        .unwrap();
    assert_eq!(attr_proj.kind(), UiProjectedNodeKind::PropertyCarrier);
}

#[test]
fn lowered_action_projects_to_action_carrier() {
    let mut ast = UiAst::new();
    let root_id = UiAstNodeId::new(1);
    let action_id = UiAstNodeId::new(2);
    let mut root_node = UiAstNode::new(root_id, UiAstNodeKind::Root);
    root_node.push_child(action_id);
    ast.push_node(root_node);
    ast.push_node(UiAstNode::with_parent(
        action_id,
        UiAstNodeKind::Action,
        root_id,
    ));

    let ir = lower_ast_to_ir(&ast, &UiLoweringConfig).expect("supported subset");
    let projection = project_ir_to_projection(&ir).expect("valid projection");

    let action_proj = projection
        .nodes()
        .iter()
        .find(|n| n.id().raw() == 2)
        .unwrap();
    assert_eq!(action_proj.kind(), UiProjectedNodeKind::ActionCarrier);
}

#[test]
fn lowered_attribute_renders_as_property_marker() {
    let mut ast = UiAst::new();
    let root_id = UiAstNodeId::new(1);
    let attr_id = UiAstNodeId::new(2);
    let mut root_node = UiAstNode::new(root_id, UiAstNodeKind::Root);
    root_node.push_child(attr_id);
    ast.push_node(root_node);
    ast.push_node(UiAstNode::with_parent(
        attr_id,
        UiAstNodeKind::Attribute,
        root_id,
    ));

    let ir = lower_ast_to_ir(&ast, &UiLoweringConfig).expect("supported subset");
    let projection = project_ir_to_projection(&ir).expect("valid projection");
    let render_model = render_projection_to_model(&projection).expect("valid render model");

    let attr_render = render_model
        .nodes()
        .iter()
        .find(|n| n.id().raw() == 2)
        .unwrap();
    assert!(attr_render.markers().contains(&UiRenderMarker::Property));
}

#[test]
fn lowered_action_renders_as_action_marker() {
    let mut ast = UiAst::new();
    let root_id = UiAstNodeId::new(1);
    let action_id = UiAstNodeId::new(2);
    let mut root_node = UiAstNode::new(root_id, UiAstNodeKind::Root);
    root_node.push_child(action_id);
    ast.push_node(root_node);
    ast.push_node(UiAstNode::with_parent(
        action_id,
        UiAstNodeKind::Action,
        root_id,
    ));

    let ir = lower_ast_to_ir(&ast, &UiLoweringConfig).expect("supported subset");
    let projection = project_ir_to_projection(&ir).expect("valid projection");
    let render_model = render_projection_to_model(&projection).expect("valid render model");

    let action_render = render_model
        .nodes()
        .iter()
        .find(|n| n.id().raw() == 2)
        .unwrap();
    assert!(action_render.markers().contains(&UiRenderMarker::Action));
}

#[test]
fn ast_binding_remains_unsupported() {
    let mut ast = UiAst::new();
    let root_id = UiAstNodeId::new(1);
    let binding_id = UiAstNodeId::new(2);
    let mut root_node = UiAstNode::new(root_id, UiAstNodeKind::Root);
    root_node.push_child(binding_id);
    ast.push_node(root_node);
    ast.push_node(UiAstNode::with_parent(
        binding_id,
        UiAstNodeKind::Binding,
        root_id,
    ));

    let err = lower_ast_to_ir(&ast, &UiLoweringConfig).expect_err("binding is unsupported");
    assert_eq!(err.len(), 1);
}

#[test]
fn binding_diagnostic_preserves_ast_node_id() {
    let mut ast = UiAst::new();
    let root_id = UiAstNodeId::new(1);
    let binding_id = UiAstNodeId::new(42);
    let mut root_node = UiAstNode::new(root_id, UiAstNodeKind::Root);
    root_node.push_child(binding_id);
    ast.push_node(root_node);
    ast.push_node(UiAstNode::with_parent(
        binding_id,
        UiAstNodeKind::Binding,
        root_id,
    ));

    let err = lower_ast_to_ir(&ast, &UiLoweringConfig).expect_err("binding is unsupported");
    assert_eq!(err.diagnostics()[0].ast_node_id(), UiAstNodeId::new(42));
    assert_eq!(
        err.diagnostics()[0].kind(),
        UiLoweringDiagnosticKind::UnsupportedAstNodeKind(UiAstNodeKind::Binding)
    );
}

#[test]
fn mixed_carrier_lowering_is_deterministic() {
    let mut ast = UiAst::new();
    let root_id = UiAstNodeId::new(1);
    let mut root_node = UiAstNode::new(root_id, UiAstNodeKind::Root);
    root_node.push_child(UiAstNodeId::new(2));
    root_node.push_child(UiAstNodeId::new(3));
    ast.push_node(root_node);
    ast.push_node(UiAstNode::with_parent(
        UiAstNodeId::new(2),
        UiAstNodeKind::Attribute,
        root_id,
    ));
    ast.push_node(UiAstNode::with_parent(
        UiAstNodeId::new(3),
        UiAstNodeKind::Action,
        root_id,
    ));

    let lowered_one = lower_ast_to_ir(&ast, &UiLoweringConfig).expect("supported subset");
    let lowered_two = lower_ast_to_ir(&ast, &UiLoweringConfig).expect("supported subset");

    assert_eq!(lowered_one, lowered_two);
}

#[test]
fn carrier_lowering_does_not_generate_effect_boundary() {
    let mut ast = UiAst::new();
    let root_id = UiAstNodeId::new(1);
    let mut root_node = UiAstNode::new(root_id, UiAstNodeKind::Root);
    root_node.push_child(UiAstNodeId::new(2));
    root_node.push_child(UiAstNodeId::new(3));
    ast.push_node(root_node);
    ast.push_node(UiAstNode::with_parent(
        UiAstNodeId::new(2),
        UiAstNodeKind::Attribute,
        root_id,
    ));
    ast.push_node(UiAstNode::with_parent(
        UiAstNodeId::new(3),
        UiAstNodeKind::Action,
        root_id,
    ));

    let lowered = lower_ast_to_ir(&ast, &UiLoweringConfig).expect("supported subset");

    assert!(!lowered
        .nodes()
        .iter()
        .any(|n| n.kind() == UiIrNodeKind::EffectBoundary));
}

#[test]
fn carrier_lowering_is_inert_and_non_authoritative() {
    let mut ast = UiAst::new();
    let root_id = UiAstNodeId::new(1);
    let mut root_node = UiAstNode::new(root_id, UiAstNodeKind::Root);
    root_node.push_child(UiAstNodeId::new(2));
    root_node.push_child(UiAstNodeId::new(3));
    ast.push_node(root_node);
    ast.push_node(UiAstNode::with_parent(
        UiAstNodeId::new(2),
        UiAstNodeKind::Attribute,
        root_id,
    ));
    ast.push_node(UiAstNode::with_parent(
        UiAstNodeId::new(3),
        UiAstNodeKind::Action,
        root_id,
    ));

    let config = UiLoweringConfig; // No execution authorities or hooks provided
    let lowered = lower_ast_to_ir(&ast, &config).expect("inert mapping");

    assert_eq!(lowered.len(), 3);
}

#[test]
fn carrier_lowering_preserves_parent_child_handles() {
    let mut ast = UiAst::new();
    let root_id = UiAstNodeId::new(10);
    let mut root_node = UiAstNode::new(root_id, UiAstNodeKind::Root);
    root_node.push_child(UiAstNodeId::new(11));
    root_node.push_child(UiAstNodeId::new(12));
    ast.push_node(root_node);

    ast.push_node(UiAstNode::with_parent(
        UiAstNodeId::new(11),
        UiAstNodeKind::Attribute,
        root_id,
    ));
    ast.push_node(UiAstNode::with_parent(
        UiAstNodeId::new(12),
        UiAstNodeKind::Action,
        root_id,
    ));

    let lowered = lower_ast_to_ir(&ast, &UiLoweringConfig).expect("supported subset");

    let root_ir = lowered.nodes().iter().find(|n| n.id().raw() == 10).unwrap();
    assert_eq!(root_ir.children().len(), 2);
    assert_eq!(root_ir.children()[0].raw(), 11);
    assert_eq!(root_ir.children()[1].raw(), 12);

    let attr_ir = lowered.nodes().iter().find(|n| n.id().raw() == 11).unwrap();
    assert_eq!(attr_ir.parent().unwrap().raw(), 10);

    let action_ir = lowered.nodes().iter().find(|n| n.id().raw() == 12).unwrap();
    assert_eq!(action_ir.parent().unwrap().raw(), 10);
}
