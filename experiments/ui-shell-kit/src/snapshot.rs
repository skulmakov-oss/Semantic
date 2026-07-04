use crate::paint::UiFrame;
use core::fmt::Write;
use prom_ui_runtime::DrawCommand;

pub fn frame_to_snapshot(frame: &UiFrame) -> alloc::string::String {
    let mut output = alloc::string::String::new();

    for command in frame.commands() {
        match command {
            DrawCommand::Clear { color } => {
                writeln!(
                    &mut output,
                    "Clear rgba({}, {}, {}, {})",
                    color.r, color.g, color.b, color.a
                )
                .unwrap();
            }
            DrawCommand::FillRect { rect, color } => {
                writeln!(
                    &mut output,
                    "FillRect x={} y={} w={} h={} rgba({}, {}, {}, {})",
                    rect.x, rect.y, rect.width, rect.height, color.r, color.g, color.b, color.a
                )
                .unwrap();
            }
            DrawCommand::DrawText { text, x, y, color } => {
                writeln!(
                    &mut output,
                    "DrawText {:?} x={} y={} rgba({}, {}, {}, {})",
                    text, x, y, color.r, color.g, color.b, color.a
                )
                .unwrap();
            }
        }
    }

    output
}
