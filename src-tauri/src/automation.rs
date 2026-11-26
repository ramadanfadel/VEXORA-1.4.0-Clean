//! Small rule engine for repetitive maintenance. Destructive actions are
//! never implicit: a rule with `dry_run = true` (the default) is evaluated
//! and reported but performs no system change.

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub dry_run: bool,
    pub action: String, // "clean_temp" | "trim_memory"
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Outcome {
    pub rule_id: String,
    pub would_run: bool,
    pub executes: bool,
    pub detail: String,
}

fn valid_action(action: &str) -> bool {
    matches!(action, "clean_temp" | "trim_memory")
}

pub fn sanitize(rules: &mut Vec<Rule>) {
    rules.retain(|r| {
        valid_action(&r.action)
            && !r.id.is_empty()
            && r.id.len() <= 64
            && r.id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    });
    for r in rules.iter_mut().take(20) {
        r.dry_run = true; // never let imported JSON enable destructive automation
    }
    rules.truncate(20);
}

/// Evaluate rules against a memory percentage. Pure: no side effects.
pub fn evaluate(rules: &[Rule], mem_used_pct: f32) -> Vec<Outcome> {
    rules
        .iter()
        .filter(|r| r.enabled)
        .map(|r| {
            let triggered = match r.action.as_str() {
                "clean_temp" => true,
                "trim_memory" => mem_used_pct >= 80.0,
                _ => false,
            };
            Outcome {
                rule_id: r.id.clone(),
                would_run: triggered,
                executes: triggered && !r.dry_run,
                detail: if triggered {
                    format!("{} ({})", r.action, if r.dry_run { "dry-run" } else { "live" })
                } else {
                    "not triggered".into()
                },
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(id: &str, action: &str, dry: bool) -> Rule {
        Rule {
            id: id.into(),
            name: id.into(),
            enabled: true,
            dry_run: dry,
            action: action.into(),
        }
    }

    #[test]
    fn dry_run_never_executes() {
        let out = evaluate(&[rule("a", "clean_temp", true)], 90.0);
        assert!(out[0].would_run);
        assert!(!out[0].executes);
    }

    #[test]
    fn memory_rule_triggers_only_when_high() {
        let r = rule("b", "trim_memory", false);
        assert!(!evaluate(&[r.clone()], 50.0)[0].would_run);
        assert!(evaluate(&[r], 85.0)[0].executes);
    }

    #[test]
    fn imported_rules_are_forced_to_dry_run() {
        let mut rules = vec![rule("a", "clean_temp", false), rule("bad id!", "wipe", false)];
        sanitize(&mut rules);
        assert_eq!(rules.len(), 1);
        assert!(rules[0].dry_run);
    }
}
