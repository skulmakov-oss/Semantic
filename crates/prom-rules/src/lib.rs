#![allow(clippy::filter_map_bool_then)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use prom_state::{FactResolution, FactValue, SemanticStateStore};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RuleId(pub String);

impl RuleId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Salience(pub i32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleCondition {
    pub key: String,
    pub expected: FactValue,
}

impl RuleCondition {
    pub fn equals(key: impl Into<String>, expected: FactValue) -> Self {
        Self {
            key: key.into(),
            expected,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleStateWriteEffect {
    pub key: String,
    pub value: FactValue,
    pub context: String,
    pub reason: String,
}

impl RuleStateWriteEffect {
    pub fn new(
        key: impl Into<String>,
        value: FactValue,
        context: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            key: key.into(),
            value,
            context: context.into(),
            reason: reason.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleAuditNoteEffect {
    pub message: String,
}

impl RuleAuditNoteEffect {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleEffect {
    StateWrite(RuleStateWriteEffect),
    AuditNote(RuleAuditNoteEffect),
}

impl RuleEffect {
    pub fn state_write(
        key: impl Into<String>,
        value: FactValue,
        context: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::StateWrite(RuleStateWriteEffect::new(key, value, context, reason))
    }

    pub fn audit_note(message: impl Into<String>) -> Self {
        Self::AuditNote(RuleAuditNoteEffect::new(message))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuleEffectPlan {
    effects: Vec<RuleEffect>,
}

impl RuleEffectPlan {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_effects(effects: Vec<RuleEffect>) -> Self {
        Self { effects }
    }

    pub fn effects(&self) -> &[RuleEffect] {
        &self.effects
    }

    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleDefinition {
    pub id: RuleId,
    pub salience: Salience,
    pub conditions: Vec<RuleCondition>,
    pub effect_plan: RuleEffectPlan,
}

impl RuleDefinition {
    pub fn new(id: impl Into<String>, salience: i32, conditions: Vec<RuleCondition>) -> Self {
        Self {
            id: RuleId::new(id),
            salience: Salience(salience),
            conditions,
            effect_plan: RuleEffectPlan::new(),
        }
    }

    pub fn with_effects(mut self, effects: Vec<RuleEffect>) -> Self {
        self.effect_plan = RuleEffectPlan::from_effects(effects);
        self
    }

    pub fn effect_plan(&self) -> &RuleEffectPlan {
        &self.effect_plan
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgendaEntry {
    pub rule_id: RuleId,
    pub salience: Salience,
    pub ordinal: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Agenda {
    entries: Vec<AgendaEntry>,
}

impl Agenda {
    pub fn entries(&self) -> &[AgendaEntry] {
        &self.entries
    }

    pub fn pop_next(&mut self) -> Option<AgendaEntry> {
        if self.entries.is_empty() {
            None
        } else {
            Some(self.entries.remove(0))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleValidationCode {
    EmptyRuleId,
    DuplicateRuleId,
    EmptyConditionKey,
    EmptyConditionSet,
    EmptyEffectKey,
    EmptyEffectContext,
    EmptyEffectReason,
    EmptyAuditNoteMessage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleValidationError {
    pub code: RuleValidationCode,
    pub message: String,
}

impl RuleValidationError {
    pub fn new(code: RuleValidationCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl core::fmt::Display for RuleValidationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for RuleValidationError {}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuleEngine {
    rules: Vec<RuleDefinition>,
}

impl RuleEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn rules(&self) -> &[RuleDefinition] {
        &self.rules
    }

    pub fn register(&mut self, rule: RuleDefinition) -> Result<(), RuleValidationError> {
        validate_rule(&rule)?;
        if self.rules.iter().any(|existing| existing.id == rule.id) {
            return Err(RuleValidationError::new(
                RuleValidationCode::DuplicateRuleId,
                format!("rule '{}' is already registered", rule.id.0),
            ));
        }
        self.rules.push(rule);
        Ok(())
    }

    pub fn evaluate(&self, state: &SemanticStateStore) -> Agenda {
        let mut entries = self
            .rules
            .iter()
            .enumerate()
            .filter_map(|(ordinal, rule)| {
                rule_matches(state, rule).then(|| AgendaEntry {
                    rule_id: rule.id.clone(),
                    salience: rule.salience,
                    ordinal,
                })
            })
            .collect::<Vec<_>>();

        entries.sort_by(|left, right| {
            right
                .salience
                .cmp(&left.salience)
                .then_with(|| left.ordinal.cmp(&right.ordinal))
                .then_with(|| left.rule_id.cmp(&right.rule_id))
        });

        Agenda { entries }
    }
}

fn validate_rule(rule: &RuleDefinition) -> Result<(), RuleValidationError> {
    if rule.id.0.trim().is_empty() {
        return Err(RuleValidationError::new(
            RuleValidationCode::EmptyRuleId,
            "rule id must not be empty",
        ));
    }
    if rule.conditions.is_empty() {
        return Err(RuleValidationError::new(
            RuleValidationCode::EmptyConditionSet,
            "rule must define at least one condition",
        ));
    }
    if rule
        .conditions
        .iter()
        .any(|condition| condition.key.trim().is_empty())
    {
        return Err(RuleValidationError::new(
            RuleValidationCode::EmptyConditionKey,
            "rule condition key must not be empty",
        ));
    }
    for effect in rule.effect_plan.effects() {
        validate_effect(effect)?;
    }
    Ok(())
}

fn validate_effect(effect: &RuleEffect) -> Result<(), RuleValidationError> {
    match effect {
        RuleEffect::StateWrite(effect) => {
            if effect.key.trim().is_empty() {
                return Err(RuleValidationError::new(
                    RuleValidationCode::EmptyEffectKey,
                    "rule state-write effect key must not be empty",
                ));
            }
            if effect.context.trim().is_empty() {
                return Err(RuleValidationError::new(
                    RuleValidationCode::EmptyEffectContext,
                    "rule state-write effect context must not be empty",
                ));
            }
            if effect.reason.trim().is_empty() {
                return Err(RuleValidationError::new(
                    RuleValidationCode::EmptyEffectReason,
                    "rule state-write effect reason must not be empty",
                ));
            }
        }
        RuleEffect::AuditNote(effect) => {
            if effect.message.trim().is_empty() {
                return Err(RuleValidationError::new(
                    RuleValidationCode::EmptyAuditNoteMessage,
                    "rule audit-note effect message must not be empty",
                ));
            }
        }
    }
    Ok(())
}

fn rule_matches(state: &SemanticStateStore, rule: &RuleDefinition) -> bool {
    rule.conditions.iter().all(|condition| {
        state
            .get(&condition.key)
            .map(|record| matches!(&record.resolution, FactResolution::Certain(value) if value == &condition.expected))
            .unwrap_or(false)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use prom_state::{ContextWindow, StateUpdate};

    fn seeded_state() -> SemanticStateStore {
        let mut state = SemanticStateStore::new();
        state
            .apply(StateUpdate::new(
                "fact.alpha",
                FactResolution::Certain(FactValue::Bool(true)),
                ContextWindow::new("root"),
                "seed alpha",
            ))
            .expect("seed alpha");
        state
            .apply(StateUpdate::new(
                "fact.beta",
                FactResolution::Certain(FactValue::I32(2)),
                ContextWindow::new("root"),
                "seed beta",
            ))
            .expect("seed beta");
        state
    }

    #[test]
    fn engine_rejects_duplicate_rule_ids() {
        let mut engine = RuleEngine::new();
        engine
            .register(RuleDefinition::new(
                "rule.alpha",
                10,
                vec![RuleCondition::equals("fact.alpha", FactValue::Bool(true))],
            ))
            .expect("register first");
        let err = engine
            .register(RuleDefinition::new(
                "rule.alpha",
                1,
                vec![RuleCondition::equals("fact.beta", FactValue::I32(2))],
            ))
            .expect_err("duplicate id must reject");
        assert_eq!(err.code, RuleValidationCode::DuplicateRuleId);
    }

    #[test]
    fn new_rule_definition_starts_with_empty_effect_plan() {
        let rule = RuleDefinition::new(
            "rule.alpha",
            10,
            vec![RuleCondition::equals("fact.alpha", FactValue::Bool(true))],
        );

        assert!(rule.effect_plan().is_empty());
        assert!(rule.effect_plan().effects().is_empty());
    }

    #[test]
    fn with_effects_preserves_declared_effect_order() {
        let rule = RuleDefinition::new(
            "rule.alpha",
            10,
            vec![RuleCondition::equals("fact.alpha", FactValue::Bool(true))],
        )
        .with_effects(vec![
            RuleEffect::state_write("fact.beta", FactValue::I32(2), "window.beta", "derive beta"),
            RuleEffect::audit_note("rule.alpha emitted note"),
        ]);

        assert!(matches!(
            &rule.effect_plan().effects()[0],
            RuleEffect::StateWrite(effect)
                if effect.key == "fact.beta"
                    && effect.value == FactValue::I32(2)
                    && effect.context == "window.beta"
                    && effect.reason == "derive beta"
        ));
        assert!(matches!(
            &rule.effect_plan().effects()[1],
            RuleEffect::AuditNote(effect)
                if effect.message == "rule.alpha emitted note"
        ));
    }

    #[test]
    fn engine_rejects_invalid_effect_payloads() {
        let mut engine = RuleEngine::new();
        let err = engine
            .register(
                RuleDefinition::new(
                    "rule.alpha",
                    10,
                    vec![RuleCondition::equals("fact.alpha", FactValue::Bool(true))],
                )
                .with_effects(vec![RuleEffect::audit_note("  ")]),
            )
            .expect_err("invalid effect must reject");

        assert_eq!(err.code, RuleValidationCode::EmptyAuditNoteMessage);
    }

    #[test]
    fn agenda_orders_by_salience_then_registration_order() {
        let state = seeded_state();
        let mut engine = RuleEngine::new();
        engine
            .register(RuleDefinition::new(
                "rule.low",
                1,
                vec![RuleCondition::equals("fact.alpha", FactValue::Bool(true))],
            ))
            .expect("register low");
        engine
            .register(RuleDefinition::new(
                "rule.high",
                5,
                vec![RuleCondition::equals("fact.alpha", FactValue::Bool(true))],
            ))
            .expect("register high");
        engine
            .register(RuleDefinition::new(
                "rule.high.second",
                5,
                vec![RuleCondition::equals("fact.beta", FactValue::I32(2))],
            ))
            .expect("register second high");

        let agenda = engine.evaluate(&state);
        let ids = agenda
            .entries()
            .iter()
            .map(|entry| entry.rule_id.0.as_str())
            .collect::<Vec<_>>();
        assert_eq!(ids, vec!["rule.high", "rule.high.second", "rule.low"]);
    }

    #[test]
    fn uncertain_state_does_not_activate_certain_match_rule() {
        let mut state = SemanticStateStore::new();
        state
            .apply(StateUpdate::new(
                "fact.alpha",
                FactResolution::Uncertain(vec![FactValue::Bool(true), FactValue::Bool(false)]),
                ContextWindow::new("root"),
                "uncertain alpha",
            ))
            .expect("seed uncertain");

        let mut engine = RuleEngine::new();
        engine
            .register(RuleDefinition::new(
                "rule.alpha",
                10,
                vec![RuleCondition::equals("fact.alpha", FactValue::Bool(true))],
            ))
            .expect("register");

        assert!(engine.evaluate(&state).entries().is_empty());
    }

    #[test]
    fn agenda_is_deterministic_for_same_state_and_rule_set() {
        let state = seeded_state();
        let mut engine = RuleEngine::new();
        for rule in [
            RuleDefinition::new(
                "rule.a",
                3,
                vec![RuleCondition::equals("fact.alpha", FactValue::Bool(true))],
            ),
            RuleDefinition::new(
                "rule.b",
                3,
                vec![RuleCondition::equals("fact.beta", FactValue::I32(2))],
            ),
        ] {
            engine.register(rule).expect("register");
        }

        let first = engine.evaluate(&state);
        let second = engine.evaluate(&state);
        assert_eq!(first, second);
    }
}
