use alloc::string::String;

use crate::hello_observation_sink::{
    HelloObservationClass, HelloObservationEvent, HelloObservationSequenceIndex,
    HelloObservationSink,
};

#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelloObservationRouteInput {
    pub admitted: bool,
    pub text: String,
    pub sequence_index: HelloObservationSequenceIndex,
}

#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelloObservationRouteResult {
    Routed,
    NotRouted(HelloObservationRouteError),
}

#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelloObservationRouteError {
    NotAdmitted,
    NonControlledText,
    ForbiddenHostOutput,
    SinkRejected,
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub fn route_hello_observation_to_sink<S: HelloObservationSink>(
    input: HelloObservationRouteInput,
    sink: &mut S,
) -> HelloObservationRouteResult {
    if !input.admitted {
        return HelloObservationRouteResult::NotRouted(
            HelloObservationRouteError::NotAdmitted,
        );
    }

    match input.text.as_str() {
        "Hello, World!" => {
            let event = HelloObservationEvent {
                operation_kind: "controlled_observation_text",
                observation_class: HelloObservationClass::ControlledText,
                text: input.text,
                sequence_index: input.sequence_index,
            };

            match sink.observe(event) {
                Ok(()) => HelloObservationRouteResult::Routed,
                Err(_) => HelloObservationRouteResult::NotRouted(
                    HelloObservationRouteError::SinkRejected,
                ),
            }
        }
        "stdout" | "print" | "io.write" | "file" | "network" | "stdin" => {
            HelloObservationRouteResult::NotRouted(
                HelloObservationRouteError::ForbiddenHostOutput,
            )
        }
        _ => HelloObservationRouteResult::NotRouted(
            HelloObservationRouteError::NonControlledText,
        ),
    }
}
