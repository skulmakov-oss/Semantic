use crate::hello_ir::{
    HelloIrCompleteQuad, HelloIrLocalQuad, HelloIrModule, HelloIrObserveText,
    HelloIrQuadLit, HelloIrRequireQuadEq, HelloIrStmt,
};
use crate::FrontendError;
use std::string::String;
use std::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelloConceptualSemCode {
    pub lines: Vec<String>,
}

pub fn emit_hello_conceptual_semcode(
    module: &HelloIrModule,
) -> Result<HelloConceptualSemCode, FrontendError> {
    let lines = render_hello_conceptual_semcode(module)?;
    Ok(HelloConceptualSemCode { lines })
}

pub fn render_hello_conceptual_semcode(
    module: &HelloIrModule,
) -> Result<Vec<String>, FrontendError> {
    let mut lines = Vec::with_capacity(module.entry.body.len());

    for stmt in &module.entry.body {
        match stmt {
            HelloIrStmt::LocalQuad(HelloIrLocalQuad { symbol, value }) => {
                lines.push(format!(
                    "declare_local_quad {} = {}",
                    symbol,
                    render_quad(*value)
                ));
            }
            HelloIrStmt::RequireQuadEq(HelloIrRequireQuadEq { symbol, expected }) => {
                lines.push(format!(
                    "require_quad_eq {} {}",
                    symbol,
                    render_quad(*expected)
                ));
            }
            HelloIrStmt::ObserveText(HelloIrObserveText { text, .. }) => {
                lines.push(format!("request_observation_text {text}"));
            }
            HelloIrStmt::CompleteQuad(HelloIrCompleteQuad { value }) => {
                lines.push(format!("complete_quad {}", render_quad(*value)));
            }
        }
    }

    if lines.is_empty() {
        return Err(FrontendError::syntax(0, "Hello conceptual SemCode requires at least one statement"));
    }

    Ok(lines)
}

fn render_quad(value: HelloIrQuadLit) -> &'static str {
    match value {
        HelloIrQuadLit::T => "T",
        HelloIrQuadLit::F => "F",
        HelloIrQuadLit::N => "N",
        HelloIrQuadLit::S => "S",
    }
}
