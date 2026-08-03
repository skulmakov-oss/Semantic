//! Proof-gate fixture (owner directive section 7): a small,
//! application-neutral Prom UI application, entirely unrelated to
//! Workbench, that exercises the full real pipeline this whole task is
//! meant to prove:
//!
//! Semantic/Prom projection -> Iced adapter -> actual Iced widgets ->
//! native WGPU/Winit frame -> Iced message -> SemanticActionId -> verified
//! state transition -> updated projection -> updated frame
//!
//! Run with: `cargo run -p prom-ui-iced-adapter --example fixture`
//!
//! Contains every element the owner directive requires of this fixture:
//! header; responsive row/column layout; button; text input; scrollable
//! list; split pane; UTF-8 text including Cyrillic; status badge; table;
//! action dispatch.

use prom_ui_iced_adapter::{
    NodeId, Overflow, PromApplication, PromContent, PromNode, PromProperties, PromRole, PromState,
    PromStatus, SemanticActionId, SizeHint, SplitAxis,
};

const ACTION_INCREMENT: u64 = 1;
const ACTION_DECREMENT: u64 = 2;
const ACTION_SELECT_ITEM_BASE: u64 = 100;

const NODE_SEARCH: u64 = 1;

/// The fixture's own tiny "Semantic-owned" state -- deliberately not
/// using any Workbench vocabulary, proving this pipeline is real
/// infrastructure any Prom UI consumer can use, not something
/// Workbench-specific smuggled in through the adapter.
struct FixtureApp {
    counter: i32,
    search: String,
    items: Vec<&'static str>,
    selected: Option<usize>,
}

impl Default for FixtureApp {
    fn default() -> Self {
        Self {
            counter: 0,
            search: String::new(),
            // Real UTF-8 content including Cyrillic, per the owner
            // directive's explicit requirement.
            items: vec![
                "artifact-alpha.log",
                "artifact-beta.log",
                "отчёт-финал.md",
                "проверка-юникода.txt",
                "artifact-gamma.log",
            ],
            selected: None,
        }
    }
}

impl PromApplication for FixtureApp {
    fn title(&self) -> String {
        "Prom UI Iced Adapter -- Proof Fixture".to_string()
    }

    fn view(&self) -> PromNode {
        let header = PromNode::new(NodeId(10), PromRole::Header)
            .push(PromNode::new(NodeId(11), PromRole::Text).with_text("PROOF FIXTURE"))
            .push(
                PromNode::new(NodeId(12), PromRole::StatusBadge)
                    .with_text(format!("counter: {}", self.counter))
                    .with_state(PromState::normal().status(if self.counter >= 0 {
                        PromStatus::Succeeded
                    } else {
                        PromStatus::Failed
                    })),
            );

        // Left pane: SearchField + a real scrollable ArtifactList (UTF-8
        // including Cyrillic), each row dispatching a real select action.
        let search =
            PromNode::new(NodeId(NODE_SEARCH), PromRole::SearchField).with_content(PromContent {
                text: Some(self.search.clone()),
                placeholder: Some("search artifacts...".to_string()),
                lines: None,
            });
        let filtered: Vec<_> = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, name)| {
                self.search.is_empty() || name.to_lowercase().contains(&self.search.to_lowercase())
            })
            .collect();
        let list_rows: Vec<PromNode> = filtered
            .into_iter()
            .map(|(i, name)| {
                PromNode::new(NodeId(200 + i as u64), PromRole::TreeRow)
                    .with_text(*name)
                    .with_state(PromState::normal().selected(self.selected == Some(i)))
                    .with_action(SemanticActionId(ACTION_SELECT_ITEM_BASE + i as u64))
            })
            .collect();
        let artifact_list =
            PromNode::new(NodeId(20), PromRole::ArtifactList).with_children(list_rows);
        let left_panel = PromNode::new(NodeId(21), PromRole::Panel)
            .push(PromNode::new(NodeId(22), PromRole::SectionHeader).with_text("ARTIFACTS"))
            .push(search)
            .push(artifact_list);

        // Right pane: a real DataTable (columns + rows), a real Button
        // dispatching a verified action, and real multi-line UTF-8 text.
        let table_header = PromNode::new(NodeId(30), PromRole::TableColumn)
            .with_text("NAME")
            .with_properties(PromProperties {
                column_width: Some(SizeHint::Portion(2)),
                ..Default::default()
            });
        let table_header2 = PromNode::new(NodeId(31), PromRole::TableColumn)
            .with_text("STATUS")
            .with_properties(PromProperties {
                column_width: Some(SizeHint::Portion(1)),
                ..Default::default()
            });
        let table_columns_row = PromNode::new(NodeId(32), PromRole::Toolbar)
            .push(table_header)
            .push(table_header2);
        let mut table_rows = Vec::new();
        for (i, name) in self.items.iter().enumerate() {
            let cell_name =
                PromNode::new(NodeId(400 + i as u64 * 2), PromRole::Text).with_text(*name);
            let cell_status = PromNode::new(NodeId(400 + i as u64 * 2 + 1), PromRole::StatusBadge)
                .with_text(if i % 2 == 0 { "ok" } else { "pending" })
                .with_state(PromState::normal().status(if i % 2 == 0 {
                    PromStatus::Succeeded
                } else {
                    PromStatus::Unknown
                }));
            table_rows.push(
                PromNode::new(NodeId(500 + i as u64), PromRole::TableRow)
                    .push(cell_name)
                    .push(cell_status),
            );
        }
        let data_table = PromNode::new(NodeId(33), PromRole::DataTable).with_children(table_rows);

        let buttons = PromNode::new(NodeId(40), PromRole::Toolbar)
            .push(
                PromNode::new(NodeId(41), PromRole::Button)
                    .with_text("increment")
                    .with_action(SemanticActionId(ACTION_INCREMENT)),
            )
            .push(
                PromNode::new(NodeId(42), PromRole::Button)
                    .with_text("decrement")
                    .with_action(SemanticActionId(ACTION_DECREMENT)),
            );

        let cyrillic_note = PromNode::new(NodeId(43), PromRole::Text)
            .with_text("Полная проверка UTF-8: кириллица отображается корректно.")
            .with_overflow(Overflow::Wrap);

        let right_panel = PromNode::new(NodeId(23), PromRole::Panel)
            .push(PromNode::new(NodeId(24), PromRole::SectionHeader).with_text("EVIDENCE TABLE"))
            .push(table_columns_row)
            .push(data_table)
            .push(buttons)
            .push(cyrillic_note);

        // Real split pane between the two panels.
        let split = PromNode::new(NodeId(2), PromRole::SplitPane)
            .with_properties(PromProperties {
                split_axis: Some(SplitAxis::LeftRight),
                split_first_size: Some(SizeHint::Fixed(280.0)),
                split_min: Some(160.0),
                ..Default::default()
            })
            .push(left_panel)
            .push(right_panel);

        PromNode::new(NodeId(1), PromRole::RootShell)
            .push(header)
            .push(split)
    }

    fn dispatch(&mut self, action: SemanticActionId) {
        match action.0 {
            ACTION_INCREMENT => self.counter += 1,
            ACTION_DECREMENT => self.counter -= 1,
            n if n >= ACTION_SELECT_ITEM_BASE => {
                let idx = (n - ACTION_SELECT_ITEM_BASE) as usize;
                if idx < self.items.len() {
                    self.selected = Some(idx);
                }
            }
            _ => {}
        }
    }

    fn set_text(&mut self, node_id: NodeId, text: String) {
        if node_id == NodeId(NODE_SEARCH) {
            self.search = text;
        }
    }
}

fn main() {
    if let Err(e) = prom_ui_iced_adapter::run(FixtureApp::default()) {
        eprintln!("fixture failed: {e}");
        std::process::exit(1);
    }
}
