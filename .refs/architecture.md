# Architecture

mail-sieve is a self-hosted mailbox control panel for any IMAP account: it
mirrors **message headers only** into a local SQLite index, lets the user
browse, group and query that index instantly, and pushes bulk actions
(delete, archive, move, flag, unsubscribe) back to the server as a few
compressed IMAP commands. Mental model: **reads hit the local index, writes
are jobs that hit IMAP.**

Code is the truth. This file explains the *why* behind the shape it takes.

## Why this shape

- **Browser-based, server-hosted.** Self-hosted means a single container on
  a NAS/VPS, reachable from any device, able to run scheduled rules while no
  client is open. A native desktop app can't do that. Rust-native GUI
  toolkits are not mature enough for dense, virtualized data tables; the
  practical "desktop" route (Tauri) is a webview anyway, so the same SPA can
  be wrapped later at near-zero cost.
- **Performance is a protocol problem first.** With IMAP the bottleneck is
  network round-trips and server work, not CPU. Rust gives low memory, no GC
  pauses and cheap concurrency, but the wins come from:
  - fetching headers only (`BODY.PEEK[HEADER.FIELDS (...)]`, ENVELOPE-free),
    about 1–2 KB per message;
  - chunked UID ranges over several parallel connections per account;
  - incremental sync via `CONDSTORE`/`QRESYNC`, falling back to
    `UIDVALIDITY` + `UIDNEXT` + flag diff;
  - bulk writes as compressed UID sets (`1:500,502,510:9000`) with
    `UID MOVE`, or `UID STORE +FLAGS (\Deleted)` + `UID EXPUNGE`, chunked
    under command-length limits;
  - the UI never waiting on IMAP for reads.

## Layout

Cargo workspace, one binary.

- `crates/server` → binary `mail-sieve`. Entry point. Owns the axum HTTP
  API, SSE progress stream, auth, the job runner, the CLI, and serves the
  embedded SPA (`web/dist`, via `rust-embed`).
- `crates/core` → pure domain: types (`Account`, `Mailbox`, `MessageMeta`,
  `Flags`, `UidSet`), the query DSL parser and its AST, rule model. No I/O.
- `crates/store` → SQLite schema, migrations, the single writer task
  (batched transactions), read pool, sender aggregates, query AST → SQL
  compiler.
- `crates/imap` → `ImapClient` trait, connection pool, full/incremental
  sync, bulk actions, provider quirks (Gmail, Outlook, generic).
- `crates/unsub` → unsubscribe execution: RFC 8058 one-click POST, mailto
  via SMTP, plain link fallback; `List-Unsubscribe` header parsing.
- `web/` → SPA (Svelte + Vite, TanStack Virtual for 100k-row lists). Built
  to `web/dist` and embedded at compile time.
- `docker/` → static musl build image and a compose file with a Dovecot
  instance for integration tests.

Dependency direction: `server → {store, imap, unsub} → core`. `store` and
`imap` never depend on each other; `server` wires sync output into the store.

## Data model (SQLite, WAL)

- `accounts(id, label, host, port, tls, auth_kind, enc_credentials,
  capabilities, settings)`
- `mailboxes(id, account_id, name, special_use, uidvalidity, uidnext,
  highestmodseq, last_synced_at)`
- `messages(id, mailbox_id, uid, message_id, gm_msgid, from_addr,
  from_name, from_domain, subject, date, size, flags, list_id, unsub_http,
  unsub_mailto, unsub_one_click)`. Unique `(mailbox_id, uid)`; indexes on
  `from_domain`, `from_addr`, `list_id`, `date`.
- `messages_fts` — FTS5 over subject and sender.
- `senders` — materialized aggregate per `(account_id, from_addr)`: count,
  total size, unread, last date, has_unsub. Updated incrementally per sync
  batch so the "who sends me what" view is instant.
- `rules`, `jobs`, `action_log` — saved queries + actions, persistent job
  queue (resumable after crash), audit trail for undo.

A `UIDVALIDITY` change invalidates every row of that mailbox: delete and
re-sync, never try to reconcile.

## The extension point: providers

`imap` has a `Provider` profile chosen from `CAPABILITY` + host:

- **Generic**: folders from `SPECIAL-USE` (`\Archive`, `\Trash`, `\Junk`),
  falling back to common names.
- **Gmail** (`X-GM-EXT-1`): canonical folder is `[Gmail]/All Mail`; dedupe
  by `X-GM-MSGID`; "delete" = move to `[Gmail]/Trash`; "archive" = remove
  from INBOX. Max 15 concurrent connections.

Adding a provider: implement the profile (folder resolution, delete and
archive semantics, connection limit), register it in the detection
function. Nothing else changes.

## Shared state

`server` builds one `AppState` (store handle, IMAP pool registry, job
runner handle, credential vault, config) at startup and injects it into
axum handlers and jobs. There are no globals.

## Reusable mechanics

- **Query DSL** (`core`): `from:*@linkedin.com older_than:1y is:unread
  larger:5mb has:unsubscribe list:<id>` → AST → SQL (`store`). The same
  AST drives the UI's live match count, bulk-action targets and rules.
- **Jobs**: every IMAP write is a job: dry-run first (count + bytes), then
  queued, chunked, progress over SSE, resumable. Delete defaults to Trash;
  permanent expunge requires explicit confirmation.
- **UidSet**: canonical compressed UID ranges, shared by sync and actions.

## Security

- Single admin login (argon2) by default; optional trusted reverse-proxy
  auth header. Multi-account from day one, multi-user later.
- IMAP credentials encrypted at rest (XChaCha20-Poly1305); the key comes
  from env or a secret file, never the DB.
- Message bodies are never stored; previews use `BODY.PEEK` so nothing is
  marked read.

## Deployment

One static binary and one SQLite file:
`docker run -v data:/data -p 8080:8080 mail-sieve`.

## Roadmap

1. **M1 sync core** — accounts, parallel header sync, incremental sync, CLI
   `mail-sieve sync`. Target: 19k messages under 60 s first sync, under 3 s
   re-sync.
2. **M2 API + senders view** — REST, aggregates, virtualized SPA, query
   builder with live counts.
3. **M3 bulk actions** — job queue, MOVE/STORE/EXPUNGE, Trash-first, action
   log, SSE progress.
4. **M4 unsubscribe** — one-click/mailto/link, "unsubscribe and purge".
5. **M5 rules + IDLE** — scheduled and real-time rules.
6. **M6 ship** — OAuth2 (Gmail/Microsoft), Docker image, optional Tauri.

## Known gaps

Everything is a scaffold: crates are empty, `web/` and `docker/` are
placeholders. The roadmap above is the order of work.
