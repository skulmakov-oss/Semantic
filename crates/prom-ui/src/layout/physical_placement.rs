use super::*;
use crate::model::UiIrNodeId;
use crate::projection::UiProjectedNodeId;
use crate::projection::UiProjectionArtifactId;
use crate::renderer::{UiRenderModelId, UiRenderNodeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UiLayoutPhysicalPlacementModelId {
    raw: u64,
}

impl UiLayoutPhysicalPlacementModelId {
    pub fn new(raw: u64) -> Self {
        Self { raw }
    }

    pub fn raw(self) -> u64 {
        self.raw
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UiLayoutPhysicalPlacementEntryId {
    raw: u64,
}

impl UiLayoutPhysicalPlacementEntryId {
    pub fn new(raw: u64) -> Self {
        Self { raw }
    }

    pub fn raw(self) -> u64 {
        self.raw
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiLayoutPhysicalPlacementKind {
    #[default]
    ParentRelativeIntent,
    PlacementPolicyIntent,
    DeferredRectIntent,
    ContainerRelationIntent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiLayoutPhysicalPlacementState {
    #[default]
    MetadataOnly,
    Deferred,
    Unavailable,
    AuditOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiLayoutPhysicalPlacementEntry {
    id: UiLayoutPhysicalPlacementEntryId,
    source_layout_node: UiLayoutNodeId,
    source_layout_slot: UiLayoutSlotId,
    source_geometry_node: UiLayoutGeometryNodeId,
    source_constraint_declaration: UiLayoutConstraintId,
    source_sizing_entry: UiLayoutSizingEntryId,
    source_sizing_algorithm_entry: UiLayoutSizingAlgorithmEntryId,
    source_measuring_entry: UiLayoutMeasuringEntryId,
    source_size_to_fit_entry: UiLayoutSizeToFitEntryId,
    source_constraint_solver_entry: UiLayoutConstraintSolverEntryId,
    source_solving_entry: UiLayoutSolvingEntryId,
    source_solving_result_entry: UiLayoutSolvingResultEntryId,
    source_render_node: UiRenderNodeId,
    source_projection_node: Option<UiProjectedNodeId>,
    source_ir_node: Option<UiIrNodeId>,
    kind: UiLayoutPhysicalPlacementKind,
    state: UiLayoutPhysicalPlacementState,
    final_rect: UiLayoutGeometryRect,
    order: usize,
}

impl UiLayoutPhysicalPlacementEntry {
    pub fn id(&self) -> UiLayoutPhysicalPlacementEntryId {
        self.id
    }

    pub fn source_layout_node(&self) -> UiLayoutNodeId {
        self.source_layout_node
    }

    pub fn source_layout_slot(&self) -> UiLayoutSlotId {
        self.source_layout_slot
    }

    pub fn source_geometry_node(&self) -> UiLayoutGeometryNodeId {
        self.source_geometry_node
    }

    pub fn source_constraint_declaration(&self) -> UiLayoutConstraintId {
        self.source_constraint_declaration
    }

    pub fn source_sizing_entry(&self) -> UiLayoutSizingEntryId {
        self.source_sizing_entry
    }

    pub fn source_sizing_algorithm_entry(&self) -> UiLayoutSizingAlgorithmEntryId {
        self.source_sizing_algorithm_entry
    }

    pub fn source_measuring_entry(&self) -> UiLayoutMeasuringEntryId {
        self.source_measuring_entry
    }

    pub fn source_size_to_fit_entry(&self) -> UiLayoutSizeToFitEntryId {
        self.source_size_to_fit_entry
    }

    pub fn source_constraint_solver_entry(&self) -> UiLayoutConstraintSolverEntryId {
        self.source_constraint_solver_entry
    }

    pub fn source_solving_entry(&self) -> UiLayoutSolvingEntryId {
        self.source_solving_entry
    }

    pub fn source_solving_result_entry(&self) -> UiLayoutSolvingResultEntryId {
        self.source_solving_result_entry
    }

    pub fn source_render_node(&self) -> UiRenderNodeId {
        self.source_render_node
    }

    pub fn source_projection_node(&self) -> Option<UiProjectedNodeId> {
        self.source_projection_node
    }

    pub fn source_ir_node(&self) -> Option<UiIrNodeId> {
        self.source_ir_node
    }

    pub fn kind(&self) -> UiLayoutPhysicalPlacementKind {
        self.kind
    }

    pub fn state(&self) -> UiLayoutPhysicalPlacementState {
        self.state
    }

    pub fn order(&self) -> usize {
        self.order
    }

    pub fn final_rect(&self) -> UiLayoutGeometryRect {
        self.final_rect
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiLayoutPhysicalPlacementModel {
    id: UiLayoutPhysicalPlacementModelId,
    source_layout_model: UiLayoutModelId,
    source_geometry_model: UiLayoutGeometryModelId,
    source_constraints_model: UiLayoutConstraintsModelId,
    source_sizing_model: UiLayoutSizingModelId,
    source_sizing_algorithm_model: UiLayoutSizingAlgorithmModelId,
    source_measuring_model: UiLayoutMeasuringModelId,
    source_size_to_fit_model: UiLayoutSizeToFitModelId,
    source_constraint_solver_model: UiLayoutConstraintSolverModelId,
    source_solving_model: UiLayoutSolvingModelId,
    source_solving_result_model: UiLayoutSolvingResultModelId,
    source_render_model: UiRenderModelId,
    source_projection: UiProjectionArtifactId,
    source_ir_root: Option<UiIrNodeId>,
    entries: Vec<UiLayoutPhysicalPlacementEntry>,
}

impl UiLayoutPhysicalPlacementModel {
    pub fn id(&self) -> UiLayoutPhysicalPlacementModelId {
        self.id
    }

    pub fn source_layout_model(&self) -> UiLayoutModelId {
        self.source_layout_model
    }

    pub fn source_geometry_model(&self) -> UiLayoutGeometryModelId {
        self.source_geometry_model
    }

    pub fn source_constraints_model(&self) -> UiLayoutConstraintsModelId {
        self.source_constraints_model
    }

    pub fn source_sizing_model(&self) -> UiLayoutSizingModelId {
        self.source_sizing_model
    }

    pub fn source_sizing_algorithm_model(&self) -> UiLayoutSizingAlgorithmModelId {
        self.source_sizing_algorithm_model
    }

    pub fn source_measuring_model(&self) -> UiLayoutMeasuringModelId {
        self.source_measuring_model
    }

    pub fn source_size_to_fit_model(&self) -> UiLayoutSizeToFitModelId {
        self.source_size_to_fit_model
    }

    pub fn source_constraint_solver_model(&self) -> UiLayoutConstraintSolverModelId {
        self.source_constraint_solver_model
    }

    pub fn source_solving_model(&self) -> UiLayoutSolvingModelId {
        self.source_solving_model
    }

    pub fn source_solving_result_model(&self) -> UiLayoutSolvingResultModelId {
        self.source_solving_result_model
    }

    pub fn source_render_model(&self) -> UiRenderModelId {
        self.source_render_model
    }

    pub fn source_projection(&self) -> UiProjectionArtifactId {
        self.source_projection
    }

    pub fn source_ir_root(&self) -> Option<UiIrNodeId> {
        self.source_ir_root
    }

    pub fn entries(&self) -> &[UiLayoutPhysicalPlacementEntry] {
        &self.entries
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

pub fn build_layout_physical_placement(
    model: &UiLayoutSolvingResultModel,
) -> UiLayoutPhysicalPlacementModel {
    let mut entries = Vec::with_capacity(model.entries().len());

    for (order, solving_result_entry) in model.entries().iter().enumerate() {
        let kind = match order % 4 {
            0 => UiLayoutPhysicalPlacementKind::ParentRelativeIntent,
            1 => UiLayoutPhysicalPlacementKind::PlacementPolicyIntent,
            2 => UiLayoutPhysicalPlacementKind::DeferredRectIntent,
            _ => UiLayoutPhysicalPlacementKind::ContainerRelationIntent,
        };

        let state = match order % 4 {
            0 => UiLayoutPhysicalPlacementState::MetadataOnly,
            1 => UiLayoutPhysicalPlacementState::Deferred,
            2 => UiLayoutPhysicalPlacementState::Unavailable,
            _ => UiLayoutPhysicalPlacementState::AuditOnly,
        };

        let final_rect = UiLayoutGeometryRect::new(
            (order as i32) * 10,
            (order as i32) * 20,
            100 + (order as u32) * 5,
            50 + (order as u32) * 5,
        );

        entries.push(UiLayoutPhysicalPlacementEntry {
            id: UiLayoutPhysicalPlacementEntryId::new(solving_result_entry.id().raw()),
            source_layout_node: solving_result_entry.source_layout_node(),
            source_layout_slot: solving_result_entry.source_layout_slot(),
            source_geometry_node: solving_result_entry.source_geometry_node(),
            source_constraint_declaration: solving_result_entry.source_constraint_declaration(),
            source_sizing_entry: solving_result_entry.source_sizing_entry(),
            source_sizing_algorithm_entry: solving_result_entry.source_sizing_algorithm_entry(),
            source_measuring_entry: solving_result_entry.source_measuring_entry(),
            source_size_to_fit_entry: solving_result_entry.source_size_to_fit_entry(),
            source_constraint_solver_entry: solving_result_entry.source_constraint_solver_entry(),
            source_solving_entry: solving_result_entry.source_solving_entry(),
            source_solving_result_entry: solving_result_entry.id(),
            source_render_node: solving_result_entry.source_render_node(),
            source_projection_node: solving_result_entry.source_projection_node(),
            source_ir_node: solving_result_entry.source_ir_node(),
            kind,
            state,
            final_rect,
            order,
        });
    }

    UiLayoutPhysicalPlacementModel {
        id: UiLayoutPhysicalPlacementModelId::new(model.id().raw()),
        source_layout_model: model.source_layout_model(),
        source_geometry_model: model.source_geometry_model(),
        source_constraints_model: model.source_constraints_model(),
        source_sizing_model: model.source_sizing_model(),
        source_sizing_algorithm_model: model.source_sizing_algorithm_model(),
        source_measuring_model: model.source_measuring_model(),
        source_size_to_fit_model: model.source_size_to_fit_model(),
        source_constraint_solver_model: model.source_constraint_solver_model(),
        source_solving_model: model.source_solving_model(),
        source_solving_result_model: model.id(),
        source_render_model: model.source_render_model(),
        source_projection: model.source_projection(),
        source_ir_root: model.source_ir_root(),
        entries,
    }
}

pub fn hit_test_placement(
    x: f64,
    y: f64,
    placement: &UiLayoutPhysicalPlacementModel,
) -> Option<UiRenderNodeId> {
    let mut hit_node = None;
    let mut highest_order = 0;

    for entry in placement.entries() {
        let rect = entry.final_rect();
        let rx = rect.x() as f64;
        let ry = rect.y() as f64;
        let rw = rect.width() as f64;
        let rh = rect.height() as f64;

        if x >= rx && y >= ry && x < rx + rw && y < ry + rh {
            // Find the topmost rect
            if hit_node.is_none() || entry.order() >= highest_order {
                hit_node = Some(entry.source_render_node());
                highest_order = entry.order();
            }
        }
    }

    hit_node
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hit_test_placement() {
        // Build a mock model for testing
        let entries = vec![
            // Background, order 0
            UiLayoutPhysicalPlacementEntry {
                id: UiLayoutPhysicalPlacementEntryId::new(1),
                source_layout_node: UiLayoutNodeId::new(1),
                source_layout_slot: UiLayoutSlotId::new(1),
                source_geometry_node: UiLayoutGeometryNodeId::new(1),
                source_constraint_declaration: UiLayoutConstraintId::new(1),
                source_sizing_entry: UiLayoutSizingEntryId::new(1),
                source_sizing_algorithm_entry: UiLayoutSizingAlgorithmEntryId::new(1),
                source_measuring_entry: UiLayoutMeasuringEntryId::new(1),
                source_size_to_fit_entry: UiLayoutSizeToFitEntryId::new(1),
                source_constraint_solver_entry: UiLayoutConstraintSolverEntryId::new(1),
                source_solving_entry: UiLayoutSolvingEntryId::new(1),
                source_solving_result_entry: UiLayoutSolvingResultEntryId::new(1),
                source_render_node: UiRenderNodeId::new(10),
                source_projection_node: None,
                source_ir_node: None,
                kind: UiLayoutPhysicalPlacementKind::ParentRelativeIntent,
                state: UiLayoutPhysicalPlacementState::MetadataOnly,
                final_rect: UiLayoutGeometryRect::new(0, 0, 100, 100),
                order: 0,
            },
            // Foreground small rect, order 1
            UiLayoutPhysicalPlacementEntry {
                id: UiLayoutPhysicalPlacementEntryId::new(2),
                source_layout_node: UiLayoutNodeId::new(2),
                source_layout_slot: UiLayoutSlotId::new(2),
                source_geometry_node: UiLayoutGeometryNodeId::new(2),
                source_constraint_declaration: UiLayoutConstraintId::new(2),
                source_sizing_entry: UiLayoutSizingEntryId::new(2),
                source_sizing_algorithm_entry: UiLayoutSizingAlgorithmEntryId::new(2),
                source_measuring_entry: UiLayoutMeasuringEntryId::new(2),
                source_size_to_fit_entry: UiLayoutSizeToFitEntryId::new(2),
                source_constraint_solver_entry: UiLayoutConstraintSolverEntryId::new(2),
                source_solving_entry: UiLayoutSolvingEntryId::new(2),
                source_solving_result_entry: UiLayoutSolvingResultEntryId::new(2),
                source_render_node: UiRenderNodeId::new(20),
                source_projection_node: None,
                source_ir_node: None,
                kind: UiLayoutPhysicalPlacementKind::ParentRelativeIntent,
                state: UiLayoutPhysicalPlacementState::MetadataOnly,
                final_rect: UiLayoutGeometryRect::new(10, 10, 50, 50),
                order: 1,
            },
        ];

        let model = UiLayoutPhysicalPlacementModel {
            id: UiLayoutPhysicalPlacementModelId::new(1),
            source_layout_model: UiLayoutModelId::new(1),
            source_geometry_model: UiLayoutGeometryModelId::new(1),
            source_constraints_model: UiLayoutConstraintsModelId::new(1),
            source_sizing_model: UiLayoutSizingModelId::new(1),
            source_sizing_algorithm_model: UiLayoutSizingAlgorithmModelId::new(1),
            source_measuring_model: UiLayoutMeasuringModelId::new(1),
            source_size_to_fit_model: UiLayoutSizeToFitModelId::new(1),
            source_constraint_solver_model: UiLayoutConstraintSolverModelId::new(1),
            source_solving_model: UiLayoutSolvingModelId::new(1),
            source_solving_result_model: UiLayoutSolvingResultModelId::new(1),
            source_render_model: UiRenderModelId::new(1),
            source_projection: UiProjectionArtifactId::new(1),
            source_ir_root: None,
            entries,
        };

        // Point inside both, should return the one with higher order
        assert_eq!(
            hit_test_placement(20.0, 20.0, &model),
            Some(UiRenderNodeId::new(20))
        );

        // Point inside background only
        assert_eq!(
            hit_test_placement(80.0, 80.0, &model),
            Some(UiRenderNodeId::new(10))
        );

        // Point outside all
        assert_eq!(hit_test_placement(150.0, 150.0, &model), None);

        // Point on top-left edge
        assert_eq!(
            hit_test_placement(10.0, 10.0, &model),
            Some(UiRenderNodeId::new(20))
        );

        // Point on bottom-right edge (exclusive)
        assert_eq!(
            hit_test_placement(60.0, 60.0, &model),
            Some(UiRenderNodeId::new(10))
        );
    }
}
