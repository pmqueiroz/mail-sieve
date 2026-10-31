# Architecture

mail-sieve is a self-hosted mailbox control panel for any IMAP account. It
mirrors **message headers only** into a local SQLite index, answers every
read (grouping, counting, searching, paging) from that index, and pushes
bulk changes back to the server as a few compressed IMAP commands.

Mental model: **reads hit the local index, writes are jobs that hit IMAP.**

Code is the truth. This file explains the *why* behind the shape it takes.

## Why this shape

- **Browser-based, server-hosted.** Self-hosted means one container on a
  NAS or VPS, reachable from any device, running scheduled rules while no
  client is open. A native desktop app cannot do that. Rust-native GUI
  toolkits are not mature for dense virtualized tables, and the practical
  desktop route (Tauri) is a webview anyway, so the same SPA can be wrapped
  later.
- **Performance is a protocol problem first.** IMAP cost is round trips and
  server work, not CPU. The wins:
  - header-only fetch: `BODY.PEEK[HEADER.FIELDS (...)]` for seven headers,
    about 1 to 2 KB per message, never bodies, never `\Seen` changes;
  - `COMPRESS=DEFLATE` whenever the server offers it;
  - UID sets split into chunks and fetched over N parallel connections,
    newest chunks first so the UI is useful during a first sync;
  - set-difference sync: one `UID SEARCH ALL` versus local UIDs gives new
    and vanished messages, which also self-heals interrupted syncs;
  - `CONDSTORE` `CHANGEDSINCE` for flag changes; a full flag fetch only
    without it;
  - an "unchanged" short circuit when UIDVALIDITY, UIDNEXT, HIGHESTMODSEQ
    and the message count all match (about 10 ms per mailbox);
  - bulk writes as compressed UID sets (`1:500,502,510:9000`) chunked by
    count (1000) and encoded length (4000 bytes).
- **Measured** on local Dovecot, release build: first sync of 20,000
  messages in 0.66 s, unchanged re-sync 12 ms, counts 7 to 14 ms, sender
  groups 0.5 ms, archive of 4,000 messages 480 ms. Real providers are
  network bound; the design target is 19k messages under 60 s.

## Layout

Cargo workspace producing one binary, plus a Svelte SPA embedded into it.

- `crates/core`: pure domain, no I/O. `UidSet`, `Flags`, `MessageMeta`,
  `Action`, the query DSL (`Query::parse`), header parsers
  (`headers::parse_list_unsubscribe`, `parse_list_id`), label
  normalization.
- `crates/store`: SQLite. Schema in `schema.sql`, migrations by
  `PRAGMA user_version`, one writer connection plus a pool of read-only
  connections. Modules per table; `search.rs` compiles the query AST to
  SQL.
- `crates/imap`: async IMAP over `async-imap` + rustls (ring, webpki
  roots). `connect`, `list_folders`, `fetch_headers`,
  `fetch_headers_parallel`, `fetch_flags`, `fetch_raw`, `apply(Operation)`.
  Knows nothing about the store.
- `crates/unsub`: unsubscribe execution (RFC 8058 one-click POST, mailto
  over SMTP, link fallback).
- `crates/server` (binary `mail-sieve`): config, credential vault, account
  service, sync and action engines, job runner, rules, scheduler, IDLE
  watchers, OAuth, axum API, embedded assets, CLI.
- `web/`: Svelte 5 + Vite + TypeScript SPA, built to `web/dist` and embedded
  with `rust-embed`. `build.rs` writes a placeholder `index.html` when the
  UI was not built, so `cargo build` always works.
- `docker/`: Dovecot config and compose files for tests and deployment.

Dependency direction: `server -> {store, imap, unsub} -> core`. `store` and
`imap` never depend on each other; the server wires sync output into the
store.

## Data model

See `crates/store/src/schema.sql`. Highlights:

- `messages` is keyed by `(mailbox_id, uid)` and denormalizes `account_id`
  for fast filtering. `from_addr` and `from_domain` are lowercased on
  insert. `date` falls back to INTERNALDATE, then 0; header dates more than
  a day after INTERNALDATE are discarded as bogus.
- `senders` is a materialized aggregate per `(account_id, from_addr)`,
  maintained **by triggers** on insert, delete and seen-flag change. The
  unfiltered sender and domain views read it directly; filtered views fall
  back to `GROUP BY` over `messages`. `last_date` only grows (a delete does
  not recompute it).
- `messages_fts` is an external-content FTS5 table over subject and sender,
  kept in sync by the same triggers.
- `labels` holds normalized Gmail labels joined by `\u{1f}`
  (`inbox\u{1f}receipts`).
- `jobs` + `job_targets` freeze the exact `(mailbox_id, uid)` selection at
  enqueue time, so the action that runs is the one previewed, and a job
  resumes after a crash with whatever targets are left.
- UIDVALIDITY change: every row of that mailbox is deleted and re-synced.
  Never reconcile across UIDVALIDITY.

## Query DSL

`Query::parse` produces AND-ed terms; any term can be negated with `-`.
Fields: `from:` (exact with `@`, wildcard with `*`, otherwise substring of
address or name), `domain:` (matches subdomains), `subject:`, `list:`,
`in:` (folder name or Gmail label), `is:unread|read|flagged|answered`,
`has:unsubscribe|list`, `older_than:`/`newer_than:` (`h d w m y`),
`before:`/`after:` (`YYYY-MM-DD`, UTC), `larger:`/`smaller:` (`kb mb gb`),
`category:` (see below), bare words go to FTS5 as prefix matches. Every compiled clause is wrapped
in `COALESCE(..., 0)` so negation is NULL-safe. There is no OR; the action
API accepts `queries: []` and unions their targets instead.

### Categories

`category:` classifies each message into exactly one of six buckets with a
single SQL `CASE` (`search::category_sql`), first match wins:

1. `receipts`: subject contains receipt, invoice, order, payment, purchase,
   booking or statement;
2. `social`: sender domain (or subdomain) is a known social network;
3. `newsletters`: has a `List-Id`;
4. `notifications`: sender address looks automated (noreply, notification,
   alert, ...);
5. `promotions`: offers unsubscribe but has no list id;
6. `people`: everything else.

It is a heuristic over headers only, chosen so the Overview "Inbox mix" and
the `category:` filter always agree: both use the same expression. Order
matters: a receipt from a mailing list is a receipt. Singular names
(`category:receipt`) and `cat:` are accepted.

## Providers

`imap::Provider` is detected from capabilities (`X-GM-EXT-1` means Gmail).

- **Generic**: syncs every selectable folder except `\Trash`, `\Junk`,
  `\Drafts`, `\Sent`, `\All`, `\Flagged`. Archive moves to `\Archive`
  (created and left unsynced if missing). Trash moves to `\Trash`. Delete
  is `\Deleted` + `UID EXPUNGE`.
- **Gmail**: syncs only `[Gmail]/All Mail` plus `X-GM-MSGID` and
  `X-GM-LABELS`, because label folders duplicate messages and archived mail
  exists only in All Mail. Archive removes the `\Inbox` label. Trash and
  Delete both move to `[Gmail]/Trash` (Gmail purges it after 30 days).
- Special-use folders come from LIST attributes, falling back to common
  names (`Deleted Items`, `INBOX.Trash`, `Spam`, ...).
- No MOVE: `COPY` + `\Deleted` + expunge. No UIDPLUS: plain `EXPUNGE`,
  which also removes other messages already flagged `\Deleted`.

Adding a provider: extend `Provider` (detection, `default_sync`,
`default_connections`) and its arms in `server/src/actions.rs::plan`.

## Jobs

- Kinds: `sync`, `action`. One job per account runs at a time (lanes keyed
  by account id); different accounts run in parallel. The runner wakes on
  `Notify` or every 5 s.
- Startup requeues jobs left `running`. Cancellation is a state flip that
  workers check between chunks; `finish` never overwrites `cancelled`.
- Progress is written to the row and broadcast as an SSE `job` event
  (throttled to about 3 per second during sync).
- After a successful sync, rules with `run_after_sync` enqueue their
  actions.

## Rules, scheduler, IDLE

- Rules are saved query + action, run after each sync and/or every N
  minutes. They cannot use `delete` and cannot have an empty query.
- The scheduler ticks every 60 s: enqueue due syncs, run due rules,
  reconcile IDLE watchers, purge expired sessions, prune finished jobs
  older than 30 days.
- One IDLE watcher per synced account watches All Mail (Gmail) or INBOX,
  re-IDLEs every 25 minutes, and enqueues a sync 5 s after new data.

## Unsubscribe

Order: one-click POST if the sender advertises it, then mailto via the
account's SMTP settings, then hand the HTTPS link back to the UI. Outcomes
are stored in `unsubscribes` and shown as sender badges. An optional
follow-up action (archive or trash, never delete) is enqueued for
`from:<sender>`. See [security.md](./security.md) for the SSRF guard.

## Web UI

- `src/lib/session.svelte.ts`: auth status, accounts, jobs, SSE
  connection. Bumps `dataVersion` when jobs finish so views reload.
- `src/lib/router.svelte.ts`: hash router (`#/senders`, `#/messages?q=`,
  `#/rules`, `#/activity`, `#/accounts`).
- Shared components in `src/lib/components`: `VirtualList` (fixed row
  height, overscan, end-reached paging), `QueryBar` (debounced query with
  live count and syntax help), `ActionBar` + `BulkActions` (one action
  vocabulary everywhere), `UnsubscribeDialog`, `Modal`, `ConfirmDialog`,
  `Button`, `Badge`, `Field`, `Segmented`, `ProgressBar`, `Toasts`.
- Design tokens live in `src/app.css` as CSS variables with a dark scheme
  via `prefers-color-scheme`; components never hardcode colors.
- Message previews render `text/plain` (or text extracted from HTML) inside
  a `<pre>`; HTML is never injected.

## Known gaps

- OAuth flows are implemented against Google and Microsoft endpoints but
  are only exercised by unit tests; they need real client credentials.
- No STARTTLS: use implicit TLS (993) or plain for local servers only. No
  "accept invalid certificates" option for self-signed homelab servers.
- Single admin user; multi-user is not modeled.
- No undo beyond Trash-first defaults and the `action_log` audit table.
- Gmail permanent delete is not exposed; Gmail purges Trash itself.
- `in:` matches folder names exactly (case-insensitive) or Gmail labels; no
  folder hierarchy wildcards.
