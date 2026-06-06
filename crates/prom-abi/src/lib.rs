#![allow(clippy::derivable_impls)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::string::{String, ToString};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HostCallId {
    GateRead,
    GateWrite,
    PulseEmit,
    StateQuery,
    StateUpdate,
    EventPost,
    ClockRead,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectClass {
    HostQuery,
    HostWrite,
    EventEmit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeterminismClass {
    Deterministic,
    HostBound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostCallStability {
    StableV1,
    PlannedPostStable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HostCallDescriptor {
    pub id: HostCallId,
    pub effect: EffectClass,
    pub determinism: DeterminismClass,
    pub returns_value: bool,
    pub stability: HostCallStability,
}

pub const fn descriptor_for_call(id: HostCallId) -> HostCallDescriptor {
    match id {
        HostCallId::GateRead => HostCallDescriptor {
            id,
            effect: EffectClass::HostQuery,
            determinism: DeterminismClass::HostBound,
            returns_value: true,
            stability: HostCallStability::StableV1,
        },
        HostCallId::GateWrite => HostCallDescriptor {
            id,
            effect: EffectClass::HostWrite,
            determinism: DeterminismClass::HostBound,
            returns_value: false,
            stability: HostCallStability::StableV1,
        },
        HostCallId::PulseEmit => HostCallDescriptor {
            id,
            effect: EffectClass::EventEmit,
            determinism: DeterminismClass::HostBound,
            returns_value: false,
            stability: HostCallStability::StableV1,
        },
        HostCallId::StateQuery => HostCallDescriptor {
            id,
            effect: EffectClass::HostQuery,
            determinism: DeterminismClass::HostBound,
            returns_value: true,
            stability: HostCallStability::PlannedPostStable,
        },
        HostCallId::StateUpdate => HostCallDescriptor {
            id,
            effect: EffectClass::HostWrite,
            determinism: DeterminismClass::HostBound,
            returns_value: false,
            stability: HostCallStability::PlannedPostStable,
        },
        HostCallId::EventPost => HostCallDescriptor {
            id,
            effect: EffectClass::EventEmit,
            determinism: DeterminismClass::HostBound,
            returns_value: false,
            stability: HostCallStability::PlannedPostStable,
        },
        HostCallId::ClockRead => HostCallDescriptor {
            id,
            effect: EffectClass::HostQuery,
            determinism: DeterminismClass::HostBound,
            returns_value: true,
            stability: HostCallStability::PlannedPostStable,
        },
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AbiValue {
    Quad(u8),
    Bool(bool),
    I32(i32),
    U32(u32),
    Fx(i32),
    F64(f64),
    Unit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbiFailureKind {
    Unavailable,
    InvalidInput,
    HostFault,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbiError {
    pub call: HostCallId,
    pub kind: AbiFailureKind,
    pub message: String,
}

impl AbiError {
    pub fn new(call: HostCallId, kind: AbiFailureKind, message: impl Into<String>) -> Self {
        Self {
            call,
            kind,
            message: message.into(),
        }
    }
}

impl core::fmt::Display for AbiError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "ABI {:?} failure [{:?}]: {}",
            self.call, self.kind, self.message
        )
    }
}

#[cfg(feature = "std")]
impl std::error::Error for AbiError {}

pub trait PrometheusHostAbi {
    fn gate_read(&mut self, device_id: u16, port: u16) -> Result<AbiValue, AbiError>;
    fn gate_write(&mut self, device_id: u16, port: u16, value: AbiValue) -> Result<(), AbiError>;
    fn pulse_emit(&mut self, signal: &str) -> Result<(), AbiError>;
    fn state_query(&mut self, key: &str) -> Result<AbiValue, AbiError>;
    fn state_update(&mut self, key: &str, value: AbiValue) -> Result<(), AbiError>;
    fn event_post(&mut self, signal: &str) -> Result<(), AbiError>;
    fn clock_read(&mut self) -> Result<u32, AbiError>;
}

#[derive(Debug, Default)]
pub struct RecordingHostAbi {
    pub reads: alloc::vec::Vec<(u16, u16)>,
    pub writes: alloc::vec::Vec<(u16, u16, AbiValue)>,
    pub pulses: alloc::vec::Vec<String>,
    pub state_queries: alloc::vec::Vec<String>,
    pub state_updates: alloc::vec::Vec<(String, AbiValue)>,
    pub event_posts: alloc::vec::Vec<String>,
    pub clock_reads: usize,
    pub next_read: AbiValue,
    pub next_state_query: AbiValue,
    pub next_clock_read: u32,
}

impl RecordingHostAbi {
    pub fn with_read_value(next_read: AbiValue) -> Self {
        Self {
            next_read,
            ..Self::default()
        }
    }

    pub fn with_state_query_value(next_state_query: AbiValue) -> Self {
        Self {
            next_state_query,
            ..Self::default()
        }
    }

    pub fn with_clock_read_value(next_clock_read: u32) -> Self {
        Self {
            next_clock_read,
            ..Self::default()
        }
    }
}

impl PrometheusHostAbi for RecordingHostAbi {
    fn gate_read(&mut self, device_id: u16, port: u16) -> Result<AbiValue, AbiError> {
        self.reads.push((device_id, port));
        Ok(self.next_read.clone())
    }

    fn gate_write(&mut self, device_id: u16, port: u16, value: AbiValue) -> Result<(), AbiError> {
        self.writes.push((device_id, port, value));
        Ok(())
    }

    fn pulse_emit(&mut self, signal: &str) -> Result<(), AbiError> {
        self.pulses.push(signal.to_string());
        Ok(())
    }

    fn state_query(&mut self, key: &str) -> Result<AbiValue, AbiError> {
        self.state_queries.push(key.to_string());
        Ok(self.next_state_query.clone())
    }

    fn state_update(&mut self, key: &str, value: AbiValue) -> Result<(), AbiError> {
        self.state_updates.push((key.to_string(), value));
        Ok(())
    }

    fn event_post(&mut self, signal: &str) -> Result<(), AbiError> {
        self.event_posts.push(signal.to_string());
        Ok(())
    }

    fn clock_read(&mut self) -> Result<u32, AbiError> {
        self.clock_reads += 1;
        Ok(self.next_clock_read)
    }
}

impl Default for AbiValue {
    fn default() -> Self {
        Self::Unit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptor_table_is_explicit() {
        assert!(descriptor_for_call(HostCallId::GateRead).returns_value);
        assert_eq!(
            descriptor_for_call(HostCallId::GateWrite).effect,
            EffectClass::HostWrite
        );
        assert_eq!(
            descriptor_for_call(HostCallId::PulseEmit).determinism,
            DeterminismClass::HostBound
        );
        assert_eq!(
            descriptor_for_call(HostCallId::StateQuery).stability,
            HostCallStability::PlannedPostStable
        );
        assert!(descriptor_for_call(HostCallId::ClockRead).returns_value);
    }

    #[test]
    fn recording_host_captures_state_query_calls() {
        let mut host = RecordingHostAbi::with_state_query_value(AbiValue::I32(7));
        let result = host.state_query("decision.mode").expect("state query");
        assert_eq!(result, AbiValue::I32(7));
        assert_eq!(host.state_queries, alloc::vec!["decision.mode".to_string()]);
    }

    #[test]
    fn recording_host_captures_state_update_calls() {
        let mut host = RecordingHostAbi::default();
        host.state_update("decision.mode", AbiValue::Bool(true))
            .expect("state update");
        assert_eq!(
            host.state_updates,
            alloc::vec![("decision.mode".to_string(), AbiValue::Bool(true))]
        );
    }

    #[test]
    fn recording_host_captures_event_post_calls() {
        let mut host = RecordingHostAbi::default();
        host.event_post("alert.raised").expect("event post");
        assert_eq!(host.event_posts, alloc::vec!["alert.raised".to_string()]);
    }

    #[test]
    fn recording_host_captures_clock_read_calls() {
        let mut host = RecordingHostAbi::with_clock_read_value(42);
        let result = host.clock_read().expect("clock read");
        assert_eq!(result, 42);
        assert_eq!(host.clock_reads, 1);
    }
}
