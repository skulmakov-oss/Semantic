use sm_ir::hello_ir::{
    HelloIrCompleteQuad, HelloIrLocalQuad, HelloIrModule, HelloIrObservationClass,
    HelloIrObserveText, HelloIrQuadLit, HelloIrRequireQuadEq, HelloIrStmt,
};
use std::string::String;
use std::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelloRealSemCodeModule {
    pub ops: Vec<HelloRealSemCodeOp>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HelloRealSemCodeOp {
    DeclareLocalQuad { name: String, value: String },
    RequireQuadEq { name: String, expected: String },
    ObserveTextLiteral { text: String },
    CompleteQuad { value: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HelloRealSemCodeError {
    UnsupportedShape,
}

pub fn emit_hello_real_semcode_skeleton(
    module: &HelloIrModule,
) -> Result<HelloRealSemCodeModule, HelloRealSemCodeError> {
    let ops = render_hello_real_semcode_skeleton(module)?;
    Ok(HelloRealSemCodeModule { ops })
}

pub fn render_hello_real_semcode_skeleton(
    module: &HelloIrModule,
) -> Result<Vec<HelloRealSemCodeOp>, HelloRealSemCodeError> {
    if module.entry.name != "HelloWorld" {
        return Err(HelloRealSemCodeError::UnsupportedShape);
    }

    let ops = match module.entry.body.as_slice() {
        [HelloIrStmt::LocalQuad(HelloIrLocalQuad {
            symbol,
            value: HelloIrQuadLit::T,
        }), HelloIrStmt::RequireQuadEq(HelloIrRequireQuadEq {
            symbol: require_symbol,
            expected: HelloIrQuadLit::T,
        }), HelloIrStmt::ObserveText(HelloIrObserveText {
            text,
            observation_class: HelloIrObservationClass::Controlled,
        }), HelloIrStmt::CompleteQuad(HelloIrCompleteQuad {
            value: HelloIrQuadLit::T,
        })] => vec![
            HelloRealSemCodeOp::DeclareLocalQuad {
                name: symbol.clone(),
                value: render_quad(HelloIrQuadLit::T).to_string(),
            },
            HelloRealSemCodeOp::RequireQuadEq {
                name: require_symbol.clone(),
                expected: render_quad(HelloIrQuadLit::T).to_string(),
            },
            HelloRealSemCodeOp::ObserveTextLiteral { text: text.clone() },
            HelloRealSemCodeOp::CompleteQuad {
                value: render_quad(HelloIrQuadLit::T).to_string(),
            },
        ],
        _ => return Err(HelloRealSemCodeError::UnsupportedShape),
    };

    if ops.len() != 4 {
        return Err(HelloRealSemCodeError::UnsupportedShape);
    }

    Ok(ops)
}

pub fn render_hello_real_semcode_skeleton_text(
    module: &HelloIrModule,
) -> Result<Vec<String>, HelloRealSemCodeError> {
    let ops = render_hello_real_semcode_skeleton(module)?;
    Ok(ops
        .into_iter()
        .map(|op| match op {
            HelloRealSemCodeOp::DeclareLocalQuad { name, value } => {
                format!("declare_local_quad {name} = {value}")
            }
            HelloRealSemCodeOp::RequireQuadEq { name, expected } => {
                format!("require_quad_eq {name} {expected}")
            }
            HelloRealSemCodeOp::ObserveTextLiteral { text } => {
                format!("observe_text_literal {text}")
            }
            HelloRealSemCodeOp::CompleteQuad { value } => format!("complete_quad {value}"),
        })
        .collect())
}

fn render_quad(value: HelloIrQuadLit) -> &'static str {
    match value {
        HelloIrQuadLit::T => "T",
        HelloIrQuadLit::F => "F",
        HelloIrQuadLit::N => "N",
        HelloIrQuadLit::S => "S",
    }
}
