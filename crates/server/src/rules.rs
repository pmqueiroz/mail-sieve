use mail_sieve_core::{Action, Query};
use mail_sieve_store::rules::{self, Rule, RuleInput};

use crate::actions::{self, Selection};
use crate::error::{Error, Result};
use crate::state::AppState;

pub fn validate(input: &RuleInput) -> Result<()> {
    if input.name.trim().is_empty() {
        return Err(Error::BadRequest("rule name is required".into()));
    }
    let query = Query::parse(&input.query)?;
    if query.is_empty() {
        return Err(Error::BadRequest(
            "a rule needs a query; an empty query matches every message".into(),
        ));
    }
    let action: Action = input
        .action
        .parse()
        .map_err(|e: mail_sieve_core::UnknownAction| Error::BadRequest(e.to_string()))?;
    if action.is_destructive() {
        return Err(Error::BadRequest(
            "rules cannot permanently delete; use trash instead".into(),
        ));
    }
    if input.interval_minutes == Some(0) {
        return Err(Error::BadRequest(
            "interval must be at least one minute".into(),
        ));
    }
    Ok(())
}

pub async fn run_rule(state: &AppState, rule: &Rule) -> Result<Option<i64>> {
    let query = Query::parse(&rule.query)?;
    let action: Action = rule
        .action
        .parse()
        .map_err(|e: mail_sieve_core::UnknownAction| Error::BadRequest(e.to_string()))?;
    let rule_id = rule.id;
    state.write(move |tx| rules::mark_run(tx, rule_id)).await?;
    actions::enqueue(
        state,
        rule.account_id,
        Selection::Query(query),
        &action,
        &format!("rule:{}", rule.id),
    )
    .await
}

pub async fn run_after_sync(state: &AppState, account_id: i64) -> Result<()> {
    for rule in state
        .read(move |c| rules::after_sync(c, account_id))
        .await?
    {
        run_rule(state, &rule).await?;
    }
    Ok(())
}

pub async fn run_due(state: &AppState) -> Result<()> {
    let now = mail_sieve_store::now();
    for rule in state.read(move |c| rules::due(c, now)).await? {
        if let Err(error) = run_rule(state, &rule).await {
            tracing::warn!(%error, rule_id = rule.id, "scheduled rule failed");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(query: &str, action: &str) -> RuleInput {
        RuleInput {
            account_id: 1,
            name: "newsletters".into(),
            query: query.into(),
            action: action.into(),
            enabled: true,
            interval_minutes: Some(60),
            run_after_sync: false,
        }
    }

    #[test]
    fn rejects_unsafe_rules() {
        assert!(validate(&input("has:unsubscribe older_than:30d", "archive")).is_ok());
        assert!(validate(&input("", "archive")).is_err());
        assert!(validate(&input("from:x@y.com", "delete")).is_err());
        assert!(validate(&input("from:x@y.com", "explode")).is_err());
        assert!(validate(&input("bogus:field", "archive")).is_err());
    }
}
