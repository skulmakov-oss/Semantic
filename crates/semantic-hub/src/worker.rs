//! Worker health state machine and restart policy.
//!
//! In-process execution is not process isolation: `HubWorkerState` and its
//! transitions exist to contain and classify failures inside one process,
//! not to claim hard memory-corruption isolation.

use std::fmt;

/// Health state of one registered tool worker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HubWorkerState {
    Registered,
    Starting,
    Ready,
    Busy,
    Degraded,
    Restarting,
    Quarantined,
    Disabled,
    Stopped,
}

impl HubWorkerState {
    pub const fn as_str(&self) -> &'static str {
        match self {
            HubWorkerState::Registered => "Registered",
            HubWorkerState::Starting => "Starting",
            HubWorkerState::Ready => "Ready",
            HubWorkerState::Busy => "Busy",
            HubWorkerState::Degraded => "Degraded",
            HubWorkerState::Restarting => "Restarting",
            HubWorkerState::Quarantined => "Quarantined",
            HubWorkerState::Disabled => "Disabled",
            HubWorkerState::Stopped => "Stopped",
        }
    }

    /// Whether a worker in this state may accept a new dispatched request.
    pub const fn accepts_dispatch(&self) -> bool {
        matches!(self, HubWorkerState::Ready | HubWorkerState::Degraded)
    }

    /// Whether `self -> next` is an allowed transition. Kept as an explicit
    /// table (not "anything goes") so tests can pin the full state machine
    /// and a bug can't silently create an unreachable or illegal state.
    pub const fn can_transition_to(&self, next: HubWorkerState) -> bool {
        use HubWorkerState::*;
        matches!(
            (*self, next),
            (Registered, Starting)
                | (Registered, Disabled)
                | (Starting, Ready)
                | (Starting, Degraded)
                | (Starting, Quarantined)
                | (Ready, Busy)
                | (Ready, Degraded)
                | (Ready, Disabled)
                | (Ready, Stopped)
                | (Busy, Ready)
                | (Busy, Degraded)
                | (Busy, Quarantined)
                | (Busy, Stopped)
                | (Degraded, Ready)
                | (Degraded, Restarting)
                | (Degraded, Quarantined)
                | (Degraded, Disabled)
                | (Restarting, Ready)
                | (Restarting, Degraded)
                | (Restarting, Quarantined)
                | (Quarantined, Disabled)
                | (Quarantined, Restarting)
                | (Disabled, Starting)
                | (Disabled, Stopped)
        )
    }
}

impl fmt::Display for HubWorkerState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Supervisor policy for restarting a worker after failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HubRestartPolicy {
    Never,
    OnCrash,
    OnTransientFailure,
    Always,
    ManualOnly,
}

/// Attempting an illegal state transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IllegalWorkerTransition {
    pub from: HubWorkerState,
    pub to: HubWorkerState,
}

impl fmt::Display for IllegalWorkerTransition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "illegal worker state transition {} -> {}",
            self.from, self.to
        )
    }
}

/// Escalation thresholds the supervisor applies after repeated failures.
/// Kept explicit and finite so a worker can never restart forever without
/// eventually reaching a visible, auditable quarantine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HubSupervisionPolicy {
    pub restart_policy: HubRestartPolicy,
    pub max_restarts_before_quarantine: u32,
    pub max_protocol_violations_before_quarantine: u32,
}

impl HubSupervisionPolicy {
    pub const fn conservative_default() -> Self {
        Self {
            restart_policy: HubRestartPolicy::OnCrash,
            max_restarts_before_quarantine: 3,
            max_protocol_violations_before_quarantine: 1,
        }
    }
}

/// Tracks one worker's health state and failure counters, and is the sole
/// authority for whether a transition is applied.
#[derive(Debug, Clone)]
pub struct HubWorkerHealth {
    state: HubWorkerState,
    policy: HubSupervisionPolicy,
    crash_count: u32,
    protocol_violation_count: u32,
}

impl HubWorkerHealth {
    pub const fn new(policy: HubSupervisionPolicy) -> Self {
        Self {
            state: HubWorkerState::Registered,
            policy,
            crash_count: 0,
            protocol_violation_count: 0,
        }
    }

    pub const fn state(&self) -> HubWorkerState {
        self.state
    }

    pub const fn crash_count(&self) -> u32 {
        self.crash_count
    }

    fn transition(&mut self, next: HubWorkerState) -> Result<(), IllegalWorkerTransition> {
        if self.state.can_transition_to(next) {
            self.state = next;
            Ok(())
        } else {
            Err(IllegalWorkerTransition {
                from: self.state,
                to: next,
            })
        }
    }

    pub fn mark_starting(&mut self) -> Result<(), IllegalWorkerTransition> {
        self.transition(HubWorkerState::Starting)
    }

    pub fn mark_ready(&mut self) -> Result<(), IllegalWorkerTransition> {
        self.transition(HubWorkerState::Ready)
    }

    pub fn mark_busy(&mut self) -> Result<(), IllegalWorkerTransition> {
        self.transition(HubWorkerState::Busy)
    }

    /// Report a crash. Escalates to `Quarantined` once the configured
    /// threshold is reached instead of allowing an unbounded restart loop;
    /// otherwise returns to `Degraded` so the caller can decide whether to
    /// restart per `HubRestartPolicy`.
    pub fn report_crash(&mut self) -> Result<(), IllegalWorkerTransition> {
        self.crash_count += 1;
        if self.policy.restart_policy == HubRestartPolicy::Never
            || self.crash_count >= self.policy.max_restarts_before_quarantine
        {
            self.transition(HubWorkerState::Quarantined)
        } else {
            self.transition(HubWorkerState::Degraded)
        }
    }

    /// Report a protocol violation (malformed reply). Threshold is
    /// deliberately stricter than crash tolerance: a tool that violates the
    /// wire contract is untrustworthy in a way a transient crash is not.
    pub fn report_protocol_violation(&mut self) -> Result<(), IllegalWorkerTransition> {
        self.protocol_violation_count += 1;
        if self.protocol_violation_count >= self.policy.max_protocol_violations_before_quarantine {
            self.transition(HubWorkerState::Quarantined)
        } else {
            self.transition(HubWorkerState::Degraded)
        }
    }

    pub fn restart(&mut self) -> Result<(), IllegalWorkerTransition> {
        self.transition(HubWorkerState::Restarting)?;
        self.transition(HubWorkerState::Ready)
    }

    pub fn disable(&mut self) -> Result<(), IllegalWorkerTransition> {
        self.transition(HubWorkerState::Disabled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_worker_starts_registered_and_does_not_accept_dispatch() {
        let health = HubWorkerHealth::new(HubSupervisionPolicy::conservative_default());
        assert_eq!(health.state(), HubWorkerState::Registered);
        assert!(!health.state().accepts_dispatch());
    }

    #[test]
    fn happy_path_registered_to_ready_to_busy_to_ready() {
        let mut health = HubWorkerHealth::new(HubSupervisionPolicy::conservative_default());
        health.mark_starting().unwrap();
        health.mark_ready().unwrap();
        assert!(health.state().accepts_dispatch());
        health.mark_busy().unwrap();
        assert!(!health.state().accepts_dispatch());
        health.mark_ready().unwrap();
        assert_eq!(health.state(), HubWorkerState::Ready);
    }

    #[test]
    fn illegal_transition_is_rejected_and_state_is_unchanged() {
        let mut health = HubWorkerHealth::new(HubSupervisionPolicy::conservative_default());
        let err = health.mark_busy().unwrap_err();
        assert_eq!(err.from, HubWorkerState::Registered);
        assert_eq!(err.to, HubWorkerState::Busy);
        assert_eq!(health.state(), HubWorkerState::Registered);
    }

    #[test]
    fn repeated_crashes_escalate_to_quarantine_and_stop_before_infinite_restart() {
        let mut health = HubWorkerHealth::new(HubSupervisionPolicy {
            restart_policy: HubRestartPolicy::OnCrash,
            max_restarts_before_quarantine: 2,
            max_protocol_violations_before_quarantine: 5,
        });
        health.mark_starting().unwrap();
        health.mark_ready().unwrap();
        health.mark_busy().unwrap();

        health.report_crash().unwrap(); // 1st crash -> Degraded
        assert_eq!(health.state(), HubWorkerState::Degraded);
        health.restart().unwrap();
        health.mark_busy().unwrap();

        health.report_crash().unwrap(); // 2nd crash reaches threshold -> Quarantined
        assert_eq!(health.state(), HubWorkerState::Quarantined);
        assert_eq!(health.crash_count(), 2);

        // Quarantine is terminal except for explicit manual restart/disable.
        assert!(health.mark_ready().is_err());
    }

    #[test]
    fn a_single_protocol_violation_quarantines_under_the_conservative_default_policy() {
        let mut health = HubWorkerHealth::new(HubSupervisionPolicy::conservative_default());
        health.mark_starting().unwrap();
        health.mark_ready().unwrap();
        health.mark_busy().unwrap();
        health.report_protocol_violation().unwrap();
        assert_eq!(health.state(), HubWorkerState::Quarantined);
    }

    #[test]
    fn quarantined_worker_can_be_disabled_but_not_silently_revived() {
        let mut health = HubWorkerHealth::new(HubSupervisionPolicy {
            restart_policy: HubRestartPolicy::Never,
            max_restarts_before_quarantine: 1,
            max_protocol_violations_before_quarantine: 1,
        });
        health.mark_starting().unwrap();
        health.mark_ready().unwrap();
        health.mark_busy().unwrap();
        health.report_crash().unwrap();
        assert_eq!(health.state(), HubWorkerState::Quarantined);
        health.disable().unwrap();
        assert_eq!(health.state(), HubWorkerState::Disabled);
    }
}
