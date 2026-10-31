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
