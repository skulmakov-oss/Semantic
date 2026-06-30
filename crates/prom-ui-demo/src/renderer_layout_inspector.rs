use std::fmt::Write;

#[derive(Debug, Clone, PartialEq)]
pub struct InspectorNode {
    pub id: &'static str,
    pub kind: &'static str,
    pub rect: LayoutRect,
    pub children: Vec<InspectorNode>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl LayoutRect {
    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

pub fn demo_inspector_tree() -> InspectorNode {
    InspectorNode {
        id: "root",
        kind: "container",
        rect: LayoutRect::new(0, 0, 960, 540),
        children: vec![
            InspectorNode {
                id: "header",
                kind: "bar",
                rect: LayoutRect::new(0, 0, 960, 64),
                children: Vec::new(),
            },
            InspectorNode {
                id: "sidebar",
                kind: "panel",
                rect: LayoutRect::new(0, 64, 220, 420),
                children: Vec::new(),
            },
            InspectorNode {
                id: "content",
                kind: "panel",
                rect: LayoutRect::new(220, 64, 740, 420),
                children: vec![
                    InspectorNode {
                        id: "card_a",
                        kind: "card",
                        rect: LayoutRect::new(244, 88, 320, 160),
                        children: Vec::new(),
                    },
                    InspectorNode {
                        id: "card_b",
                        kind: "card",
                        rect: LayoutRect::new(588, 88, 320, 160),
                        children: Vec::new(),
                    },
                ],
            },
            InspectorNode {
                id: "footer",
                kind: "bar",
                rect: LayoutRect::new(0, 484, 960, 56),
                children: Vec::new(),
            },
        ],
    }
}

pub fn flatten_nodes<'a>(root: &'a InspectorNode) -> Vec<&'a InspectorNode> {
    fn visit<'a>(node: &'a InspectorNode, nodes: &mut Vec<&'a InspectorNode>) {
        nodes.push(node);
        for child in &node.children {
            visit(child, nodes);
        }
    }

    let mut nodes = Vec::new();
    visit(root, &mut nodes);
    nodes
}

pub fn find_node<'a>(root: &'a InspectorNode, id: &str) -> Option<&'a InspectorNode> {
    if root.id == id {
        return Some(root);
    }

    for child in &root.children {
        if let Some(found) = find_node(child, id) {
            return Some(found);
        }
    }

    None
}

pub fn render_inspector_text(root: &InspectorNode, selected_id: &str) -> String {
    let nodes = flatten_nodes(root);
    let selected = find_node(root, selected_id).unwrap_or(root);
    let id_width = nodes
        .iter()
        .map(|node| node.id.len())
        .max()
        .unwrap_or(0)
        .max(9);
    let kind_width = nodes
        .iter()
        .map(|node| node.kind.len())
        .max()
        .unwrap_or(0)
        .max(9);

    let mut output = String::new();
    writeln!(&mut output, "Renderer/Layout Inspector").unwrap();
    writeln!(&mut output).unwrap();
    writeln!(&mut output, "Nodes:").unwrap();

    for node in nodes {
        let marker = if node.id == selected.id { ">" } else { " " };
        writeln!(
            &mut output,
            "{marker} {:<id_width$} {:<kind_width$} x={} y={} w={} h={} children={}",
            node.id,
            node.kind,
            node.rect.x,
            node.rect.y,
            node.rect.width,
            node.rect.height,
            node.children.len(),
            id_width = id_width,
            kind_width = kind_width,
        )
        .unwrap();
    }

    writeln!(&mut output).unwrap();
    writeln!(&mut output, "Selected:").unwrap();
    writeln!(&mut output, "id: {}", selected.id).unwrap();
    writeln!(&mut output, "kind: {}", selected.kind).unwrap();
    writeln!(
        &mut output,
        "rect: x={} y={} w={} h={}",
        selected.rect.x, selected.rect.y, selected.rect.width, selected.rect.height
    )
    .unwrap();
    writeln!(&mut output, "children: {}", selected.children.len()).unwrap();

    output
}
