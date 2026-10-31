# CI/CD

GitHub Actions, `.github/workflows/ci.yml`. Runs on pushes to `main`, every
PR into `main`, and manual dispatch. A newer run on the same ref cancels the
older one.

The Rust toolchain is `stable` (matches `rust-toolchain.toml`) via
`dtolnay/rust-toolchain`; the workspace declares `rust-version = "1.88"`
because it uses let chains and `as_chunks`. Node is 24.

## Jobs

- **`web`**: `npm ci`, `npm run check` (svelte-check, warnings fail the
  build), `npm run build`. Uploads `web/dist` as an artifact so the Rust job
  embeds the real UI.
- **`rust`**: needs `web`. Starts Dovecot (`dovecot/dovecot:2.3-latest`
  with `docker/dovecot/dovecot.conf`) on `127.0.0.1:3143`, then
  `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`. With
  `MAIL_SIEVE_TEST_IMAP` set, the IMAP integration suite
  (`crates/imap/tests/dovecot.rs`) and the server end-to-end flow
  (`crates/server/src/e2e_tests.rs`) run against it. Without the variable
  those tests print a skip message and pass, so local `cargo test` works
  offline.
- **`docker`**: needs `rust`. Builds the multi-stage `Dockerfile` with
  Buildx and the GitHub Actions cache. Nothing is pushed.

## Local equivalents

```sh
docker compose -f docker/compose.test.yml up -d
MAIL_SIEVE_TEST_IMAP=127.0.0.1:3143 cargo test --workspace
(cd web && npm ci && npm run check && npm run build)
docker build -t mail-sieve .
```

## Not set up yet

- Publishing images (GHCR) and release binaries. Deliberately deferred
  until the first tagged release and a decided license; publishing earlier
  would ship an artifact with no versioning promise.
- Multi-arch images (arm64 for NAS boxes). The Dockerfile is portable; it
  needs `docker/setup-qemu-action` and `platforms` once publishing exists.
- Real-provider tests (Gmail, Outlook) need secrets and accounts and are
  not automated.
