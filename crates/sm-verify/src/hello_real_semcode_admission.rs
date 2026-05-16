use std::string::String;
use std::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlledObservationAdmissionKind {
    ControlledTextLiteral,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelloRealSemCodeAdmissionInput {
    pub ops: Vec<HelloRealSemCodeAdmissionOp>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HelloRealSemCodeAdmissionOp {
    DeclareLocalQuad { name: String, value: String },
    RequireQuadEq { name: String, expected: String },
    ObserveTextLiteral { text: String },
    CompleteQuad { value: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelloRealSemCodeAdmissionDecision {
    Admit,
    Reject(HelloRealSemCodeAdmissionError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelloRealSemCodeAdmissionError {
    InvalidOperationOrder,
    MissingRequirement,
    MissingObservation,
    MissingCompletion,
    NonTextObservation,
    StdoutNotAllowed,
    PrintNotAllowed,
    GenericIoNotAllowed,
    OpcodeOrBytecodeNotAllowed,
    UnsupportedShape,
}

pub fn builtin_call_controlled_observation_admission(
    name: &str,
) -> Option<ControlledObservationAdmissionKind> {
    match name {
        "print" => Some(ControlledObservationAdmissionKind::ControlledTextLiteral),
        _ => None,
    }
}

pub fn admit_controlled_text_observation_shape(
    text: &str,
) -> Result<ControlledObservationAdmissionKind, HelloRealSemCodeAdmissionError> {
    match text {
        "\"Hello, World!\"" => Ok(ControlledObservationAdmissionKind::ControlledTextLiteral),
        "\"stdout\"" => Err(HelloRealSemCodeAdmissionError::StdoutNotAllowed),
        "\"print\"" => Err(HelloRealSemCodeAdmissionError::PrintNotAllowed),
        "\"io.write\"" | "\"file\"" | "\"network\"" | "\"stdin\"" => {
            Err(HelloRealSemCodeAdmissionError::GenericIoNotAllowed)
        }
        "\"opcode\"" | "\"bytecode\"" => {
            Err(HelloRealSemCodeAdmissionError::OpcodeOrBytecodeNotAllowed)
        }
        _ => Err(HelloRealSemCodeAdmissionError::NonTextObservation),
    }
}

pub fn admit_hello_real_semcode_skeleton(
    input: &HelloRealSemCodeAdmissionInput,
) -> HelloRealSemCodeAdmissionDecision {
    match input.ops.as_slice() {
        [] | [HelloRealSemCodeAdmissionOp::DeclareLocalQuad { .. }] => {
            HelloRealSemCodeAdmissionDecision::Reject(
                HelloRealSemCodeAdmissionError::MissingRequirement,
            )
        }
        [HelloRealSemCodeAdmissionOp::DeclareLocalQuad { .. }, HelloRealSemCodeAdmissionOp::ObserveTextLiteral { .. }] => {
            HelloRealSemCodeAdmissionDecision::Reject(
                HelloRealSemCodeAdmissionError::MissingRequirement,
            )
        }
        [
            HelloRealSemCodeAdmissionOp::DeclareLocalQuad { .. },
            HelloRealSemCodeAdmissionOp::RequireQuadEq { .. },
        ] => HelloRealSemCodeAdmissionDecision::Reject(
            HelloRealSemCodeAdmissionError::MissingObservation,
        ),
        [
            HelloRealSemCodeAdmissionOp::DeclareLocalQuad { .. },
            HelloRealSemCodeAdmissionOp::ObserveTextLiteral { .. },
            HelloRealSemCodeAdmissionOp::CompleteQuad { .. },
        ] => HelloRealSemCodeAdmissionDecision::Reject(
            HelloRealSemCodeAdmissionError::MissingRequirement,
        ),
        [
            HelloRealSemCodeAdmissionOp::DeclareLocalQuad { .. },
            HelloRealSemCodeAdmissionOp::RequireQuadEq { .. },
            HelloRealSemCodeAdmissionOp::CompleteQuad { .. },
        ] => HelloRealSemCodeAdmissionDecision::Reject(
            HelloRealSemCodeAdmissionError::MissingObservation,
        ),
        [
            HelloRealSemCodeAdmissionOp::DeclareLocalQuad { .. },
            HelloRealSemCodeAdmissionOp::RequireQuadEq { .. },
            HelloRealSemCodeAdmissionOp::ObserveTextLiteral { .. },
        ] => HelloRealSemCodeAdmissionDecision::Reject(
            HelloRealSemCodeAdmissionError::MissingCompletion,
        ),
        [
            HelloRealSemCodeAdmissionOp::DeclareLocalQuad { name, value },
            HelloRealSemCodeAdmissionOp::RequireQuadEq {
                name: require_name,
                expected,
            },
            HelloRealSemCodeAdmissionOp::ObserveTextLiteral { text },
            HelloRealSemCodeAdmissionOp::CompleteQuad {
                value: completion_value,
            },
        ] => {
            if name != "boot" || value != "T" {
                return HelloRealSemCodeAdmissionDecision::Reject(
                    HelloRealSemCodeAdmissionError::UnsupportedShape,
                );
            }
            if require_name != "boot" || expected != "T" {
                return HelloRealSemCodeAdmissionDecision::Reject(
                    HelloRealSemCodeAdmissionError::MissingRequirement,
                );
            }
            match admit_controlled_text_observation_shape(text) {
                Ok(ControlledObservationAdmissionKind::ControlledTextLiteral) => {}
                Err(error) => {
                    return HelloRealSemCodeAdmissionDecision::Reject(error);
                }
            }

            if completion_value != "T" {
                return HelloRealSemCodeAdmissionDecision::Reject(
                    HelloRealSemCodeAdmissionError::MissingCompletion,
                );
            }

            HelloRealSemCodeAdmissionDecision::Admit
        }
        [
            HelloRealSemCodeAdmissionOp::RequireQuadEq { .. },
            ..
        ]
        | [
            HelloRealSemCodeAdmissionOp::ObserveTextLiteral { .. },
            ..
        ]
        | [HelloRealSemCodeAdmissionOp::CompleteQuad { .. }, ..] => {
            HelloRealSemCodeAdmissionDecision::Reject(
                HelloRealSemCodeAdmissionError::InvalidOperationOrder,
            )
        }
        _ if input.ops.len() == 4 => HelloRealSemCodeAdmissionDecision::Reject(
            HelloRealSemCodeAdmissionError::InvalidOperationOrder,
        ),
        _ => HelloRealSemCodeAdmissionDecision::Reject(
            HelloRealSemCodeAdmissionError::UnsupportedShape,
        ),
    }
}
