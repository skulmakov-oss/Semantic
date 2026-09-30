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
        return HelloObservationRouteResult::NotRouted(HelloObservationRouteError::NotAdmitted);
    }

    let controlled_text = input
        .text
        .strip_prefix('"')
        .and_then(|text| text.strip_suffix('"'))
        .unwrap_or(&input.text);

    match controlled_text {
        "Hello, World!" => {
            let event = HelloObservationEvent {
                operation_kind: "controlled_observation_text",
                observation_class: HelloObservationClass::ControlledText,
                text: controlled_text.into(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hello_observation_sink::HelloObservationSinkError;
    use alloc::vec::Vec;

    #[derive(Default)]
    struct TestSink(Vec<HelloObservationEvent>);

    impl HelloObservationSink for TestSink {
        fn observe(
            &mut self,
            event: HelloObservationEvent,
        ) -> Result<(), HelloObservationSinkError> {
            self.0.push(event);
            Ok(())
        }
    }

    #[test]
    fn routes_verifier_admitted_quoted_text() {
        let mut sink = TestSink::default();
        let result = route_hello_observation_to_sink(
            HelloObservationRouteInput {
                admitted: true,
                text: String::from("\"Hello, World!\""),
                sequence_index: HelloObservationSequenceIndex(0),
            },
            &mut sink,
        );

        assert_eq!(result, HelloObservationRouteResult::Routed);
        assert_eq!(sink.0[0].text, "Hello, World!");
    }
}
