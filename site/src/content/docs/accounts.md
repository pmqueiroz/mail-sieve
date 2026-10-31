---
title: Connecting accounts
description: IMAP settings for Gmail, Outlook, iCloud, Yahoo, Fastmail and self-hosted servers.
lead: Any IMAP server works. Most providers need an app password rather than your normal one.
group: Getting started
order: 3
---

| Provider | Settings |
| --- | --- |
| Gmail | `imap.gmail.com` · app password, or Google sign-in |
| Outlook / Microsoft 365 | `outlook.office365.com` · Microsoft sign-in |
| iCloud | `imap.mail.me.com` · app-specific password |
| Yahoo | `imap.mail.yahoo.com` · app password |
| Fastmail | `imap.fastmail.com` · app password with IMAP access |
| Other | any host and port, with TLS |

The first sync downloads headers only, newest first, so you can start browsing senders while it runs.

## Gmail

Either create an [app password](https://myaccount.google.com/apppasswords) (it requires 2-step verification) and pick the Gmail preset, or set up Google sign-in. mail-sieve indexes `[Gmail]/All Mail` with its labels, so every message is counted once.

## Outlook / Microsoft 365

Microsoft disabled password IMAP login for most accounts, so set up Microsoft sign-in.

## Sign in with Google or Microsoft

Register an OAuth app with the provider and set its redirect URI to `<public url>/api/oauth/google/callback` (or `/microsoft/callback`). Then set the client id and secret variables from [Configuration](../configuration/) and restart.

| Provider | Scopes |
| --- | --- |
| Google | `openid email https://mail.google.com/` |
| Microsoft | `openid email offline_access https://outlook.office.com/IMAP.AccessAsUser.All https://outlook.office.com/SMTP.Send` |

:::note
SMTP is only used to send unsubscribe emails to senders that ask for one by mailto. mail-sieve never sends anything else.
:::
