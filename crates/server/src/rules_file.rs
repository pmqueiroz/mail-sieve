use std::collections::HashSet;

use mail_sieve_store::Connection;
use mail_sieve_store::rules::{self, Rule, RuleInput};
use serde::{Deserialize, Serialize};
use serde_yaml_ng::Value;

use crate::error::{Error, Result};
use crate::rules::validate;

pub const VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OnConflict {
    #[default]
    Skip,
    Replace,
}

#[derive(Serialize)]
struct ExportFile {
    version: u32,
    rules: Vec<Entry>,
}

#[derive(Deserialize)]
struct ImportFile {
    version: u32,
    #[serde(default)]
    rules: Vec<Value>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    name: String,
    query: String,
    action: String,
    #[serde(default)]
    run_after_sync: bool,
    #[serde(default)]
    interval_minutes: Option<u32>,
    #[serde(default = "enabled_by_default")]
    enabled: bool,
}

fn enabled_by_default() -> bool {
    true
}

#[derive(Debug, Clone, Serialize)]
pub struct InvalidRule {
    pub name: String,
    pub error: String,
}

#[derive(Debug, Serialize)]
pub struct ImportReport {
    pub added: Vec<String>,
    pub existing: Vec<String>,
    pub invalid: Vec<InvalidRule>,
    pub applied: bool,
}

pub struct RulesFile {
    account_id: i64,
    valid: Vec<RuleInput>,
    invalid: Vec<InvalidRule>,
}

pub struct ImportPlan {
    added: Vec<RuleInput>,
    existing: Vec<(i64, RuleInput)>,
    invalid: Vec<InvalidRule>,
}

pub fn export(rules: &[Rule]) -> Result<String> {
    let file = ExportFile {
        version: VERSION,
        rules: rules
            .iter()
            .map(|rule| Entry {
                name: rule.name.clone(),
                query: rule.query.clone(),
                action: rule.action.clone(),
                run_after_sync: rule.run_after_sync,
                interval_minutes: rule.interval_minutes,
                enabled: rule.enabled,
            })
            .collect(),
    };
    serde_yaml_ng::to_string(&file).map_err(|e| Error::Internal(e.to_string()))
}

pub fn parse(yaml: &str, account_id: i64) -> Result<RulesFile> {
    let file: ImportFile = serde_yaml_ng::from_str(yaml)
        .map_err(|e| Error::BadRequest(format!("not a valid rules file: {e}")))?;
    if file.version != VERSION {
        return Err(Error::BadRequest(format!(
            "unsupported rules file version {}; this server reads version {VERSION}",
            file.version
        )));
    }
    let mut parsed = RulesFile {
        account_id,
        valid: Vec::new(),
        invalid: Vec::new(),
    };
    let mut seen = HashSet::new();
    for (index, value) in file.rules.into_iter().enumerate() {
        let label = entry_label(&value, index);
        match parse_entry(value, account_id) {
            Ok(input) if seen.insert(input.name.clone()) => parsed.valid.push(input),
            Ok(input) => parsed.invalid.push(InvalidRule {
                name: input.name,
                error: "another rule in the file has the same name".into(),
            }),
            Err(error) => parsed.invalid.push(InvalidRule { name: label, error }),
        }
    }
    Ok(parsed)
}

impl RulesFile {
    pub fn plan(self, current: &[Rule]) -> ImportPlan {
        let mut plan = ImportPlan {
            added: Vec::new(),
            existing: Vec::new(),
            invalid: self.invalid,
        };
        for input in self.valid {
            match current.iter().find(|rule| rule.name == input.name) {
                Some(rule) => plan.existing.push((rule.id, input)),
                None => plan.added.push(input),
            }
        }
        plan
    }
}

fn entry_label(value: &Value, index: usize) -> String {
    value
        .get("name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map_or_else(|| format!("Rule {}", index + 1), str::to_string)
}

fn parse_entry(value: Value, account_id: i64) -> std::result::Result<RuleInput, String> {
    let entry: Entry = serde_yaml_ng::from_value(value).map_err(|e| e.to_string())?;
    let input = RuleInput {
        account_id,
        name: entry.name.trim().to_string(),
        query: entry.query.trim().to_string(),
        action: entry.action,
        enabled: entry.enabled,
        interval_minutes: entry.interval_minutes,
        run_after_sync: entry.run_after_sync,
    };
    validate(&input).map_err(|e| e.to_string())?;
    Ok(input)
}

impl ImportPlan {
    pub fn report(&self, applied: bool) -> ImportReport {
        ImportReport {
            added: self.added.iter().map(|input| input.name.clone()).collect(),
            existing: self
                .existing
                .iter()
                .map(|(_, input)| input.name.clone())
                .collect(),
            invalid: self.invalid.clone(),
            applied,
        }
    }

    fn apply(&self, conn: &Connection, on_conflict: OnConflict) -> mail_sieve_store::Result<()> {
        for input in &self.added {
            rules::insert(conn, input)?;
        }
        if on_conflict == OnConflict::Replace {
            for (id, input) in &self.existing {
                rules::update(conn, *id, input)?;
            }
        }
        Ok(())
    }
}

pub fn import(
    conn: &Connection,
    file: RulesFile,
    on_conflict: OnConflict,
    apply: bool,
) -> mail_sieve_store::Result<ImportReport> {
    let current = rules::for_account(conn, file.account_id)?;
    let plan = file.plan(&current);
    if apply {
        plan.apply(conn, on_conflict)?;
    }
    Ok(plan.report(apply))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(id: i64, name: &str) -> Rule {
        Rule {
            id,
            account_id: 1,
            name: name.into(),
            query: "has:list".into(),
            action: "archive".into(),
            enabled: true,
            interval_minutes: None,
            run_after_sync: true,
            last_run_at: None,
            created_at: 0,
        }
    }

    #[test]
    fn exports_the_documented_shape() {
        let mut shipping = rule(2, "Shipping notifications");
        shipping.query = "from:*@ups.com older_than:14d".into();
        shipping.action = "trash".into();
        shipping.run_after_sync = false;
        shipping.interval_minutes = Some(1440);
        let yaml = export(&[rule(1, "Old newsletters"), shipping]).unwrap();
        assert_eq!(
            yaml,
            "version: 1\nrules:\n- name: Old newsletters\n  query: has:list\n  action: archive\n  run_after_sync: true\n  interval_minutes: null\n  enabled: true\n- name: Shipping notifications\n  query: from:*@ups.com older_than:14d\n  action: trash\n  run_after_sync: false\n  interval_minutes: 1440\n  enabled: true\n"
        );
    }

    #[test]
    fn round_trips_an_export() {
        let yaml = export(&[rule(1, "Old newsletters")]).unwrap();
        let report = parse(&yaml, 7).unwrap().plan(&[]).report(false);
        assert_eq!(report.added, ["Old newsletters"]);
        assert!(report.existing.is_empty() && report.invalid.is_empty());
    }

    #[test]
    fn sorts_entries_into_added_existing_and_invalid() {
        let yaml = r#"
version: 1
rules:
  - name: Old newsletters
    query: has:list
    action: archive
  - name: Fresh
    query: older_than:30d
    action: mark_read
  - name: Purge
    query: from:x@y.com
    action: delete
  - name: Everything
    query: ""
    action: archive
  - name: Typo
    query: has:list
    action: archive
    run_afer_sync: true
  - query: has:list
    action: archive
  - name: Fresh
    query: has:list
    action: trash
"#;
        let plan = parse(yaml, 1).unwrap().plan(&[rule(4, "Old newsletters")]);
        let report = plan.report(false);
        assert_eq!(report.added, ["Fresh"]);
        assert_eq!(report.existing, ["Old newsletters"]);
        assert_eq!(plan.existing[0].0, 4);
        let invalid = report
            .invalid
            .iter()
            .map(|invalid| invalid.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(invalid, ["Purge", "Everything", "Typo", "Rule 6", "Fresh"]);
        assert!(report.invalid[0].error.contains("permanently delete"));
    }

    #[test]
    fn rejects_unknown_versions_and_garbage() {
        assert!(parse("version: 2\nrules: []\n", 1).is_err());
        assert!(parse("rules: []\n", 1).is_err());
        assert!(parse("- just\n- a list\n", 1).is_err());
        assert!(parse("version: 1\n", 1).is_ok());
    }

    #[test]
    fn imports_skipping_or_replacing_existing_names() {
        let dir = tempfile::tempdir().unwrap();
        let store = mail_sieve_store::Store::open(dir.path().join("test.db")).unwrap();
        let account = store
            .write(|tx| {
                mail_sieve_store::accounts::insert(
                    tx,
                    &mail_sieve_store::accounts::NewAccount {
                        label: "Test".into(),
                        email: "me@example.com".into(),
                        imap_host: "imap.example.com".into(),
                        imap_port: 993,
                        imap_tls: true,
                        smtp_host: None,
                        smtp_port: None,
                        username: "me@example.com".into(),
                        auth_kind: "password".into(),
                        oauth_provider: None,
                        credentials: vec![1],
                        provider: "generic".into(),
                        max_connections: 4,
                    },
                )
            })
            .unwrap();
        let yaml = "version: 1\nrules:\n  - name: Old newsletters\n    query: has:list older_than:30d\n    action: archive\n";
        let run = |on_conflict, apply| {
            let file = parse(yaml, account).unwrap();
            store
                .write(|tx| import(tx, file, on_conflict, apply))
                .unwrap()
        };
        let current = || store.read(|c| rules::for_account(c, account)).unwrap();

        let preview = run(OnConflict::Skip, false);
        assert_eq!(preview.added, ["Old newsletters"]);
        assert!(!preview.applied && current().is_empty());

        assert!(run(OnConflict::Skip, true).applied);
        assert_eq!(current().len(), 1);

        let again = run(OnConflict::Skip, true);
        assert_eq!(again.existing, ["Old newsletters"]);
        assert_eq!(current()[0].query, "has:list older_than:30d");

        let mut edited = current()[0].clone();
        edited.query = "has:list".into();
        store
            .write(|tx| {
                rules::update(
                    tx,
                    edited.id,
                    &RuleInput {
                        account_id: account,
                        name: edited.name,
                        query: edited.query,
                        action: edited.action,
                        enabled: edited.enabled,
                        interval_minutes: edited.interval_minutes,
                        run_after_sync: edited.run_after_sync,
                    },
                )
            })
            .unwrap();
        run(OnConflict::Replace, true);
        let rules = current();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].query, "has:list older_than:30d");
    }
}
