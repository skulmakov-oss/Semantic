//! ARG v3 policy authority and deterministic budget transitions.
//!
//! This module deliberately keeps authorization separate from VM accounting:
//! the host constructs and validates an immutable policy, while the VM may
//! consume a governor derived from it. Guest code has no constructor path.

use sm_format::sha256::sha256;
use sm_runtime_core::{ExecutionConfig, ExecutionContext, RuntimeQuotas};

const DOMAIN: &[u8] = b"SEMANTIC-ARG-B-POLICY-V1\0";
pub const POLICY_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ArgProfile {
    V2 = 0,
    God = 1,
    Custom = 2,
    CustomAdaptive = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ArgBudgets {
    pub steps: u64,
    pub calls: u64,
    pub frames: u64,
    pub stack_depth: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArgGrowth {
    pub enabled: bool,
    pub minimum_increment: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArgPolicy {
    schema_version: u16,
    profile: ArgProfile,
    context: ExecutionContext,
    initial: ArgBudgets,
    hard_max: ArgBudgets,
    growth: ArgGrowth,
    identity: [u8; 32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgPolicyError {
    UnsupportedSchema(u16),
    ZeroBudget,
    InitialExceedsMaximum,
    FramesBelowStackDepth,
    AdaptiveWithoutGrowth,
    IdentityMismatch,
    IntegerOverflow,
}

impl ArgPolicy {
    pub fn v2() -> Self {
        Self::from_parts(
            ArgProfile::V2,
            ExecutionContext::VerifiedLocal,
            ArgBudgets {
                steps: 100_000,
                calls: 16_384,
                frames: 256,
                stack_depth: 256,
            },
            ArgBudgets {
                steps: 100_000,
                calls: 16_384,
                frames: 256,
                stack_depth: 256,
            },
            ArgGrowth {
                enabled: false,
                minimum_increment: 0,
            },
        )
        .expect("v2 policy is valid")
    }

    pub fn god() -> Self {
        Self::from_parts(
            ArgProfile::God,
            ExecutionContext::VerifiedLocal,
            ArgBudgets {
                steps: 40_000_000,
                calls: 100_000,
                frames: 512,
                stack_depth: 512,
            },
            ArgBudgets {
                steps: 40_000_000,
                calls: 100_000,
                frames: 512,
                stack_depth: 512,
            },
            ArgGrowth {
                enabled: false,
                minimum_increment: 0,
            },
        )
        .expect("god policy is valid")
    }

    pub fn custom(
        initial: ArgBudgets,
        hard_max: ArgBudgets,
        adaptive: bool,
        minimum_increment: u64,
    ) -> Result<Self, ArgPolicyError> {
        Self::from_parts(
            if adaptive {
                ArgProfile::CustomAdaptive
            } else {
                ArgProfile::Custom
            },
            ExecutionContext::VerifiedLocal,
            initial,
            hard_max,
            ArgGrowth {
                enabled: adaptive,
                minimum_increment,
            },
        )
    }

    fn from_parts(
        profile: ArgProfile,
        context: ExecutionContext,
        initial: ArgBudgets,
        hard_max: ArgBudgets,
        growth: ArgGrowth,
    ) -> Result<Self, ArgPolicyError> {
        if [
            initial.steps,
            initial.calls,
            initial.frames,
            initial.stack_depth,
            hard_max.steps,
            hard_max.calls,
            hard_max.frames,
            hard_max.stack_depth,
        ]
        .contains(&0)
        {
            return Err(ArgPolicyError::ZeroBudget);
        }
        if initial.steps > hard_max.steps
            || initial.calls > hard_max.calls
            || initial.frames > hard_max.frames
            || initial.stack_depth > hard_max.stack_depth
        {
            return Err(ArgPolicyError::InitialExceedsMaximum);
        }
        if initial.frames != initial.stack_depth || hard_max.frames != hard_max.stack_depth {
            return Err(ArgPolicyError::FramesBelowStackDepth);
        }
        if profile == ArgProfile::CustomAdaptive
            && (!growth.enabled || growth.minimum_increment == 0)
        {
            return Err(ArgPolicyError::AdaptiveWithoutGrowth);
        }
        let mut policy = Self {
            schema_version: POLICY_SCHEMA_VERSION,
            profile,
            context,
            initial,
            hard_max,
            growth,
            identity: [0; 32],
        };
        policy.identity = sha256(&policy.canonical_bytes_without_identity());
        Ok(policy)
    }

    pub fn validate(&self) -> Result<(), ArgPolicyError> {
        if self.schema_version != POLICY_SCHEMA_VERSION {
            return Err(ArgPolicyError::UnsupportedSchema(self.schema_version));
        }
        let expected = sha256(&self.canonical_bytes_without_identity());
        if expected != self.identity {
            return Err(ArgPolicyError::IdentityMismatch);
        }
        Ok(())
    }

    pub fn profile(&self) -> ArgProfile {
        self.profile
    }
    pub fn context(&self) -> ExecutionContext {
        self.context
    }
    pub fn initial(&self) -> ArgBudgets {
        self.initial
    }
    pub fn hard_max(&self) -> ArgBudgets {
        self.hard_max
    }
    pub fn growth(&self) -> ArgGrowth {
        self.growth
    }
    pub fn identity(&self) -> [u8; 32] {
        self.identity
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = self.canonical_bytes_without_identity();
        bytes.extend_from_slice(&self.identity);
        bytes
    }

    fn canonical_bytes_without_identity(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(2 + 2 + 1 + 1 + 8 * 8 + 1 + 8);
        out.extend_from_slice(DOMAIN);
        out.extend_from_slice(&self.schema_version.to_le_bytes());
        out.push(self.profile as u8);
        out.push(context_tag(self.context));
        encode_budgets(&mut out, self.initial);
        encode_budgets(&mut out, self.hard_max);
        out.push(u8::from(self.growth.enabled));
        out.extend_from_slice(&self.growth.minimum_increment.to_le_bytes());
        out
    }

    pub fn execution_config(&self) -> Result<ExecutionConfig, ArgPolicyError> {
        self.validate()?;
        let base = RuntimeQuotas::verified_local();
        let quotas = RuntimeQuotas {
            max_steps: to_usize(self.initial.steps)?,
            max_calls: to_usize(self.initial.calls)?,
            max_frames: to_usize(self.initial.frames)?,
            max_stack_depth: to_usize(self.initial.stack_depth)?,
            ..base
        };
        Ok(ExecutionConfig::new(self.context, quotas))
    }
}

fn encode_budgets(out: &mut Vec<u8>, b: ArgBudgets) {
    out.extend_from_slice(&b.steps.to_le_bytes());
    out.extend_from_slice(&b.calls.to_le_bytes());
    out.extend_from_slice(&b.frames.to_le_bytes());
    out.extend_from_slice(&b.stack_depth.to_le_bytes());
}

fn context_tag(context: ExecutionContext) -> u8 {
    match context {
        ExecutionContext::PureCompute => 0,
        ExecutionContext::VerifiedLocal => 1,
        ExecutionContext::RuleExecution => 2,
        ExecutionContext::KernelBound => 3,
    }
}

fn to_usize(value: u64) -> Result<usize, ArgPolicyError> {
    usize::try_from(value).map_err(|_| ArgPolicyError::IntegerOverflow)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdaptiveGovernor {
    pub budgets: ArgBudgets,
    pub used: ArgBudgets,
    pub hard_max: ArgBudgets,
    pub growth: ArgGrowth,
    pub events: Vec<AdaptiveEvent>,
    effect_calls: u64,
    effect_calls_budget: u64,
    effect_calls_hard_max: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdaptiveEvent {
    pub resource: ArgResource,
    pub used_before: u64,
    pub budget_before: u64,
    pub budget_after: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgResource {
    Steps,
    Calls,
    Frames,
    StackDepth,
    EffectCalls,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdaptiveError {
    Exhausted,
    Overflow,
}

impl AdaptiveGovernor {
    pub fn from_policy(policy: &ArgPolicy) -> Result<Self, ArgPolicyError> {
        policy.validate()?;
        let base = RuntimeQuotas::verified_local();
        Ok(Self {
            budgets: policy.initial,
            used: ArgBudgets::default(),
            hard_max: policy.hard_max,
            growth: policy.growth,
            events: Vec::new(),
            effect_calls: 0,
            effect_calls_budget: base.max_effect_calls as u64,
            effect_calls_hard_max: base.max_effect_calls as u64,
        })
    }

    pub fn charge(
        &mut self,
        resource: ArgResource,
        amount: u64,
    ) -> Result<Option<AdaptiveEvent>, AdaptiveError> {
        let used = self.used_value(resource);
        let budget = self.budget_value(resource);
        let required = used.checked_add(amount).ok_or(AdaptiveError::Overflow)?;
        if required <= budget {
            self.set_used(resource, required);
            return Ok(None);
        }
        if !self.growth.enabled {
            return Err(AdaptiveError::Exhausted);
        }
        let deficit = required - budget;
        let half = budget.checked_add(1).ok_or(AdaptiveError::Overflow)? / 2;
        let increment = self.growth.minimum_increment.max(deficit).max(half);
        let hard = self.hard_value(resource);
        let after = budget
            .checked_add(increment)
            .ok_or(AdaptiveError::Overflow)?
            .min(hard);
        if after < required {
            return Err(AdaptiveError::Exhausted);
        }
        self.set_budget(resource, after);
        self.set_used(resource, required);
        if matches!(resource, ArgResource::Frames | ArgResource::StackDepth) {
            self.budgets.frames = self.budgets.frames.max(after);
            self.budgets.stack_depth = self.budgets.stack_depth.max(after);
        }
        let event = AdaptiveEvent {
            resource,
            used_before: used,
            budget_before: budget,
            budget_after: after,
        };
        self.events.push(event);
        Ok(Some(event))
    }

    pub fn used(&self, resource: ArgResource) -> u64 {
        self.used_value(resource)
    }

    pub fn budget(&self, resource: ArgResource) -> u64 {
        self.budget_value(resource)
    }

    fn used_value(&self, r: ArgResource) -> u64 {
        match r {
            ArgResource::Steps => self.used.steps,
            ArgResource::Calls => self.used.calls,
            ArgResource::Frames => self.used.frames,
            ArgResource::StackDepth => self.used.stack_depth,
            ArgResource::EffectCalls => self.effect_calls,
        }
    }
    fn budget_value(&self, r: ArgResource) -> u64 {
        match r {
            ArgResource::Steps => self.budgets.steps,
            ArgResource::Calls => self.budgets.calls,
            ArgResource::Frames => self.budgets.frames,
            ArgResource::StackDepth => self.budgets.stack_depth,
            ArgResource::EffectCalls => self.effect_calls_budget,
        }
    }
    fn hard_value(&self, r: ArgResource) -> u64 {
        match r {
            ArgResource::Steps => self.hard_max.steps,
            ArgResource::Calls => self.hard_max.calls,
            ArgResource::Frames => self.hard_max.frames,
            ArgResource::StackDepth => self.hard_max.stack_depth,
            ArgResource::EffectCalls => self.effect_calls_hard_max,
        }
    }
    fn set_used(&mut self, r: ArgResource, v: u64) {
        match r {
            ArgResource::Steps => self.used.steps = v,
            ArgResource::Calls => self.used.calls = v,
            ArgResource::Frames => self.used.frames = v,
            ArgResource::StackDepth => self.used.stack_depth = v,
            ArgResource::EffectCalls => self.effect_calls = v,
        }
    }
    fn set_budget(&mut self, r: ArgResource, v: u64) {
        match r {
            ArgResource::Steps => self.budgets.steps = v,
            ArgResource::Calls => self.budgets.calls = v,
            ArgResource::Frames => self.budgets.frames = v,
            ArgResource::StackDepth => self.budgets.stack_depth = v,
            ArgResource::EffectCalls => self.effect_calls_budget = v,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small(adaptive: bool) -> ArgPolicy {
        ArgPolicy::custom(
            ArgBudgets {
                steps: 10,
                calls: 10,
                frames: 2,
                stack_depth: 2,
            },
            ArgBudgets {
                steps: 100,
                calls: 100,
                frames: 8,
                stack_depth: 8,
            },
            adaptive,
            1,
        )
        .unwrap()
    }

    #[test]
    fn policy_identity_is_deterministic_and_validates() {
        let a = small(true);
        let b = small(true);
        assert_eq!(a.identity(), b.identity());
        assert_eq!(a.canonical_bytes(), b.canonical_bytes());
        a.validate().unwrap();
    }

    #[test]
    fn fixed_profiles_have_exact_bounds() {
        assert_eq!(
            ArgPolicy::god().initial(),
            ArgBudgets {
                steps: 40_000_000,
                calls: 100_000,
                frames: 512,
                stack_depth: 512
            }
        );
        assert_eq!(
            ArgPolicy::god()
                .execution_config()
                .unwrap()
                .quotas
                .max_frames,
            512
        );
    }

    #[test]
    fn adaptive_growth_is_deterministic_and_bounded() {
        let mut a = AdaptiveGovernor::from_policy(&small(true)).unwrap();
        let mut b = a.clone();
        assert!(a.charge(ArgResource::Steps, 11).unwrap().is_some());
        assert!(b.charge(ArgResource::Steps, 11).unwrap().is_some());
        assert_eq!(a, b);
        assert!(a.budgets.steps <= 100);
    }

    #[test]
    fn fixed_policy_exhaustion_is_explicit() {
        let mut g = AdaptiveGovernor::from_policy(&small(false)).unwrap();
        assert_eq!(
            g.charge(ArgResource::Steps, 11),
            Err(AdaptiveError::Exhausted)
        );
    }

    #[test]
    fn effect_calls_use_their_own_preserved_runtime_ceiling() {
        let mut g = AdaptiveGovernor::from_policy(&small(true)).unwrap();
        assert_eq!(g.budget(ArgResource::EffectCalls), 1_024);
        assert_eq!(g.charge(ArgResource::EffectCalls, 1), Ok(None));
        assert_eq!(g.used(ArgResource::EffectCalls), 1);
        assert_eq!(g.used(ArgResource::Calls), 0);
        assert_eq!(g.budget(ArgResource::Calls), 10);
    }

    #[test]
    fn depth_budget_is_jointly_validated() {
        let err = ArgPolicy::custom(
            ArgBudgets {
                steps: 1,
                calls: 1,
                frames: 2,
                stack_depth: 1,
            },
            ArgBudgets {
                steps: 1,
                calls: 1,
                frames: 2,
                stack_depth: 1,
            },
            false,
            0,
        )
        .unwrap_err();
        assert_eq!(err, ArgPolicyError::FramesBelowStackDepth);
    }

    #[test]
    fn adaptive_policy_is_consumed_by_real_vm_execution() {
        let source = "fn value() -> i32 { let a: i32 = 1; let b: i32 = 2; let c: i32 = a + b; return c; } fn main() { return; }";
        let bytes = sm_emit::compile_program_to_semcode(source).expect("compile");
        let token = sm_verify::verify_semcode_token(&bytes).expect("verify");
        let token = token.require_entry("value").expect("entry");
        let policy = ArgPolicy::custom(
            ArgBudgets {
                steps: 1,
                calls: 1,
                frames: 1,
                stack_depth: 1,
            },
            ArgBudgets {
                steps: 64,
                calls: 8,
                frames: 8,
                stack_depth: 8,
            },
            true,
            1,
        )
        .expect("policy");
        let (result, events) = crate::run_verified_function_semcode_with_args_and_policy_events(
            &token,
            Vec::new(),
            policy,
        )
        .expect("adaptive execution");
        assert_eq!(result, crate::Value::I32(3));
        assert!(
            events.len() >= 2,
            "real VM should record multiple growth events"
        );
    }

    #[test]
    fn fixed_profiles_are_consumed_by_real_vm_execution() {
        let source = "fn value() -> i32 { return 7; } fn main() { return; }";
        let bytes = sm_emit::compile_program_to_semcode(source).expect("compile");
        let verified = sm_verify::verify_semcode_token(&bytes).expect("verify");
        let token = verified.require_entry("value").expect("entry");

        for policy in [
            ArgPolicy::god(),
            ArgPolicy::custom(
                ArgBudgets {
                    steps: 128,
                    calls: 8,
                    frames: 4,
                    stack_depth: 4,
                },
                ArgBudgets {
                    steps: 128,
                    calls: 8,
                    frames: 4,
                    stack_depth: 4,
                },
                false,
                0,
            )
            .expect("fixed custom policy"),
        ] {
            let result = crate::run_verified_function_semcode_with_args_and_policy(
                &token,
                Vec::new(),
                policy,
            )
            .expect("fixed policy execution");
            assert_eq!(result, crate::Value::I32(7));
        }
    }
}
