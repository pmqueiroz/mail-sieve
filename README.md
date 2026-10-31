![banner](.github/assets/banner.png)

Self-hosted control panel for any IMAP mailbox. See who sends you what,
unsubscribe in bulk, and archive, trash or label thousands of messages at a
time, without handing your inbox to a third-party service.

- **Senders view**: every sender, domain or mailing list with message count,
  unread count, size and last date, sorted however you like.
- **Bulk actions**: archive, move to trash, mark read or unread, flag, move
  to a folder, or delete permanently, for whole senders or any query.
- **Unsubscribe**: RFC 8058 one-click, mailto, or the sender's link, with an
  optional "and archive everything they sent".
- **Gmail-style search**: `from:*@linkedin.com older_than:1y is:unread`.
- **Rules**: run a query + action after every sync or on a schedule.
- **Fast**: one Rust binary with an embedded web UI, SQLite index, parallel
  header-only sync. A first sync of 20,000 messages takes under a second
  against a local Dovecot server; queries answer in milliseconds.
- **Private**: message bodies stay on your mail server. Credentials are
  encrypted at rest. Nothing phones home.

Works with Gmail, Outlook / Microsoft 365, iCloud, Yahoo, Fastmail, Dovecot
and any IMAP4rev1 server.

## Quick start

```sh
docker build -t mail-sieve .
docker run -d --name mail-sieve -p 127.0.0.1:8080:8080 -v mail-sieve-data:/data mail-sieve
```

Or with compose: `docker compose -f docker/compose.yml up -d` (set `MAIL_SIEVE_PORT` to use a host port other than 8080, and `MAIL_SIEVE_HOST_IP=0.0.0.0` to reach it from other devices; both can live in `docker/.env`).

Open http://localhost:8080, choose an admin password, then add an account.

> The data volume holds `secret.key`, which encrypts stored credentials.
> Back it up; without it saved accounts must be re-added.

### Gmail

Either create an [app password](https://myaccount.google.com/apppasswords)
(requires 2-step verification) and pick the Gmail preset, or configure
Google sign-in (below). mail-sieve indexes `[Gmail]/All Mail` with labels,
so every message is counted once. Archive removes the Inbox label; trash and
delete move to Gmail's Trash, which Gmail empties after 30 days.

### Outlook / Microsoft 365

Microsoft disabled password IMAP login for most accounts, so configure
Microsoft sign-in.

### Sign in with Google or Microsoft (OAuth)

Register an OAuth app with the provider and set the redirect URI to
`<public url>/api/oauth/google/callback` (or `/microsoft/callback`).

| Provider | Scopes |
| --- | --- |
| Google | `openid email https://mail.google.com/` |
| Microsoft | `openid email offline_access https://outlook.office.com/IMAP.AccessAsUser.All https://outlook.office.com/SMTP.Send` |

Then set the client id and secret variables below and restart.

## Configuration

Every option is an environment variable (or the matching `--flag`).

| Variable | Default | Purpose |
| --- | --- | --- |
| `MAIL_SIEVE_DATA_DIR` | `./data` (`/data` in Docker) | Database and `secret.key` |
| `MAIL_SIEVE_BIND` | `127.0.0.1:8080` (`0.0.0.0:8080` in Docker) | Listen address |
| `MAIL_SIEVE_PUBLIC_URL` | | External URL, e.g. `https://mail.example.com`. Enables `Secure` cookies and builds OAuth redirect URIs |
| `MAIL_SIEVE_SECRET_KEY` | | Base64 32-byte key, instead of `secret.key` |
| `MAIL_SIEVE_ADMIN_PASSWORD` | | Sets or resets the admin password at startup |
| `MAIL_SIEVE_TRUSTED_HEADER` | | Trust a reverse-proxy auth header (for example `Remote-User`). Only behind a proxy that strips it from clients |
| `MAIL_SIEVE_GOOGLE_CLIENT_ID` / `_SECRET` | | Enables Google sign-in |
| `MAIL_SIEVE_MICROSOFT_CLIENT_ID` / `_SECRET` | | Enables Microsoft sign-in |
| `RUST_LOG` | `info` | Log filter |

Put it behind a TLS reverse proxy (Caddy, Traefik, nginx) when exposed
beyond localhost. Server-sent events are used for live progress, so disable
response buffering for `/api/events`.

## CLI

```sh
mail-sieve serve                 # default
mail-sieve accounts              # list accounts
mail-sieve sync --account 1      # one-shot sync with timing, handy for benchmarks
mail-sieve set-password          # reset the admin password from stdin
mail-sieve health                # exit 0 when the server answers (Docker healthcheck)
```

## Query syntax

| Example | Meaning |
| --- | --- |
| `from:deals@shop.com` | exact sender |
| `from:*@shop.com` | wildcard sender |
| `from:alice` | sender address or name contains |
| `domain:shop.com` | sender domain, including subdomains |
| `subject:invoice` | subject contains |
| `list:newsletter` | mailing list id contains |
| `in:inbox` | folder, or Gmail label |
| `is:unread`, `is:read`, `is:flagged`, `is:answered` | state |
| `has:unsubscribe`, `has:list` | newsletter signals |
| `older_than:1y`, `newer_than:7d` | age, units `h d w m y` |
| `before:2024-01-01`, `after:2023-06-01` | dates |
| `larger:5mb`, `smaller:100kb` | size |
| `category:newsletters` | inbox mix category: `newsletters notifications promotions receipts social people` |
| `-from:boss@work.com` | negate any term |
| `receipt` | free text over subject and sender |

Terms are combined with AND.

## Development

Requirements: Rust 1.88+, Node 24, Docker (for the IMAP test server).

```sh
docker compose -f docker/compose.test.yml up -d          # Dovecot on 127.0.0.1:3143, any user, password "pass"
cargo run -p mail-sieve -- serve                          # API on :8080
(cd web && npm install && npm run dev)                    # UI on :5173, proxies /api
MAIL_SIEVE_TEST_IMAP=127.0.0.1:3143 cargo test --workspace
cargo run --release -p mail-sieve-imap --example seed -- 127.0.0.1:3143 someone 20000
```

Project documentation for contributors and coding agents lives in
[`AGENTS.md`](./AGENTS.md) and [`.refs/`](./.refs): architecture, security
model, CI and conventions.
