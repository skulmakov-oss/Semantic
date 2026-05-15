use std::string::String;
use std::vec::Vec;

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
            match text.as_str() {
                "\"Hello, World!\"" => {}
                "\"stdout\"" => {
                    return HelloRealSemCodeAdmissionDecision::Reject(
                        HelloRealSemCodeAdmissionError::StdoutNotAllowed,
                    );
                }
                "\"print\"" => {
                    return HelloRealSemCodeAdmissionDecision::Reject(
                        HelloRealSemCodeAdmissionError::PrintNotAllowed,
                    );
                }
                "\"io.write\"" => {
                    return HelloRealSemCodeAdmissionDecision::Reject(
                        HelloRealSemCodeAdmissionError::GenericIoNotAllowed,
                    );
                }
                "\"opcode\"" | "\"bytecode\"" => {
                    return HelloRealSemCodeAdmissionDecision::Reject(
                        HelloRealSemCodeAdmissionError::OpcodeOrBytecodeNotAllowed,
                    );
                }
                _ => {
                    return HelloRealSemCodeAdmissionDecision::Reject(
                        HelloRealSemCodeAdmissionError::NonTextObservation,
                    );
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
