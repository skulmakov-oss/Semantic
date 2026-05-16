use crate::hello_real_semcode::{HelloRealSemCodeModule, HelloRealSemCodeOp};
use std::string::String;
use std::vec::Vec;

const FORMAT_STATUS: &str = "gated_provisional_not_production";
const CONTROLLED_OBSERVATION_SYMBOL: &str = "OBSERVE_TEXT_LITERAL";
const CONTROLLED_OBSERVATION_CLASS: &str = "ControlledText";
const CANONICAL_HELLO_TEXT: &str = "Hello, World!";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelloObservationByteModule {
    pub records: Vec<HelloObservationByteRecord>,
    pub format_status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HelloObservationByteRecord {
    DeclareLocalQuad { symbol: String, value: String },
    RequireQuadEq { symbol: String, expected: String },
    ObserveTextLiteral {
        symbolic_op: &'static str,
        text_const_index: u32,
        observation_class: &'static str,
        sequence_index: u32,
        policy_ref: u32,
    },
    CompleteQuad { value: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HelloObservationByteEmissionError {
    UnsupportedShape,
    MissingControlledObservation,
    ForbiddenHostOutput,
}

pub fn emit_hello_observation_bytes_gated(
    module: &HelloRealSemCodeModule,
) -> Result<HelloObservationByteModule, HelloObservationByteEmissionError> {
    let _ = validate_and_extract_controlled_text(module)?;
    Ok(HelloObservationByteModule {
        records: vec![
            HelloObservationByteRecord::DeclareLocalQuad {
                symbol: "boot".to_string(),
                value: "T".to_string(),
            },
            HelloObservationByteRecord::RequireQuadEq {
                symbol: "boot".to_string(),
                expected: "T".to_string(),
            },
            HelloObservationByteRecord::ObserveTextLiteral {
                symbolic_op: CONTROLLED_OBSERVATION_SYMBOL,
                text_const_index: 0,
                observation_class: CONTROLLED_OBSERVATION_CLASS,
                sequence_index: 0,
                policy_ref: 0,
            },
            HelloObservationByteRecord::CompleteQuad {
                value: "T".to_string(),
            },
        ],
        format_status: FORMAT_STATUS,
    })
}

pub fn render_hello_observation_bytes_gated(
    module: &HelloRealSemCodeModule,
) -> Result<Vec<u8>, HelloObservationByteEmissionError> {
    let text = validate_and_extract_controlled_text(module)?;
    let rendered = format!(
        "FORMAT_STATUS {FORMAT_STATUS}\nDECLARE_LOCAL_QUAD boot T\nREQUIRE_QUAD_EQ boot T\nOBSERVE_TEXT_LITERAL const#0 ControlledText seq#0 policy#0\nCOMPLETE_QUAD T\nCONST_TEXT #0 {text:?}",
    );
    Ok(rendered.into_bytes())
}

fn validate_and_extract_controlled_text(
    module: &HelloRealSemCodeModule,
) -> Result<&str, HelloObservationByteEmissionError> {
    if !module
        .ops
        .iter()
        .any(|stmt| matches!(stmt, HelloRealSemCodeOp::ObserveTextLiteral { .. }))
    {
        return Err(HelloObservationByteEmissionError::MissingControlledObservation);
    }

    match module.ops.as_slice() {
        [
            HelloRealSemCodeOp::DeclareLocalQuad { name, value },
            HelloRealSemCodeOp::RequireQuadEq {
                name: require_symbol,
                expected,
            },
            HelloRealSemCodeOp::ObserveTextLiteral { text },
            HelloRealSemCodeOp::CompleteQuad { value: complete_value },
        ] if name == "boot" && value == "T" && require_symbol == "boot" && expected == "T" && complete_value == "T" =>
        {
            let text = strip_rendered_text(text);
            if is_forbidden_host_output(text) {
                return Err(HelloObservationByteEmissionError::ForbiddenHostOutput);
            }
            if text != CANONICAL_HELLO_TEXT {
                return Err(HelloObservationByteEmissionError::UnsupportedShape);
            }
            Ok(text)
        }
        _ => Err(HelloObservationByteEmissionError::UnsupportedShape),
    }
}

fn strip_rendered_text(text: &str) -> &str {
    text.strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(text)
}

fn is_forbidden_host_output(text: &str) -> bool {
    matches!(text, "stdout" | "print" | "io.write" | "file" | "network" | "stdin")
}
