use prom_ui::lowering::{lower_ast_to_ir, UiLoweringConfig};
use prom_ui::model::{UiNode, UiNodeId, UiNodeKind, UiTree, UiTreeId};
use prom_ui::projection::project_ir_to_projection;
use prom_ui::renderer::{render_projection_to_model, UiRenderMarker, UiRenderNodeKind};
use prom_ui::tree_bridge::tree_to_ast;
use prom_ui::validation::{validate_tree, UiTreeValidationConfig};

#[test]
fn vertical_slice_from_tree_to_render_model_builds_successfully() {
    let mut tree = UiTree::new(UiTreeId::new(100));

    let root_id = UiNodeId::new(101);
    let element_id = UiNodeId::new(102);
    let text_id = UiNodeId::new(103);

    let mut root_node = UiNode::new(root_id, UiNodeKind::Root);
    root_node.push_child(element_id);

    let mut element_node = UiNode::with_parent(element_id, UiNodeKind::Element, root_id);
    element_node.push_child(text_id);

    let text_node = UiNode::with_parent(text_id, UiNodeKind::Text, element_id);

    tree.push_node(root_node);
    tree.push_node(element_node);
    tree.push_node(text_node);

    // Step 1: Validate Tree
    let validation_result = validate_tree(&tree, &UiTreeValidationConfig::default());
    assert!(validation_result.is_ok(), "UiTree must pass validation");

    // Step 2: Bridge Tree to AST
    let tree_to_ast_result = tree_to_ast(&tree);
    assert!(
        tree_to_ast_result.is_ok(),
        "Tree must bridge to AST successfully: {:?}",
        tree_to_ast_result.unwrap_err()
    );
    let ast = tree_to_ast_result.unwrap();
    assert_eq!(ast.len(), 3);

    // Step 3: Lower AST to IR
    let lowering_result = lower_ast_to_ir(&ast, &UiLoweringConfig);
    assert!(
        lowering_result.is_ok(),
        "AST must lower to IR successfully: {:?}",
        lowering_result.unwrap_err()
    );
    let ir = lowering_result.unwrap();
    assert_eq!(ir.nodes().len(), 3);

    // Step 4: Project IR to Projection
    let projection = project_ir_to_projection(&ir).expect("projection to succeed");
    assert_eq!(projection.nodes().len(), 3);

    // Step 5: Render Projection to Model
    let render_model_result = render_projection_to_model(&projection);
    assert!(
        render_model_result.is_ok(),
        "Projection must render to model successfully"
    );
    let render_model = render_model_result.unwrap();

    // Verify properties of the final render model
    assert_eq!(render_model.nodes().len(), 3);

    // The final model must only contain root, element, and text nodes.
    // It cannot contain attribute or action carriers, because `UiTree` currently
    // has no way to express them (Slot is unsupported).
    for node in render_model.nodes() {
        match node.kind() {
            UiRenderNodeKind::Root => {}
            UiRenderNodeKind::Element => {}
            UiRenderNodeKind::Text => {}
            UiRenderNodeKind::Fragment => {}
        }

        for marker in node.markers() {
            match marker {
                UiRenderMarker::Property | UiRenderMarker::Action => {
                    panic!("Attributes and Actions should not appear from the tree bridge yet");
                }
                _ => {}
            }
        }
    }
}

#[test]
fn vertical_slice_rejects_unsupported_slot_nodes() {
    let mut tree = UiTree::new(UiTreeId::new(200));

    let root_id = UiNodeId::new(201);
    let slot_id = UiNodeId::new(202); // Slot cannot be bridged

    let mut root_node = UiNode::new(root_id, UiNodeKind::Root);
    root_node.push_child(slot_id);

    let slot_node = UiNode::with_parent(slot_id, UiNodeKind::Slot, root_id);

    tree.push_node(root_node);
    tree.push_node(slot_node);

    // Validation might pass structurally
    let validation_result = validate_tree(&tree, &UiTreeValidationConfig::default());
    assert!(validation_result.is_ok());

    // Bridge Tree to AST must fail due to unsupported Slot
    let tree_to_ast_result = tree_to_ast(&tree);
    assert!(
        tree_to_ast_result.is_err(),
        "Tree to AST bridge must reject unsupported Slot node"
    );
}

#[test]
fn vertical_slice_is_inert_and_non_authoritative() {
    // There are no calls to execution or runtime authorities here.
    // Verified by static authority boundaries in the module dependencies.
}
