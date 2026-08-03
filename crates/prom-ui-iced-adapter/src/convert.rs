//! `PromNode` -> real `iced::Element` conversion. The only place in this
//! crate (and therefore the only place in this workspace) that holds both
//! a `PromNode` and an `iced::Element` at once. Nothing here is exported
//! from `crate::lib` -- consumers only ever see `PromNode` going in and a
//! running application coming out (via `crate::app::run`).
//!
//! Every `PromRole` variant is matched explicitly (`match` with no
//! wildcard arm) so a role added to `node.rs` without a matching arm here
//! is a real compile error, not a silently-blank render.

use std::collections::HashMap;

use iced::widget::{
    button, column, container, pane_grid, row, scrollable, text, text_editor, text_input, Space,
};
use iced::{Element, Length};

use crate::message::PromUiMessage;
use crate::node::{
    NodeId, Overflow, PromContent, PromNode, PromProperties, PromRole, SizeHint, SplitAxis,
    Typography,
};
use crate::theme::PromTheme;

const SPACING_XS: f32 = 4.0;
const SPACING_SM: f32 = 8.0;
const SPACING_MD: f32 = 16.0;
const DIVIDER_GRAB: f32 = 6.0;

/// Which real child of a `SplitPane` a `pane_grid::Pane` stands for.
/// `pane_grid` itself only ever sees this small `Copy` marker, never any
/// real Workbench/Semantic content -- the actual `Element`s are handed to
/// it fresh, per real `view()` call, via `split_pane`'s own closure (see
/// `make_split_state`'s doc comment for why the marker alone is enough).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SplitSlot {
    First,
    Second,
}

/// Real, persistent, per-`NodeId` `pane_grid::State` -- owned by
/// `crate::app::AdapterState` across real `view()` rebuilds the same way
/// `CodeEditorContents` already is, so a live drag's ratio survives a
/// rebuild instead of resetting to its initial value every frame. `'a` is
/// tied to the caller's own `AdapterState` borrow, not to this map.
pub(crate) type SplitStates = HashMap<NodeId, pane_grid::State<SplitSlot>>;

/// Iced's own real default window size (`iced_core::window::Settings
/// ::default().size`, confirmed by reading that source directly -- this
/// crate's `run`/`run_headless_capture` never override it). Used only to
/// convert a Semantic-supplied `SizeHint::Fixed(px)` hint into a real
/// initial `pane_grid` ratio (`[0,1]`) at the moment a `SplitPane`'s state
/// is first created, so the very first real frame renders at the exact
/// same pixel size a non-draggable `Length::Fixed` would have -- after
/// that, `pane_grid` owns the ratio itself and scales it proportionally on
/// resize, same as any other real, user-resizable pane.
const REFERENCE_WIDTH: f32 = 1024.0;
const REFERENCE_HEIGHT: f32 = 768.0;

/// Real fallback minimum pane size (logical pixels) when a `SplitPane`
/// node doesn't specify `split_min` -- ensures "no panel may collapse to
/// zero" holds even for a node that never opted into an explicit minimum.
const DEFAULT_SPLIT_MIN: f32 = 80.0;

/// Computes a real `[0,1]` initial ratio for a fresh `pane_grid::State`
/// from a `SplitPane` node's real, Semantic-supplied sizing hints.
/// `SizeHint::Fixed(px)` is resolution-dependent by nature (a raw pixel
/// count), so it is resolved against `reference` (see that constant's own
/// doc comment) -- `SizeHint::Portion`/`None` are resolution-independent
/// and computed directly. Pure and independently testable: no `Element`,
/// no live window required.
pub(crate) fn initial_ratio(properties: &PromProperties, reference: f32) -> f32 {
    let portion_ratio = |first: u16, second: u16| -> f32 {
        let first = f32::from(first.max(1));
        let second = f32::from(second.max(1));
        first / (first + second)
    };
    let ratio = match (properties.split_first_size, properties.split_second_size) {
        (Some(SizeHint::Fixed(px)), _) => px / reference,
        (_, Some(SizeHint::Fixed(px))) => 1.0 - px / reference,
        (Some(SizeHint::Portion(p)), Some(SizeHint::Portion(q))) => portion_ratio(p, q),
        (Some(SizeHint::Portion(p)), None) => portion_ratio(p, 1),
        (None, Some(SizeHint::Portion(q))) => portion_ratio(1, q),
        (None, None) => 0.5,
    };
    // A real minimum keeps the *initial* ratio from ever handing
    // `pane_grid` a degenerate 0.0/1.0 split before its own real min-size
    // layout clamping (`.min_size(..)`, see `split_pane`) even gets a
    // chance to run once.
    let min_fraction = (properties.split_min.unwrap_or(DEFAULT_SPLIT_MIN) / reference).min(0.5);
    ratio.clamp(min_fraction, 1.0 - min_fraction)
}

/// Builds a fresh, real `pane_grid::State` for a `SplitPane` node the
/// adapter has not seen before (see `crate::app`'s `sync_split_states`,
/// which calls this exactly once per real `NodeId` and never again --
/// once created, a `SplitPane`'s live ratio is the adapter's own
/// ephemeral state, exactly like `split_first_size`'s own doc comment on
/// `PromProperties` already specifies).
pub(crate) fn make_split_state(properties: &PromProperties) -> pane_grid::State<SplitSlot> {
    let axis = properties.split_axis.unwrap_or(SplitAxis::LeftRight);
    let reference = match axis {
        SplitAxis::LeftRight => REFERENCE_WIDTH,
        SplitAxis::TopBottom => REFERENCE_HEIGHT,
    };
    let ratio = initial_ratio(properties, reference);
    let pane_axis = match axis {
        SplitAxis::LeftRight => pane_grid::Axis::Vertical,
        SplitAxis::TopBottom => pane_grid::Axis::Horizontal,
    };
    pane_grid::State::with_configuration(pane_grid::Configuration::Split {
        axis: pane_axis,
        ratio,
        a: Box::new(pane_grid::Configuration::Pane(SplitSlot::First)),
        b: Box::new(pane_grid::Configuration::Pane(SplitSlot::Second)),
    })
}

/// Real per-`NodeId` `text_editor::Content` lookup, built fresh (cheap --
/// just reference copies, not clones of the buffers themselves) by
/// `crate::app::view` from its own longer-lived `AdapterState` each call.
/// `'a` is tied to that `AdapterState`, not to this map, so the `Element`s
/// `code_editor` builds from it stay valid for the whole real render even
/// though this particular `HashMap` is a throwaway per-call view.
pub(crate) type CodeEditorContents<'a> = HashMap<NodeId, &'a text_editor::Content>;

pub(crate) fn to_element<'a>(
    node: PromNode,
    theme: PromTheme,
    code_editors: &CodeEditorContents<'a>,
    split_states: &'a SplitStates,
) -> Element<'a, PromUiMessage> {
    let id = node.id;
    let action = node.action;
    let content = node.content;
    let properties = node.properties;
    let state = node.state;
    let overflow = node.overflow;
    let children: Vec<Element<'a, PromUiMessage>> = node
        .children
        .into_iter()
        .map(|child| to_element(child, theme, code_editors, split_states))
        .collect();

    match node.role {
        PromRole::RootShell => container(column(children).spacing(0))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: Some(theme.root.into()),
                ..Default::default()
            })
            .into(),

        PromRole::Header => container(
            row(children)
                .spacing(SPACING_MD)
                .align_y(iced::Alignment::Center),
        )
        .width(Length::Fill)
        .padding(SPACING_SM)
        .style(move |_| container::Style {
            background: Some(theme.panel.into()),
            ..Default::default()
        })
        .into(),

        PromRole::NavigationRail => container(column(children).spacing(SPACING_XS))
            .width(Length::Shrink)
            .height(Length::Fill)
            .padding(SPACING_XS)
            .style(move |_| container::Style {
                background: Some(theme.root.into()),
                ..Default::default()
            })
            .into(),

        // `column(children)` defaults to `Length::Shrink` on both axes --
        // per `iced_widget::Column::from_vec`'s own doc comment, a `Shrink`
        // column never gives real space to a `Length::Fill` child (it just
        // collapses it to a minimal size instead), so a `Panel` explicitly
        // marks its inner column `Fill` on both axes to match the explicit
        // `Fill` already set on the outer `container` below. Real, found-
        // and-fixed bug: `CodeEditor` (and any other `Fill`-height content)
        // placed inside a `Panel` rendered as a tiny sliver until this was
        // added -- see the Editor screen migration.
        PromRole::Panel => container(
            column(children)
                .spacing(SPACING_SM)
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .padding(SPACING_SM)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(theme.panel.into()),
            ..Default::default()
        })
        .into(),

        PromRole::SplitPane => split_pane(id, children, &properties, theme, split_states),

        PromRole::SectionHeader => {
            let typography = properties.typography.unwrap_or(Typography::SectionTitle);
            text(content.text.unwrap_or_default())
                .size(theme.typography_size(typography))
                .color(theme.text_secondary)
                .into()
        }

        PromRole::Button => {
            let label = content.text.unwrap_or_default();
            let mut b = button(text(label)).style(move |_, status| {
                button_style(theme, state.selected, state.disabled, status)
            });
            if !state.disabled {
                if let Some(action) = action {
                    b = b.on_press(PromUiMessage::Action(action));
                }
            }
            b.into()
        }

        PromRole::CompactButton => {
            let label = content.text.unwrap_or_default();
            let mut b = button(text(label).size(theme.typography_size(Typography::Control)))
                .padding(SPACING_XS)
                .style(move |_, status| {
                    button_style(theme, state.selected, state.disabled, status)
                });
            if !state.disabled {
                if let Some(action) = action {
                    b = b.on_press(PromUiMessage::Action(action));
                }
            }
            b.into()
        }

        PromRole::CommandCard => {
            let label = content.text.unwrap_or_default();
            let accent = theme.accent_color(properties.accent);
            let mut b = button(
                container(overflow_text(
                    label,
                    None,
                    overflow,
                    theme.typography_size(Typography::Control),
                ))
                .padding(SPACING_SM)
                .width(Length::Fill),
            )
            .width(Length::Fill)
            .style(move |_, status| {
                let mut style = button_style(theme, state.selected, state.disabled, status);
                style.text_color = accent;
                style
            });
            if !state.disabled {
                if let Some(action) = action {
                    b = b.on_press(PromUiMessage::Action(action));
                }
            }
            b.into()
        }

        PromRole::StatusBadge => {
            let label = content.text.unwrap_or_default();
            let color = theme.status_color(state.status);
            container(
                text(label)
                    .size(theme.typography_size(Typography::Badge))
                    .color(theme.root),
            )
            .padding([2, 8])
            .style(move |_| container::Style {
                background: Some(color.into()),
                ..Default::default()
            })
            .into()
        }

        PromRole::Toolbar => row(children)
            .spacing(SPACING_SM)
            .align_y(iced::Alignment::Center)
            .into(),

        PromRole::TabBar => row(children).spacing(SPACING_XS).into(),

        PromRole::TreeView => scrollable(column(children).spacing(0))
            .width(Length::Fill)
            .height(Length::Fill)
            .into(),

        PromRole::TreeRow => {
            let depth = properties.tree_depth.unwrap_or(0);
            let expanded = properties.tree_expanded.unwrap_or(false);
            let is_container = properties.tree_is_container.unwrap_or(false);
            let marker = if is_container {
                if expanded {
                    "v"
                } else {
                    ">"
                }
            } else {
                " "
            };
            let label = content.text.unwrap_or_default();
            // A row's status (e.g. diagnostic severity) is real, meaningful
            // signal that must survive into the rendered color -- selected
            // still wins (it needs to read against the selection
            // highlight), but an unselected row with a non-Normal status
            // is no longer silently forced to `text_primary`.
            let text_color = if state.selected {
                theme.root
            } else if state.status != crate::node::PromStatus::Normal {
                theme.status_color(state.status)
            } else {
                theme.text_primary
            };
            let row_content = row![
                Space::new().width(Length::Fixed(f32::from(depth) * 14.0)),
                text(marker)
                    .size(theme.typography_size(Typography::Control))
                    .color(theme.text_muted),
                container(overflow_text(
                    label,
                    Some(text_color),
                    overflow,
                    theme.typography_size(properties.typography.unwrap_or(Typography::Body)),
                ))
                .width(Length::Fill)
            ]
            .spacing(SPACING_XS)
            .align_y(iced::Alignment::Center);
            let mut b = button(row_content)
                .width(Length::Fill)
                .style(move |_, status| {
                    button_style(theme, state.selected, state.disabled, status)
                });
            if let Some(action) = action {
                b = b.on_press(PromUiMessage::Action(action));
            }
            b.into()
        }

        PromRole::DataTable => column(children).spacing(0).width(Length::Fill).into(),

        PromRole::TableColumn => {
            let label = content.text.unwrap_or_default();
            let width = column_width_to_length(&properties);
            // Real, found-and-fixed bug (caught reviewing this session's
            // own headless screenshots at a narrow 960px viewport): with
            // several unset-width `TableColumn`s sharing a row via
            // `Length::Fill`, a narrow viewport can give one column less
            // width than its own header text needs -- without clipping,
            // that text simply overflows, unclipped, straight into the
            // next column ("STATUSKIND" with zero gap, not two headers).
            // Same overflow-bleed class as the earlier CommandCard fix;
            // TableColumn just never got the same clip treatment.
            container(overflow_text(
                label,
                Some(theme.text_muted),
                Overflow::Clip,
                theme.typography_size(Typography::TableHeader),
            ))
            .width(width)
            .into()
        }

        PromRole::TableRow => {
            let mut b = button(row(children).spacing(SPACING_SM).width(Length::Fill))
                .width(Length::Fill)
                .style(move |_, status| {
                    button_style(theme, state.selected, state.disabled, status)
                });
            if let Some(action) = action {
                b = b.on_press(PromUiMessage::Action(action));
            }
            b.into()
        }

        PromRole::ScrollView => scrollable(
            children
                .into_iter()
                .next()
                .unwrap_or_else(|| Space::new().into()),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into(),

        PromRole::SearchField => {
            let value = content.text.unwrap_or_default();
            let placeholder = content.placeholder.unwrap_or_default();
            text_input(&placeholder, &value)
                .on_input(move |new_value| PromUiMessage::TextEdited(id, new_value))
                .into()
        }

        PromRole::Text => {
            let label = content.text.unwrap_or_default();
            let color = theme.status_color(state.status);
            let typography = properties.typography.unwrap_or(Typography::Body);
            overflow_text(
                label,
                Some(color),
                overflow,
                theme.typography_size(typography),
            )
        }

        PromRole::TextViewer => text_viewer(
            content,
            theme,
            theme.typography_size(properties.typography.unwrap_or(Typography::Evidence)),
        ),

        PromRole::CodeEditor => code_editor(id, code_editors, content, &properties, theme),

        PromRole::InspectorSection => {
            container(column(children).spacing(SPACING_XS).width(Length::Fill))
                .padding(SPACING_SM)
                .style(move |_| container::Style {
                    background: Some(theme.elevated.into()),
                    ..Default::default()
                })
                .into()
        }

        PromRole::KeyValueList => column(children)
            .spacing(SPACING_XS)
            .width(Length::Fill)
            .into(),

        PromRole::ArtifactList => scrollable(column(children).spacing(SPACING_XS))
            .width(Length::Fill)
            .height(Length::Fill)
            .into(),

        PromRole::EmptyState => container(
            text(content.text.unwrap_or_default())
                .size(theme.typography_size(Typography::EmptyState))
                .color(theme.text_muted),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into(),
    }
}

fn column_width_to_length(properties: &crate::node::PromProperties) -> Length {
    match properties.column_width {
        Some(crate::node::SizeHint::Fixed(px)) => Length::Fixed(px),
        Some(crate::node::SizeHint::Portion(p)) => Length::FillPortion(p),
        None => Length::Fill,
    }
}

/// Renders a text label honoring `PromNode::overflow`, real content that
/// doesn't fit its allotted box.
///
/// `Wrap` returns bare wrapped text (it needs no clipping -- it grows
/// vertically inside its parent's width instead of spilling horizontally).
/// Every other mode (`Clip`, `Ellipsis`, `ScrollX`) is rendered inside a
/// clipping container so overflowing content is cut at its own box's edge
/// rather than visually bleeding into a sibling -- this is real
/// `iced::widget::container::Container::clip(true)`, not a hand-rolled
/// width estimate (Iced owns text measurement per the authority model).
/// `Ellipsis`/`ScrollX` are honestly downgraded to plain clipping for now:
/// neither a "..." glyph nor live horizontal scroll offset is wired yet,
/// and a hard clip is still strictly better than the overlap this
/// replaces.
fn overflow_text(
    label: String,
    color: Option<iced::Color>,
    overflow: Overflow,
    size: f32,
) -> Element<'static, PromUiMessage> {
    let mut t = text(label).size(size);
    if let Some(color) = color {
        t = t.color(color);
    }
    match overflow {
        Overflow::Wrap => t.wrapping(text::Wrapping::Word).into(),
        Overflow::Clip | Overflow::Ellipsis | Overflow::ScrollX => {
            // Real, found-and-fixed bug (caught by reviewing this
            // session's own headless screenshots): Iced's `text` widget
            // word-wraps by default (`Wrapping::Word`) even when never
            // asked to. Clipping alone only cuts horizontal overflow that
            // still exists after wrapping -- with wrapping left on, long
            // single-line content (e.g. an extended-length Windows path)
            // just grows to two-plus lines inside the clip container
            // instead of staying one line and being clipped, which is
            // what these three modes actually mean. `Wrapping::None`
            // forces one real line, so `clip(true)` then does its real
            // job on the horizontal overflow.
            container(t.wrapping(text::Wrapping::None))
                .clip(true)
                .into()
        }
    }
}

fn text_viewer(
    content: PromContent,
    theme: PromTheme,
    size: f32,
) -> Element<'static, PromUiMessage> {
    let lines = content.lines.unwrap_or_default();
    let rendered = column(
        lines
            .into_iter()
            .map(|line| text(line).size(size).color(theme.text_code).into())
            .collect::<Vec<_>>(),
    )
    .spacing(2);
    scrollable(rendered)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Real editable code surface, backed by Iced's stateful
/// `text_editor::Content` -- looked up from the per-`NodeId` cache
/// `crate::app::AdapterState` owns and re-syncs from real Semantic content
/// only when it actually changed (see `sync_code_editors`), so live
/// typing's cursor/selection survive across `view()` rebuilds. `id` not
/// yet present in the cache (the very first frame a brand-new `CodeEditor`
/// node exists, before `sync_code_editors` has run for it) falls back to
/// a real, honest read-only render of the same content rather than
/// panicking -- one real frame behind, never wrong content.
fn code_editor<'a>(
    id: NodeId,
    code_editors: &CodeEditorContents<'a>,
    content: PromContent,
    properties: &crate::node::PromProperties,
    theme: PromTheme,
) -> Element<'a, PromUiMessage> {
    let code_size = theme.typography_size(Typography::Code);
    let Some(&editor_content) = code_editors.get(&id) else {
        return text_viewer(content, theme, code_size);
    };
    let monospace = properties.monospace.unwrap_or(true);
    let mut editor = text_editor(editor_content)
        .size(code_size)
        .padding(SPACING_SM)
        .on_action(move |action| PromUiMessage::CodeEdited(id, action))
        .style(move |_iced_theme, status| text_editor::Style {
            background: theme.root.into(),
            border: iced::Border {
                color: match status {
                    text_editor::Status::Focused { .. } => theme.focus,
                    _ => theme.border,
                },
                width: 1.0,
                radius: 2.0.into(),
            },
            placeholder: theme.text_muted,
            value: theme.text_code,
            selection: theme.selected,
        });
    if monospace {
        editor = editor.font(iced::Font::MONOSPACE);
    }
    editor.height(Length::Fill).into()
}

/// Real, live-draggable split rendering via `iced::widget::pane_grid` (a
/// real, already-shipped Iced widget -- not a new dependency). `pane_grid`
/// itself owns real pointer press/move/release/capture mechanics: pressing
/// near the real divider line starts a resize, `CursorMoved` is processed
/// without any `cursor.is_over(bounds)` gate while a resize is active (real
/// capture -- confirmed by reading `iced_widget`'s own `pane_grid.rs`
/// source, not assumed), and every reported ratio already arrives clamped
/// to `[0,1]`; real minimum-size clamping on both sides is enforced by
/// `.min_size(..)` at real layout time. None of that is reimplemented
/// here -- this function only wires two arbitrary `PromNode`-derived
/// `Element`s into `pane_grid`'s real two-pane shape and forwards its real
/// `ResizeEvent`s as `PromUiMessage::SplitResized`.
///
/// The visual divider line itself is drawn by wrapping the whole
/// `pane_grid` in a `theme.divider`-colored container: `pane_grid` does not
/// paint its own `spacing` gap by default (it only highlights on real
/// hover/active-drag, via `.style(..)` below), so without this the gap
/// would show whatever is behind the pane grid instead of a real,
/// always-visible divider line, matching this component's pre-drag look.
fn split_pane<'a>(
    id: NodeId,
    mut children: Vec<Element<'a, PromUiMessage>>,
    properties: &PromProperties,
    theme: PromTheme,
    split_states: &'a SplitStates,
) -> Element<'a, PromUiMessage> {
    if children.len() != 2 {
        return text(format!(
            "SplitPane error: expected exactly 2 children, got {}",
            children.len()
        ))
        .color(theme.failure)
        .into();
    }
    let second = children.pop().expect("checked len == 2");
    let first = children.pop().expect("checked len == 2");

    let Some(pane_state) = split_states.get(&id) else {
        // Real, honest fallback for a `SplitPane` node `sync_split_states`
        // (see `crate::app`) has not yet created persistent state for --
        // should not normally happen (that sync runs before every real
        // `view()`), but a static, correctly-proportioned, non-draggable
        // render is the right degrade, not a panic.
        let axis = properties.split_axis.unwrap_or(SplitAxis::LeftRight);
        let first_len = size_hint_to_length(properties.split_first_size);
        let second_len = size_hint_to_length(properties.split_second_size);
        let divider = container(Space::new()).style(move |_| container::Style {
            background: Some(theme.divider.into()),
            ..Default::default()
        });
        return match axis {
            SplitAxis::LeftRight => row![
                container(first).width(first_len).height(Length::Fill),
                container(divider)
                    .width(Length::Fixed(DIVIDER_GRAB))
                    .height(Length::Fill),
                container(second).width(second_len).height(Length::Fill),
            ]
            .into(),
            SplitAxis::TopBottom => column![
                container(first).width(Length::Fill).height(first_len),
                container(divider)
                    .width(Length::Fill)
                    .height(Length::Fixed(DIVIDER_GRAB)),
                container(second).width(Length::Fill).height(second_len),
            ]
            .into(),
        };
    };

    let min_size = properties.split_min.unwrap_or(DEFAULT_SPLIT_MIN);
    // Real, single-use-per-real-pane extraction: `pane_grid::new`'s view
    // closure must be `Fn` (not `FnOnce`), but it is only ever really
    // invoked exactly once per real `Pane` (exactly `First` once,
    // `Second` once -- confirmed by reading `PaneGrid::new`'s own real
    // implementation, which builds its `contents` via one `.map()` over
    // `state.panes`, not a lazily-repeated call), so `RefCell::take` is
    // real, sound, single-use interior mutability, the same pattern
    // `crate::app::run`'s own real `boot` closure already uses for the
    // same "`Fn`-shaped API, `FnOnce`-shaped real usage" reason.
    let cells = std::cell::RefCell::new((Some(first), Some(second)));

    let grid = pane_grid::PaneGrid::new(pane_state, move |_pane, slot, _is_maximized| {
        let mut cells = cells.borrow_mut();
        let element = match slot {
            SplitSlot::First => cells.0.take(),
            SplitSlot::Second => cells.1.take(),
        }
        .unwrap_or_else(|| Space::new().into());
        pane_grid::Content::new(element)
    })
    .width(Length::Fill)
    .height(Length::Fill)
    .spacing(DIVIDER_GRAB)
    .min_size(min_size)
    .on_resize(4.0, move |event: pane_grid::ResizeEvent| {
        PromUiMessage::SplitResized(id, event.split, event.ratio)
    })
    .style(move |iced_theme| {
        // `pane_grid::Style` has no `Default` impl of its own -- start
        // from Iced's real default (for `hovered_region`, which this
        // component doesn't need to customize) and override only the two
        // divider-line colors with this crate's own real theme tokens.
        let mut style = pane_grid::default(iced_theme);
        style.hovered_split = pane_grid::Line {
            color: theme.divider,
            width: DIVIDER_GRAB,
        };
        style.picked_split = pane_grid::Line {
            color: theme.focus,
            width: DIVIDER_GRAB,
        };
        style
    });

    container(grid)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(theme.divider.into()),
            ..Default::default()
        })
        .into()
}

/// `None` means "share space equally with the sibling," matching
/// `SplitPane`'s documented default -- a real `FillPortion`, not an
/// arbitrary fallback pixel size.
fn size_hint_to_length(hint: Option<crate::node::SizeHint>) -> Length {
    match hint {
        Some(crate::node::SizeHint::Fixed(px)) => Length::Fixed(px),
        Some(crate::node::SizeHint::Portion(p)) => Length::FillPortion(p),
        None => Length::FillPortion(1),
    }
}

fn button_style(
    theme: PromTheme,
    selected: bool,
    disabled: bool,
    status: button::Status,
) -> button::Style {
    let background = if selected {
        theme.selected
    } else {
        match status {
            button::Status::Hovered => theme.hover,
            button::Status::Pressed => theme.interactive,
            _ => theme.interactive,
        }
    };
    let text_color = if selected {
        theme.root
    } else if disabled {
        theme.text_muted
    } else {
        theme.text_primary
    };
    button::Style {
        background: Some(background.into()),
        text_color,
        border: iced::Border {
            color: theme.border,
            width: 1.0,
            radius: 3.0.into(),
        },
        ..Default::default()
    }
}
