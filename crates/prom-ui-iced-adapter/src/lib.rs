//! Private Iced-backed native UI substrate for Prom UI.
//!
//! This is the ONLY crate in this workspace authorized to depend on `iced`
//! directly (see `third_party/iced/UPSTREAM.md` and
//! `.harness/current.task.yaml`'s `owner_directive_2026_08_02`). Its
//! public surface (this module's `pub use`s) is entirely Iced-free: a
//! consumer builds a `PromNode` tree describing what should be on screen
//! and implements `PromApplication`, then calls `run` -- no `iced` type
//! ever needs to be named outside this crate. See
//! `tests/dependency_boundary.rs` for the enforcement of that boundary
//! from this crate's own side, and
//! `examples/workbench_semantic/tests/no_iced_dependency.rs` for the
//! enforcement from the consumer's side.

mod app;
mod convert;
mod message;
mod node;
mod png_encode;
mod theme;

pub use app::{run, run_headless_capture, CaptureRequest, PromApplication, PromRunError};
pub use node::{
    NodeId, Overflow, PromContent, PromNode, PromProperties, PromRole, PromState, PromStatus,
    SemanticActionId, SizeHint, SplitAxis, Typography,
};
pub use theme::PromTheme;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prom_node_builds_a_real_tree_without_panicking() {
        let tree = PromNode::new(NodeId(0), PromRole::RootShell)
            .push(PromNode::new(NodeId(1), PromRole::Text).with_text("hello, мир"));
        assert_eq!(tree.children.len(), 1);
        assert_eq!(tree.children[0].content.text.as_deref(), Some("hello, мир"));
    }

    #[test]
    fn to_element_converts_every_role_without_panicking() {
        // Real, exhaustive proof that every `PromRole` variant has a real
        // conversion arm and doesn't panic when given minimal/empty
        // content -- not just a compile-time exhaustiveness check (the
        // `match` in `convert.rs` already gets that from the compiler),
        // but a real runtime pass through `to_element` for each role.
        let theme = PromTheme::default();
        let roles = [
            PromRole::RootShell,
            PromRole::Header,
            PromRole::NavigationRail,
            PromRole::Panel,
            PromRole::SectionHeader,
            PromRole::Button,
            PromRole::CompactButton,
            PromRole::CommandCard,
            PromRole::StatusBadge,
            PromRole::Toolbar,
            PromRole::TabBar,
            PromRole::TreeView,
            PromRole::TreeRow,
            PromRole::DataTable,
            PromRole::TableColumn,
            PromRole::TableRow,
            PromRole::ScrollView,
            PromRole::SearchField,
            PromRole::Text,
            PromRole::TextViewer,
            PromRole::CodeEditor,
            PromRole::InspectorSection,
            PromRole::KeyValueList,
            PromRole::ArtifactList,
            PromRole::EmptyState,
        ];
        // No real `CodeEditor` buffer has been synced for any of these
        // nodes -- `to_element` must fall back to a real read-only render
        // rather than panicking (see `convert::code_editor`'s cache-miss
        // path), which this empty map exercises for `PromRole::CodeEditor`.
        let code_editors: convert::CodeEditorContents = std::collections::HashMap::new();
        let split_states: convert::SplitStates = std::collections::HashMap::new();
        for role in roles {
            let node = PromNode::new(NodeId(1), role).with_text("проверка UTF-8");
            let _element: iced::Element<'_, message::PromUiMessage> =
                convert::to_element(node, theme, &code_editors, &split_states);
        }

        // SplitPane needs exactly 2 children to be meaningful; proved
        // separately since every other role above is meaningful with
        // zero children. No real synced `pane_grid::State` exists for
        // `NodeId(2)` in this empty `split_states` map, so this exercises
        // `split_pane`'s real, honest static fallback path (see its own
        // doc comment) -- the real, synced/draggable path is covered by
        // `app::tests` instead, since it needs `AdapterState`'s private
        // sync machinery to construct meaningfully.
        let split = PromNode::new(NodeId(2), PromRole::SplitPane)
            .push(PromNode::new(NodeId(3), PromRole::Panel))
            .push(PromNode::new(NodeId(4), PromRole::Panel));
        let _element: iced::Element<'_, message::PromUiMessage> =
            convert::to_element(split, theme, &code_editors, &split_states);
    }
}
