//! Test-only controlled-text delivery used by the isolated Hello harnesses
//! (capability gate, audit decision, CLI smoke pipeline).
//!
//! This is NOT a canonical route. Since PB-06 (#1764/#1765) `sm-runtime-core`
//! owns no routing, and canonical observation semantics live in `sm-vm`.
//! There is deliberately no `admitted` input: each harness gates on its own
//! verifier result before calling this helper. The Hello-only text rule below
//! is a provisional harness restriction, not a language rule.

use sm_runtime_core::hello_observation_sink::{
    HelloObservationClass, HelloObservationEvent, HelloObservationSequenceIndex,
    HelloObservationSink,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelloObservationRouteInput {
    pub text: String,
    pub sequence_index: HelloObservationSequenceIndex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelloObservationRouteResult {
    Routed,
    NotRouted(HelloObservationRouteError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelloObservationRouteError {
    NonControlledText,
    ForbiddenHostOutput,
    SinkRejected,
}

pub fn route_hello_observation_to_sink<S: HelloObservationSink>(
    input: HelloObservationRouteInput,
    sink: &mut S,
) -> HelloObservationRouteResult {
    let text = input
        .text
        .strip_prefix('"')
        .and_then(|text| text.strip_suffix('"'))
        .unwrap_or(&input.text);
    match text {
        "Hello, World!" => {
            let event = HelloObservationEvent {
                operation_kind: "controlled_observation_text",
                observation_class: HelloObservationClass::ControlledText,
                text: text.into(),
                sequence_index: input.sequence_index,
            };
            match sink.observe(event) {
                Ok(()) => HelloObservationRouteResult::Routed,
                Err(_) => {
                    HelloObservationRouteResult::NotRouted(HelloObservationRouteError::SinkRejected)
                }
            }
        }
        "stdout" | "print" | "io.write" | "file" | "network" | "stdin" => {
            HelloObservationRouteResult::NotRouted(HelloObservationRouteError::ForbiddenHostOutput)
        }
        _ => HelloObservationRouteResult::NotRouted(HelloObservationRouteError::NonControlledText),
    }
}
