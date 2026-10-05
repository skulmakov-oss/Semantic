//! Shared controlled-observation vocabulary (#1764). `sm-runtime-core` owns
//! these types only; it owns no admission, text policy, dispatch or routing.
//! Observation semantics and sequencing belong to `sm-vm`.

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

/// Position of an observation within one execution. Minted by `sm-vm` as a
/// contiguous `0, 1, 2, …` sequence; exhaustion fails closed in the VM (#1766).
#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct HelloObservationSequenceIndex(pub u64);

#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelloObservationSinkError {
    Unavailable,
    Denied,
}

/// Consumes events in the order the VM delivers them. A sink is not a
/// sequence authority and the trait makes no ordering guarantee of its own
/// (#1766); it may only refuse delivery.
#[cfg(any(feature = "alloc", feature = "std"))]
pub trait HelloObservationSink {
    fn observe(&mut self, event: HelloObservationEvent) -> Result<(), HelloObservationSinkError>;
}
