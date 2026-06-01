use alloc::string::String;

#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelloObservationEvent {
    pub operation_kind: &'static str,
    pub observation_class: HelloObservationClass,
    pub text: String,
    pub sequence_index: HelloObservationSequenceIndex,
}

#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelloObservationClass {
    ControlledText,
}

#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct HelloObservationSequenceIndex(pub u64);

#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelloObservationSinkError {
    Unavailable,
    Denied,
    NondeterministicOrder,
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub trait HelloObservationSink {
    fn observe(&mut self, event: HelloObservationEvent) -> Result<(), HelloObservationSinkError>;
}
