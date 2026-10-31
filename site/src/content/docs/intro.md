---
title: Introduction
description: What mail-sieve is, what it is not, and what you need to run it.
lead: mail-sieve is a self-hosted web app for bulk-cleaning IMAP mailboxes by sender.
group: Getting started
order: 1
---

It connects to one or more mail accounts over IMAP, builds a local index of message headers, and groups everything by sender, domain and mailing list. From there you can archive, trash, mark read or unsubscribe in bulk, and save searches as rules that run on a schedule.

## What it is not

- Not an email client. There is no compose window and no threading.
- Not a hosted service. You run it, you own the data.
- Not destructive by default. Trash goes to your provider's Trash folder, and permanent delete always asks for confirmation first.

## Requirements

- Docker on `amd64` or `arm64`, or the standalone binary for Linux, macOS or Windows
- An IMAP account: Gmail, Outlook / Microsoft 365, iCloud, Yahoo, Fastmail, Dovecot or any IMAP4rev1 server
- A little disk for the index: about 1 to 2 KB of headers per message
