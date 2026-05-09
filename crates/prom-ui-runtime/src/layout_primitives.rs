//! Semantic UI layout primitive type scaffold.
//!
//! This module defines semantic layout primitive roles.
//! It does not implement a renderer, layout engine, geometry solver,
//! Workbench UI, components, widgets, or runtime presentation.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiLayoutPrimitive {
    pub id: &'static str,
    pub kind: UiLayoutPrimitiveKind,
    pub domain: UiLayoutPrimitiveDomain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiLayoutPrimitiveDomain {
    Root,
    Structure,
    Content,
    Trace,
    Inspector,
    Boundary,
    Overlay,
    Map,
    Workbench,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiLayoutPrimitiveKind {
    Root,
    Pane,
    Panel,
    Lane,
    Region,
    Slot,
    Stack,
    Split,
    Overlay,
    InspectorPane,
    TraceLane,
    EffectLane,
    CapabilityGateRegion,
    ConflictBoundary,
    QuarantineRegion,
    SystemMap,
    WorkbenchSurface,
    WorkbenchPanel,
}

pub const UI_LAYOUT_PRIMITIVES: &[UiLayoutPrimitive] = &[
    UiLayoutPrimitive {
        id: "ui.layout.root",
        kind: UiLayoutPrimitiveKind::Root,
        domain: UiLayoutPrimitiveDomain::Root,
    },
    UiLayoutPrimitive {
        id: "ui.layout.pane",
        kind: UiLayoutPrimitiveKind::Pane,
        domain: UiLayoutPrimitiveDomain::Structure,
    },
    UiLayoutPrimitive {
        id: "ui.layout.panel",
        kind: UiLayoutPrimitiveKind::Panel,
        domain: UiLayoutPrimitiveDomain::Structure,
    },
    UiLayoutPrimitive {
        id: "ui.layout.lane",
        kind: UiLayoutPrimitiveKind::Lane,
        domain: UiLayoutPrimitiveDomain::Structure,
    },
    UiLayoutPrimitive {
        id: "ui.layout.region",
        kind: UiLayoutPrimitiveKind::Region,
        domain: UiLayoutPrimitiveDomain::Structure,
    },
    UiLayoutPrimitive {
        id: "ui.layout.slot",
        kind: UiLayoutPrimitiveKind::Slot,
        domain: UiLayoutPrimitiveDomain::Structure,
    },
    UiLayoutPrimitive {
        id: "ui.layout.stack",
        kind: UiLayoutPrimitiveKind::Stack,
        domain: UiLayoutPrimitiveDomain::Structure,
    },
    UiLayoutPrimitive {
        id: "ui.layout.split",
        kind: UiLayoutPrimitiveKind::Split,
        domain: UiLayoutPrimitiveDomain::Structure,
    },
    UiLayoutPrimitive {
        id: "ui.layout.overlay",
        kind: UiLayoutPrimitiveKind::Overlay,
        domain: UiLayoutPrimitiveDomain::Overlay,
    },
    UiLayoutPrimitive {
        id: "ui.layout.inspector_pane",
        kind: UiLayoutPrimitiveKind::InspectorPane,
        domain: UiLayoutPrimitiveDomain::Inspector,
    },
    UiLayoutPrimitive {
        id: "ui.layout.trace_lane",
        kind: UiLayoutPrimitiveKind::TraceLane,
        domain: UiLayoutPrimitiveDomain::Trace,
    },
    UiLayoutPrimitive {
        id: "ui.layout.effect_lane",
        kind: UiLayoutPrimitiveKind::EffectLane,
        domain: UiLayoutPrimitiveDomain::Trace,
    },
    UiLayoutPrimitive {
        id: "ui.layout.capability_gate_region",
        kind: UiLayoutPrimitiveKind::CapabilityGateRegion,
        domain: UiLayoutPrimitiveDomain::Boundary,
    },
    UiLayoutPrimitive {
        id: "ui.layout.conflict_boundary",
        kind: UiLayoutPrimitiveKind::ConflictBoundary,
        domain: UiLayoutPrimitiveDomain::Boundary,
    },
    UiLayoutPrimitive {
        id: "ui.layout.quarantine_region",
        kind: UiLayoutPrimitiveKind::QuarantineRegion,
        domain: UiLayoutPrimitiveDomain::Boundary,
    },
    UiLayoutPrimitive {
        id: "ui.layout.system_map",
        kind: UiLayoutPrimitiveKind::SystemMap,
        domain: UiLayoutPrimitiveDomain::Map,
    },
    UiLayoutPrimitive {
        id: "ui.layout.workbench_surface",
        kind: UiLayoutPrimitiveKind::WorkbenchSurface,
        domain: UiLayoutPrimitiveDomain::Workbench,
    },
    UiLayoutPrimitive {
        id: "ui.layout.workbench_panel",
        kind: UiLayoutPrimitiveKind::WorkbenchPanel,
        domain: UiLayoutPrimitiveDomain::Workbench,
    },
];

pub fn ui_layout_primitives() -> &'static [UiLayoutPrimitive] {
    UI_LAYOUT_PRIMITIVES
}

pub fn find_ui_layout_primitive(id: &str) -> Option<&'static UiLayoutPrimitive> {
    UI_LAYOUT_PRIMITIVES
        .iter()
        .find(|primitive| primitive.id == id)
}
