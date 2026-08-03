//! The public, Iced-free Prom UI component contract.
//!
//! `PromNode` is an application-neutral, role-typed component tree: any
//! Prom UI consumer (Workbench or otherwise) builds one of these to
//! describe *what* should be on screen, in Semantic's own vocabulary
//! (content, state, which verified action a click dispatches) -- never
//! *how* it's laid out, shaped, or drawn. Nothing in this module may name
//! or import `iced`; that conversion lives in `crate::convert`, which is
//! the only place in this crate (and therefore the only place in this
//! workspace) allowed to hold both a `PromNode` and an `iced::Element` at
//! the same time.
//!
//! Deliberately NOT a duplicate of `prom_ui`'s existing `UiProjectedNode`/
//! `UiRenderNode`/`UiLayoutNode` family: those are handle-only structural
//! mirrors over a closed, deliberately narrow *semantic/action* role
//! dictionary (`prom_ui::role_dictionary`, which has a real test rejecting
//! visual widget names like `"button"`). `PromNode` is a different,
//! additive layer: a *visual component* role tree, carrying real content
//! and state, that this crate's adapter turns into actual Iced widgets.

/// Stable identity for a `PromNode`, used to correlate Iced-side ephemeral
/// events (scroll offset, text edits, splitter drags, focus) back to the
/// node that produced them. Not necessarily the same value space as
/// `SemanticActionId`, though a consumer is free to reuse the same ids for
/// both (Workbench does, for the nodes that are both identifiable and
/// clickable) since they're both thin `u64` newtypes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(pub u64);

/// A verified Semantic action identity. Never constructed or interpreted
/// by this crate -- purely an opaque token round-tripped from a
/// `PromApplication`'s `view()` output, through a real user interaction,
/// back to that same `PromApplication`'s `dispatch()`. What id space this
/// actually is (and what it means) is entirely up to the consumer; for
/// Workbench it is literally the same `u64` action-node-id space
/// `WorkbenchApp::on_click` already dispatches on, so migrating a screen
/// does not require inventing a second id system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SemanticActionId(pub u64);

/// Which way a `SplitPane`'s two children are arranged, and therefore
/// which way its divider drags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitAxis {
    /// Children sit side by side; the divider is a vertical line the user
    /// drags left/right.
    LeftRight,
    /// Children are stacked; the divider is a horizontal line the user
    /// drags up/down.
    TopBottom,
}

/// How much of a `SplitPane`'s or `DataTable` column's available space its
/// first/a given child should prefer, before min/max constraints apply.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SizeHint {
    /// A specific size in logical pixels.
    Fixed(f32),
    /// A fraction of the available space, roughly `iced`'s `FillPortion`
    /// concept (kept generic/Iced-free at this layer).
    Portion(u16),
}

/// Per-role configuration that doesn't fit a single generic shape.
/// Deliberately small: most roles carry no extra properties (their
/// meaning comes entirely from `content`/`state`/`children`) and use
/// `PromProperties::default()`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PromProperties {
    /// `SplitPane`: which way the divider runs.
    pub split_axis: Option<SplitAxis>,
    /// `SplitPane`: the first child's preferred size along `split_axis`.
    /// `None` means "share space equally with the second child." The
    /// adapter treats live drags as its own ephemeral state (see
    /// `PromEvent::SplitterMoved`) and does not require the consumer to
    /// round-trip every drag frame through a full `view()` rebuild --
    /// this field only matters for the tree's *initial*/externally-driven
    /// size (e.g. restored from settings).
    pub split_first_size: Option<SizeHint>,
    /// `SplitPane`: the second child's preferred size, same semantics as
    /// `split_first_size`. Giving exactly one side a `Fixed` size and
    /// leaving the other `None` is the common case (that side fills the
    /// remaining space); giving both a `Fixed` size is also valid, though
    /// then neither side truly "fills."
    pub split_second_size: Option<SizeHint>,
    /// `SplitPane`: minimum size (in logical pixels) either side may
    /// shrink to.
    pub split_min: Option<f32>,
    /// `TreeRow`: nesting depth (0 = root-level), used for indentation.
    pub tree_depth: Option<u16>,
    /// `TreeRow`: whether this row currently has its children expanded.
    pub tree_expanded: Option<bool>,
    /// `TreeRow`: whether this row represents a container (directory,
    /// group) rather than a leaf.
    pub tree_is_container: Option<bool>,
    /// `TableColumn`: how this column should size itself relative to its
    /// siblings.
    pub column_width: Option<SizeHint>,
    /// `CommandCard`/`StatusBadge`: a short semantic accent name (e.g.
    /// `"teal"`, `"amber"`) resolved to a real color by the adapter's
    /// theme -- never a raw color value here, keeping this layer
    /// palette-agnostic.
    pub accent: Option<&'static str>,
    /// `CodeEditor`/`TextViewer`: 0-based (line, column) cursor position.
    pub cursor: Option<(u32, u32)>,
    /// `CodeEditor`/`TextViewer`: 0-based (line, column) selection anchor
    /// and active-end positions, when a selection is active.
    pub selection: Option<((u32, u32), (u32, u32))>,
    /// `CodeEditor`: whether monospace font metrics should be used for
    /// `content.lines` (true) vs. proportional UI text (false, the
    /// default for every other role that renders text).
    pub monospace: Option<bool>,
    /// `ScrollView`/`TreeView`/`DataTable`: current scroll offset in
    /// logical pixels, for a consumer that wants to drive scroll
    /// position explicitly (e.g. "jump to next search match") rather
    /// than leaving it entirely to the adapter's own ephemeral state.
    pub scroll_offset: Option<f32>,
    /// Explicit typography role override -- `None` means "use this node's
    /// `PromRole`'s own real default" (see `Typography`'s doc comment).
    pub typography: Option<Typography>,
}

/// Real content payload. Kept as a small, explicit struct rather than one
/// big free-form string bag, so a role that legitimately has no text
/// content (a bare `Panel` used only for grouping/background) doesn't need
/// to carry meaningless `None` noise beyond the one field it actually
/// needs.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PromContent {
    /// The primary single-line text for roles like `Text`, `Button`,
    /// `SectionHeader`, `StatusBadge`, `TreeRow`, a table cell, etc.
    pub text: Option<String>,
    /// Multi-line real content for `TextViewer`/`CodeEditor` -- one
    /// `String` per real line, UTF-8, no artificial wrapping applied here
    /// (wrapping is the adapter/Iced's job, per `PromRole`'s overflow
    /// intent).
    pub lines: Option<Vec<String>>,
    /// Placeholder/hint text for `SearchField` when its `text` is empty.
    pub placeholder: Option<String>,
}

impl PromContent {
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            text: Some(text.into()),
            ..Self::default()
        }
    }

    pub fn lines(lines: Vec<String>) -> Self {
        Self {
            lines: Some(lines),
            ..Self::default()
        }
    }
}

/// Domain lifecycle/status meaning a role's visual state should carry --
/// this is where N/F/T/S and job-lifecycle distinctions live, kept
/// distinct from purely generic interaction state (`hover`/`pressed`/
/// `focus`, which `PromState` deliberately does NOT track: those are pure
/// Iced-owned ephemeral interaction mechanics per the adapter's authority
/// split, not something Semantic recomputes every `view()`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PromStatus {
    #[default]
    Normal,
    Loading,
    Succeeded,
    Failed,
    Warning,
    Denied,
    Cancelled,
    Interrupted,
    Stale,
    Unknown,
    Conflict,
}

/// Semantic-owned visual/interaction state for a node. Deliberately does
/// NOT include `hover`/`pressed`/`focused` -- those are real-time pointer/
/// keyboard mechanics Iced owns internally (see this crate's module docs
/// and the owner directive's AUTHORITY MODEL); re-deriving them here on
/// every `view()` call would make Semantic a shadow input-tracking layer,
/// which the directive explicitly forbids.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PromState {
    pub selected: bool,
    pub disabled: bool,
    pub status: PromStatus,
}

impl PromState {
    pub fn normal() -> Self {
        Self::default()
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn status(mut self, status: PromStatus) -> Self {
        self.status = status;
        self
    }
}

/// Named typography roles (owner directive section 17's required list) --
/// the adapter's `PromTheme` resolves each to a real point size, so no
/// screen view builder ever writes a raw font-size number. Every text-
/// bearing `PromRole` has a real, sensible default typography role baked
/// into `crate::convert` (e.g. `SectionHeader` always renders as
/// `SectionTitle`); `PromProperties::typography` only needs to be set
/// explicitly when a plain `Text`/`SectionHeader` node wants a role other
/// than its `PromRole`'s default -- e.g. a `Text` node used as a screen's
/// page title vs. one used as ordinary body copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Typography {
    /// The application's own name/wordmark (the header's title text).
    Product,
    /// A screen's own top-level title (e.g. a `SectionHeader` naming the
    /// whole screen, "COCKPIT"/"JOBS"/etc.).
    PageTitle,
    /// A subsection heading within a screen.
    SectionTitle,
    /// Ordinary prose/content text.
    #[default]
    Body,
    /// Secondary, de-emphasized descriptive text (subtitles, hints, known-
    /// limits notices).
    Metadata,
    /// Text inside an interactive control (buttons, chips, nav rail).
    Control,
    /// Text inside a `StatusBadge`.
    Badge,
    /// A `TableColumn` header label.
    TableHeader,
    /// A `TableRow` cell's text.
    TableRow,
    /// Monospace source-code content (`CodeEditor`).
    Code,
    /// Monospace evidence/log content (`TextViewer` showing job evidence).
    Evidence,
    /// An `EmptyState` placeholder message.
    EmptyState,
}

/// How a `TextViewer`/`CodeEditor`/single-line text role should handle
/// content that doesn't fit its bounds. The adapter maps this to real
/// Iced wrapping/scrolling behavior -- never to silent, undocumented
/// truncation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Overflow {
    Clip,
    #[default]
    Ellipsis,
    Wrap,
    ScrollX,
}

/// The fixed vocabulary of application-neutral visual component roles
/// this adapter knows how to render. Adding a role here is a real,
/// reviewed change (a new arm in `crate::convert`'s match), not silent
/// drift -- deliberately exhaustive-matched everywhere, never a catch-all
/// wildcard arm, so a forgotten role fails to compile rather than
/// rendering as nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromRole {
    /// The single top-level node of an application's whole UI tree.
    RootShell,
    /// A full-width persistent header band.
    Header,
    /// A persistent, narrow vertical navigation strip.
    NavigationRail,
    /// A generic content-grouping surface with no inherent semantics of
    /// its own beyond "a panel."
    Panel,
    /// Two children, one real draggable divider between them (see
    /// `PromProperties::split_axis`/`split_position`/`split_min`).
    /// Exactly two children are expected; the adapter treats a `SplitPane`
    /// with any other child count as a real, reported error rather than
    /// silently rendering the first two.
    SplitPane,
    /// A section/subsection title label.
    SectionHeader,
    /// A standard clickable action control with a text label.
    Button,
    /// A visually smaller/denser `Button` variant (icon-only or a few
    /// characters), for tight spaces like a compact nav rail.
    CompactButton,
    /// A card-shaped clickable action control with a label and an accent
    /// (see `PromProperties::accent`), meant to reflow in a responsive
    /// grid.
    CommandCard,
    /// A small, colored (via `accent`/`state.status`) status pill.
    StatusBadge,
    /// A horizontal strip of controls (filters, actions) above content.
    Toolbar,
    /// A row of selectable tabs; each child is expected to itself carry
    /// an `action` (the tab-select action) and a `text` label.
    TabBar,
    /// A scrollable container of `TreeRow` children (only; the adapter
    /// treats any other child role under a `TreeView` as a real error).
    TreeView,
    /// One row inside a `TreeView`. See `PromProperties::tree_depth`/
    /// `tree_expanded`/`tree_is_container`.
    TreeRow,
    /// A scrollable, columned data grid. Expects `TableColumn` children
    /// (as its header) intermixed with `TableRow` children (as its body)
    /// -- concretely, the adapter expects a `DataTable`'s children to be
    /// zero or more `TableColumn`s followed by zero or more `TableRow`s.
    DataTable,
    /// One column header definition inside a `DataTable`. See
    /// `PromProperties::column_width`.
    TableColumn,
    /// One data row inside a `DataTable`; each child is one cell (a
    /// `Text` node, typically).
    TableRow,
    /// A generic scrollable viewport around a single child.
    ScrollView,
    /// A single-line text-entry control specialized for search/filter
    /// input (distinct from a general text field so the adapter can give
    /// it search-appropriate chrome).
    SearchField,
    /// Plain text content -- the most common leaf role.
    Text,
    /// A scrollable, read-only, multi-line text viewer (see
    /// `content.lines`).
    TextViewer,
    /// An editable, multi-line, typically-monospace text surface with
    /// real cursor/selection (see `PromProperties::cursor`/`selection`/
    /// `monospace`).
    CodeEditor,
    /// A titled sub-section inside an inspector-style panel, grouping a
    /// small set of key facts or a sub-view.
    InspectorSection,
    /// A vertical list of label/value pairs.
    KeyValueList,
    /// A list of produced artifacts/evidence references.
    ArtifactList,
    /// A centered placeholder shown when a container has no real content
    /// yet (e.g. "no project open").
    EmptyState,
}

/// The application-neutral, Iced-free component tree. See this module's
/// doc comment for the full rationale.
#[derive(Debug, Clone, PartialEq)]
pub struct PromNode {
    pub id: NodeId,
    pub role: PromRole,
    pub content: PromContent,
    pub properties: PromProperties,
    pub state: PromState,
    /// The verified Semantic action a real click/activation on this node
    /// dispatches, if any (many nodes -- `Panel`, `Text`, `SectionHeader`
    /// -- are purely structural/informational and carry `None`).
    pub action: Option<SemanticActionId>,
    pub overflow: Overflow,
    pub children: Vec<PromNode>,
}

impl PromNode {
    pub fn new(id: NodeId, role: PromRole) -> Self {
        Self {
            id,
            role,
            content: PromContent::default(),
            properties: PromProperties::default(),
            state: PromState::default(),
            action: None,
            overflow: Overflow::default(),
            children: Vec::new(),
        }
    }

    pub fn with_content(mut self, content: PromContent) -> Self {
        self.content = content;
        self
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.content.text = Some(text.into());
        self
    }

    pub fn with_properties(mut self, properties: PromProperties) -> Self {
        self.properties = properties;
        self
    }

    pub fn with_state(mut self, state: PromState) -> Self {
        self.state = state;
        self
    }

    pub fn with_action(mut self, action: SemanticActionId) -> Self {
        self.action = Some(action);
        self
    }

    pub fn with_overflow(mut self, overflow: Overflow) -> Self {
        self.overflow = overflow;
        self
    }

    pub fn with_children(mut self, children: Vec<PromNode>) -> Self {
        self.children = children;
        self
    }

    pub fn push(mut self, child: PromNode) -> Self {
        self.children.push(child);
        self
    }
}
