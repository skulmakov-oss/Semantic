//! The adapter's own internal Iced `Message` type. Crate-private: nothing
//! outside `crate::convert`/`crate::app` ever names it. Narrow by design
//! (owner directive section 5) -- every variant maps to exactly one of:
//! an admitted Semantic action, a generic local ephemeral UI-state
//! update, or an explicitly-ignored unsupported event.

use iced::widget::{pane_grid, text_editor};

use crate::node::{NodeId, SemanticActionId};

#[derive(Debug, Clone)]
pub(crate) enum PromUiMessage {
    /// A real click/activation on a node that carries a Semantic action.
    Action(SemanticActionId),
    /// A text field's (`SearchField`) content changed.
    TextEdited(NodeId, String),
    /// A real edit/cursor/selection interaction on a `CodeEditor`. Carries
    /// Iced's own `text_editor::Action` rather than a pre-computed line
    /// set: the adapter replays it against the persistent, per-`NodeId`
    /// `text_editor::Content` it owns in `AdapterState` (see `crate::app`),
    /// then reads the resulting plain lines back out and hands them to
    /// `PromApplication::set_code` -- Semantic remains the sole owner of
    /// real document truth, Iced only supplies the editing mechanics.
    CodeEdited(NodeId, text_editor::Action),
    /// A `ScrollView`/`TreeView`/`DataTable`'s scroll offset changed.
    Scrolled(NodeId, f32),
    /// A real, live `SplitPane` divider drag reported a new ratio for one
    /// of its (possibly several, in a nested tree) real `pane_grid::Split`
    /// dividers. `pane_grid` (a real, already-shipped Iced widget, not a
    /// new dependency) owns the actual pointer-press/move/release/capture
    /// mechanics and hands back an already-clamped-to-`[0,1]` ratio; this
    /// message only identifies *which* `SplitPane` node's persistent
    /// `pane_grid::State` (see `crate::app::AdapterState::split_states`)
    /// should apply it.
    SplitResized(NodeId, pane_grid::Split, f32),
    /// The native window's real size changed.
    WindowResized(u32, u32),
}
