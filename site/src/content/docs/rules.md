---
title: Rules
description: Save a search and an action as a rule that runs after every sync or on a timer.
lead: A rule is a saved search plus an action plus a schedule.
group: Using mailsieve
order: 4
---

Create one from the Rules screen with any search query and an action. Rules run after every sync, every N minutes, or both. Each run is recorded in Activity with the number of messages affected.

```text
Name:     Old newsletters
Query:    has:list older_than:30d
Action:   archive
Schedule: after every sync
```

:::note
Rules can never permanently delete, and a rule needs a non-empty query. Use trash instead; your provider empties it on its own schedule.
:::

## Export and import

Rules belong to one account. **Export** on the Rules screen downloads that account's rules as a YAML file you can back up, keep in version control or bring to another instance.

```yaml
version: 1
rules:
  - name: Old newsletters
    query: has:list older_than:30d
    action: archive
    run_after_sync: true
    interval_minutes: null
    enabled: true
```

Only `name`, `query` and `action` are required. `run_after_sync` defaults to `false`, `interval_minutes` to no timer and `enabled` to `true`. Unknown keys are rejected so a typo does not silently change a schedule.

**Import** reads a file and shows a preview before anything is saved: new rules, rules whose name already exists (keep yours or replace them with the file's), and rules that cannot be imported, with the reason. Imported rules are checked exactly like rules you create by hand, so a file can never bring in a permanent delete.

:::note
`move:` actions name a folder of the account they came from. Importing into another account keeps the folder name as written; check it exists there.
:::

The same works from the command line:

```sh
mail-sieve rules export --account 1 > rules.yaml
mail-sieve rules import --account 2 rules.yaml --dry-run
mail-sieve rules import --account 2 rules.yaml --replace
```
