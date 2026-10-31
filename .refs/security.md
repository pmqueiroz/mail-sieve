# Security

mail-sieve holds credentials to someone's mailbox and can delete mail in
bulk. These are the decisions that protect that, and the reasons.

## Credentials at rest

- IMAP passwords and OAuth refresh tokens are sealed with
  XChaCha20-Poly1305 (`server/src/vault.rs`) before they reach SQLite. Each
  seal uses a fresh random 24-byte nonce stored in front of the ciphertext.
- The 32-byte key comes from `MAIL_SIEVE_SECRET_KEY` (base64) or
  `<data dir>/secret.key`, generated on first start with mode `0600`.
  Losing the key makes every stored credential unreadable; back it up
  separately from the database.
- `Account.credentials` is `#[serde(skip)]`, so it never leaves the API.
  `ImapConfig`'s `Debug` omits credentials.

## Admin authentication

- One admin password, Argon2id (`argon2` defaults), stored in `settings`.
  First run is a `setup` call that only works while no hash exists.
  `MAIL_SIEVE_ADMIN_PASSWORD` resets it at boot when it differs.
- Sessions are 32 random bytes, stored only as SHA-256 hex in `sessions`,
  30-day expiry, cookie `HttpOnly; SameSite=Lax` plus `Secure` when
  `MAIL_SIEVE_PUBLIC_URL` is https.
- `Lax` and not `Strict`: the OAuth provider redirects back with a
  top-level GET that must carry the session. Mutations are POST, PATCH,
  PUT or DELETE with JSON bodies, which `Lax` keeps out of cross-site
  requests.
- Failed logins sleep 750 ms. Changing the password signs out every
  session.
- `MAIL_SIEVE_TRUSTED_HEADER` trusts any request carrying that header.
  Only use it behind a reverse proxy that strips the header from clients.

## Destructive operations

- Bulk actions snapshot their targets at enqueue time (`job_targets`).
- `delete` (permanent) requires `confirm: true` from the API and a
  confirmation dialog in the UI. An empty query (every message) also
  requires `confirm`.
- Rules cannot permanently delete. Unsubscribe follow-ups cannot either.
  Imported rule files go through the same validation, so a YAML file can
  never introduce a permanent-delete rule.
- Pinned senders and domains are excluded when action targets are
  resolved (`search::resolve`), so rules, bulk actions and unsubscribe
  follow-ups skip them. Overriding needs `include_pinned` plus `confirm`.
- Before writing, the action engine checks UIDVALIDITY against the index
  and refuses on mismatch, so stale UIDs never hit the wrong messages.

## Unsubscribe SSRF guard

`List-Unsubscribe` URLs are attacker-controlled. Before the server POSTs
(`unsub/src/one_click.rs`):

- https only, no userinfo, host must be a domain name with a dot (no IP
  literals, no `localhost`);
- DNS is resolved once and every address must be public (no loopback,
  RFC 1918, link-local, CGNAT, ULA, benchmark, multicast or reserved
  ranges, including IPv4-mapped IPv6);
- the request is pinned to those vetted addresses (`resolve_to_addrs`), so
  DNS rebinding cannot swap targets, and redirects are disabled.

Plain links that fail these checks are handed to the user's browser, not
fetched by the server.

## Web surface

- Strict CSP (`default-src 'self'`, no inline scripts, `frame-ancestors
  'none'`), `X-Frame-Options: DENY`, `nosniff`, `Referrer-Policy:
  no-referrer`.
- Message previews are plain text in a `<pre>`; mail HTML is never
  rendered, so remote images and scripts cannot run or track.
- Bodies are fetched with `BODY.PEEK[]` and never stored.

## Plugins

- Community plugins run as same-origin ES modules inside the signed-in
  app, so they can do anything the admin can. The trust boundary is the
  filesystem: only someone who can write to `<data dir>/plugins/` can add
  one, and each is off until enabled on the Plugins page, which warns
  about it. Core plugins are part of the build.
- Plugin files are served only to authenticated sessions
  (`/api/plugins/{id}/files/...`). The id must match
  `[a-z0-9][a-z0-9-]{0,63}`; the path must be plain `Normal` components
  (no `..`, no root), and the canonicalized file must still be inside the
  canonicalized plugin folder, so symlinks cannot escape it either.
- Plugin secrets (AI API keys) are sealed with the same vault key as
  mailbox credentials and stored apart from the plain settings. The API
  returns only their names. The wingman form drops a saved key when the
  provider or base URL changes, so a stored key is not replayed against
  a new endpoint.
- `POST /api/ai/models` makes the server request an admin-supplied URL.
  That is the point (local Ollama or Model Runner lives on private
  addresses), so unlike the unsubscribe guard it does not block private
  ranges. It is behind the session, only issues a GET, rejects non-http(s)
  schemes and URLs with credentials, times out after 10 s, and returns
  only parsed model ids or a short error, never the raw body.
- The CSP is unchanged: modules come from `'self'`, so no inline script or
  remote origin is needed.
- Plugins act through the public API, so every destructive-operation guard
  above (confirm, pins, rule validation) applies to them as well. The
  wingman only trashes, starts syncs and creates archive rules.

## Transport

- IMAP uses implicit TLS with the Mozilla root store compiled in
  (`webpki-roots`); the container needs no CA bundle.
- `security: plain` exists for local servers and tests. Do not use it over
  a network.
