---
title: Senders view
description: Rank every sender, domain or mailing list by messages, size, unread count or recency.
lead: The main screen ranks every sender by message count, unread count and storage used.
group: Using mailsieve
order: 1
---

Click a sender to see its messages, or select several and apply an action to all of them at once. Grouping can be switched between individual sender, domain and mailing list.

- Sort by most messages, largest, most unread or most recent
- Filter the list with any [search](../search/) query
- Previews fetch a single message body on demand and are not stored

## Pinned senders

Pin a sender or a whole domain to protect it from accidents. The pin button sits on each row of the Senders view (sender and domain grouping) and in the message preview.

- Pinned senders and domains sort to the top of the list. Filter them with `is:pinned`.
- A domain pin also covers its subdomains: pinning `bank.com` protects `alerts@mail.bank.com`.
- Bulk actions and rules skip messages from pinned senders. The action bar says how many are skipped, and so does the rule editor.
- Selecting a single pinned sender, or one of its messages, and acting on it still works after a confirmation.

:::note
Pins guard actions on messages only. Unsubscribing from a pinned sender still works, but an archive or trash follow-up leaves its messages where they are. Unpin first if you want them gone.
:::

## Overview

The Overview screen summarizes the whole index: totals, unread mail, how much comes from mailing lists, and the senders you never open. Those never-opened senders are usually the fastest place to start.
