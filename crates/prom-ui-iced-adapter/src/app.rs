//! The thin native Iced host. Its own state is only the wrapped
//! `PromApplication` implementor plus generic widget-local ephemeral
//! caches (owner directive section 6) -- it never shadows Semantic
//! domain state.

use std::collections::HashMap;

use iced::widget::text_editor;
use iced::Element;

use crate::convert::{make_split_state, to_element, SplitStates};
use crate::message::PromUiMessage;
use crate::node::{NodeId, PromNode, PromRole, SemanticActionId};
use crate::theme::PromTheme;

/// Implemented by a Prom UI consumer's own application state (e.g.
/// Workbench's `WorkbenchApp`). Every method here routes through the
/// consumer's *existing* verified logic -- this trait is the entire
/// surface through which Iced ever reaches Semantic state, and Semantic
/// state never reaches back into Iced.
pub trait PromApplication {
    /// The native window title.
    fn title(&self) -> String;

    /// Builds the current real component tree from current state. Called
    /// fresh on every real state change -- implementors should keep this
    /// cheap and side-effect-free (pure read of current state), matching
    /// Iced's own Elm-architecture `view()` contract.
    fn view(&self) -> crate::node::PromNode;

    /// A node carrying a `SemanticActionId` was activated (clicked,
    /// selected, etc.). Must route into the consumer's own existing
    /// verified action-dispatch path -- never reimplemented here.
    fn dispatch(&mut self, action: SemanticActionId);

    /// A `SearchField`'s text changed.
    fn set_text(&mut self, node_id: NodeId, text: String) {
        let _ = (node_id, text);
    }

    /// A `CodeEditor`'s real content changed as a result of a genuine
    /// edit/cursor/selection interaction. `cursor` is the 0-based
    /// (line, column) Iced's own editor now reports; `selection_anchor` is
    /// the other end of an active selection (`None` when there is none) --
    /// together the exact same shape as Workbench's own `EditorTab::
    /// cursor_line`/`cursor_col`/`selection_anchor`. Implementors that
    /// track cursor/selection themselves should adopt these as the new
    /// real values rather than recomputing them, so Iced's editing
    /// mechanics and Semantic's document-truth state never drift apart.
    fn set_code(
        &mut self,
        node_id: NodeId,
        lines: Vec<String>,
        cursor: (usize, usize),
        selection_anchor: Option<(usize, usize)>,
    ) {
        let _ = (node_id, lines, cursor, selection_anchor);
    }

    /// A `ScrollView`/`TreeView`/`DataTable` scrolled. Default no-op:
    /// most screens treat scroll position as the adapter's own ephemeral
    /// state and never need Semantic to know about it.
    fn on_scrolled(&mut self, node_id: NodeId, offset: f32) {
        let _ = (node_id, offset);
    }

    /// A `SplitPane` divider was dragged. Default no-op for the same
    /// reason as `on_scrolled` -- override only if a screen wants the
    /// position persisted into Semantic-owned settings.
    fn on_splitter_moved(&mut self, node_id: NodeId, position: f32) {
        let _ = (node_id, position);
    }

    /// The real native window size changed.
    fn on_window_resized(&mut self, width: u32, height: u32) {
        let _ = (width, height);
    }
}

/// A `CodeEditor` node's persistent, real Iced-owned editing buffer, plus
/// the last real Semantic-side line set it was synced from -- so a repeat
/// sync (run after every message; see `sync_code_editors`) can tell "the
/// user is still typing into this same buffer, leave it alone" apart from
/// "Semantic's real content changed out from under this node (tab switch,
/// reload, external reformat), rebuild the buffer for real."
struct CodeEditorEntry {
    content: text_editor::Content,
    synced_lines: Vec<String>,
}

/// Internal Iced `State` wrapper. Not exported -- consumers never see
/// this type, only the `run` function below.
struct AdapterState<App: PromApplication> {
    app: App,
    theme: PromTheme,
    /// Real per-`NodeId` `text_editor::Content` cache (owner directive:
    /// keep Iced's stateful editor buffer out of `PromNode`, which is
    /// rebuilt fresh every `view()`, or live typing would reset its own
    /// cursor/selection every keystroke). Never touched by `App` --
    /// `PromApplication` only ever sees plain `Vec<String>` lines via
    /// `set_code`.
    code_editors: HashMap<NodeId, CodeEditorEntry>,
    /// Real, persistent per-`NodeId` `pane_grid::State` for every
    /// `SplitPane` node -- see `SplitStates`'s own doc comment. Never
    /// touched by `App`: a live drag ratio is the adapter's own ephemeral
    /// state, exactly like `code_editors` above.
    split_states: SplitStates,
}

/// Walks a real `PromNode` tree looking for `CodeEditor` nodes and, for
/// each, either creates a fresh persistent buffer (first time this
/// `NodeId` is seen) or replaces it (Semantic's real `content.lines`
/// changed since the last sync -- an external change, not a live edit
/// this same buffer already knows about) -- otherwise leaves the existing
/// buffer (and its real cursor/selection) untouched.
fn sync_code_editors<App: PromApplication>(state: &mut AdapterState<App>) {
    fn walk(node: &PromNode, code_editors: &mut HashMap<NodeId, CodeEditorEntry>) {
        if node.role == PromRole::CodeEditor {
            let lines = node.content.lines.clone().unwrap_or_default();
            let needs_rebuild = match code_editors.get(&node.id) {
                Some(entry) => entry.synced_lines != lines,
                None => true,
            };
            if needs_rebuild {
                let text = lines.join("\n");
                code_editors.insert(
                    node.id,
                    CodeEditorEntry {
                        content: text_editor::Content::with_text(&text),
                        synced_lines: lines,
                    },
                );
            }
        }
        for child in &node.children {
            walk(child, code_editors);
        }
    }
    let tree = state.app.view();
    walk(&tree, &mut state.code_editors);
}

/// Walks a real `PromNode` tree looking for `SplitPane` nodes and, for
/// each real `NodeId` not already present in the cache, creates one fresh
/// `pane_grid::State` (see `make_split_state`). Deliberately never
/// replaces an *existing* entry -- once a `SplitPane` has real, persistent
/// adapter-owned state, a live/previously-dragged ratio must survive every
/// later `view()` rebuild regardless of what `PromProperties` a
/// re-rendered node carries (matching `split_first_size`'s own documented
/// "the adapter treats live drags as its own ephemeral state" contract).
fn sync_split_states<App: PromApplication>(state: &mut AdapterState<App>) {
    fn walk(node: &PromNode, split_states: &mut SplitStates) {
        if node.role == PromRole::SplitPane && !split_states.contains_key(&node.id) {
            split_states.insert(node.id, make_split_state(&node.properties));
        }
        for child in &node.children {
            walk(child, split_states);
        }
    }
    let tree = state.app.view();
    walk(&tree, &mut state.split_states);
}

fn update<App: PromApplication>(state: &mut AdapterState<App>, message: PromUiMessage) {
    match message {
        PromUiMessage::Action(action) => {
            state.app.dispatch(action);
            sync_code_editors(state);
            sync_split_states(state);
        }
        PromUiMessage::TextEdited(id, text) => state.app.set_text(id, text),
        PromUiMessage::CodeEdited(id, action) => {
            if let Some(entry) = state.code_editors.get_mut(&id) {
                entry.content.perform(action);
                let lines: Vec<String> = entry
                    .content
                    .text()
                    .split('\n')
                    .map(str::to_string)
                    .collect();
                entry.synced_lines = lines.clone();
                let cursor = entry.content.cursor();
                let position = (cursor.position.line, cursor.position.column);
                let selection_anchor = cursor.selection.map(|s| (s.line, s.column));
                state.app.set_code(id, lines, position, selection_anchor);
            }
        }
        PromUiMessage::Scrolled(id, offset) => state.app.on_scrolled(id, offset),
        PromUiMessage::SplitResized(id, split, ratio) => {
            // Real, adapter-owned mutation only -- never routes through
            // `App::dispatch`/any Semantic action (owner requirement:
            // "no domain action or effect caused by divider dragging").
            // The default-no-op `on_splitter_moved` hook still runs
            // afterward purely as an informational forward (matching its
            // own existing doc comment); it causes no domain effect on its
            // own and no real consumer overrides it today.
            if let Some(pane_state) = state.split_states.get_mut(&id) {
                pane_state.resize(split, ratio);
            }
            state.app.on_splitter_moved(id, ratio);
        }
        PromUiMessage::WindowResized(width, height) => state.app.on_window_resized(width, height),
    }
}

fn view<App: PromApplication>(state: &AdapterState<App>) -> Element<'_, PromUiMessage> {
    let code_editors: HashMap<NodeId, &text_editor::Content> = state
        .code_editors
        .iter()
        .map(|(id, entry)| (*id, &entry.content))
        .collect();
    to_element(
        state.app.view(),
        state.theme,
        &code_editors,
        &state.split_states,
    )
}

/// Real error surface for `run`, kept Iced-free at the type level (wraps
/// `iced::Error`'s message, not the type itself) so a caller can match on
/// this without ever naming an Iced type.
#[derive(Debug)]
pub struct PromRunError(pub String);

impl core::fmt::Display for PromRunError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for PromRunError {}

/// Launches a real native Iced application driven entirely by `initial`'s
/// `PromApplication` implementation. Blocks until the window closes, the
/// same shape as this workspace's existing
/// `DesktopSession::run`/`winit::EventLoop::run` entry points.
pub fn run<App>(initial: App) -> Result<(), PromRunError>
where
    App: PromApplication + 'static,
{
    let title = initial.title();
    let mut state = AdapterState {
        app: initial,
        theme: PromTheme::default(),
        code_editors: HashMap::new(),
        split_states: HashMap::new(),
    };
    // Seed any `CodeEditor`/`SplitPane` already present in the real
    // initial `view()` (e.g. a project reopened with a file already open,
    // or the persistent shell's real dividers) before the first real
    // render, the same as both sync functions do after every later
    // message.
    sync_code_editors(&mut state);
    sync_split_states(&mut state);
    // `iced::application`'s `boot` parameter requires `Fn`, not `FnOnce`,
    // even though it is only ever really called once at real startup --
    // a `RefCell<Option<_>>` lets a `move` closure hand `state` out by
    // value on that one real call while still satisfying `Fn`.
    let state_cell = std::cell::RefCell::new(Some(state));
    let boot = move || {
        let state = state_cell
            .borrow_mut()
            .take()
            .expect("iced's boot function must only be called once");
        (state, iced::Task::none())
    };
    iced::application(boot, update, view)
        .title(move |_state: &AdapterState<App>| title.clone())
        .run()
        .map_err(|e| PromRunError(e.to_string()))
}

/// A deterministic, non-interactive capture run: open the real native
/// window, resize it to an exact real viewport, let Iced render at least
/// one real frame at that size, capture that frame via Iced's own real
/// WGPU compositor readback (`iced::window::screenshot`), write it to a
/// real PNG, then exit -- never a Win32 `CopyFromScreen` of the visible
/// desktop, so this does not depend on the window holding OS foreground
/// focus or even being visible above other windows (owner directive
/// section 21/22: "do not depend on foreground-window focus").
#[derive(Debug, Clone)]
pub struct CaptureRequest {
    pub width: u32,
    pub height: u32,
    pub out_path: std::path::PathBuf,
    /// Real, deterministic divider-drag simulation: `(SplitPane NodeId,
    /// ratio)` pairs applied to that node's real, already-synced
    /// `pane_grid::State` (the exact same `.resize(split, ratio)` call a
    /// real live drag produces -- see `update`'s own `SplitResized`
    /// handling) before the frame is captured. Lets a capture prove what
    /// the real rendered result of a completed real drag looks like at an
    /// exact real viewport, through the same focus-independent WGPU
    /// readback as every other capture, when live OS-level pointer
    /// injection into a real foreground window is not reliably available
    /// (real, disclosed reason recorded in the qualification report, not
    /// silently substituted).
    pub split_overrides: Vec<(NodeId, f32)>,
}

#[derive(Debug, Clone)]
enum CaptureMessage {
    Ui(PromUiMessage),
    WindowId(Option<iced::window::Id>),
    Frame,
    Screenshot(iced::window::Screenshot),
}

struct CaptureState<App: PromApplication> {
    inner: AdapterState<App>,
    request: CaptureRequest,
    window_id: Option<iced::window::Id>,
    /// `Some(n)` once resize has been issued -- counts real frames
    /// rendered since, so the screenshot fires only after the compositor
    /// has actually drawn at the new size, not on some fixed wall-clock
    /// guess. `None` before the window/resize round-trip completes.
    frames_since_resize: Option<u32>,
    done: bool,
}

/// How many real frames to let render at the target size before capturing
/// -- generous relative to a real frame's ~16ms budget, cheap insurance
/// against capturing a transitional layout frame mid-resize.
const CAPTURE_SETTLE_FRAMES: u32 = 5;

fn capture_update<App: PromApplication>(
    state: &mut CaptureState<App>,
    message: CaptureMessage,
) -> iced::Task<CaptureMessage> {
    match message {
        CaptureMessage::Ui(msg) => {
            update(&mut state.inner, msg);
            iced::Task::none()
        }
        CaptureMessage::WindowId(Some(id)) => {
            state.window_id = Some(id);
            state.frames_since_resize = Some(0);
            iced::window::resize(
                id,
                iced::Size::new(state.request.width as f32, state.request.height as f32),
            )
        }
        CaptureMessage::WindowId(None) => iced::Task::none(),
        CaptureMessage::Frame => {
            if state.done {
                return iced::Task::none();
            }
            let Some(n) = state.frames_since_resize else {
                return iced::Task::none();
            };
            let n = n + 1;
            state.frames_since_resize = Some(n);
            if n >= CAPTURE_SETTLE_FRAMES {
                if let Some(id) = state.window_id {
                    return iced::window::screenshot(id).map(CaptureMessage::Screenshot);
                }
            }
            iced::Task::none()
        }
        CaptureMessage::Screenshot(shot) => {
            state.done = true;
            match crate::png_encode::write_rgba_png(
                &state.request.out_path,
                shot.size.width,
                shot.size.height,
                &shot.rgba,
            ) {
                Ok(()) => println!(
                    "CAPTURE_DONE: {} size={}x{} scale={}",
                    state.request.out_path.display(),
                    shot.size.width,
                    shot.size.height,
                    shot.scale_factor
                ),
                Err(e) => eprintln!(
                    "CAPTURE_FAILED: could not write {}: {e}",
                    state.request.out_path.display()
                ),
            }
            iced::exit()
        }
    }
}

fn capture_view<App: PromApplication>(state: &CaptureState<App>) -> Element<'_, CaptureMessage> {
    view(&state.inner).map(CaptureMessage::Ui)
}

fn capture_subscription<App: PromApplication>(
    state: &CaptureState<App>,
) -> iced::Subscription<CaptureMessage> {
    if state.done || state.frames_since_resize.is_none() {
        iced::Subscription::none()
    } else {
        iced::window::frames().map(|_| CaptureMessage::Frame)
    }
}

/// Runs one real, deterministic capture and exits -- see `CaptureRequest`.
/// Blocks until the capture completes (or the process is otherwise
/// terminated), the same shape as `run`.
pub fn run_headless_capture<App>(initial: App, request: CaptureRequest) -> Result<(), PromRunError>
where
    App: PromApplication + 'static,
{
    let title = initial.title();
    let mut inner = AdapterState {
        app: initial,
        theme: PromTheme::default(),
        code_editors: HashMap::new(),
        split_states: HashMap::new(),
    };
    sync_code_editors(&mut inner);
    sync_split_states(&mut inner);
    for &(node_id, ratio) in &request.split_overrides {
        if let Some(pane_state) = inner.split_states.get_mut(&node_id) {
            let split = pane_state.layout().splits().next().copied();
            if let Some(split) = split {
                pane_state.resize(split, ratio);
            }
        }
    }
    let state = CaptureState {
        inner,
        request,
        window_id: None,
        frames_since_resize: None,
        done: false,
    };
    let state_cell = std::cell::RefCell::new(Some(state));
    let boot = move || {
        let state = state_cell
            .borrow_mut()
            .take()
            .expect("iced's boot function must only be called once");
        (state, iced::window::latest().map(CaptureMessage::WindowId))
    };
    iced::application(boot, capture_update, capture_view)
        .subscription(capture_subscription)
        .title(move |_state: &CaptureState<App>| title.clone())
        .run()
        .map_err(|e| PromRunError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::{PromProperties, PromRole, SizeHint, SplitAxis};
    use iced::widget::pane_grid;

    /// A minimal, real `PromApplication` stub -- real enough to construct
    /// a real `AdapterState<StubApp>` and drive `update`/`sync_split_states`
    /// against it, with no Workbench/Semantic vocabulary at all (this test
    /// module proves the *generic* adapter behavior, not any one
    /// consumer's).
    struct StubApp {
        tree: PromNode,
        dispatch_count: u32,
        splitter_moved: Vec<(NodeId, f32)>,
    }

    impl PromApplication for StubApp {
        fn title(&self) -> String {
            "stub".to_string()
        }
        fn view(&self) -> PromNode {
            self.tree.clone()
        }
        fn dispatch(&mut self, _action: SemanticActionId) {
            self.dispatch_count += 1;
        }
        fn on_splitter_moved(&mut self, node_id: NodeId, position: f32) {
            self.splitter_moved.push((node_id, position));
        }
    }

    const REAL_MIN: f32 = 80.0;

    fn split_node(id: u64) -> PromNode {
        PromNode::new(NodeId(id), PromRole::SplitPane)
            .with_properties(PromProperties {
                split_axis: Some(SplitAxis::LeftRight),
                split_first_size: Some(SizeHint::Fixed(220.0)),
                split_min: Some(REAL_MIN),
                ..Default::default()
            })
            .push(PromNode::new(NodeId(id * 1000 + 1), PromRole::Panel))
            .push(PromNode::new(NodeId(id * 1000 + 2), PromRole::Panel))
    }

    /// A real tree with three independent, real `SplitPane` nodes -- the
    /// exact same shape (three real dividers) the owner directive's
    /// closure task requires be independently draggable.
    fn three_dividers_tree() -> PromNode {
        PromNode::new(NodeId(0), PromRole::RootShell)
            .push(split_node(1))
            .push(split_node(2))
            .push(split_node(3))
    }

    fn stub_state(tree: PromNode) -> AdapterState<StubApp> {
        let mut state = AdapterState {
            app: StubApp {
                tree,
                dispatch_count: 0,
                splitter_moved: Vec::new(),
            },
            theme: PromTheme::default(),
            code_editors: HashMap::new(),
            split_states: HashMap::new(),
        };
        sync_split_states(&mut state);
        state
    }

    /// The real `pane_grid::Split` id for a real, already-synced
    /// `SplitPane` node -- every real node has exactly one, since this
    /// crate only ever builds a single real `Configuration::Split` per
    /// `SplitPane` (two `Pane` leaves, never a deeper real split tree).
    fn real_split_id(state: &AdapterState<StubApp>, node: NodeId) -> pane_grid::Split {
        *state.split_states[&node]
            .layout()
            .splits()
            .next()
            .expect("a real, synced SplitPane must have exactly one real Split")
    }

    /// Real first/second pane pixel widths for a real `LeftRight` node at
    /// `container_width`, computed the same way `split_pane`'s real
    /// rendering would (`pane_regions` with the same `spacing`/`min_size`
    /// this crate's own `split_pane` passes to the real `PaneGrid`).
    fn real_widths(
        state: &AdapterState<StubApp>,
        node: NodeId,
        container_width: f32,
    ) -> (f32, f32) {
        let pane_state = &state.split_states[&node];
        let regions = pane_state.layout().pane_regions(
            0.0,
            REAL_MIN,
            iced::Size::new(container_width, 600.0),
        );
        let first_pane = pane_state
            .iter()
            .find(|(_, slot)| **slot == crate::convert::SplitSlot::First)
            .map(|(pane, _)| *pane)
            .expect("real First pane must exist");
        let second_pane = pane_state
            .iter()
            .find(|(_, slot)| **slot == crate::convert::SplitSlot::Second)
            .map(|(pane, _)| *pane)
            .expect("real Second pane must exist");
        (regions[&first_pane].width, regions[&second_pane].width)
    }

    #[test]
    fn drag_start_applies_a_real_ratio_immediately() {
        // "Pointer press on a divider starts dragging" -- in this
        // architecture, the very first real `SplitResized` a press+move
        // produces (see `split_pane`'s real `on_resize` wiring) must take
        // real, immediate effect, not be buffered/ignored.
        let mut state = stub_state(three_dividers_tree());
        let split = real_split_id(&state, NodeId(1));
        let (before_first, _) = real_widths(&state, NodeId(1), 1000.0);

        update(
            &mut state,
            PromUiMessage::SplitResized(NodeId(1), split, 0.75),
        );

        let (after_first, after_second) = real_widths(&state, NodeId(1), 1000.0);
        assert_ne!(
            after_first, before_first,
            "the very first resize must take real, immediate effect"
        );
        assert!(
            (after_first - 750.0).abs() < 1.0,
            "expected ~75% of 1000px, got {after_first}"
        );
        assert!(
            (after_second - 250.0).abs() < 1.0,
            "expected ~25% of 1000px, got {after_second}"
        );
    }

    #[test]
    fn ratio_updates_continuously_during_movement() {
        // "Pointer movement updates the split ratio continuously" -- a
        // real sequence of `SplitResized` messages (the real shape
        // continuous `CursorMoved` dragging over this crate's real
        // `PaneGrid.on_resize` wiring produces) must each take real,
        // independent effect in order, not just the first or the last.
        let mut state = stub_state(three_dividers_tree());
        let split = real_split_id(&state, NodeId(1));

        let steps = [0.3, 0.4, 0.5, 0.6, 0.7];
        let mut seen = Vec::new();
        for ratio in steps {
            update(
                &mut state,
                PromUiMessage::SplitResized(NodeId(1), split, ratio),
            );
            let (first, _) = real_widths(&state, NodeId(1), 1000.0);
            seen.push(first);
        }

        // Real, strictly increasing widths -- proves every intermediate
        // move was really applied, not just the final one.
        for pair in seen.windows(2) {
            assert!(
                pair[1] > pair[0],
                "each move step must really move the divider further: {seen:?}"
            );
        }
    }

    #[test]
    fn drag_release_persists_the_final_ratio_across_later_view_rebuilds() {
        // "Pointer release ends dragging" -- there is no separate release
        // message in this architecture (see `message.rs`'s own doc
        // comment: `pane_grid` owns real press/move/release mechanics
        // internally), so what must be real and testable is the
        // *consequence* of a release: the last real ratio reported before
        // release stays in effect through every later real `view()`
        // rebuild, exactly like a real completed drag would leave it.
        let mut state = stub_state(three_dividers_tree());
        let split = real_split_id(&state, NodeId(1));
        update(
            &mut state,
            PromUiMessage::SplitResized(NodeId(1), split, 0.8),
        );
        let (after_drag, _) = real_widths(&state, NodeId(1), 1000.0);

        // A real, later, unrelated Semantic action -- this is exactly what
        // a real subsequent `view()` rebuild after the drag ended looks
        // like (see `update`'s own `Action` arm, which calls
        // `sync_split_states` again on every real dispatch).
        update(&mut state, PromUiMessage::Action(SemanticActionId(999)));

        let (after_rebuild, _) = real_widths(&state, NodeId(1), 1000.0);
        assert_eq!(
            after_drag, after_rebuild,
            "the dragged ratio must survive a later real view() rebuild, not reset"
        );
    }

    #[test]
    fn lower_and_upper_ratio_are_clamped_to_a_real_nonzero_minimum() {
        // "no panel may collapse to zero" / "ratios are clamped to valid
        // minimum panel dimensions" -- a real `pane_grid::State::resize`
        // call does not itself clamp the *stored* ratio (confirmed by
        // reading `iced_widget`'s own `pane_grid/node.rs` source), so the
        // real guarantee comes entirely from real layout-time clamping via
        // `.min_size(..)` (wired in `split_pane`) -- this test proves that
        // real guarantee holds even for a deliberately extreme, real
        // out-of-[0,1] ratio, not just a well-behaved one.
        let mut state = stub_state(three_dividers_tree());
        let split = real_split_id(&state, NodeId(1));

        update(
            &mut state,
            PromUiMessage::SplitResized(NodeId(1), split, -5.0),
        );
        let (low_first, low_second) = real_widths(&state, NodeId(1), 1000.0);
        assert!(
            low_first >= REAL_MIN - 1.0,
            "first pane collapsed below real min: {low_first}"
        );
        assert!(
            low_second >= REAL_MIN - 1.0,
            "second pane collapsed below real min: {low_second}"
        );

        update(
            &mut state,
            PromUiMessage::SplitResized(NodeId(1), split, 5.0),
        );
        let (high_first, high_second) = real_widths(&state, NodeId(1), 1000.0);
        assert!(
            high_first >= REAL_MIN - 1.0,
            "first pane collapsed below real min: {high_first}"
        );
        assert!(
            high_second >= REAL_MIN - 1.0,
            "second pane collapsed below real min: {high_second}"
        );
    }

    #[test]
    fn each_divider_operates_independently() {
        // "only the active divider moves" -- dragging one real SplitPane's
        // divider must not touch either of the other two's real,
        // independent `pane_grid::State`.
        let mut state = stub_state(three_dividers_tree());
        let split1 = real_split_id(&state, NodeId(1));
        let (before2, _) = real_widths(&state, NodeId(2), 1000.0);
        let (before3, _) = real_widths(&state, NodeId(3), 1000.0);

        update(
            &mut state,
            PromUiMessage::SplitResized(NodeId(1), split1, 0.9),
        );

        let (after2, _) = real_widths(&state, NodeId(2), 1000.0);
        let (after3, _) = real_widths(&state, NodeId(3), 1000.0);
        assert_eq!(
            before2, after2,
            "dragging divider 1 must not move divider 2"
        );
        assert_eq!(
            before3, after3,
            "dragging divider 1 must not move divider 3"
        );
    }

    #[test]
    fn resize_behavior_after_a_ratio_has_changed_scales_proportionally() {
        // "resize behavior after a ratio has changed" / "existing
        // responsive behavior remains intact" -- once a real ratio has
        // been set by a real drag, a real later container-size change
        // (a real window resize) must scale both panes proportionally to
        // that *new* ratio, not silently revert to the original one.
        let mut state = stub_state(three_dividers_tree());
        let split = real_split_id(&state, NodeId(1));
        update(
            &mut state,
            PromUiMessage::SplitResized(NodeId(1), split, 0.25),
        );

        let (narrow_first, _) = real_widths(&state, NodeId(1), 800.0);
        let (wide_first, _) = real_widths(&state, NodeId(1), 1600.0);

        assert!(
            (narrow_first - 200.0).abs() < 1.0,
            "expected ~25% of 800px, got {narrow_first}"
        );
        assert!(
            (wide_first - 400.0).abs() < 1.0,
            "expected ~25% of 1600px, got {wide_first}"
        );
    }

    #[test]
    fn dragging_causes_no_domain_action_or_effect() {
        // "no domain action or effect caused by divider dragging" -- a
        // real `SplitResized` message must never reach `App::dispatch`
        // (the one real path to a Semantic action/effect in this whole
        // adapter -- see `PromApplication`'s own doc comment). The
        // informational `on_splitter_moved` hook may still fire (it is a
        // real, pre-existing, default-no-op hook, not a domain action on
        // its own), so this test checks both: zero dispatches, and that
        // the informational hook alone doesn't constitute one.
        let mut state = stub_state(three_dividers_tree());
        let split = real_split_id(&state, NodeId(1));
        assert_eq!(state.app.dispatch_count, 0);

        for ratio in [0.3, 0.4, 0.5, 0.6] {
            update(
                &mut state,
                PromUiMessage::SplitResized(NodeId(1), split, ratio),
            );
        }

        assert_eq!(
            state.app.dispatch_count, 0,
            "no real Semantic action may ever be dispatched by divider dragging"
        );
        assert_eq!(
            state.app.splitter_moved.len(),
            4,
            "the informational hook should still see every real move"
        );
    }
}
