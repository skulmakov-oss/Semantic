use std::vec::Vec;

use sm_runtime_core::hello_observation_sink::{
    HelloObservationClass, HelloObservationEvent, HelloObservationSequenceIndex,
    HelloObservationSink, HelloObservationSinkError,
};

#[derive(Default)]
struct InMemoryHelloObservationSink {
    events: Vec<HelloObservationEvent>,
}

impl HelloObservationSink for InMemoryHelloObservationSink {
    fn observe(
        &mut self,
        event: HelloObservationEvent,
    ) -> Result<(), HelloObservationSinkError> {
        self.events.push(event);
        Ok(())
    }
}

fn controlled_event(sequence_index: u64, text: &str) -> HelloObservationEvent {
    HelloObservationEvent {
        operation_kind: "controlled_observation_text",
        observation_class: HelloObservationClass::ControlledText,
        text: text.to_string(),
        sequence_index: HelloObservationSequenceIndex(sequence_index),
    }
}

#[test]
fn hello_observation_sink_skeleton_preserves_deterministic_order() {
    let mut sink = InMemoryHelloObservationSink::default();
    sink.observe(controlled_event(0, "Hello, World!"))
        .expect("first observation should store locally");
    sink.observe(controlled_event(1, "Hello, World!"))
        .expect("second observation should store locally");

    assert_eq!(sink.events.len(), 2);
    assert_eq!(sink.events[0].sequence_index, HelloObservationSequenceIndex(0));
    assert_eq!(sink.events[1].sequence_index, HelloObservationSequenceIndex(1));
    assert_eq!(sink.events[0].observation_class, HelloObservationClass::ControlledText);
    assert_eq!(sink.events[0].operation_kind, "controlled_observation_text");
    assert_ne!(sink.events[0].operation_kind, "print");
    assert_ne!(sink.events[0].operation_kind, "stdout");
    assert_ne!(sink.events[0].operation_kind, "io.write");
}

#[test]
fn hello_observation_sink_skeleton_stores_controlled_text_only_locally() {
    let mut sink = InMemoryHelloObservationSink::default();
    let event = controlled_event(9, "Hello, World!");

    sink.observe(event.clone())
        .expect("controlled observation should stay local to the test sink");

    assert_eq!(sink.events, vec![event]);
    assert!(sink.events.iter().all(|evt| matches!(
        evt.observation_class,
        HelloObservationClass::ControlledText
    )));
}
