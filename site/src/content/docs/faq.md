---
title: FAQ
description: Short answers about storage, privacy, Gmail labels and performance.
lead: Short answers to common questions.
group: Reference
order: 2
---

## Does it store my email?

Only headers (sender, subject, date, size, flags and list headers) in a local SQLite index. Message bodies stay on your mail server; previews fetch one body on demand and do not keep it.

## Does opening :mailsieve mark my mail as read?

No. Headers are fetched with `BODY.PEEK`, which never changes the read state.

## Will it work with Gmail labels?

Yes. :mailsieve indexes All Mail with its labels, so `in:` matches labels and archive removes the Inbox label the way Gmail does.

## How fast is it?

Against a local Dovecot server, a first sync of 20,000 messages takes under a second and queries answer in milliseconds. Real providers are limited by the network, so expect longer first syncs.
