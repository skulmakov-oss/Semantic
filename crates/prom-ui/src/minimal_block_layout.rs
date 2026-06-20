use core::convert::TryFrom;

use crate::layout_rect::{UiLayoutRect, UiLayoutRectEntry, UiLayoutRectModel};
use crate::renderer::UiRenderModel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiMinimalBlockLayoutConfig {
    origin_x: i32,
    origin_y: i32,
    width: u32,
    row_height: u32,
    row_gap: u32,
}

impl UiMinimalBlockLayoutConfig {
    pub const fn new(
        origin_x: i32,
        origin_y: i32,
        width: u32,
        row_height: u32,
        row_gap: u32,
    ) -> Self {
        Self {
            origin_x,
            origin_y,
            width,
            row_height,
            row_gap,
        }
    }

    pub const fn origin_x(&self) -> i32 {
        self.origin_x
    }

    pub const fn origin_y(&self) -> i32 {
        self.origin_y
    }

    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn row_height(&self) -> u32 {
        self.row_height
    }

    pub const fn row_gap(&self) -> u32 {
        self.row_gap
    }
}

pub fn solve_minimal_block_layout(
    render_model: &UiRenderModel,
    config: UiMinimalBlockLayoutConfig,
) -> UiLayoutRectModel {
    let mut layout = UiLayoutRectModel::new();
    let row_step = i64::from(config.row_height()) + i64::from(config.row_gap());

    for (index, render_node) in render_model.nodes().iter().enumerate() {
        let index = i64::try_from(index).expect("render node index fits in i64");
        let y = i64::from(config.origin_y()) + index * row_step;
        let y = i32::try_from(y).expect("layout y fits in i32 for deterministic seed");
        let rect = UiLayoutRect::new(config.origin_x(), y, config.width(), config.row_height());

        layout.push_entry(UiLayoutRectEntry::new(
            render_node.id(),
            render_node.source_projection_node(),
            render_node.source_ir_node(),
            rect,
        ));
    }

    layout
}
